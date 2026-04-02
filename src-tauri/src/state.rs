use crate::api::oauth::pkg_auth::{AuthEnv, AuthScope};
use crate::api::openspace::pub_api_env;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tauri_plugin_log::log::info;

static CONFIG_FILENAME: &str = "altoid_config.json";
static UPLOADED_FILES_FILENAME: &str = "uploaded_files.json";
static LEGACY_UPLOADED_FILES_FILENAME: &str = "skipped_files.json";

/// Unified config file persisted as `altoid_config.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AltoidConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_config: Option<UserConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_config: Option<OAuthConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_override: Option<String>,
}

#[derive(Debug)]
pub struct AppState {
    config_path: PathBuf,
    config: Mutex<AltoidConfig>,
    uploaded_files_path: PathBuf,
    uploaded_files: Mutex<Option<Vec<UploadedFile>>>,
}

impl AppState {
    pub fn new(app_dir: PathBuf) -> Self {
        let config_path = app_dir.join(CONFIG_FILENAME);
        let uploaded_files_path = app_dir.join(UPLOADED_FILES_FILENAME);

        let config = load_json::<AltoidConfig>(&config_path).unwrap_or_default();
        // Also try loading from legacy path for backwards compatibility
        let uploaded_files = load_json::<Vec<UploadedFile>>(&uploaded_files_path)
            .or_else(|| load_json::<Vec<UploadedFile>>(&app_dir.join(LEGACY_UPLOADED_FILES_FILENAME)));

        Self {
            config_path,
            config: Mutex::new(config),
            uploaded_files_path,
            uploaded_files: Mutex::new(uploaded_files),
        }
    }

    // ── Config persistence ──────────────────────────────────────────

    fn lock_config(&self) -> Result<MutexGuard<'_, AltoidConfig>, AppError> {
        self.config.lock().map_err(|_| AppError::LockingError)
    }

    fn save_config(&self, config: &AltoidConfig) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::internal(&format!("Failed to serialize config: {}", e)))?;
        std::fs::write(&self.config_path, content).map_err(AppError::from)
    }

    // ── UserConfig ──────────────────────────────────────────────────

    pub fn get_user_config(&self) -> Option<UserConfig> {
        self.lock_config().ok()?.user_config.clone()
    }

    pub fn set_user_config(&self, user_config: UserConfig) -> Result<(), AppError> {
        let mut guard = self.lock_config()?;
        guard.user_config = Some(user_config);
        self.save_config(&guard)
    }

    // ── OAuthConfig ─────────────────────────────────────────────────

    pub fn get_auth_config(&self) -> Option<OAuthConfig> {
        self.lock_config().ok()?.oauth_config.clone()
    }

    pub fn set_auth_config(&self, oauth_config: OAuthConfig) -> Result<(), AppError> {
        let mut guard = self.lock_config()?;
        guard.oauth_config = Some(oauth_config);
        self.save_config(&guard)
    }

    // ── Host Override ───────────────────────────────────────────────

    pub fn get_host_override(&self) -> Option<String> {
        let val = self.lock_config().ok()?.host_override.clone();
        info!("get_host_override() -> {:?}", val);
        val
    }

    pub fn set_host_override(&self, host: Option<String>) -> Result<(), AppError> {
        info!("set_host_override({:?})", host);
        let mut guard = self.lock_config()?;
        guard.host_override = host;
        self.save_config(&guard)?;
        info!("host_override saved successfully");
        Ok(())
    }

    // ── Uploaded files ──────────────────────────────────────────────

    fn lock_uploaded_files(&self) -> Result<MutexGuard<'_, Option<Vec<UploadedFile>>>, AppError> {
        self.uploaded_files.lock().map_err(|_| AppError::LockingError)
    }

    fn save_uploaded_files(&self, files: &[UploadedFile]) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(files)
            .map_err(|e| AppError::internal(&format!("Failed to serialize uploaded files: {}", e)))?;
        std::fs::write(&self.uploaded_files_path, content).map_err(AppError::from)
    }

    pub fn get_uploaded_files(&self) -> Vec<UploadedFile> {
        self.lock_uploaded_files()
            .ok()
            .and_then(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn add_uploaded_file(&self, file: UploadedFile) -> Result<(), AppError> {
        let mut guard = self.lock_uploaded_files()?;
        let files = guard.get_or_insert_with(Vec::new);
        // Avoid duplicates (same filename + device_id)
        if !files.iter().any(|f| f.filename == file.filename && f.device_id == file.device_id) {
            files.push(file);
        }
        self.save_uploaded_files(files)
    }

    // ── Clear (logout) ──────────────────────────────────────────────

    pub fn clear_state(&self) -> Result<(), AppError> {
        // Clear user config but preserve oauth_config and host_override
        let mut config_guard = self.lock_config()?;
        config_guard.user_config = None;
        self.save_config(&config_guard)?;

        // Clear uploaded files
        let mut files_guard = self.uploaded_files
            .lock()
            .map_err(|_| AppError::LockingError)?;
        *files_guard = None;
        match std::fs::remove_file(&self.uploaded_files_path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::from(e)),
        }
    }
}

// ── File helpers ────────────────────────────────────────────────────

fn load_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let file = File::open(path).ok()?;
    serde_json::from_reader(file).ok()
}

// ── Data structs ────────────────────────────────────────────────────

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
pub struct UploadedFile {
    pub filename: String,
    pub size: i64,
    pub device_id: String,
}

impl UploadedFile {
    pub fn new(filename: String, size: i64, device_id: String) -> Self {
        Self {
            filename,
            size,
            device_id,
        }
    }
}
