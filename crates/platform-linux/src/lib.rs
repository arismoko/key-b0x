use anyhow::{Context, Result, anyhow, bail};
use evdev::{Device, EventSummary, KeyCode, enumerate};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardInfo {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyChange {
    pub code: KeyCode,
    pub pressed: bool,
}

pub fn key_code_from_name(name: &str) -> Result<KeyCode> {
    KeyCode::from_str(name).map_err(|_| anyhow!("unknown evdev key code: {name}"))
}

pub fn key_name(code: KeyCode) -> String {
    format!("{code:?}")
}

pub fn list_keyboards() -> Vec<KeyboardInfo> {
    let mut keyboards = enumerate()
        .filter_map(|(path, device)| {
            is_keyboard_device(&device).then(|| KeyboardInfo {
                path,
                name: device.name().unwrap_or("Unnamed keyboard").to_string(),
            })
        })
        .collect::<Vec<_>>();

    keyboards.sort_by(|lhs, rhs| lhs.path.cmp(&rhs.path));
    keyboards
}

pub fn auto_detect_keyboard() -> Option<KeyboardInfo> {
    list_keyboards().into_iter().find(|keyboard| {
        Device::open(&keyboard.path)
            .ok()
            .as_ref()
            .is_some_and(is_keyboard_device)
    })
}

fn is_keyboard_device(device: &Device) -> bool {
    device.supported_keys().is_some_and(|keys| {
        keys.contains(KeyCode::KEY_A)
            && keys.contains(KeyCode::KEY_Z)
            && keys.contains(KeyCode::KEY_SPACE)
    })
}

pub struct KeyboardCapture {
    device: Device,
    info: KeyboardInfo,
    grabbed: bool,
}

impl KeyboardCapture {
    pub fn open(path: &Path, exclusive: bool) -> Result<Self> {
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
            path: path.to_path_buf(),
            name: device.name().unwrap_or("Unnamed keyboard").to_string(),
        };

        Ok(Self {
            device,
            info,
            grabbed: exclusive,
        })
    }

    pub fn info(&self) -> &KeyboardInfo {
        &self.info
    }

    pub fn poll_events(&mut self) -> Result<Vec<KeyChange>> {
        let events = match self.device.fetch_events() {
            Ok(events) => events,
            Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(Vec::new()),
            Err(err) => return Err(err).context("failed to read keyboard events"),
        };

        let mut changes = Vec::new();
        for event in events {
            if let EventSummary::Key(_, code, value) = event.destructure() {
                match value {
                    0 => changes.push(KeyChange {
                        code,
                        pressed: false,
                    }),
                    1 => changes.push(KeyChange {
                        code,
                        pressed: true,
                    }),
                    _ => {}
                }
            }
        }

        Ok(changes)
    }

    pub fn release(&mut self) -> Result<()> {
        if self.grabbed {
            self.device.ungrab().context("failed to ungrab keyboard")?;
            self.grabbed = false;
        }
        Ok(())
    }
}

impl Drop for KeyboardCapture {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    WaitingForReader,
    Connected,
    NewlyConnected,
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

    pub fn ensure_connected(&mut self) -> Result<ConnectionStatus> {
        if self.file.is_some() {
            return Ok(ConnectionStatus::Connected);
        }

        match OpenOptions::new()
            .write(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(&self.pipe_path)
        {
            Ok(file) => {
                self.file = Some(file);
                Ok(ConnectionStatus::NewlyConnected)
            }
            Err(err) if err.raw_os_error() == Some(nix::libc::ENXIO) => {
                Ok(ConnectionStatus::WaitingForReader)
            }
            Err(err) => Err(err)
                .with_context(|| format!("failed to connect to fifo {}", self.pipe_path.display())),
        }
    }

    pub fn send_line(&mut self, line: &str) -> Result<ConnectionStatus> {
        let status = self.ensure_connected()?;
        if status == ConnectionStatus::WaitingForReader {
            return Ok(status);
        }

        let Some(file) = self.file.as_mut() else {
            return Ok(ConnectionStatus::WaitingForReader);
        };

        match writeln!(file, "{line}") {
            Ok(()) => Ok(status),
            Err(err) if err.kind() == ErrorKind::BrokenPipe => {
                self.file = None;
                Ok(ConnectionStatus::WaitingForReader)
            }
            Err(err) => Err(err)
                .with_context(|| format!("failed to write to fifo {}", self.pipe_path.display())),
        }
    }

    pub fn disconnect(&mut self) {
        self.file = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions as FsOpenOptions;
    use std::io::Read;

    #[test]
    fn key_names_round_trip() {
        let code = key_code_from_name("KEY_A").unwrap();
        assert_eq!(code, KeyCode::KEY_A);
        assert_eq!(key_name(code), "KEY_A");
    }

    #[test]
    fn ensure_fifo_creates_a_fifo() {
        let temp = tempfile::tempdir().unwrap();
        let transport = LinuxFifoTransport::new(temp.path(), 1).unwrap();
        transport.ensure_fifo().unwrap();

        let metadata = fs::metadata(transport.pipe_path()).unwrap();
        assert!(metadata.file_type().is_fifo());
    }

    #[test]
    fn connect_waits_when_reader_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let mut transport = LinuxFifoTransport::new(temp.path(), 1).unwrap();
        transport.ensure_fifo().unwrap();

        assert_eq!(
            transport.ensure_connected().unwrap(),
            ConnectionStatus::WaitingForReader
        );
    }

    #[test]
    fn send_line_writes_when_reader_is_connected() {
        let temp = tempfile::tempdir().unwrap();
        let mut transport = LinuxFifoTransport::new(temp.path(), 1).unwrap();
        transport.ensure_fifo().unwrap();

        let mut reader = FsOpenOptions::new()
            .read(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(transport.pipe_path())
            .unwrap();

        let status = transport.send_line("PRESS A").unwrap();
        assert_eq!(status, ConnectionStatus::NewlyConnected);

        let mut buffer = [0_u8; 64];
        let bytes = reader.read(&mut buffer).unwrap();
        assert_eq!(std::str::from_utf8(&buffer[..bytes]).unwrap(), "PRESS A\n");
    }
}
