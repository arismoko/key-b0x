#[cfg(windows)]
mod imp {
    #![allow(unsafe_op_in_unsafe_fn)]

    use anyhow::{Result, anyhow, bail};
    use key_b0x_platform::{
        KeyChange, KeyboardBackend, KeyboardCaptureSession, KeyboardId, KeyboardInfo,
        NormalizedKey, SlippiTransport, TransportStatus,
    };
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::path::Path;
    use std::ptr::{null, null_mut};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::OnceLock;
    use std::thread::{self, JoinHandle};
    use windows_sys::Win32::Devices::HumanInterfaceDevice::{
        HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_PAGE_GENERIC,
    };
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_BROKEN_PIPE, ERROR_CLASS_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND,
        ERROR_INVALID_HANDLE, ERROR_NO_DATA, ERROR_PIPE_BUSY, GetLastError, HANDLE, HINSTANCE,
        HWND, INVALID_HANDLE_VALUE, LPARAM, LRESULT, WPARAM,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, OPEN_EXISTING, WriteFile,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Pipes::WaitNamedPipeW;
    use windows_sys::Win32::UI::Input::{
        GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList, HRAWINPUT, RAWINPUT,
        RAWINPUTDEVICE, RAWINPUTDEVICELIST, RAWINPUTHEADER, RAWKEYBOARD, RID_INPUT,
        RIDI_DEVICENAME, RIDEV_INPUTSINK, RIM_TYPEKEYBOARD, RegisterRawInputDevices,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
        GetMessageW, GetWindowLongPtrW, HWND_MESSAGE, MSG, PostMessageW, PostQuitMessage,
        RI_KEY_BREAK, RI_KEY_E0, RI_KEY_E1, RegisterClassW, SetWindowLongPtrW,
        TranslateMessage, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_INPUT, WNDCLASSW,
    };

    macro_rules! base_scancode_pairs {
        ($(($code:expr, $key:ident)),+ $(,)?) => {
            fn key_from_base_scancode(scancode: u16) -> Option<NormalizedKey> {
                match scancode {
                    $($code => Some(NormalizedKey::$key),)+
                    _ => None,
                }
            }
        };
    }

    macro_rules! extended_scancode_pairs {
        ($(($code:expr, $key:ident)),+ $(,)?) => {
            fn key_from_extended_scancode(scancode: u16) -> Option<NormalizedKey> {
                match scancode {
                    $($code => Some(NormalizedKey::$key),)+
                    _ => None,
                }
            }
        };
    }

    base_scancode_pairs! {
        (0x02, Digit1),
        (0x03, Digit2),
        (0x04, Digit3),
        (0x05, Digit4),
        (0x06, Digit5),
        (0x07, Digit6),
        (0x08, Digit7),
        (0x09, Digit8),
        (0x0A, Digit9),
        (0x0B, Digit0),
        (0x0C, Minus),
        (0x0D, Equal),
        (0x0E, Backspace),
        (0x0F, Tab),
        (0x10, KeyQ),
        (0x11, KeyW),
        (0x12, KeyE),
        (0x13, KeyR),
        (0x14, KeyT),
        (0x15, KeyY),
        (0x16, KeyU),
        (0x17, KeyI),
        (0x18, KeyO),
        (0x19, KeyP),
        (0x1A, BracketLeft),
        (0x1B, BracketRight),
        (0x1C, Enter),
        (0x1D, ControlLeft),
        (0x1E, KeyA),
        (0x1F, KeyS),
        (0x20, KeyD),
        (0x21, KeyF),
        (0x22, KeyG),
        (0x23, KeyH),
        (0x24, KeyJ),
        (0x25, KeyK),
        (0x26, KeyL),
        (0x27, Semicolon),
        (0x28, Quote),
        (0x29, Backquote),
        (0x2A, ShiftLeft),
        (0x2B, Backslash),
        (0x2C, KeyZ),
        (0x2D, KeyX),
        (0x2E, KeyC),
        (0x2F, KeyV),
        (0x30, KeyB),
        (0x31, KeyN),
        (0x32, KeyM),
        (0x33, Comma),
        (0x34, Period),
        (0x35, Slash),
        (0x36, ShiftRight),
        (0x38, AltLeft),
        (0x39, Space),
        (0x3A, CapsLock)
    }

    extended_scancode_pairs! {
        (0x1D, ControlRight),
        (0x38, AltRight),
        (0x48, ArrowUp),
        (0x4B, ArrowLeft),
        (0x4D, ArrowRight),
        (0x50, ArrowDown),
        (0x5B, MetaLeft),
        (0x5C, MetaRight)
    }

