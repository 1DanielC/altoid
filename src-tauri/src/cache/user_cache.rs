use crate::cache::pub_user_config::{UserConfig, USER_CONFIG_FILE};
use crate::cache::root_cache;

pub fn get_user_config() -> Result<UserConfig, Box<dyn std::error::Error>> {
    root_cache::read_cache_file(USER_CONFIG_FILE)
}

pub fn save_user_config(access_token: String, token_type: String) {
    let auth_data = UserConfig {
        access_token,
        token_type,
        // TODO load API config
        api_config: Default::default(),
    };

    root_cache::write_cache_file(USER_CONFIG_FILE, &auth_data).expect("Error Saving Auth Data");
}

pub fn clear_user_config() -> Result<(), Box<dyn std::error::Error>> {
    root_cache::clear_cache_file(USER_CONFIG_FILE)?;
    Ok(())
}
