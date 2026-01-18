use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::make_request;
use crate::api::openspace::upload_all_files;
use crate::api::openspace::upload_all_files::{upload_all_files, FileInfo};
use crate::cache::root_cache::clear_all_cache;
use std::sync::mpsc;
use std::thread;
use serde_json::Value;
use tauri::Emitter;

mod api;
mod cache;
mod camera;

#[tauri::command]
async fn login() -> Result<bool, String> {
    println!("Logging in...");
    // Run OAuth flow and save tokens
    let auth_result = authenticate_user().await.map_err(|e| e.to_string())?;

    Ok(auth_result)
}

// 4. Upload files command (spawns background thread, emits events)
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
    clear_all_cache().map_err(|e| e.to_string())
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
            login,
            start_upload,
            clear_cache,
            get_camera_files,
            req,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
