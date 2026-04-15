use crate::runtime_loop::run_worker;
use crate::state::{
    RuntimeState, RuntimeStatus, SharedRuntimeState, StateListener, now_timestamp_ms,
    read_runtime_state, write_runtime_state,
};
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

#[derive(Clone)]
pub struct RuntimeWorkerContext {
    pub config_path: PathBuf,
    pub cancel: Arc<AtomicBool>,
    pub state: SharedRuntimeState,
    pub listener: Option<StateListener>,
}

pub type RuntimeWorkerSpawner =
    Arc<dyn Fn(RuntimeWorkerContext) -> Result<JoinHandle<()>> + Send + Sync + 'static>;

struct WorkerHandle {
    cancel: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

pub struct RuntimeController {
    config_path: PathBuf,
    state: SharedRuntimeState,
    listener: Option<StateListener>,
    worker: Option<WorkerHandle>,
    worker_spawner: RuntimeWorkerSpawner,
}

impl RuntimeController {
    pub fn new(config_path: PathBuf, listener: Option<StateListener>) -> Self {
        Self::with_spawner(config_path, listener, default_worker_spawner())
    }

    pub fn with_spawner(
        config_path: PathBuf,
        listener: Option<StateListener>,
        worker_spawner: RuntimeWorkerSpawner,
    ) -> Self {
        Self {
            config_path,
            state: Default::default(),
            listener,
            worker: None,
            worker_spawner,
        }
    }

    pub fn get_state(&mut self) -> RuntimeState {
        self.refresh_worker();
        read_runtime_state(&self.state)
    }

    pub fn start(&mut self) -> Result<RuntimeState> {
        self.refresh_worker();
        if self.worker.is_some() {
            return Ok(read_runtime_state(&self.state));
        }

        write_runtime_state(
            &self.state,
            RuntimeState::starting(now_timestamp_ms()),
            self.listener.as_ref(),
        );

        let cancel = Arc::new(AtomicBool::new(false));
        let join = match (self.worker_spawner)(RuntimeWorkerContext {
            config_path: self.config_path.clone(),
            cancel: Arc::clone(&cancel),
            state: Arc::clone(&self.state),
            listener: self.listener.clone(),
        }) {
            Ok(join) => join,
            Err(error) => {
                write_runtime_state(
                    &self.state,
                    RuntimeState::error(error.to_string()),
                    self.listener.as_ref(),
                );
                return Err(error);
            }
        };

        self.worker = Some(WorkerHandle { cancel, join });
        Ok(read_runtime_state(&self.state))
    }

    pub fn stop(&mut self) -> RuntimeState {
        self.refresh_worker();
        let Some(worker) = self.worker.take() else {
            write_runtime_state(&self.state, RuntimeState::idle(), self.listener.as_ref());
            return read_runtime_state(&self.state);
        };

        let current = read_runtime_state(&self.state);
        write_runtime_state(
            &self.state,
            RuntimeState {
                status: RuntimeStatus::Stopping,
                started_at: current.started_at,
                last_error: current.last_error,
            },
            self.listener.as_ref(),
        );
        worker.cancel.store(true, Ordering::Relaxed);

        if worker.join.join().is_err() {
            write_runtime_state(
                &self.state,
                RuntimeState::error("runtime worker panicked"),
                self.listener.as_ref(),
            );
            return read_runtime_state(&self.state);
        }

        write_runtime_state(&self.state, RuntimeState::idle(), self.listener.as_ref());
        read_runtime_state(&self.state)
    }

    pub fn shutdown(&mut self) -> RuntimeState {
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
            write_runtime_state(
                &self.state,
                RuntimeState::error("runtime worker panicked"),
                self.listener.as_ref(),
            );
        }
    }
}

impl Drop for RuntimeController {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn default_worker_spawner() -> RuntimeWorkerSpawner {
    Arc::new(|context| {
        thread::Builder::new()
            .name("key-b0x-runtime-worker".to_string())
            .spawn(move || run_worker(context))
            .map_err(|error| anyhow!("failed to spawn runtime worker: {error}"))
    })
}
