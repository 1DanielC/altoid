use std::fs;
use crate::api::oauth::auth::authenticate_user;
use crate::api::openspace::api::{fetch_bootstrap_config, get_user_info, make_request, upload_file_bytes};
use crate::state::OAuthConfig;
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
        .map_err(|e| err_response(e))?;

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
    // Step 1: Quick USB scan with 5s timeout
    let detect = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::task::spawn_blocking(|| camera::camera::detect_camera()),
    )
    .await;

    let detected = match detect {
        Ok(Ok(Some(cam))) => cam,
        _ => return Ok(serde_json::json!({ "message": "No camera found" })),
    };

    let device_id = detected.device_id.clone();

    // Step 2: List files (no timeout - can be slow for PTP/mass storage)
    let result = tokio::task::spawn_blocking(move || camera::camera::find_camera())
        .await
        .map_err(|e| err_response(AppError::internal(&format!("File listing failed: {}", e))))?;

    match result {
        Some(cam) => cam.to_json().map_err(|e| err_response(AppError::from(e))),
        None => Ok(serde_json::json!({
            "message": "Camera detected but could not list files",
            "device_id": device_id,
        })),
    }
}

#[tauri::command]
async fn create_uploads(device_id: String, files: Vec<Value>) -> Result<Value, Value> {
    info!("Creating uploads for {} files on device {}", files.len(), device_id);

    let mut results = Vec::new();

    for file in &files {
        let filename = file["filename"].as_str().unwrap_or("unknown");
        let content_type = file["content_type"].as_str().unwrap_or("application/octet-stream");
        let size = file["size"].as_i64().unwrap_or(0);

        let body = serde_json::json!({
            "deviceId": device_id,
            "deviceFilename": filename,
            "contentType": content_type,
            "size": size,
        });

        match make_request("POST", "/api/desktop-client/uploads", body, None).await {
            Ok(response) => {
                info!("Created upload for {}: {}", filename, response);
                results.push(serde_json::json!({
                    "filename": filename,
                    "response": response,
                }));
            }
            Err(e) => {
                error!("Failed to create upload for {}: {}", filename, e);
                results.push(serde_json::json!({
                    "filename": filename,
                    "error": format!("{}", e),
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "total": files.len(),
        "results": results,
    }))
}

#[tauri::command]
async fn upload_file(
    upload_id: String,
    file_path: String,
    content_type: String,
) -> Result<(), Value> {
    info!("Uploading file {} for upload {}", file_path, upload_id);
    upload_file_bytes(&upload_id, &file_path, &content_type)
        .await
        .map_err(|e| err_response(e))
}

#[tauri::command]
async fn load_config() -> Result<Value, Value> {
    let state = APP_STATE
        .get()
        .ok_or(err_response(AppError::internal("App not initialized")))?;

    // If we already have oauth_config, return it
    if let Some(config) = state.get_auth_config() {
        info!("OAuth config already exists, skipping bootstrap");
        return config
            .to_json()
            .map_err(|e| err_response(AppError::from(e)));
    }

    // Fetch from remote
    info!("No OAuth config found, fetching bootstrap config...");
    let response = fetch_bootstrap_config()
        .await
        .map_err(|e| err_response(e))?;

    // Parse and save
    info!("Bootstrap config response: {}", serde_json::to_string(&response).unwrap_or_default());

    // The API returns camelCase keys and PascalCase enum values,
    // so we translate to our internal format.
    let oauth_config = OAuthConfig {
        client_id: response["clientId"]
            .as_str()
            .ok_or_else(|| err_response(AppError::internal("Bootstrap config missing clientId")))?
            .to_string(),
        env: serde_json::from_value(response["env"].clone())
            .map_err(|e| err_response(AppError::internal_with(
                format!("Invalid env in bootstrap config: {}", response["env"]),
                e,
            )))?,
        scope: match response["scope"].as_str().unwrap_or("Email") {
            "OpenId" | "openid" | "Openid" => crate::api::oauth::pkg_auth::AuthScope::Openid,
            "OfflineAccess" | "offline_access" => crate::api::oauth::pkg_auth::AuthScope::OfflineAccess,
            _ => crate::api::oauth::pkg_auth::AuthScope::Email,
        },
    };

    state
        .set_auth_config(oauth_config)
        .map_err(|e| err_response(e))?;

    info!("Bootstrap config saved successfully");
    Ok(response)
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
            clear_state,
            load_config,
            create_uploads,
            upload_file,
            get_host,
            set_host,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
