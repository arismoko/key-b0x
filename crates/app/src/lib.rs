mod bindings;
mod config;
mod platform;
mod profile;
mod runtime_controller;
mod runtime_loop;
mod setup;
mod state;
mod transport;

use anyhow::Result;
use key_b0x_platform::KeyboardBackend;
use std::path::PathBuf;

pub use config::{AppConfig, CONFIG_VERSION, default_config_path, render_default_config};
pub use key_b0x_platform::KeyboardInfo;
pub use profile::{InstallProfileResult, PIPE_TARGET_LABEL, PROFILE_FILE_NAME};
pub use runtime_controller::{RuntimeWorkerContext, RuntimeWorkerSpawner};
pub use setup::SetupStatus;
pub use state::{RuntimeState, RuntimeStatus, StateListener};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub config_path: PathBuf,
}

impl AppPaths {
    pub fn from_default_location() -> Result<Self> {
        Ok(Self {
            config_path: default_config_path()?,
        })
    }
}

pub struct AppService {
    paths: AppPaths,
    runtime_controller: runtime_controller::RuntimeController,
}

impl AppService {
    pub fn new(listener: Option<StateListener>) -> Result<Self> {
        let paths = AppPaths::from_default_location()?;
        Ok(Self::with_paths(paths, listener))
    }

    pub fn with_paths(paths: AppPaths, listener: Option<StateListener>) -> Self {
        Self {
            runtime_controller: runtime_controller::RuntimeController::new(
                paths.config_path.clone(),
                listener,
            ),
            paths,
        }
    }

    pub fn with_paths_and_spawner(
        paths: AppPaths,
        listener: Option<StateListener>,
        worker_spawner: RuntimeWorkerSpawner,
    ) -> Self {
        Self {
            runtime_controller: runtime_controller::RuntimeController::with_spawner(
                paths.config_path.clone(),
                listener,
                worker_spawner,
            ),
            paths,
        }
    }

    pub fn load_config(&self) -> Result<AppConfig> {
        config::load_or_create(&self.paths.config_path)
    }

    pub fn save_config(&self, config: AppConfig) -> Result<AppConfig> {
        let normalized = config::prepare(config)?;
        config::save(&self.paths.config_path, &normalized)?;
        Ok(normalized)
    }

    pub fn check_setup(&self, config: Option<AppConfig>) -> Result<SetupStatus> {
        let resolved = match config {
            Some(config) => config::prepare(config)?,
            None => self.load_config()?,
        };

        Ok(setup::check_setup(&resolved))
    }

    pub fn install_profile(
        &self,
        slippi_user_path: Option<PathBuf>,
    ) -> Result<InstallProfileResult> {
        let slippi_user_path = match slippi_user_path {
            Some(path) => path,
            None => self.load_config()?.slippi_user_path,
        };

        profile::install_profile(&slippi_user_path)
    }

    pub fn get_runtime_state(&mut self) -> RuntimeState {
        self.runtime_controller.get_state()
    }

    pub fn start_runtime(&mut self) -> Result<RuntimeState> {
        self.runtime_controller.start()
    }

    pub fn stop_runtime(&mut self) -> RuntimeState {
        self.runtime_controller.stop()
    }

    pub fn shutdown(&mut self) -> RuntimeState {
        self.runtime_controller.shutdown()
    }

    pub fn list_keyboards(&self) -> Result<Vec<KeyboardInfo>> {
        let backend = platform::active_keyboard_backend();
        backend.list_keyboards()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{
        RuntimeState, RuntimeStatus, StateListener, now_timestamp_ms, read_runtime_state,
        write_runtime_state,
    };
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    fn temp_paths() -> AppPaths {
        let temp = tempfile::tempdir().unwrap();
        AppPaths {
            config_path: temp.keep().join("config.toml"),
        }
    }

    #[test]
    fn load_config_creates_default_file() {
        let service = AppService::with_paths(temp_paths(), None);

        let config = service.load_config().unwrap();

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.melee, key_b0x_core::MeleeConfig::default());
        assert!(service.paths.config_path.exists());
    }