    fn normalized_key_from_raw(raw: &RAWKEYBOARD) -> Option<NormalizedKey> {
        let scancode = raw.MakeCode;
        let flags = u32::from(raw.Flags);
        if scancode == 0 {
            return None;
        }
        if flags & RI_KEY_E1 != 0 {
            return None;
        }
        if flags & RI_KEY_E0 != 0 {
            key_from_extended_scancode(scancode)
        } else {
            key_from_base_scancode(scancode)
        }
    }

    fn is_key_release(raw: &RAWKEYBOARD) -> bool {
        u32::from(raw.Flags) & RI_KEY_BREAK != 0
    }

    #[derive(Clone)]
    struct RawInputContext {
        tx: Sender<KeyChange>,
    }

    struct WindowReady {
        hwnd: isize,
    }

    pub struct WindowsKeyboardBackend;

    impl WindowsKeyboardBackend {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for WindowsKeyboardBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl KeyboardBackend for WindowsKeyboardBackend {
        type Session = WindowsKeyboardCapture;

        fn list_keyboards(&self) -> Result<Vec<KeyboardInfo>> {
            enumerate_keyboards()
        }

        fn open(&self) -> Result<Self::Session> {
            WindowsKeyboardCapture::open()
        }
    }

    fn enumerate_keyboards() -> Result<Vec<KeyboardInfo>> {
        let devices = raw_input_devices()?;
        let mut keyboards = Vec::new();

        for device in devices {
            let device_name = unsafe { raw_input_device_name(device.hDevice) }?;
            let name = device_name.clone();
            keyboards.push(KeyboardInfo {
                id: KeyboardId::new(device_name),
                name,
            });
        }

        keyboards.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
        keyboards.dedup_by(|lhs, rhs| lhs.id == rhs.id);
        Ok(keyboards)
    }

    fn raw_input_devices() -> Result<Vec<RAWINPUTDEVICELIST>> {
        unsafe {
            let mut count = 0u32;
            let result =
                GetRawInputDeviceList(null_mut(), &mut count, size_of::<RAWINPUTDEVICELIST>() as u32);
            if result == u32::MAX {
                return Err(last_os_error("failed to query raw input device count"));
            }

            let mut devices = vec![zeroed::<RAWINPUTDEVICELIST>(); count as usize];
            let result = GetRawInputDeviceList(
                devices.as_mut_ptr(),
                &mut count,
                size_of::<RAWINPUTDEVICELIST>() as u32,
            );
            if result == u32::MAX {
                return Err(last_os_error("failed to enumerate raw input devices"));
            }
            devices.truncate(count as usize);
            Ok(devices
                .into_iter()
                .filter(|device| device.dwType == RIM_TYPEKEYBOARD)
                .collect())
        }
    }

    unsafe fn raw_input_device_name(device: HANDLE) -> Result<String> {
        let mut size = 0u32;
        let result = GetRawInputDeviceInfoW(device, RIDI_DEVICENAME, null_mut(), &mut size);
        if result == u32::MAX {
            return Err(last_os_error("failed to query raw input device name length"));
        }

        let mut buffer = vec![0u16; size as usize];
        let result = GetRawInputDeviceInfoW(
            device,
            RIDI_DEVICENAME,
            buffer.as_mut_ptr().cast::<c_void>(),
            &mut size,
        );
        if result == u32::MAX {
            return Err(last_os_error("failed to query raw input device name"));
        }
        if let Some(0) = buffer.last().copied() {
            buffer.pop();
        }
        Ok(String::from_utf16_lossy(&buffer))
    }

    pub struct WindowsKeyboardCapture {
        rx: Receiver<KeyChange>,
        hwnd: HWND,
        thread: Option<JoinHandle<()>>,
        released: bool,
    }

    impl WindowsKeyboardCapture {
        pub fn open() -> Result<Self> {
            if enumerate_keyboards()?.is_empty() {
                bail!("no keyboards detected");
            }

            let (events_tx, events_rx) = mpsc::channel();
            let (ready_tx, ready_rx) = mpsc::channel();

            let thread = thread::spawn(move || {
                let _ = run_raw_input_thread(events_tx, ready_tx);
            });

            let ready = ready_rx
                .recv()
                .map_err(|_| anyhow!("failed to initialize Windows raw input thread"))?
                .map_err(|err| anyhow!(err))?;

            Ok(Self {
                rx: events_rx,
                hwnd: ready.hwnd as HWND,
                thread: Some(thread),
                released: false,
            })
        }
    }

    impl KeyboardCaptureSession for WindowsKeyboardCapture {
        fn poll_events(&mut self) -> Result<Vec<KeyChange>> {
            let mut changes = Vec::new();
            while let Ok(change) = self.rx.try_recv() {
                changes.push(change);
            }
            Ok(changes)
        }

