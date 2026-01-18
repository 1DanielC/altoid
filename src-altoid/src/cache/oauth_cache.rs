use crate::cache::pub_oauth_config::{OAuthConfig, OAUTH_CONFIG_FILE};
use crate::cache::root_cache;

pub fn get_oauth_config() -> Result<OAuthConfig, Box<dyn std::error::Error>> {
    root_cache::read_cache_file(OAUTH_CONFIG_FILE)
}

pub fn save_auth_data(auth_data: &OAuthConfig) {
    root_cache::write_cache_file(OAUTH_CONFIG_FILE, auth_data).expect("Error Saving Auth Data");
}

pub fn clear_auth_data() -> Result<(), Box<dyn std::error::Error>> {
    root_cache::clear_cache_file(OAUTH_CONFIG_FILE)?;
    Ok(())
}
