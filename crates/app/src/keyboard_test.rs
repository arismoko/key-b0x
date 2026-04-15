use crate::platform::active_keyboard_backend;
use anyhow::Result;
use key_b0x_platform::{KeyChange, KeyboardBackend, KeyboardCaptureSession, NormalizedKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KEYBOARD_TEST_WAIT_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardTestStatus {
    Idle,
    Running,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardTestState {
    pub status: KeyboardTestStatus,
    pub pressed_keys: Vec<NormalizedKey>,
    pub last_error: Option<String>,
}

impl KeyboardTestState {
    pub fn idle() -> Self {
        Self {
            status: KeyboardTestStatus::Idle,
            pressed_keys: Vec::new(),
            last_error: None,
        }
    }

    pub fn running(pressed_keys: impl Into<Vec<NormalizedKey>>) -> Self {
        Self {
            status: KeyboardTestStatus::Running,
            pressed_keys: pressed_keys.into(),
            last_error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: KeyboardTestStatus::Error,
            pressed_keys: Vec::new(),
            last_error: Some(message.into()),
        }
    }
}

impl Default for KeyboardTestState {
    fn default() -> Self {
        Self::idle()
    }
}

pub type KeyboardTestListener = Arc<dyn Fn(KeyboardTestState) + Send + Sync + 'static>;
pub type SharedKeyboardTestState = Arc<Mutex<KeyboardTestState>>;

#[derive(Clone)]
pub struct KeyboardTestWorkerContext {
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
    pub state: SharedKeyboardTestState,
    pub listener: Option<KeyboardTestListener>,
}

pub fn run_worker(context: KeyboardTestWorkerContext) {
    if let Err(error) = run_capture_loop(&context) {
        if !context.cancel.load(Ordering::Relaxed) {
            write_keyboard_test_state(
                &context.state,
                KeyboardTestState::error(error.to_string()),
                context.listener.as_ref(),
            );
        }
    }
}

fn run_capture_loop(context: &KeyboardTestWorkerContext) -> Result<()> {
    let backend = active_keyboard_backend();
    let mut capture = backend.open()?;
    let mut pressed_keys = BTreeSet::new();

    write_keyboard_test_state(
        &context.state,
        KeyboardTestState::running(Vec::<NormalizedKey>::new()),
        context.listener.as_ref(),
    );

    while !context.cancel.load(Ordering::Relaxed) {
        let changes = capture.wait_for_events(KEYBOARD_TEST_WAIT_INTERVAL)?;
        if changes.is_empty() {
            continue;
        }

        if apply_key_changes(&mut pressed_keys, changes) {
            write_keyboard_test_state(
                &context.state,
                KeyboardTestState::running(pressed_keys.iter().copied().collect::<Vec<_>>()),
                context.listener.as_ref(),
            );
        }
    }

    let _ = capture.release();
    Ok(())
}

fn apply_key_changes(pressed_keys: &mut BTreeSet<NormalizedKey>, changes: Vec<KeyChange>) -> bool {
    let mut changed = false;

    for change in changes {
        if change.pressed {
            changed |= pressed_keys.insert(change.key);
        } else {
            changed |= pressed_keys.remove(&change.key);
        }
    }

    changed
}

pub fn read_keyboard_test_state(state: &SharedKeyboardTestState) -> KeyboardTestState {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn write_keyboard_test_state(
    state: &SharedKeyboardTestState,
    next: KeyboardTestState,
    listener: Option<&KeyboardTestListener>,
) {
    {
        let mut guard = state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = next.clone();
    }

    if let Some(listener) = listener {
        listener(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn apply_key_changes_tracks_pressed_keys() {
        let mut pressed_keys = BTreeSet::new();

        assert!(apply_key_changes(
            &mut pressed_keys,
            vec![
                KeyChange {
                    key: NormalizedKey::KeyA,
                    pressed: true,
                    observed_at: Instant::now(),
                },
                KeyChange {
                    key: NormalizedKey::KeyS,
                    pressed: true,
                    observed_at: Instant::now(),
                },
            ],
        ));
        assert_eq!(
            pressed_keys.iter().copied().collect::<Vec<_>>(),
            vec![NormalizedKey::KeyA, NormalizedKey::KeyS]
        );

        assert!(apply_key_changes(
            &mut pressed_keys,
            vec![KeyChange {
                key: NormalizedKey::KeyA,
                pressed: false,
                observed_at: Instant::now(),
            }],
        ));
        assert_eq!(
            pressed_keys.iter().copied().collect::<Vec<_>>(),
            vec![NormalizedKey::KeyS]
        );
    }

    #[test]
    fn apply_key_changes_ignores_duplicate_state_updates() {
        let mut pressed_keys = BTreeSet::from([NormalizedKey::KeyA]);

        assert!(!apply_key_changes(
            &mut pressed_keys,
            vec![
                KeyChange {
                    key: NormalizedKey::KeyA,
                    pressed: true,
                    observed_at: Instant::now(),
                },
                KeyChange {
                    key: NormalizedKey::KeyS,
                    pressed: false,
                    observed_at: Instant::now(),
                },
            ],
        ));
        assert_eq!(
            pressed_keys.iter().copied().collect::<Vec<_>>(),
            vec![NormalizedKey::KeyA]
        );
    }
}
