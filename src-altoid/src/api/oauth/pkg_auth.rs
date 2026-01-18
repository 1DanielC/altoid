use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub device_code: String,
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct UserInfo {
    pub email: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeRequest {
    pub client_id: String,
    pub scope: String,
    pub audience: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct LoginConfig {
    pub client_id: String,
    pub auth_env: AuthEnv,
    pub scope: AuthScope,
}

impl Default for LoginConfig {
    fn default() -> Self {
        Self {
            client_id: "37lfJRh96Q9MT86n7MigrFcRLBsNxIXD".to_string(),
            auth_env: AuthEnv::Dev,
            scope: AuthScope::Email,
        }
    }
}

impl LoginConfig {
    pub fn new(client_id: String, auth_env: AuthEnv, scope: AuthScope) -> Self {
        Self { client_id, auth_env, scope }
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum AuthScope {
    Openid,
    Email,
    OfflineAccess,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
)]
#[strum(serialize_all = "camelCase")]
pub enum AuthEnv {
    Dev,
    Prod,
}

impl AuthEnv {
    pub fn get_host(&self) -> &'static str {
        match self {
            AuthEnv::Dev => "https://login.osdevenv.net",
            AuthEnv::Prod => "https://login.openspace.ai",
        }
    }

    pub fn get_audience(&self) -> &'static str {
        match self {
            AuthEnv::Dev => "openspace-dev.ai",
            AuthEnv::Prod => "openspace.ai",
        }
    }

    pub fn get_auth_url(&self) -> String {
        format!("{}/oauth/device/code", self.get_host())
    }

    pub fn get_token_url(&self) -> String {
        format!("{}/oauth/token", self.get_host())
    }
}
