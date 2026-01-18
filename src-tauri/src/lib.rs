use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::{get_user_info, make_request};
use crate::api::openspace::pub_user_info::UserInfo;
use crate::api::openspace::upload_all_files;
use crate::api::openspace::upload_all_files::{upload_all_files, FileInfo};
use crate::cache::root_cache::clear_all_cache;
use crate::cache::user_cache::clear_user_config;
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use tauri::Emitter;
use crate::cache::file_cache::clear_skipped_files;

mod api;
mod cache;
mod camera;

#[tauri::command]
async fn get_user() -> Result<UserInfo, String> {
    // Try to get user info first
    if let Ok(ui) = get_user_info().await {
        return Ok(ui);
    }

    authenticate_user().await.map_err(|e| {
        eprintln!("Error authenticating user: {}", e);
        "Unable to authenticate user".to_string()
    })?;

    // Then try to get user info again
    get_user_info().await.map_err(|e| {
        eprintln!("Get user info error: {}", e);
        "Unable to get user info after authentication".to_string()
    })
}

#[tauri::command]
async fn start_upload(app_handle: tauri::AppHandle) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();

    // Spawn blocking upload in background thread
    thread::spawn(move || {
        if let Err(e) = upload_all_files(Some(tx)) {
            eprintln!("Upload failed: {}", e);
        }
    });

    // Forward events from channel to Tauri event system
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.try_recv() {
                Ok(event) => {
                    // Emit event to frontend
                    let _ = app_handle.emit("upload-event", event);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Upload finished
                    break;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn clear_cache() -> Result<(), String> {
    println!("Clearing cache");
    clear_user_cache().map_err(|e| e.to_string())?;
    clear_skipped_files().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn clear_user_cache() -> Result<(), String> {
    clear_user_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_camera_files() -> Result<Vec<FileInfo>, String> {
    upload_all_files::get_uploadable_files().map_err(|e| e.to_string())
}

#[tauri::command]
async fn req(
    method: String,
    path: String,
    body: Value,
    content_type: Option<String>,
) -> Result<Value, String> {
    make_request(&method, &path, body, content_type).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_user,
            start_upload,
            clear_cache,
            clear_user_cache,
            get_camera_files,
            req,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
