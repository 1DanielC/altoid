use crate::api::oauth::pkg_auth::{DeviceCodeRequest, DeviceCodeResponse, TokenRequest, TokenResponse};
use crate::cache::{oauth_cache};
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;
use crate::cache::pub_user_config::UserConfig;
use crate::cache::user_cache::{get_user_config, save_user_config};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

pub async fn authenticate_user() -> Result<UserConfig, Box<dyn std::error::Error>> {
    let login_config = oauth_cache::get_oauth_config()?;
    let auth_url = login_config.env.get_auth_url();
    let token_url = login_config.env.get_token_url();
    let audience = login_config.env.get_audience();

    // Step 1: Request device code from auth server
    println!("Requesting device code...");
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
        .await?;

    println!(
        "Device code received. User code: {}",
        device_code_response.user_code
    );

    // Step 2: Open browser with verification_uri_complete
    println!("Opening browser for authentication...");
    if let Err(e) = open::that(&device_code_response.verification_uri_complete) {
        eprintln!("Failed to open browser automatically: {}", e);
        println!(
            "Please manually visit: {}",
            device_code_response.verification_uri_complete
        );
    } else {
        println!("Browser opened. Please complete the authentication in your browser.");
    }

    // Step 3: Poll for token after user authenticates
    println!("Waiting for authentication...");
    let interval = Duration::from_secs(device_code_response.interval);
    let expires_at =
        std::time::Instant::now() + Duration::from_secs(device_code_response.expires_in);

    let token_response = loop {
        if std::time::Instant::now() > expires_at {
            return Err("Device code expired. Please try again.".into());
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
            let token_response: TokenResponse = response.json().await?;
            break token_response;
        } else {
            // Check for authorization_pending or slow_down errors (expected during polling)
            let status = response.status();
            let error_text = response.text().await?;

            if error_text.contains("authorization_pending") {
                // Still waiting for user to authorize
                continue;
            } else if error_text.contains("slow_down") {
                // Server asked us to slow down, add extra delay
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            } else {
                return Err(format!("Token request failed ({}): {}", status, error_text).into());
            }
        }
    };

    save_user_config(token_response.access_token.clone(), token_response.token_type.clone());
    get_user_config()
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
