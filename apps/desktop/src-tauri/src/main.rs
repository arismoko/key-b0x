#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use key_b0x_app::{
    AppConfig, AppService, InstallProfileResult, KeyboardTestState, RuntimeState, SetupStatus,
    StateListener,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

struct AppState {
    service: Mutex<AppService>,
    updater: Mutex<UpdaterStore>,
}

struct UpdaterStore {
    state: UpdateState,
    pending_update: Option<Update>,
    downloaded_bundle: Option<Vec<u8>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum UpdateStatus {
    Idle,
    Checking,
    Available,
    Downloading,
    Downloaded,
    Installing,
    UpToDate,
    Error,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInfo {
    version: String,
    current_version: String,
    notes: Option<String>,
    published_at: Option<String>,
    target: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateState {
    status: UpdateStatus,
    current_version: String,
    latest_version: Option<String>,
    notes: Option<String>,
    published_at: Option<String>,
    target: Option<String>,
    downloaded_bytes: Option<u64>,
    content_length: Option<u64>,
    last_error: Option<String>,
}

impl UpdaterStore {
    fn new(current_version: String) -> Self {
        Self {
            state: UpdateState::idle(current_version),
            pending_update: None,
            downloaded_bundle: None,
        }
    }
}

impl UpdateState {
    fn idle(current_version: String) -> Self {
        Self {
            status: UpdateStatus::Idle,
            current_version,
            latest_version: None,
            notes: None,
            published_at: None,
            target: None,
            downloaded_bytes: None,
            content_length: None,
            last_error: None,
        }
    }

    fn from_info(status: UpdateStatus, info: &UpdateInfo) -> Self {
        Self {
            status,
            current_version: info.current_version.clone(),
            latest_version: Some(info.version.clone()),
            notes: info.notes.clone(),
            published_at: info.published_at.clone(),
            target: Some(info.target.clone()),
            downloaded_bytes: None,
            content_length: None,
            last_error: None,
        }
    }
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
fn get_keyboard_test_state(state: State<'_, AppState>) -> KeyboardTestState {
    with_service_state(&state, |service| service.get_keyboard_test_state())
}

#[tauri::command]
fn start_keyboard_test(state: State<'_, AppState>) -> Result<KeyboardTestState, String> {
    with_service(&state, |service| service.start_keyboard_test())
}

#[tauri::command]
fn stop_keyboard_test(state: State<'_, AppState>) -> KeyboardTestState {
    with_service_state(&state, |service| service.stop_keyboard_test())
}

#[tauri::command]
fn list_keyboards(state: State<'_, AppState>) -> Result<Vec<key_b0x_app::KeyboardInfo>, String> {
    with_service(&state, |service| service.list_keyboards())
}

#[tauri::command]
fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<UpdateInfo>, String> {
    let current_version = app.package_info().version.to_string();

    {
        let mut updater = lock_updater(&state);
        ensure_update_operation_allowed(&updater.state.status)?;
        updater.pending_update = None;
        updater.downloaded_bundle = None;
        updater.state = UpdateState {
            status: UpdateStatus::Checking,
            current_version: current_version.clone(),
            latest_version: None,
            notes: None,
            published_at: None,
            target: None,
            downloaded_bytes: None,
            content_length: None,
            last_error: None,
        };
        emit_updater_state(&app, &updater.state);
    }

    match app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
    {
        Ok(Some(update)) => {
            let info = update_info_from_update(&update);
            let next_state = UpdateState::from_info(UpdateStatus::Available, &info);
            let mut updater = lock_updater(&state);
            updater.pending_update = Some(update);
            updater.downloaded_bundle = None;
            updater.state = next_state.clone();
            emit_updater_state(&app, &next_state);
            Ok(Some(info))
        }
        Ok(None) => {
            let next_state = UpdateState {
                status: UpdateStatus::UpToDate,
                current_version,
                latest_version: None,
                notes: None,
                published_at: None,
                target: None,
                downloaded_bytes: None,
                content_length: None,
                last_error: None,
            };
            let mut updater = lock_updater(&state);
            updater.pending_update = None;
            updater.downloaded_bundle = None;
            updater.state = next_state.clone();
            emit_updater_state(&app, &next_state);
            Ok(None)
        }
        Err(error) => {
            let message = error.to_string();
            mark_update_error(&app, &state, message.clone());
            Err(message)
        }
    }
}

#[tauri::command]
async fn download_update(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let update = {
        let mut updater = lock_updater(&state);
        ensure_download_allowed(&updater)?;
        updater.state.status = UpdateStatus::Downloading;
        updater.state.downloaded_bytes = Some(0);
        updater.state.content_length = None;
        updater.state.last_error = None;
        emit_updater_state(&app, &updater.state);
        updater
            .pending_update
            .clone()
            .ok_or_else(|| "There is no pending update.".to_string())?
    };

    let mut downloaded = 0_u64;
    match update
        .download(
            |chunk_length, content_length| {
                downloaded += chunk_length as u64;
                let mut updater = lock_updater(&state);
                updater.state.status = UpdateStatus::Downloading;
                updater.state.downloaded_bytes = Some(downloaded);
                updater.state.content_length = content_length;
                updater.state.last_error = None;
                emit_updater_state(&app, &updater.state);
            },
            || {},
        )
        .await
    {
        Ok(bundle) => {
            let info = update_info_from_update(&update);
            let mut next_state = UpdateState::from_info(UpdateStatus::Downloaded, &info);
            let bundle_length = bundle.len() as u64;
            next_state.downloaded_bytes = Some(bundle_length);
            next_state.content_length = Some(bundle_length);

            let mut updater = lock_updater(&state);
            updater.pending_update = Some(update);
            updater.downloaded_bundle = Some(bundle);
            updater.state = next_state.clone();
            emit_updater_state(&app, &next_state);
            Ok(())
        }
        Err(error) => {
            let message = error.to_string();
            {
                let mut updater = lock_updater(&state);
                updater.pending_update = Some(update);
                updater.downloaded_bundle = None;
            }
            mark_update_error(&app, &state, message.clone());
            Err(message)
        }
    }
}

#[tauri::command]
async fn install_update(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut service = state
            .service
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        service.stop_keyboard_test();
        service.stop_runtime();
    }

    let (update, bundle) = {
        let mut updater = lock_updater(&state);
        ensure_install_allowed(&updater)?;
        updater.state.status = UpdateStatus::Installing;
        updater.state.last_error = None;
        emit_updater_state(&app, &updater.state);
        let update = updater
            .pending_update
            .clone()
            .ok_or_else(|| "There is no downloaded update ready to install.".to_string())?;
        let bundle = updater
            .downloaded_bundle
            .take()
            .ok_or_else(|| "The pending update has not been downloaded yet.".to_string())?;
        (update, bundle)
    };

    if let Err(error) = update.install(&bundle) {
        let message = friendly_install_error(error.to_string());
        let mut updater = lock_updater(&state);
        updater.pending_update = Some(update);
        updater.downloaded_bundle = Some(bundle);
        updater.state.status = UpdateStatus::Error;
        updater.state.last_error = Some(message.clone());
        emit_updater_state(&app, &updater.state);
        return Err(message);
    }

    #[cfg(target_os = "linux")]
    {
        app.restart();
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let listener: StateListener = Arc::new(move |runtime_state| {
                let _ = app_handle.emit("runtime://state", runtime_state);
            });
            let keyboard_test_handle = app.handle().clone();
            let keyboard_test_listener = Arc::new(move |keyboard_test_state| {
                let _ = keyboard_test_handle.emit("keyboard-test://state", keyboard_test_state);
            });
            let service =
                AppService::new_with_listeners(Some(listener), Some(keyboard_test_listener))
                    .map_err(|error| {
                        Box::new(std::io::Error::other(error.to_string()))
                            as Box<dyn std::error::Error>
                    })?;

            app.manage(AppState {
                service: Mutex::new(service),
                updater: Mutex::new(UpdaterStore::new(app.package_info().version.to_string())),
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
            get_keyboard_test_state,
            start_keyboard_test,
            stop_keyboard_test,
            list_keyboards,
            get_app_version,
            check_for_update,
            download_update,
            install_update
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

fn update_info_from_update(update: &Update) -> UpdateInfo {
    UpdateInfo {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone(),
        published_at: update.date.map(|date| date.to_string()),
        target: update.target.clone(),
    }
}

fn emit_updater_state(app: &AppHandle, state: &UpdateState) {
    let _ = app.emit("updater://state", state);
}

fn lock_updater<'a>(state: &'a State<'_, AppState>) -> std::sync::MutexGuard<'a, UpdaterStore> {
    state
        .updater
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn ensure_update_operation_allowed(status: &UpdateStatus) -> Result<(), String> {
    if matches!(
        status,
        UpdateStatus::Checking | UpdateStatus::Downloading | UpdateStatus::Installing
    ) {
        return Err("An update operation is already in progress.".to_string());
    }

    Ok(())
}

fn ensure_download_allowed(updater: &UpdaterStore) -> Result<(), String> {
    ensure_update_operation_allowed(&updater.state.status)?;

    if updater.downloaded_bundle.is_some() {
        return Err("The pending update is already downloaded.".to_string());
    }

    if updater.pending_update.is_none() {
        return Err("There is no pending update to download.".to_string());
    }

    Ok(())
}

fn ensure_install_allowed(updater: &UpdaterStore) -> Result<(), String> {
    ensure_update_operation_allowed(&updater.state.status)?;

    if updater.pending_update.is_none() {
        return Err("There is no downloaded update ready to install.".to_string());
    }

    if updater.downloaded_bundle.is_none() {
        return Err("The pending update has not been downloaded yet.".to_string());
    }

    Ok(())
}

fn mark_update_error(app: &AppHandle, state: &State<'_, AppState>, message: String) {
    let mut updater = lock_updater(state);
    updater.state.status = UpdateStatus::Error;
    updater.state.last_error = Some(message);
    updater.state.downloaded_bytes = None;
    updater.state.content_length = None;
    emit_updater_state(app, &updater.state);
}

fn friendly_install_error(message: String) -> String {
    #[cfg(target_os = "linux")]
    {
        let normalized = message.to_ascii_lowercase();
        if normalized.contains("permission denied")
            || normalized.contains("operation not permitted")
            || normalized.contains("read-only file system")
        {
            return "Move key-b0x.AppImage to a writable location such as ~/Applications/key-b0x.AppImage and try again.".to_string();
        }
    }

    message
}

fn string_error(error: anyhow::Error) -> String {
    error.to_string()
}
