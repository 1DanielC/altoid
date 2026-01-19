use serde::{Deserialize, Serialize};
use crate::api::openspace::pub_api_env;

pub const USER_CONFIG_FILE: &str = "user_config.json";

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct UserConfig {
    pub access_token: String,
    pub token_type: String,
    pub api_config: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ApiConfig {
    pub env: pub_api_env::ApiEnv,
    host: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            env: pub_api_env::ApiEnv::Local,
            host: Some("http://localhost:8080".to_string()),
        }
    }
}

impl ApiConfig {
    pub fn new(env: pub_api_env::ApiEnv, url: Option<String>) -> Self {
        Self { env, host: url }
    }

    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or(self.env.get_host())
    }
}
