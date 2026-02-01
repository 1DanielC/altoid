use crate::cache::pub_user_config::{UserConfig, USER_CONFIG_FILE};
use crate::cache::root_cache;
use crate::error::AppError;

pub fn get_user_config() -> Option<UserConfig> {
    root_cache::read_cache_file(USER_CONFIG_FILE)
}

pub fn save_user_config(access_token: String, token_type: String) -> Result<(), AppError> {
    let auth_data = UserConfig {
        access_token,
        token_type,
        // TODO load API config
        api_config: Default::default(),
    };

    root_cache::write_cache_file(USER_CONFIG_FILE, &auth_data)?;
    Ok(())
}

pub fn clear_user_config() -> Result<(), AppError> {
    root_cache::clear_cache_file(USER_CONFIG_FILE)
}

pub fn get_host_override() -> Option<String> {
    get_user_config().and_then(|config| {
        let host = config.api_config.host().to_string();
        let default_host = config.api_config.env.get_host();
        if host != default_host {
            Some(host)
        } else {
            None
        }
    })
}

pub fn set_host_override(host: Option<String>) -> Result<(), AppError> {
    let mut config = get_user_config().ok_or(AppError::NotAuthenticated)?;
    config.api_config.set_host(host);
    root_cache::write_cache_file(USER_CONFIG_FILE, &config)?;
    Ok(())
}