    #[test]
    fn check_setup_uses_passed_config_without_touching_disk() {
        let paths = temp_paths();
        let service = AppService::with_paths(paths.clone(), None);
        let config = AppConfig {
            slippi_user_path: std::path::PathBuf::from("/tmp/key-b0x-missing-slippi"),
            ..AppConfig::default()
        };

        let setup = service.check_setup(Some(config)).unwrap();

        assert!(!setup.slippi_found);
        assert!(!paths.config_path.exists());
    }

    #[test]
    fn install_profile_uses_saved_config_when_path_is_omitted() {
        let paths = temp_paths();
        let service = AppService::with_paths(paths, None);
        let slippi_root = tempfile::tempdir().unwrap();
        let config = AppConfig {
            slippi_user_path: slippi_root.path().to_path_buf(),
            ..AppConfig::default()
        };
        service.save_config(config).unwrap();

        let installed = service.install_profile(None).unwrap();

        assert!(installed.profile_path.exists());
    }

    #[test]
    fn runtime_lifecycle_is_idempotent_and_shutdown_cleans_up() {
        let paths = temp_paths();
        let start_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let event_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let listener: StateListener = Arc::new({
            let event_count = Arc::clone(&event_count);
            move |_| {
                event_count.fetch_add(1, Ordering::Relaxed);
            }
        });
        let spawner: RuntimeWorkerSpawner = Arc::new({
            let start_count = Arc::clone(&start_count);
            move |context| {
                start_count.fetch_add(1, Ordering::Relaxed);
                Ok(thread::spawn(move || {
                    let started_at = read_runtime_state(&context.state).started_at;
                    write_runtime_state(
                        &context.state,
                        RuntimeState {
                            status: RuntimeStatus::Running,
                            started_at,
                            last_error: None,
                        },
                        context.listener.as_ref(),
                    );

                    while !context.cancel.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(5));
                    }
                }))
            }
        });
        let mut service = AppService::with_paths_and_spawner(paths, Some(listener), spawner);

        let first = service.start_runtime().unwrap();
        let second = service.start_runtime().unwrap();

        assert_eq!(first.status, RuntimeStatus::Starting);
        assert!(matches!(
            second.status,
            RuntimeStatus::Starting | RuntimeStatus::Running
        ));
        assert_eq!(start_count.load(Ordering::Relaxed), 1);

        for _ in 0..20 {
            if service.get_runtime_state().status == RuntimeStatus::Running {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(service.get_runtime_state().status, RuntimeStatus::Running);
        assert_eq!(service.shutdown().status, RuntimeStatus::Idle);
        assert_eq!(service.stop_runtime().status, RuntimeStatus::Idle);
        assert_eq!(start_count.load(Ordering::Relaxed), 1);
        assert!(event_count.load(Ordering::Relaxed) >= 2);
    }

    #[test]
    fn runtime_restart_after_worker_error_is_allowed() {
        let paths = temp_paths();
        let start_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let spawner: RuntimeWorkerSpawner = Arc::new({
            let start_count = Arc::clone(&start_count);
            move |context| {
                start_count.fetch_add(1, Ordering::Relaxed);
                Ok(thread::spawn(move || {
                    let _ = now_timestamp_ms();
                    write_runtime_state(
                        &context.state,
                        RuntimeState::error("boom"),
                        context.listener.as_ref(),
                    );
                }))
            }
        });
        let mut service = AppService::with_paths_and_spawner(paths, None, spawner);

        service.start_runtime().unwrap();
        for _ in 0..20 {
            if service.get_runtime_state().status == RuntimeStatus::Error {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(service.get_runtime_state().status, RuntimeStatus::Error);

        service.start_runtime().unwrap();

        assert_eq!(start_count.load(Ordering::Relaxed), 2);
    }
}
