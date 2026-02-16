use crate::api::oauth::pkg_auth::{
    AuthEnv, DeviceCodeRequest, DeviceCodeResponse, TokenRequest, TokenResponse,
};
use crate::api::openspace::pub_api_env::ApiEnv;
use crate::error::AppError;
use crate::state::{ApiConfig, UserConfig};
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;
use tauri_plugin_log::log::{error, info};
use crate::APP_STATE;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

pub async fn authenticate_user() -> Result<(), AppError> {
    let login_config = APP_STATE.get().unwrap().get_auth_config()
        .ok_or(AppError::auth_failed("Cannot authenticate"))?;

    let auth_url = login_config.env.get_auth_url();
    let token_url = login_config.env.get_token_url();
    let audience = login_config.env.get_audience();

    // Step 1: Request device code from auth server
    info!("Requesting device code...");
    let device_code_request = DeviceCodeRequest {
        client_id: login_config.client_id.clone(),
        scope: login_config.scope.as_ref().to_string(),
        audience: audience.to_string(),
    };

    let device_code_response: DeviceCodeResponse = HTTP_CLIENT
        .post(&auth_url)
        .json(&device_code_request)
        .send()
        .await?
        .json()
        .await
        .map_err(|e| AppError::api_parse_failed("Failed to parse device code response", e))?;

    info!(
        "Device code received. User code: {}",
        device_code_response.user_code
    );

    // Step 2: Open browser with verification_uri_complete
    info!("Opening browser for authentication...");
    if let Err(e) = open::that(&device_code_response.verification_uri_complete) {
        error!("Failed to open browser automatically: {}", e);
        info!(
            "Please manually visit: {}",
            device_code_response.verification_uri_complete
        );
    } else {
        info!("Browser opened. Please complete the authentication in your browser.");
    }

    // Step 3: Poll for token after user authenticates
    info!("Waiting for authentication...");
    let interval = Duration::from_secs(device_code_response.interval);
    let expires_at =
        std::time::Instant::now() + Duration::from_secs(device_code_response.expires_in);

    let token_response = loop {
        if std::time::Instant::now() > expires_at {
            return Err(AppError::DeviceCodeExpired);
        }

        tokio::time::sleep(interval).await;

        let token_request = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            device_code: device_code_response.device_code.clone(),
            client_id: login_config.client_id.clone(),
        };

        let response = HTTP_CLIENT
            .post(&token_url)
            .json(&token_request)
            .send()
            .await?;


        if response.status().is_success() {
            let token_response: TokenResponse = response
                .json()
                .await
                .map_err(|e| AppError::api_parse_failed("Failed to parse token response", e))?;
            break token_response;
        } else {
            // Check for authorization_pending or slow_down errors (expected during polling)
            let status = response.status();
            let error_text = response.text().await;

            match error_text {
                Ok(text) => info!("Token request failed ({}): {}", status, text),
                Err(e) => info!("Failed to read error text: {}", e)
            }
        }
    };

    // Build UserConfig from the token response
    let api_env = match login_config.env {
        AuthEnv::Prod => ApiEnv::US,
        AuthEnv::Dev => ApiEnv::Dev,
    };

    let user_config = UserConfig {
        access_token: token_response.access_token,
        token_type: token_response.token_type,
        api_config: ApiConfig::new(api_env, None),
    };

    APP_STATE.get().unwrap().set_user_config(user_config)?;
    info!("Authentication successful, user config saved");

    Ok(())
}

pub fn get_user_initials(full_name: Option<String>) -> String {
    match full_name {
        Some(name) if !name.trim().is_empty() => {
            let parts: Vec<&str> = name.trim().split_whitespace().collect();
            match parts.len() {
                0 => "OS".to_string(),
                1 => {
                    // Single name, take first char
                    parts[0]
                        .chars()
                        .next()
                        .unwrap_or('O')
                        .to_uppercase()
                        .to_string()
                }
                _ => {
                    // Multiple names, take first char of first and last
                    let first = parts[0].chars().next().unwrap_or('O');
                    let last = parts[parts.len() - 1].chars().next().unwrap_or('S');
                    format!("{}{}", first.to_uppercase(), last.to_uppercase())
                }
            }
        }
        _ => "OS".to_string(),
    }
}
