use crate::cache::pkg_file_config::{SkippedFile, SKIPPED_FILES_FILE};
use crate::cache::root_cache::{clear_cache_file, read_cache_file, write_cache_file};
use crate::error::AppError;
use std::collections::HashSet;

pub fn load_skipped_files() -> Option<HashSet<SkippedFile>> {
    read_cache_file(SKIPPED_FILES_FILE)
}

pub fn save_skipped_files(
    skipped: &HashSet<SkippedFile>,
) -> Result<(), AppError> {
    write_cache_file(SKIPPED_FILES_FILE, skipped)
}

pub fn add_skipped_file(
    filename: &str,
    size: i64,
    device_id: &str,
) -> Result<(), AppError> {
    let mut skipped = load_skipped_files().unwrap_or(HashSet::new());
    skipped.insert(SkippedFile::new(
        filename.to_string(),
        size,
        device_id.to_string(),
    ));
    save_skipped_files(&skipped)?;

    Ok(())
}

pub fn clear_skipped_files() -> Result<(), AppError> {
    clear_cache_file(SKIPPED_FILES_FILE)
}

pub fn is_file_skipped(filename: &str, size: i64, device_id: &str) -> bool {
    match load_skipped_files() {
        Some(skipped) => {
            let file = SkippedFile::new(filename.to_string(), size, device_id.to_string());
            skipped.contains(&file)
        }
        None => false,
    }
}
