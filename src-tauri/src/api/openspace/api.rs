use crate::api::http::client::create_http_client;
use crate::api::openspace::pub_user_info::UserInfo;
use crate::cache::user_cache::get_user_config;
use crate::error::AppError;
use crate::ipc::ipc_error::IpcError;
use crate::ipc::pub_ipc_response::IpcStatus;
use reqwest::{Client, Method};
use serde_json::{from_value, Value};
use std::sync::LazyLock;

static USER_AGENT: &str = "ai.openspace.tactic/0.0.1";
static API_CLIENT: LazyLock<Client> = LazyLock::new(|| create_http_client());
static API: LazyLock<Option<OSApi>> = LazyLock::new(|| create_os_api());

struct OSApi {
    api_host: String,
    access_token: String,
    token_type: String,
}

impl OSApi {
    pub fn new(api_host: String, access_token: String, token_type: String) -> Self {
        Self {
            api_host,
            access_token,
            token_type,
        }
    }

    pub async fn request(
        &self,
        method: &str,
        path: &str,
        body: Value,
        content_type: Option<String>,
    ) -> Result<Value, AppError> {
        println!("Requesting {} {}", method, path);
        let url = format!("{}{}", self.api_host, path);
        let method = Method::from_bytes(method.as_bytes())
            .map_err(|e| AppError::InvalidArgument(format!("Invalid HTTP method: {}", e)))?;
        let response = API_CLIENT
            .request(method, &url)
            .header(
                "Authorization",
                format!("{} {}", self.token_type, self.access_token),
            )
            .header("User-Agent", USER_AGENT)
            .header(
                "Content-Type",
                content_type.unwrap_or_else(|| "application/json".into()),
            )
            .body(serde_json::to_vec(&body)?)
            .send()
            .await?;

        let status = response.status();
        let json = response.json::<Value>().await?;
        println!("Response: the sauce");
        if status.as_u16() >= 300 {
            return Err(AppError::ApiRequest {
                status: status.as_u16(),
                message: format!("Request failed: {}", status),
            });
        }

        Ok(json)
    }
}
fn create_os_api() -> Option<OSApi> {
    get_user_config().map(|config| {
        OSApi::new(
            config.api_config.host().to_string(),
            config.access_token,
            config.token_type,
        )
    })
}

pub async fn make_request(
    method: &str,
    path: &str,
    body: Value,
    content_type: Option<String>,
) -> Result<Value, AppError> {
    let res = API
        .as_ref()
        .ok_or(AppError::ApiNotInitialized)?
        .request(method, path, body, content_type)
        .await?;

    println!("{}", serde_json::to_string(&res).unwrap());

    Ok(res)
}
/// Special case function that returns IpcError to handle 401 as Ok(None).
///
/// This is one of the rare cases where we use IpcError instead of AppError,
/// because we need to treat a 401 response as Ok(None) rather than an error,
/// but still need explicit status codes for other errors.
pub async fn get_user_info() -> Result<Option<UserInfo>, IpcError> {
    match make_request("GET", "/api/self", Value::Null, None).await {
        Ok(res) => {
            let user_info = from_value(res).map_err(|e| {
                IpcError::new(
                    IpcStatus::InternalError,
                    format!("Unable to parse user info: {}", e),
                )
            })?;

            Ok(Some(user_info))
        }

        Err(AppError::ApiRequest { status, message }) if status == 401 => {
            // Not authenticated → recoverable, return Ok(None)
            Ok(None)
        }

        Err(e) => {
            // Convert AppError to IpcError
            Err(IpcError::from(e))
        }
    }
}
