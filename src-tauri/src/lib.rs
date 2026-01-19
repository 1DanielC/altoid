use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::{get_user_info, make_request};
use crate::api::openspace::pub_user_info::UserInfo;
use crate::api::openspace::upload_all_files::{upload_all_files};
use crate::cache::file_cache::clear_skipped_files;
use crate::cache::user_cache::{clear_user_config, get_user_config};
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use tauri::Emitter;
use crate::camera::camera_finder::find_camera;

mod api;
mod cache;
mod camera;

#[tauri::command]
async fn get_user() -> Result<UserInfo, String> {
    if get_user_config().is_none() || get_user_info().await?.is_none() {
        authenticate_user().await?;
    }

    get_user_info().await?.ok_or("User is not logged in. Please log in again".into())
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
    clear_user_config().map_err(|e| e.to_string())?;
    clear_skipped_files().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_camera_files() -> Result<(), String> {
    find_camera().ok_or("No camera found".into())
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
