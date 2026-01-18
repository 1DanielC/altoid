use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

pub const STORAGE_DIR: &str = ".openspace_sync";

pub static STORAGE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    get_or_create_storage_path().expect("Failed to initialize storage path")
});

fn get_or_create_storage_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let storage_dir = home.join(STORAGE_DIR);

    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir)?;
    }

    Ok(storage_dir)
}

fn get_cache_file(rel_path: &str) -> PathBuf {
    STORAGE_PATH.join(rel_path)
}

pub fn read_cache_file<T: serde::de::DeserializeOwned>(rel_path: &str) -> Result<T, Box<dyn std::error::Error>> {
    let path = get_cache_file(rel_path);
    let content = fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn write_cache_file<T: serde::Serialize>(rel_path: &str, data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_cache_file(rel_path);
    let content = serde_json::to_string_pretty(data)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn clear_cache_file(rel_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_cache_file(rel_path);
    fs::remove_file(path)?;
    Ok(())
}

pub fn clear_all_cache() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(storage_dir) = STORAGE_PATH.parent() {
        if storage_dir.exists() {
            fs::remove_dir_all(storage_dir)?;
        }
    }
    Ok(())
}
