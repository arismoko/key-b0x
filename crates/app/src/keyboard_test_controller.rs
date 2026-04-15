use crate::keyboard_test::{
    KeyboardTestListener, KeyboardTestState, KeyboardTestWorkerContext, SharedKeyboardTestState,
    read_keyboard_test_state, run_worker, write_keyboard_test_state,
};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

pub type KeyboardTestWorkerSpawner =
    Arc<dyn Fn(KeyboardTestWorkerContext) -> Result<JoinHandle<()>> + Send + Sync + 'static>;

struct WorkerHandle {
    cancel: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

pub struct KeyboardTestController {
    state: SharedKeyboardTestState,
    listener: Option<KeyboardTestListener>,
    worker: Option<WorkerHandle>,
    worker_spawner: KeyboardTestWorkerSpawner,
}

impl KeyboardTestController {
    pub fn new(listener: Option<KeyboardTestListener>) -> Self {
        Self::with_spawner(listener, default_worker_spawner())
    }

    pub fn with_spawner(
        listener: Option<KeyboardTestListener>,
        worker_spawner: KeyboardTestWorkerSpawner,
    ) -> Self {
        Self {
            state: Default::default(),
            listener,
            worker: None,
            worker_spawner,
        }
    }

    pub fn get_state(&mut self) -> KeyboardTestState {
        self.refresh_worker();
        read_keyboard_test_state(&self.state)
    }

    pub fn start(&mut self) -> Result<KeyboardTestState> {
        self.refresh_worker();
        if self.worker.is_some() {
            return Ok(read_keyboard_test_state(&self.state));
        }

        write_keyboard_test_state(
            &self.state,
            KeyboardTestState::running(Vec::new()),
            self.listener.as_ref(),
        );

        let cancel = Arc::new(AtomicBool::new(false));
        let join = match (self.worker_spawner)(KeyboardTestWorkerContext {
            cancel: Arc::clone(&cancel),
            state: Arc::clone(&self.state),
            listener: self.listener.clone(),
        }) {
            Ok(join) => join,
            Err(error) => {
                write_keyboard_test_state(
                    &self.state,
                    KeyboardTestState::error(error.to_string()),
                    self.listener.as_ref(),
                );
                return Err(error);
            }
        };

        self.worker = Some(WorkerHandle { cancel, join });
        Ok(read_keyboard_test_state(&self.state))
    }

    pub fn stop(&mut self) -> KeyboardTestState {
        self.refresh_worker();
        let Some(worker) = self.worker.take() else {
            write_keyboard_test_state(
                &self.state,
                KeyboardTestState::idle(),
                self.listener.as_ref(),
            );
            return read_keyboard_test_state(&self.state);
        };

        worker.cancel.store(true, Ordering::Relaxed);

        if worker.join.join().is_err() {
            write_keyboard_test_state(
                &self.state,
                KeyboardTestState::error("keyboard test worker panicked"),
                self.listener.as_ref(),
            );
            return read_keyboard_test_state(&self.state);
        }

        write_keyboard_test_state(
            &self.state,
            KeyboardTestState::idle(),
            self.listener.as_ref(),
        );
        read_keyboard_test_state(&self.state)
    }

    pub fn shutdown(&mut self) -> KeyboardTestState {
        self.stop()
    }

    fn refresh_worker(&mut self) {
        let finished = self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.join.is_finished());

        if !finished {
            return;
        }

        let worker = self.worker.take().expect("worker handle disappeared");
        if worker.join.join().is_err() {
            write_keyboard_test_state(
                &self.state,
                KeyboardTestState::error("keyboard test worker panicked"),
                self.listener.as_ref(),
            );
        }
    }
}

impl Drop for KeyboardTestController {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn default_worker_spawner() -> KeyboardTestWorkerSpawner {
    Arc::new(|context| {
        thread::Builder::new()
            .name("key-b0x-keyboard-test-worker".to_string())
            .spawn(move || run_worker(context))
            .map_err(|error| anyhow!("failed to spawn keyboard test worker: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn keyboard_test_lifecycle_is_idempotent_and_shutdown_cleans_up() {
        let start_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let spawner: KeyboardTestWorkerSpawner = Arc::new({
            let start_count = Arc::clone(&start_count);
            move |context| {
                start_count.fetch_add(1, Ordering::Relaxed);
                Ok(thread::spawn(move || {
                    write_keyboard_test_state(
                        &context.state,
                        KeyboardTestState::running(vec![key_b0x_platform::NormalizedKey::KeyA]),
                        context.listener.as_ref(),
                    );

                    while !context.cancel.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(5));
                    }
                }))
            }
        });
        let mut controller = KeyboardTestController::with_spawner(None, spawner);

        let first = controller.start().unwrap();
        let second = controller.start().unwrap();

        assert_eq!(
            first.status,
            crate::keyboard_test::KeyboardTestStatus::Running
        );
        assert_eq!(
            second.status,
            crate::keyboard_test::KeyboardTestStatus::Running
        );
        assert_eq!(start_count.load(Ordering::Relaxed), 1);

        for _ in 0..20 {
            if controller.get_state().pressed_keys == vec![key_b0x_platform::NormalizedKey::KeyA] {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(
            controller.get_state().pressed_keys,
            vec![key_b0x_platform::NormalizedKey::KeyA]
        );
        assert_eq!(
            controller.shutdown().status,
            crate::keyboard_test::KeyboardTestStatus::Idle
        );
        assert_eq!(
            controller.stop().status,
            crate::keyboard_test::KeyboardTestStatus::Idle
        );
    }

    #[test]
    fn keyboard_test_restart_after_worker_error_is_allowed() {
        let start_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let spawner: KeyboardTestWorkerSpawner = Arc::new({
            let start_count = Arc::clone(&start_count);
            move |context| {
                start_count.fetch_add(1, Ordering::Relaxed);
                Ok(thread::spawn(move || {
                    write_keyboard_test_state(
                        &context.state,
                        KeyboardTestState::error("boom"),
                        context.listener.as_ref(),
                    );
                }))
            }
        });
        let mut controller = KeyboardTestController::with_spawner(None, spawner);

        controller.start().unwrap();

        for _ in 0..20 {
            if controller.get_state().status == crate::keyboard_test::KeyboardTestStatus::Error {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(controller.get_state(), KeyboardTestState::error("boom"));

        controller.start().unwrap();
        assert_eq!(start_count.load(Ordering::Relaxed), 2);
    }
}
