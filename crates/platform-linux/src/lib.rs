#![cfg(unix)]

use anyhow::{Context, Result, anyhow, bail};
use evdev::{Device, EventSummary, KeyCode, enumerate};
use key_b0x_platform::{
    BackendCapabilities, KeyChange, KeyboardBackend, KeyboardCaptureSession, KeyboardId,
    KeyboardInfo, NormalizedKey, SlippiTransport, TransportStatus,
};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

macro_rules! normalized_key_pairs {
    ($(($normalized:ident, $evdev:ident)),+ $(,)?) => {
        pub fn key_code_from_normalized(key: NormalizedKey) -> Option<KeyCode> {
            match key {
                $(NormalizedKey::$normalized => Some(KeyCode::$evdev),)+
            }
        }

        pub fn normalized_key_from_code(code: KeyCode) -> Option<NormalizedKey> {
            match code {
                $(KeyCode::$evdev => Some(NormalizedKey::$normalized),)+
                _ => None,
            }
        }
    };
}

normalized_key_pairs! {
    (Digit0, KEY_0),
    (Digit1, KEY_1),
    (Digit2, KEY_2),
    (Digit3, KEY_3),
    (Digit4, KEY_4),
    (Digit5, KEY_5),
    (Digit6, KEY_6),
    (Digit7, KEY_7),
    (Digit8, KEY_8),
    (Digit9, KEY_9),
    (KeyA, KEY_A),
    (KeyB, KEY_B),
    (KeyC, KEY_C),
    (KeyD, KEY_D),
    (KeyE, KEY_E),
    (KeyF, KEY_F),
    (KeyG, KEY_G),
    (KeyH, KEY_H),
    (KeyI, KEY_I),
    (KeyJ, KEY_J),
    (KeyK, KEY_K),
    (KeyL, KEY_L),
    (KeyM, KEY_M),
    (KeyN, KEY_N),
    (KeyO, KEY_O),
    (KeyP, KEY_P),
    (KeyQ, KEY_Q),
    (KeyR, KEY_R),
    (KeyS, KEY_S),
    (KeyT, KEY_T),
    (KeyU, KEY_U),
    (KeyV, KEY_V),
    (KeyW, KEY_W),
    (KeyX, KEY_X),
    (KeyY, KEY_Y),
    (KeyZ, KEY_Z),
    (Minus, KEY_MINUS),
    (Equal, KEY_EQUAL),
    (BracketLeft, KEY_LEFTBRACE),
    (BracketRight, KEY_RIGHTBRACE),
    (Backslash, KEY_BACKSLASH),
    (Semicolon, KEY_SEMICOLON),
    (Quote, KEY_APOSTROPHE),
    (Backquote, KEY_GRAVE),
    (Comma, KEY_COMMA),
    (Period, KEY_DOT),
    (Slash, KEY_SLASH),
    (Space, KEY_SPACE),
    (Tab, KEY_TAB),
    (Enter, KEY_ENTER),
    (Backspace, KEY_BACKSPACE),
    (Escape, KEY_ESC),
    (CapsLock, KEY_CAPSLOCK),
    (ShiftLeft, KEY_LEFTSHIFT),
    (ShiftRight, KEY_RIGHTSHIFT),
    (ControlLeft, KEY_LEFTCTRL),
    (ControlRight, KEY_RIGHTCTRL),
    (AltLeft, KEY_LEFTALT),
    (AltRight, KEY_RIGHTALT),
    (MetaLeft, KEY_LEFTMETA),
    (MetaRight, KEY_RIGHTMETA),
    (ArrowUp, KEY_UP),
    (ArrowDown, KEY_DOWN),
    (ArrowLeft, KEY_LEFT),
    (ArrowRight, KEY_RIGHT)
}

pub struct LinuxKeyboardBackend;

impl LinuxKeyboardBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxKeyboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardBackend for LinuxKeyboardBackend {
    type Session = LinuxKeyboardCapture;

