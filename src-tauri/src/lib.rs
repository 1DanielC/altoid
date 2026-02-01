use std::fmt::format;
use std::time;
use chrono::Local;
use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::{get_user_info, make_request};
use crate::api::openspace::pub_user_info::UserInfo;
use crate::cache::file_cache::clear_skipped_files;
use crate::cache::user_cache::{clear_user_config, get_host_override, get_user_config, set_host_override};
use crate::error::AppError;
use crate::ipc::pub_ipc_response::ToIpcResponse;
use crate::traits::traits::ToJson;
use serde_json::Value;
use tauri_plugin_log::{Builder, Target, TargetKind};
use tauri_plugin_log::log::{error, info};

mod api;
mod cache;
pub mod camera;
mod error;
mod ipc;
mod traits;

fn err_response(app_error: AppError) -> Value {
    error!("{}", app_error);
    app_error.to_ipc_response().to_json().unwrap()
}

#[tauri::command]
async fn get_user() -> Result<UserInfo, Value> {
    if get_user_config().is_none() {
        authenticate_user()
            .await
            .map_err(|e: AppError| e.to_ipc_response().to_json().unwrap())?;
    }

    get_user_info()
        .await
        .map_err(|e| e.to_ipc_response().to_json().unwrap())?
        .ok_or_else(|| err_response(AppError::NotAuthenticated))
}

#[tauri::command]
async fn clear_cache() -> Result<(), Value> {
    info!("Clearing cache");
    clear_user_config()
        .and_then(|_| clear_skipped_files())
        .map_err(|e: AppError| err_response(e))
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
fn get_host() -> Option<String> {
    get_host_override()
}

#[tauri::command]
fn set_host(host: Option<String>) -> Result<(), Value> {
    set_host_override(host).map_err(|e: AppError| err_response(e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let timestamp = Local::now().format("%Y-%m-%d");
    let filename = format!("log-{}.log", timestamp);
    let logger = Builder::new().targets([
        Target::new(TargetKind::Stdout),
        Target::new(TargetKind::LogDir { file_name: Some(filename) })
    ]).build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(logger)
        .invoke_handler(tauri::generate_handler![
            get_user,
            req,
            get_camera,
            get_camera_files,
            clear_cache,
            get_host,
            set_host,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
