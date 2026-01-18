use crate::cache::pub_oauth_config::{OAuthConfig, OAUTH_CONFIG_FILE};
use crate::cache::root_cache;

pub fn get_oauth_config() -> Option<OAuthConfig> {
    root_cache::read_cache_file(OAUTH_CONFIG_FILE)
}

pub fn save_auth_data(auth_data: &OAuthConfig) {
    root_cache::write_cache_file(OAUTH_CONFIG_FILE, auth_data).expect("Error Saving Auth Data");
}
