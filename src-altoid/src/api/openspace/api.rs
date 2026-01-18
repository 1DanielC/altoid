use crate::api::http::client::create_http_client;
use crate::cache::user_cache::get_user_config;
use reqwest::{Client, Method};
use serde_json::Value;
use std::sync::LazyLock;

static USER_AGENT: &str = "ai.openspace.tactic/0.0.1";
static API_CLIENT: LazyLock<Client> = LazyLock::new(|| create_http_client());
static API: LazyLock<OSApi> = LazyLock::new(|| create_os_api());

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
    ) -> Result<Value, Box<dyn std::error::Error>> {
        println!("Requesting {} {}", method, path);
        let url = format!("{}{}", self.api_host, path);
        let method = Method::from_bytes(method.as_bytes())?;
        let response = API_CLIENT
            .request(method, &url)
            .header(
                "Authorization",
                format!("{} {}", self.token_type, self.access_token),
            )
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", content_type.unwrap_or_else(|| "application/json".into()))
            .body(serde_json::to_vec(&body)?)
            .send()
            .await?;

        let status = response.status();
        let json = response.json::<Value>().await?;
        println!("Response: the sauce");
        if status.as_u16() >= 300 {
            return Err(format!("Request failed: {}", status).into());
        }

        Ok(json)
    }
}

fn create_os_api() -> OSApi {
    let config = get_user_config().expect("Failed to get user config");

    OSApi::new(
        config.api_config.host().to_string(),
        config.access_token,
        config.token_type,
    )
}

pub async fn make_request(
    method: &str,
    path: &str,
    body: Value,
    content_type: Option<String>,
) -> Result<Value, String> {
    let res = API
        .request(method, path, body, content_type)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string(&res).unwrap());

    Ok(res)
}
