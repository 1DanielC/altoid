use crate::api::http::client::create_http_client;
use crate::api::openspace::pub_user_info::UserInfo;
use crate::error::AppError;
use reqwest::{Client, Method};
use serde_json::{from_value, Value};
use std::sync::LazyLock;
use std::time::Duration;
use tauri_plugin_log::log::info;
use url::Url;
use crate::APP_STATE;

static USER_AGENT: &str = "ai.openspace.tactic/0.0.1";
static API_CLIENT: LazyLock<Client> = LazyLock::new(|| create_http_client());

pub async fn make_request(
    method: &str,
    path: &str,
    body: Value,
    content_type: Option<String>,
) -> Result<Value, AppError> {
    info!("Requesting {} {}", method, path);
    let method = Method::from_bytes(method.as_bytes())
        .map_err(|e| AppError::invalid_arg_with(format!("Invalid HTTP method: {}", method), e))?;

    let state = APP_STATE
        .get()
        .ok_or(AppError::internal("App State not configured"))?;

    let user_config = state
        .get_user_config()
        .ok_or(AppError::ApiRequest { status: 401, message: "Not authenticated".into() })?;

    let override_host = state.get_host_override();
    let config_host = user_config.api_config.host().to_string();
    let host = override_host.clone().unwrap_or(config_host.clone());
    info!("Host override: {:?}, config host: {}, using: {}", override_host, config_host, host);

    let base = Url::parse(&host)
        .map_err(|e| AppError::url_parse(format!("Invalid Host: {}", host), e))?;
    let url = base.join(path)
        .map_err(|e| AppError::url_parse(format!("Could not build url: {}", path), e))?;

    let response = API_CLIENT
        .request(method, url)
        .timeout(Duration::from_secs(30))
        .header("Authorization", format!("{} {}", user_config.token_type, user_config.access_token))
        .header("User-Agent", USER_AGENT)
        .header(
            "Content-Type",
            content_type.unwrap_or_else(|| "application/json".into()),
        )
        .body(serde_json::to_vec(&body)?)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    info!("Response status: {}, body length: {}", status, body_text.len());

    if status.as_u16() >= 300 {
        return Err(AppError::ApiRequest {
            status: status.as_u16(),
            message: format!("Request failed ({}): {}", status, body_text.chars().take(200).collect::<String>()),
        });
    }

    let json: Value = serde_json::from_str(&body_text)
        .map_err(|e| AppError::internal_with(
            format!("Failed to parse response as JSON. Body preview: {}", body_text.chars().take(100).collect::<String>()),
            e
        ))?;

    info!("{}", serde_json::to_string(&json).unwrap());

    Ok(json)
}
/// Read file bytes - handles both local filesystem and PTP cameras.
pub async fn read_file_bytes(file_path: &str, mount_point: &str) -> Result<Vec<u8>, AppError> {
    if mount_point == "PTP" {
        download_ptp_file(file_path).await
    } else {
        let full_path = format!("{}/{}", mount_point, file_path);
        tokio::fs::read(&full_path).await
            .map_err(|e| AppError::internal(&format!("Failed to read file {}: {}", full_path, e)))
    }
}

/// Upload raw bytes to an existing upload via PUT /api/desktop-client/uploads/{uploadId}.
pub async fn upload_bytes_to_server(
    upload_id: &str,
    content_type: &str,
    file_bytes: Vec<u8>,
) -> Result<(), AppError> {
    let state = APP_STATE
        .get()
        .ok_or(AppError::internal("App State not configured"))?;

    let user_config = state
        .get_user_config()
        .ok_or(AppError::ApiRequest { status: 401, message: "Not authenticated".into() })?;

    let host = state
        .get_host_override()
        .unwrap_or_else(|| user_config.api_config.host().to_string());

    let base = Url::parse(&host)
        .map_err(|e| AppError::url_parse(format!("Invalid Host: {}", host), e))?;
    let path = format!("/api/desktop-client/uploads/{}", upload_id);
    let url = base.join(&path)
        .map_err(|e| AppError::url_parse(format!("Could not build url: {}", path), e))?;

    let file_size = file_bytes.len();
    let content_range = if file_size > 0 {
        format!("bytes 0-{}/{}", file_size - 1, file_size)
    } else {
        "bytes 0-0/0".to_string()
    };

    info!("Uploading {} bytes to {} (Content-Range: {})", file_size, url, content_range);

    let response = API_CLIENT
        .put(url)
        .timeout(Duration::from_secs(300))
        .header("Authorization", format!("{} {}", user_config.token_type, user_config.access_token))
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", content_type)
        .header("Content-Range", content_range)
        .body(file_bytes)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    info!("Upload response: status={}, body length={}", status, body_text.len());

    if status.as_u16() >= 300 {
        return Err(AppError::ApiRequest {
            status: status.as_u16(),
            message: format!("Upload failed ({}): {}", status, body_text.chars().take(200).collect::<String>()),
        });
    }

    Ok(())
}