    fn list_keyboards(&self) -> Result<Vec<KeyboardInfo>> {
        let mut keyboards = enumerate()
            .filter_map(|(path, device)| {
                is_keyboard_device(&device).then(|| KeyboardInfo {
                    id: keyboard_id_for_path(&path),
                    name: device.name().unwrap_or("Unnamed keyboard").to_string(),
                })
            })
            .collect::<Vec<_>>();

        keyboards.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
        Ok(keyboards)
    }

    fn auto_detect_keyboard(&self) -> Result<Option<KeyboardInfo>> {
        let keyboards = self.list_keyboards()?;
        Ok(keyboards
            .iter()
            .find(|keyboard| keyboard.name.eq_ignore_ascii_case("keyd virtual keyboard"))
            .cloned()
            .or_else(|| keyboards.into_iter().next()))
    }

    fn open(&self, id: &KeyboardId, exclusive: bool) -> Result<Self::Session> {
        LinuxKeyboardCapture::open(id, exclusive)
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            exclusive_capture: true,
        }
    }
}

fn is_keyboard_device(device: &Device) -> bool {
    device.supported_keys().is_some_and(|keys| {
        keys.contains(KeyCode::KEY_A)
            && keys.contains(KeyCode::KEY_Z)
            && keys.contains(KeyCode::KEY_SPACE)
    })
}

fn keyboard_id_for_path(path: &Path) -> KeyboardId {
    preferred_keyboard_path(path)
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
        .parse()
        .expect("keyboard id must be non-empty")
}

fn preferred_keyboard_path(path: &Path) -> Option<PathBuf> {
    let canonical = fs::canonicalize(path).ok()?;
    let by_id_root = Path::new("/dev/input/by-id");
    let entries = fs::read_dir(by_id_root).ok()?;

    for entry in entries.flatten() {
        let candidate = entry.path();
        let file_name = candidate.file_name()?.to_string_lossy();
        if !file_name.contains("event-kbd") {
            continue;
        }
        if fs::canonicalize(&candidate).ok().as_ref() == Some(&canonical) {
            return Some(candidate);
        }
    }

    None
}

pub struct LinuxKeyboardCapture {
    device: Device,
    info: KeyboardInfo,
    grabbed: bool,
}

impl LinuxKeyboardCapture {
    pub fn open(id: &KeyboardId, exclusive: bool) -> Result<Self> {
        let path = Path::new(id.as_str());
        let mut device = Device::open(path)
            .with_context(|| format!("failed to open keyboard device {}", path.display()))?;
        device
            .set_nonblocking(true)
            .with_context(|| format!("failed to configure {}", path.display()))?;
        if exclusive {
            device
                .grab()
                .with_context(|| format!("failed to grab {}", path.display()))?;
        }

        let info = KeyboardInfo {
            id: id.clone(),
            name: device.name().unwrap_or("Unnamed keyboard").to_string(),
        };

        Ok(Self {
            device,
            info,
            grabbed: exclusive,
        })
    }
}

impl KeyboardCaptureSession for LinuxKeyboardCapture {
    fn info(&self) -> &KeyboardInfo {
        &self.info
    }

    fn poll_events(&mut self) -> Result<Vec<KeyChange>> {
        let events = match self.device.fetch_events() {
            Ok(events) => events,
            Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(Vec::new()),
            Err(err) => return Err(err).context("failed to read keyboard events"),
        };

        let mut changes = Vec::new();
        for event in events {
            if let EventSummary::Key(_, code, value) = event.destructure() {
                let Some(key) = normalized_key_from_code(code) else {
                    continue;
                };
                match value {
                    0 => changes.push(KeyChange {
                        key,
                        pressed: false,
                    }),
                    1 => changes.push(KeyChange { key, pressed: true }),
                    _ => {}
                }
            }
        }

        Ok(changes)
    }

    fn release(&mut self) -> Result<()> {
        if self.grabbed {
            self.device.ungrab().context("failed to ungrab keyboard")?;
            self.grabbed = false;
        }
        Ok(())
    }
}

impl Drop for LinuxKeyboardCapture {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

pub struct LinuxFifoTransport {
    pipe_path: PathBuf,
    file: Option<File>,
}

impl LinuxFifoTransport {
    pub fn new(slippi_user_path: impl AsRef<Path>, port: u8) -> Result<Self> {
        if port != 1 {
            bail!("only Slippi port 1 is supported in this proof of concept");
        }

        let pipe_path = slippi_user_path
            .as_ref()
            .join("Pipes")
            .join(format!("slippibot{port}"));

        Ok(Self {
            pipe_path,
            file: None,
        })
    }

