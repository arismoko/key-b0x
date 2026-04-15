use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Idle,
    Starting,
    Running,
    WaitingForSlippi,
    Stopping,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeState {
    pub status: RuntimeStatus,
    pub started_at: Option<u64>,
    pub last_error: Option<String>,
}

impl RuntimeState {
    pub fn idle() -> Self {
        Self {
            status: RuntimeStatus::Idle,
            started_at: None,
            last_error: None,
        }
    }

    pub fn starting(started_at: u64) -> Self {
        Self {
            status: RuntimeStatus::Starting,
            started_at: Some(started_at),
            last_error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: RuntimeStatus::Error,
            started_at: None,
            last_error: Some(message.into()),
        }
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::idle()
    }
}

pub type StateListener = Arc<dyn Fn(RuntimeState) + Send + Sync + 'static>;
pub type SharedRuntimeState = Arc<Mutex<RuntimeState>>;

pub fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn read_runtime_state(state: &SharedRuntimeState) -> RuntimeState {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn write_runtime_state(
    state: &SharedRuntimeState,
    next: RuntimeState,
    listener: Option<&StateListener>,
) {
    {
        let mut guard = state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = next.clone();
    }

    if let Some(listener) = listener {
        listener(next);
    }
}
