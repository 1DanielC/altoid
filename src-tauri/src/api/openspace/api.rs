use crate::api::http::client::create_http_client;
use crate::api::openspace::pub_user_info::UserInfo;
use crate::error::AppError;
use futures_util::StreamExt;
use reqwest::{Body, Client, Method};
use serde_json::{from_value, Value};
use std::sync::LazyLock;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_plugin_log::log::info;
use tokio_util::io::ReaderStream;
use url::Url;
use crate::APP_STATE;

static USER_AGENT: &str = "ai.openspace.altoid/0.0.1";
static API_CLIENT: LazyLock<Client> = LazyLock::new(|| create_http_client());

pub(crate) static FILE_PROGRESS_EVENT: &str = "file-progress";
pub(crate) static API_UPLOADS_PATH: &str = "/api/v3/upcap/desktop-client/uploads";
pub(crate) static API_ACTIVITY_LOG_PATH: &str = "/api/v3/upcap/desktop-client/logs";
static API_CONFIG_PATH: &str = "/api/v3/upcap/desktop-client/config";
static API_SELF_PATH: &str = "/api/self";
pub(crate) static PTP_TEMP_DIR_NAME: &str = "altoid_ptp";

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
/// Resolve the local file path for a camera file.
/// For PTP cameras, downloads via gphoto2 to a temp file first.
/// Returns (local_path, file_size, is_temp) - caller should clean up temp files.
pub async fn resolve_local_file(
    file_path: &str,
    mount_point: &str,
    filename: &str,
    app: &AppHandle,
) -> Result<(std::path::PathBuf, u64, bool), AppError> {
    if mount_point == crate::camera::camera::PTP_MOUNT_POINT {
        let local_path = download_ptp_file_to_disk(file_path, filename, app).await?;
        let size = tokio::fs::metadata(&local_path).await
            .map_err(|e| AppError::internal(&format!("Failed to stat temp file: {}", e)))?
            .len();
        Ok((local_path, size, true))
    } else {
        let full_path = std::path::PathBuf::from(format!("{}/{}", mount_point, file_path));
        let size = tokio::fs::metadata(&full_path).await
            .map_err(|e| AppError::internal(&format!("Failed to stat file {}: {}", full_path.display(), e)))?
            .len();
        Ok((full_path, size, false))
    }
}

/// Stream a file from disk to the server with progress events.
pub async fn upload_file_streaming(
    upload_id: &str,
    local_path: &std::path::Path,
    file_size: u64,
    content_type: &str,
    filename: &str,
    app: &AppHandle,
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
    let path = format!("{}/{}", API_UPLOADS_PATH, upload_id);
    let url = base.join(&path)
        .map_err(|e| AppError::url_parse(format!("Could not build url: {}", path), e))?;

    let content_range = if file_size > 0 {
        format!("bytes 0-{}/{}", file_size - 1, file_size)
    } else {
        "bytes 0-0/0".to_string()
    };

    info!("Streaming upload: {} bytes to {} (Content-Range: {})", file_size, url, content_range);

    // Open file and create a progress-tracking stream
    let file = tokio::fs::File::open(local_path).await
        .map_err(|e| AppError::internal(&format!("Failed to open file: {}", e)))?;

    let reader_stream = ReaderStream::with_capacity(file, 256 * 1024); // 256KB chunks
    let mut bytes_sent: u64 = 0;
    let app_clone = app.clone();
    let filename_owned = filename.to_string();

    let progress_stream = reader_stream.map(move |chunk| {
        if let Ok(ref bytes) = chunk {
            bytes_sent += bytes.len() as u64;
            let _ = app_clone.emit(FILE_PROGRESS_EVENT, serde_json::json!({
                "filename": filename_owned,
                "stage": "uploading",
                "bytes": bytes_sent,
                "total": file_size,
            }));
        }
        chunk
    });

    let body = Body::wrap_stream(progress_stream);

    let response = API_CLIENT
        .put(url)
        .header("Authorization", format!("{} {}", user_config.token_type, user_config.access_token))
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", content_type)
        .header("Content-Range", &content_range)
        .header("Content-Length", file_size)
        .body(body)
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

/// Download a file from the camera via gphoto2 PTP to a temp file on disk.
/// Emits progress events by polling the temp file size during download.
/// Returns the path to the temp file (caller is responsible for cleanup).
async fn download_ptp_file_to_disk(
    camera_path: &str,
    filename: &str,
    app: &AppHandle,
) -> Result<std::path::PathBuf, AppError> {
    info!("Downloading PTP file to disk: {}", camera_path);

    let temp_dir = std::env::temp_dir().join(PTP_TEMP_DIR_NAME);
    tokio::fs::create_dir_all(&temp_dir).await
        .map_err(|e| AppError::internal(&format!("Failed to create temp dir: {}", e)))?;

    let temp_file = temp_dir.join(filename);

    let folder = std::path::Path::new(camera_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    let gphoto_filename = std::path::Path::new(camera_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    for attempt in 0..2 {
        let _ = tokio::fs::remove_file(&temp_file).await;

        let mut child = tokio::process::Command::new(crate::camera::camera::GPHOTO2_CMD)
            .args([
                "--folder", &folder,
                "--get-file", &gphoto_filename,
                "--filename", &temp_file.to_string_lossy(),
                "--force-overwrite",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::internal(&format!("Failed to spawn gphoto2: {}", e)))?;

        // gphoto2 buffers internally, so the temp file grows in bursts.
        // Poll it for whatever progress we can show.
        let temp_file_clone = temp_file.clone();
        let app_clone = app.clone();
        let filename_owned = filename.to_string();
        let poll_handle = tokio::spawn(async move {
            let mut last_size: u64 = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(250)).await;
                if let Ok(meta) = tokio::fs::metadata(&temp_file_clone).await {
                    let size = meta.len();
                    if size != last_size {
                        last_size = size;
                        let _ = app_clone.emit(FILE_PROGRESS_EVENT, serde_json::json!({
                            "filename": filename_owned,
                            "stage": "downloading",
                            "bytes": size,
                            "total": 0,
                        }));
                    }
                }
            }
        });

        let output = child.wait_with_output().await
            .map_err(|e| AppError::internal(&format!("gphoto2 process error: {}", e)))?;

        poll_handle.abort();

        if output.status.success() {
            // Emit final progress with actual file size
            if let Ok(meta) = tokio::fs::metadata(&temp_file).await {
                let _ = app.emit(FILE_PROGRESS_EVENT, serde_json::json!({
                    "filename": filename,
                    "stage": "downloading",
                    "bytes": meta.len(),
                    "total": 0,
                }));
            }
            info!("Downloaded PTP file to {}", temp_file.display());
            return Ok(temp_file);
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
    let url = base.join(API_CONFIG_PATH)
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
    match make_request("GET", API_SELF_PATH, Value::Null, None).await {
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