/// Kill the macOS PTPCamera process that grabs the USB device.
async fn kill_ptpcamera() {
    let _ = tokio::process::Command::new("killall")
        .args(["PTPCamera"])
        .output()
        .await;
    // Give the OS a moment to release the device
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

/// Download a file from the camera via gphoto2 PTP, returning the bytes.
/// Retries once after killing PTPCamera if USB claim fails.
async fn download_ptp_file(camera_path: &str) -> Result<Vec<u8>, AppError> {
    info!("Downloading PTP file: {}", camera_path);

    let temp_dir = std::env::temp_dir().join("altoid_ptp");
    tokio::fs::create_dir_all(&temp_dir).await
        .map_err(|e| AppError::internal(&format!("Failed to create temp dir: {}", e)))?;

    let filename = std::path::Path::new(camera_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let temp_file = temp_dir.join(&filename);

    let folder = std::path::Path::new(camera_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    // Try up to 2 times: first attempt, then retry after killing PTPCamera
    for attempt in 0..2 {
        let output = tokio::process::Command::new("gphoto2")
            .args([
                "--folder", &folder,
                "--get-file", &filename,
                "--filename", &temp_file.to_string_lossy(),
                "--force-overwrite",
            ])
            .output()
            .await
            .map_err(|e| AppError::internal(&format!("Failed to run gphoto2: {}", e)))?;

        if output.status.success() {
            info!("Downloaded PTP file to {}", temp_file.display());

            let bytes = tokio::fs::read(&temp_file).await
                .map_err(|e| AppError::internal(&format!("Failed to read temp file: {}", e)))?;

            let _ = tokio::fs::remove_file(&temp_file).await;
            return Ok(bytes);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);

        if attempt == 0 && stderr.contains("Could not claim") {
            info!("USB claim failed, killing PTPCamera and retrying...");
            kill_ptpcamera().await;
            continue;
        }

        return Err(AppError::internal(&format!(
            "gphoto2 download failed for {}: {}",
            camera_path, stderr
        )));
    }

    Err(AppError::internal(&format!("gphoto2 download failed for {} after retries", camera_path)))
}

const DEFAULT_HOST: &str = "http://localhost:8080";

/// Fetch bootstrap config from an unauthenticated endpoint.
/// Uses host_override if set, otherwise falls back to the default host.
pub async fn fetch_bootstrap_config() -> Result<Value, AppError> {
    let state = APP_STATE
        .get()
        .ok_or(AppError::internal("App State not configured"))?;

    let host = state
        .get_host_override()
        .unwrap_or_else(|| DEFAULT_HOST.to_string());

    info!("Fetching bootstrap config from {}", host);

    let base = Url::parse(&host)
        .map_err(|e| AppError::url_parse(format!("Invalid Host: {}", host), e))?;
    let url = base.join("/api/desktop-client/config")
        .map_err(|e| AppError::url_parse("Could not build bootstrap URL".to_string(), e))?;

    let response = API_CLIENT
        .get(url)
        .timeout(Duration::from_secs(10))
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    info!("Bootstrap config response: status={}, body length={}", status, body_text.len());

    if status.as_u16() >= 300 {
        return Err(AppError::ApiRequest {
            status: status.as_u16(),
            message: format!("Bootstrap config request failed ({}): {}", status, body_text.chars().take(200).collect::<String>()),
        });
    }

    let json: Value = serde_json::from_str(&body_text)
        .map_err(|e| AppError::internal_with(
            format!("Failed to parse bootstrap config: {}", body_text.chars().take(100).collect::<String>()),
            e
        ))?;

    Ok(json)
}

/// Special case function that returns IpcError to handle 401 as Ok(None).
///
/// This is one of the rare cases where we use IpcError instead of AppError,
/// because we need to treat a 401 response as Ok(None) rather than an error,
/// but still need explicit status codes for other errors.
pub async fn get_user_info() -> Result<Option<UserInfo>, AppError> {
    match make_request("GET", "/api/self", Value::Null, None).await {
        Ok(res) => {
            let user_info = from_value(res)?;

            Ok(Some(user_info))
        }
        
        Err(AppError::ApiRequest { status, message }) if status == 401 => {
            // Not authenticated → recoverable, return Ok(None)
            Ok(None)
        }

        Err(e) => {
            Err(e)
        }
    }
}
