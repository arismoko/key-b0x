use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

macro_rules! normalized_keys {
    ($($name:ident),+ $(,)?) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            Serialize,
            Deserialize,
        )]
        pub enum NormalizedKey {
            $($name),+
        }

        impl NormalizedKey {
            pub const ALL: &[NormalizedKey] = &[
                $(NormalizedKey::$name),+
            ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(NormalizedKey::$name => stringify!($name)),+
                }
            }
        }

        impl fmt::Display for NormalizedKey {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for NormalizedKey {
            type Err = anyhow::Error;

            fn from_str(value: &str) -> Result<Self> {
                match value {
                    $(stringify!($name) => Ok(Self::$name)),+,
                    _ => Err(anyhow!("unknown normalized key: {value}")),
                }
            }
        }
    };
}

normalized_keys! {
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Backquote,
    Comma,
    Period,
    Slash,
    Space,
    Tab,
    Enter,
    Backspace,
    Escape,
    CapsLock,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    MetaLeft,
    MetaRight,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyboardId(String);

impl KeyboardId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KeyboardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for KeyboardId {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        if value.is_empty() {
            return Err(anyhow!("keyboard id cannot be empty"));
        }
        Ok(Self::new(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardInfo {
    pub id: KeyboardId,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyChange {
    pub key: NormalizedKey,
    pub pressed: bool,
}

pub trait KeyboardCaptureSession {
    fn poll_events(&mut self) -> Result<Vec<KeyChange>>;
    fn release(&mut self) -> Result<()>;
}

pub trait KeyboardBackend {
    type Session: KeyboardCaptureSession;

    fn list_keyboards(&self) -> Result<Vec<KeyboardInfo>>;
    fn open(&self) -> Result<Self::Session>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportStatus {
    WaitingForReader,
    Connected,
    NewlyConnected,
}

pub trait SlippiTransport {
    fn ensure_connected(&mut self) -> Result<TransportStatus>;
    fn send_line(&mut self, line: &str) -> Result<TransportStatus>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_keys_round_trip_as_dom_codes() {
        for key in NormalizedKey::ALL {
            let encoded = key.to_string();
            let decoded: NormalizedKey = encoded.parse().unwrap();
            assert_eq!(decoded, *key);
        }
    }

    #[test]
    fn keyboard_id_rejects_empty_values() {
        assert!("".parse::<KeyboardId>().is_err());
        assert_eq!("keyboard-1".parse::<KeyboardId>().unwrap().as_str(), "keyboard-1");
    }
}