        fn release(&mut self) -> Result<()> {
            if self.released {
                return Ok(());
            }

            unsafe {
                if !self.hwnd.is_null() {
                    PostMessageW(self.hwnd, WM_CLOSE, 0, 0);
                }
            }

            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }

            self.released = true;
            Ok(())
        }
    }

    impl Drop for WindowsKeyboardCapture {
        fn drop(&mut self) {
            let _ = self.release();
        }
    }

    fn run_raw_input_thread(tx: Sender<KeyChange>, ready_tx: Sender<Result<WindowReady, String>>) -> Result<()> {
        unsafe {
            let class_name = match register_window_class() {
                Ok(class_name) => class_name,
                Err(err) => {
                    let _ = ready_tx.send(Err(err.to_string()));
                    return Err(err);
                }
            };
            let instance = GetModuleHandleW(null());
            if instance.is_null() {
                let err = last_os_error("failed to get module handle");
                let _ = ready_tx.send(Err(err.to_string()));
                return Err(err);
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name.as_ptr(),
                class_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                null_mut(),
                instance,
                null(),
            );
            if hwnd.is_null() {
                let err = last_os_error("failed to create raw input window");
                let _ = ready_tx.send(Err(err.to_string()));
                return Err(err);
            }

            let context = Box::new(RawInputContext { tx });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

            let device = RAWINPUTDEVICE {
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_KEYBOARD,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            };
            let registered =
                RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32);
            if registered == 0 {
                DestroyWindow(hwnd);
                let err = last_os_error("failed to register raw input device");
                let _ = ready_tx.send(Err(err.to_string()));
                return Err(err);
            }

            let _ = ready_tx.send(Ok(WindowReady {
                hwnd: hwnd as isize,
            }));

            let mut message: MSG = zeroed();
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }

