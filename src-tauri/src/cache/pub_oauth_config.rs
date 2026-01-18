use serde::{Deserialize, Serialize};
use crate::api::oauth::pkg_auth::{AuthEnv, AuthScope};

pub const OAUTH_CONFIG_FILE: &str = "oauth_config.json";

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct OAuthConfig {
    pub client_id: String,
    pub env: AuthEnv,
    pub scope: AuthScope,
}