    pub fn pipe_path(&self) -> &Path {
        &self.pipe_path
    }

    pub fn ensure_fifo(&self) -> Result<()> {
        let parent = self
            .pipe_path
            .parent()
            .ok_or_else(|| anyhow!("pipe path has no parent: {}", self.pipe_path.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;

        if self.pipe_path.exists() {
            let metadata = fs::metadata(&self.pipe_path)
                .with_context(|| format!("failed to inspect {}", self.pipe_path.display()))?;
            if metadata.file_type().is_fifo() {
                return Ok(());
            }
            bail!("{} exists but is not a FIFO", self.pipe_path.display());
        }

        mkfifo(&self.pipe_path, Mode::from_bits_truncate(0o600))
            .with_context(|| format!("failed to create fifo {}", self.pipe_path.display()))?;
        Ok(())
    }
}

impl SlippiTransport for LinuxFifoTransport {
    fn ensure_connected(&mut self) -> Result<TransportStatus> {
        if self.file.is_some() {
            return Ok(TransportStatus::Connected);
        }

        match OpenOptions::new()
            .write(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(&self.pipe_path)
        {
            Ok(file) => {
                self.file = Some(file);
                Ok(TransportStatus::NewlyConnected)
            }
            Err(err) if err.raw_os_error() == Some(nix::libc::ENXIO) => {
                Ok(TransportStatus::WaitingForReader)
            }
            Err(err) => Err(err)
                .with_context(|| format!("failed to connect to fifo {}", self.pipe_path.display())),
        }
    }

    fn send_line(&mut self, line: &str) -> Result<TransportStatus> {
        let status = self.ensure_connected()?;
        if status == TransportStatus::WaitingForReader {
            return Ok(status);
        }

        let Some(file) = self.file.as_mut() else {
            return Ok(TransportStatus::WaitingForReader);
        };

        match writeln!(file, "{line}") {
            Ok(()) => Ok(status),
            Err(err) if err.kind() == ErrorKind::BrokenPipe => {
                self.file = None;
                Ok(TransportStatus::WaitingForReader)
            }
            Err(err) => Err(err)
                .with_context(|| format!("failed to write to fifo {}", self.pipe_path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions as FsOpenOptions;
    use std::io::Read;

    #[test]
    fn normalized_key_round_trips_with_evdev_codes() {
        for key in NormalizedKey::ALL {
            let code = key_code_from_normalized(*key).unwrap();
            assert_eq!(normalized_key_from_code(code), Some(*key));
        }
    }

    #[test]
    fn ensure_fifo_creates_a_fifo() {
        let temp = tempfile::tempdir().unwrap();
        let transport = LinuxFifoTransport::new(temp.path(), 1).unwrap();
        transport.ensure_fifo().unwrap();

        let meta = fs::metadata(transport.pipe_path()).unwrap();
        assert!(meta.file_type().is_fifo());
    }

    #[test]
    fn connect_waits_when_reader_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let mut transport = LinuxFifoTransport::new(temp.path(), 1).unwrap();
        transport.ensure_fifo().unwrap();

        let status = transport.ensure_connected().unwrap();
        assert_eq!(status, TransportStatus::WaitingForReader);
    }

    #[test]
    fn send_line_writes_when_reader_is_connected() {
        let temp = tempfile::tempdir().unwrap();
        let mut transport = LinuxFifoTransport::new(temp.path(), 1).unwrap();
        transport.ensure_fifo().unwrap();

        let reader = FsOpenOptions::new()
            .read(true)
            .write(true)
            .open(transport.pipe_path())
            .unwrap();

        let status = transport.send_line("PRESS A").unwrap();
        assert_eq!(status, TransportStatus::NewlyConnected);
        let mut reader = reader;
        let mut buf = [0u8; 16];
        let bytes_read = reader.read(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf[..bytes_read]);
        assert!(text.contains("PRESS A"));
    }
}
