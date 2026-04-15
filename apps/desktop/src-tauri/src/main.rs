#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use key_b0x_app::{
    AppConfig, AppService, InstallProfileResult, RuntimeState, SetupStatus, StateListener,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

struct AppState {
    service: Mutex<AppService>,
}

#[tauri::command]
fn load_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    with_service(&state, |service| service.load_config())
}

#[tauri::command]
fn save_config(state: State<'_, AppState>, config: AppConfig) -> Result<AppConfig, String> {
    with_service(&state, |service| service.save_config(config))
}

#[tauri::command]
fn check_setup(
    state: State<'_, AppState>,
    config: Option<AppConfig>,
) -> Result<SetupStatus, String> {
    with_service(&state, |service| service.check_setup(config))
}

#[tauri::command]
fn install_profile(
    state: State<'_, AppState>,
    slippi_user_path: Option<String>,
) -> Result<InstallProfileResult, String> {
    with_service(&state, move |service| {
        service.install_profile(slippi_user_path.map(PathBuf::from))
    })
}

#[tauri::command]
fn get_runtime_state(state: State<'_, AppState>) -> RuntimeState {
    with_service_state(&state, |service| service.get_runtime_state())
}

#[tauri::command]
fn start_runtime(state: State<'_, AppState>) -> Result<RuntimeState, String> {
    with_service(&state, |service| service.start_runtime())
}

#[tauri::command]
fn stop_runtime(state: State<'_, AppState>) -> RuntimeState {
    with_service_state(&state, |service| service.stop_runtime())
}

#[tauri::command]
fn list_keyboards(state: State<'_, AppState>) -> Result<Vec<key_b0x_app::KeyboardInfo>, String> {
    with_service(&state, |service| service.list_keyboards())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let listener: StateListener = Arc::new(move |runtime_state| {
                let _ = app_handle.emit("runtime://state", runtime_state);
            });
            let service = AppService::new(Some(listener)).map_err(|error| {
                Box::new(std::io::Error::other(error.to_string())) as Box<dyn std::error::Error>
            })?;
            app.manage(AppState {
                service: Mutex::new(service),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            check_setup,
            install_profile,
            get_runtime_state,
            start_runtime,
            stop_runtime,
            list_keyboards
        ])
        .build(tauri::generate_context!())
        .expect("failed to build tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let state: State<'_, AppState> = app_handle.state();
                let mut service = state
                    .service
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = service.shutdown();
            }
        });
}

fn with_service<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&mut AppService) -> anyhow::Result<T>,
) -> Result<T, String> {
    let mut service = state
        .service
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut service).map_err(string_error)
}

fn with_service_state<T>(state: &State<'_, AppState>, f: impl FnOnce(&mut AppService) -> T) -> T {
    let mut service = state
        .service
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut service)
}

fn string_error(error: anyhow::Error) -> String {
    error.to_string()
}
