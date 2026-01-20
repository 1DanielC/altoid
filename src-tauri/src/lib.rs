use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::{get_user_info, make_request};
use crate::api::openspace::pub_user_info::UserInfo;
use crate::cache::file_cache::clear_skipped_files;
use crate::cache::user_cache::{clear_user_config, get_user_config};
use crate::error::AppError;
use crate::ipc::pub_ipc_response::ToIpcResponse;
use crate::traits::traits::ToJson;
use serde_json::Value;

mod api;
mod cache;
pub mod camera;
mod error;
mod ipc;
mod traits;

fn err_response(app_error: AppError) -> Value {
    eprintln!("{}", app_error);
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
    println!("Clearing cache");
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_user,
            req,
            get_camera,
            get_camera_files,
            clear_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
