use crate::error::AppError;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::LazyLock;

pub const STORAGE_DIR: &str = ".openspace_sync";

pub static STORAGE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| get_or_create_storage_path().expect("Failed to initialize storage path"));

fn get_or_create_storage_path() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::Internal("Could not find home directory".to_string()))?;
    let storage_dir = home.join(STORAGE_DIR);

    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir)?;
    }

    Ok(storage_dir)
}

fn get_cache_file(rel_path: &str) -> Option<PathBuf> {
    let p = STORAGE_PATH.join(rel_path);
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

pub fn read_cache_file<T: serde::de::DeserializeOwned>(rel_path: &str) -> Option<T> {
    let path = get_cache_file(rel_path)?;
    let content = File::open(path)
        .map_err(|e| format!("Failed to read cache file {}: {}", rel_path, e))
        .ok()?;

    serde_json::from_reader(content).ok()?
}

pub fn write_cache_file<T: serde::Serialize>(
    rel_path: &str,
    data: &T,
) -> Result<(), AppError> {
    let path: PathBuf = STORAGE_PATH.join(rel_path);
    let content = serde_json::to_string_pretty(data).map_err(AppError::JsonSerialization)?;
    fs::write(&path, content).map_err(|e| AppError::CacheWrite {
        file: rel_path.to_string(),
        source: e,
    })?;
    Ok(())
}

pub fn clear_cache_file(rel_path: &str) -> Result<(), AppError> {
    let path = STORAGE_PATH.join(rel_path);

    match fs::remove_file(&path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File didn't exist, that's fine
        }
        Err(e) => {
            return Err(AppError::CacheWrite {
                file: rel_path.to_string(),
                source: e,
            })
        }
    }

    Ok(())
}

pub fn clear_all_cache() -> Result<(), AppError> {
    if let Some(storage_dir) = STORAGE_PATH.parent() {
        if storage_dir.exists() {
            fs::remove_dir_all(storage_dir).map_err(|e| AppError::CacheWrite {
                file: storage_dir.display().to_string(),
                source: e,
            })?;
        }
    }
    Ok(())
}
