use std::fs;
use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::{get_user_info, make_request};
use crate::api::openspace::pub_user_info::UserInfo;
use crate::error::AppError;
use crate::ipc::pub_ipc_response::ToIpcResponse;
use crate::state::{AppState};
use crate::traits::traits::ToJson;
use chrono::Local;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Manager;
use tauri_plugin_log::log::{error, info};
use tauri_plugin_log::{Builder, Target, TargetKind};

mod api;
pub mod camera;
mod error;
mod extensions;
mod ipc;
mod state;
mod traits;

pub static APP_STATE: OnceLock<AppState> = OnceLock::new();

fn err_response(app_error: AppError) -> Value {
    error!("{}", app_error);
    app_error.to_ipc_response().to_json().unwrap()
}

#[tauri::command]
async fn get_user() -> Result<UserInfo, Value> {
    let ui = get_user_info()
        .await
        .map_err(|e| err_response(AppError::from(e)))?;

    if let Some(user_info) = ui {
        return Ok(user_info);
    }

    authenticate_user().await.map_err(|e| err_response(e))?;

    get_user_info()
        .await
        .unwrap()
        .ok_or(err_response(AppError::NotAuthenticated))
}

#[tauri::command]
async fn clear_state() -> Result<(), Value> {
    info!("Clearing Cache...");
    APP_STATE
        .get()
        .ok_or(err_response(AppError::internal("App not initialized")))?
        .clear_state()
        .expect("Something went wrong!");

    Ok(())
}

#[tauri::command]
async fn req(
    method: String,
    path: String,
    body: Value,
    content_type: Option<String>,
) -> Result<Value, Value> {
    make_request(&method, &path, body, content_type)
        .await
        .map_err(|e: AppError| err_response(e))
}

#[tauri::command]
async fn get_camera() -> Result<Value, Value> {
    camera::camera::find_camera()
        .to_json()
        .map_err(|e| err_response(AppError::from(e)))
}

#[tauri::command]
async fn get_camera_files() -> Result<(), Value> {
    Ok(())
}

#[tauri::command]
async fn get_host() -> Result<Option<String>, Value> {
    let state = APP_STATE
        .get()
        .ok_or(err_response(AppError::internal("App not initialized")))?;

    Ok(state.get_host_override())
}

#[tauri::command]
async fn set_host(host: Option<String>) -> Result<(), Value> {
    let state = APP_STATE
        .get()
        .ok_or(err_response(AppError::internal("App not initialized")))?;

    state
        .set_host_override(host)
        .map_err(|e| err_response(e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let timestamp = Local::now().format("%Y-%m-%d");
    let filename = format!("log-{}.log", timestamp);
    let logger = Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some(filename),
            }),
        ])
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(logger)
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_local_data_dir().unwrap();
            info!("Application data directory: {:?}", app_dir);
            fs::create_dir_all(&app_dir)?;
            let app_state = AppState::new(app_dir);
            APP_STATE
                .set(app_state)
                .expect("Could not set application state");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_user,
            req,
            get_camera,
            get_camera_files,
            clear_state,
            get_host,
            set_host,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