            Ok(())
        }
    }

    unsafe fn register_window_class() -> Result<&'static Vec<u16>> {
        static CLASS_NAME: OnceLock<Vec<u16>> = OnceLock::new();
        let class_name = CLASS_NAME.get_or_init(|| wide("key-b0x-raw-input-window"));
        let instance: HINSTANCE = GetModuleHandleW(null());
        if instance.is_null() {
            return Err(last_os_error("failed to get module handle"));
        }

        let class = WNDCLASSW {
            lpfnWndProc: Some(raw_input_window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };

        if RegisterClassW(&class) == 0 {
            let error = GetLastError();
            if error != ERROR_CLASS_ALREADY_EXISTS {
                return Err(last_os_error("failed to register raw input window class"));
            }
        }

        Ok(class_name)
    }

    unsafe extern "system" fn raw_input_window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_INPUT => {
                let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RawInputContext;
                if context_ptr.is_null() {
                    return 0;
                }
                let context = &*context_ptr;
                if let Some(change) = key_change_from_wm_input(lparam as HRAWINPUT) {
                    let _ = context.tx.send(change);
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RawInputContext;
                if !context_ptr.is_null() {
                    drop(Box::from_raw(context_ptr));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    unsafe fn key_change_from_wm_input(input: HRAWINPUT) -> Option<KeyChange> {
        let mut size = 0u32;
        if GetRawInputData(
            input,
            RID_INPUT,
            null_mut(),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        ) == u32::MAX
        {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        if GetRawInputData(
            input,
            RID_INPUT,
            buffer.as_mut_ptr().cast::<c_void>(),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        ) == u32::MAX
        {
            return None;
        }

        let raw = &*(buffer.as_ptr() as *const RAWINPUT);
        // Windows v1 accepts keyboard events from any attached keyboard in the
        // interactive desktop session. Exact raw-device handle matching proved
        // unreliable across laptop keyboard stacks and dropped all input.
        if raw.header.dwType != RIM_TYPEKEYBOARD {
            return None;
        }

        let keyboard = unsafe { raw.data.keyboard };
        let key = normalized_key_from_raw(&keyboard)?;
        Some(KeyChange {
            key,
            pressed: !is_key_release(&keyboard),
        })
    }

    pub struct WindowsNamedPipeTransport {
        pipe_name: String,
        handle: Option<HANDLE>,
    }

    impl WindowsNamedPipeTransport {
        pub fn new(_slippi_user_path: &Path, port: u8) -> Result<Self> {
            if port != 1 {
                bail!("only Slippi port 1 is supported in this proof of concept");
            }

            Ok(Self {
                pipe_name: format!(r"\\.\pipe\slippibot{port}"),
                handle: None,
            })
        }
    }

    impl SlippiTransport for WindowsNamedPipeTransport {
        fn ensure_connected(&mut self) -> Result<TransportStatus> {
            if self.handle.is_some() {
                return Ok(TransportStatus::Connected);
            }

            unsafe {
                let pipe_name = wide(&self.pipe_name);
                let handle = CreateFileW(
                    pipe_name.as_ptr(),
                    FILE_GENERIC_WRITE,
                    0,
                    null(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                );

                if handle == INVALID_HANDLE_VALUE {
                    let error = GetLastError();
                    if matches!(error, ERROR_FILE_NOT_FOUND | ERROR_PIPE_BUSY) {
                        let _ = WaitNamedPipeW(pipe_name.as_ptr(), 0);
                        return Ok(TransportStatus::WaitingForReader);
                    }
                    return Err(last_os_error("failed to connect to Slippi named pipe"));
                }

                self.handle = Some(handle);
                Ok(TransportStatus::NewlyConnected)
            }
        }

        fn send_line(&mut self, line: &str) -> Result<TransportStatus> {
            let status = self.ensure_connected()?;
            if status == TransportStatus::WaitingForReader {
                return Ok(status);
            }

            let Some(handle) = self.handle else {
                return Ok(TransportStatus::WaitingForReader);
            };

            let mut bytes = line.as_bytes().to_vec();
            bytes.push(b'\n');

            unsafe {
                let mut written = 0u32;
                if WriteFile(
                    handle,
                    bytes.as_ptr(),
                    bytes.len() as u32,
                    &mut written,
                    null_mut(),
                ) == 0
                {
                    let error = GetLastError();
                    if matches!(error, ERROR_BROKEN_PIPE | ERROR_NO_DATA | ERROR_INVALID_HANDLE) {
                        let _ = CloseHandle(handle);
                        self.handle = None;
                        return Ok(TransportStatus::WaitingForReader);
                    }
                    return Err(last_os_error("failed to write to Slippi named pipe"));
                }
            }

            Ok(status)
        }
    }

    impl Drop for WindowsNamedPipeTransport {
        fn drop(&mut self) {
            if let Some(handle) = self.handle.take() {
                unsafe {
                    let _ = CloseHandle(handle);
                }
            }
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    fn last_os_error(context: &str) -> anyhow::Error {
        anyhow!("{context}: Windows error {}", unsafe { GetLastError() })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn base_scancodes_map_to_expected_keys() {
            assert_eq!(key_from_base_scancode(0x05), Some(NormalizedKey::Digit4));
            assert_eq!(key_from_base_scancode(0x1B), Some(NormalizedKey::BracketRight));
            assert_eq!(key_from_base_scancode(0x2F), Some(NormalizedKey::KeyV));
        }

        #[test]
        fn extended_scancodes_map_to_expected_keys() {
            assert_eq!(key_from_extended_scancode(0x48), Some(NormalizedKey::ArrowUp));
            assert_eq!(key_from_extended_scancode(0x4D), Some(NormalizedKey::ArrowRight));
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use anyhow::{Result, bail};
    use key_b0x_platform::{
        KeyChange, KeyboardBackend, KeyboardCaptureSession, KeyboardInfo, SlippiTransport,
        TransportStatus,
    };
    use std::path::Path;

    pub struct WindowsKeyboardBackend;

    impl WindowsKeyboardBackend {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for WindowsKeyboardBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    pub struct WindowsKeyboardCapture;

    impl KeyboardCaptureSession for WindowsKeyboardCapture {
        fn poll_events(&mut self) -> Result<Vec<KeyChange>> {
            Ok(Vec::new())
        }

        fn release(&mut self) -> Result<()> {
            Ok(())
        }
    }

    impl KeyboardBackend for WindowsKeyboardBackend {
        type Session = WindowsKeyboardCapture;

        fn list_keyboards(&self) -> Result<Vec<KeyboardInfo>> {
            Ok(Vec::new())
        }

        fn open(&self) -> Result<Self::Session> {
            bail!("Windows keyboard capture is only available on Windows")
        }
    }

    pub struct WindowsNamedPipeTransport;

    impl WindowsNamedPipeTransport {
        pub fn new(_slippi_user_path: &Path, _port: u8) -> Result<Self> {
            bail!("Windows named-pipe transport is only available on Windows")
        }
    }

    impl SlippiTransport for WindowsNamedPipeTransport {
        fn ensure_connected(&mut self) -> Result<TransportStatus> {
            bail!("Windows named-pipe transport is only available on Windows")
        }

        fn send_line(&mut self, _line: &str) -> Result<TransportStatus> {
            bail!("Windows named-pipe transport is only available on Windows")
        }
    }
}

pub use imp::*;
