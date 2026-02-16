use crate::api::oauth::pkg_auth::{AuthEnv, AuthScope};
use crate::api::openspace::pub_api_env;
use crate::error::AppError;
use crate::extensions::LockExt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    local_storage: LocalStorage,
    user_config: Mutex<Option<UserConfig>>,
    auth_config: Mutex<Option<OAuthConfig>>,
    skipped_files: Mutex<Option<Vec<SkippedFile>>>,
}

impl AppState {
    pub fn new(app_dir: PathBuf) -> Self {
        let local_storage = LocalStorage::new(app_dir);
        let user_config = Mutex::new(local_storage.load(|ls| ls.user_config.clone()));
        let auth_config = Mutex::new(local_storage.load(|ls| ls.auth_config.clone()));
        let skipped_files = Mutex::new(local_storage.load(|ls| ls.skipped_files.clone()));

        Self {
            local_storage,
            user_config,
            auth_config,
            skipped_files,
        }
    }

    pub fn get_user_config(&self) -> Option<UserConfig> {
        self.user_config.lock_or_err().ok()?.clone()
    }

    pub fn set_user_config(&self, config: UserConfig) -> Result<(), AppError> {
        let mut guard = self.user_config.lock_or_err()?;

        self.local_storage
            .save(|ls| ls.user_config.clone(), &config)
            .map_err(AppError::from)?;

        *guard = Some(config);
        Ok(())
    }

    pub fn get_auth_config(&self) -> Option<OAuthConfig> {
        self.auth_config.lock_or_err().ok()?.clone()
    }


    pub fn clear_state(&self) -> Result<(), AppError> {
        let mut user_guard = self.user_config.lock_or_err()?;
        let mut files_guard = self.skipped_files.lock_or_err()?;

        self.local_storage.clear(|ls| ls.skipped_files.clone())?;
        self.local_storage.clear(|ls| ls.user_config.clone())?;

        *user_guard = None;
        *files_guard = None;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct LocalStorage {
    pub user_config: Option<PathBuf>,
    pub auth_config: Option<PathBuf>,
    pub skipped_files: Option<PathBuf>,
}

impl LocalStorage {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            user_config: Some(app_data_dir.join("user_config.json")),
            auth_config: Some(app_data_dir.join("oauth_config.json")),
            skipped_files: Some(app_data_dir.join("skipped_files.json")),
        }
    }

    pub fn load<T, F>(&self, get_path: F) -> Option<T>
    where
        T: DeserializeOwned,
        F: Fn(&Self) -> Option<PathBuf>,
    {
        Self::load_json(get_path(self))
    }

    pub fn save<T, F>(&self, get_path: F, data: &T) -> Result<(), std::io::Error>
    where
        T: serde::Serialize,
        F: Fn(&Self) -> Option<PathBuf>,
    {
        let path = get_path(self).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Storage path not configured")
        })?;
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    pub fn clear<F>(&self, get_path: F) -> Result<(), std::io::Error>
    where
        F: Fn(&Self) -> Option<PathBuf>,
    {
        if let Some(path) = get_path(self) {
            match std::fs::remove_file(&path) {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }

    fn load_json<T>(path: Option<PathBuf>) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let p = path?;
        let file = File::open(p).ok()?;
        serde_json::from_reader(file).ok()
    }
}

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

impl ApiConfig {
    pub fn new(env: pub_api_env::ApiEnv, url: Option<String>) -> Self {
        Self { env, host: url }
    }

    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or(self.env.get_host())
    }

    pub fn set_host(&mut self, host: Option<String>) {
        self.host = host;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct OAuthConfig {
    pub client_id: String,
    pub env: AuthEnv,
    pub scope: AuthScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SkippedFile {
    pub filename: String,
    pub size: i64,
    pub device_id: String,
}

impl SkippedFile {
    pub fn new(filename: String, size: i64, device_id: String) -> Self {
        Self {
            filename,
            size,
            device_id,
        }
    }
}
