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

struct OSApi {
    api_host: String,
    access_token: String,
    token_type: String,
}

impl OSApi {
    fn new(api_host: String, access_token: String, token_type: String) -> Self {
        Self {
            api_host,
            access_token,
            token_type,
        }
    }
}

pub async fn make_request(
    method: &str,
    path: &str,
    body: Value,
    content_type: Option<String>,
) -> Result<Value, AppError> {
    info!("Requesting {} {}", method, path);
    let method = Method::from_bytes(method.as_bytes())
        .map_err(|e| AppError::invalid_arg_with(format!("Invalid HTTP method: {}", method), e))?;

    let user_config = APP_STATE
        .get()
        .ok_or(AppError::internal("App State not configured"))?
        .get_user_config()
        .ok_or(AppError::ApiRequest { status: 401, message: "Not authenticated".into() })?;

    let host = user_config.api_config.host();

    let base = Url::parse(host)
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
