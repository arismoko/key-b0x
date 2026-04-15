use anyhow::Result;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use key_b0x_platform_linux::{
    LinuxFifoTransport as ActiveTransport, LinuxKeyboardBackend as ActiveKeyboardBackend,
};
#[cfg(target_os = "windows")]
use key_b0x_platform_windows::{
    WindowsKeyboardBackend as ActiveKeyboardBackend,
    WindowsNamedPipeTransport as ActiveTransport,
};

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("key-b0x-runtime currently supports only Linux and Windows");

pub fn active_keyboard_backend() -> ActiveKeyboardBackend {
    ActiveKeyboardBackend::new()
}

pub fn active_transport(slippi_user_path: &Path, port: u8) -> Result<ActiveTransport> {
    ActiveTransport::new(slippi_user_path, port)
}

#[cfg(target_os = "linux")]
pub fn default_slippi_user_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("SlippiOnline")
}

#[cfg(target_os = "windows")]
pub fn default_slippi_user_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public\AppData\Roaming"))
        .join("Slippi Launcher")
        .join("netplay")
        .join("User")
}
