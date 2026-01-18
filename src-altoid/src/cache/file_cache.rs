use crate::cache::pkg_file_config::{SkippedFile, SKIPPED_FILES_FILE};
use crate::cache::root_cache::{clear_cache_file, read_cache_file, write_cache_file};
use std::collections::HashSet;

pub fn load_skipped_files() -> Result<HashSet<SkippedFile>, Box<dyn std::error::Error>> {
    read_cache_file(SKIPPED_FILES_FILE)
}

pub fn save_skipped_files(
    skipped: &HashSet<SkippedFile>,
) -> Result<(), Box<dyn std::error::Error>> {
    write_cache_file(SKIPPED_FILES_FILE, skipped)
}

pub fn add_skipped_file(
    filename: &str,
    size: i64,
    device_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut skipped = load_skipped_files()?;
    skipped.insert(SkippedFile::new(
        filename.to_string(),
        size,
        device_id.to_string(),
    ));
    save_skipped_files(&skipped)?;

    Ok(())
}

pub fn clear_skipped_files() -> Result<(), Box<dyn std::error::Error>> {
    clear_cache_file(SKIPPED_FILES_FILE)
        .expect("Something went wrong when clearing skipped files cache");
    Ok(())
}

pub fn is_file_skipped(filename: &str, size: i64, device_id: &str) -> bool {
    match load_skipped_files() {
        Ok(skipped) => {
            let file = SkippedFile::new(filename.to_string(), size, device_id.to_string());
            skipped.contains(&file)
        }
        Err(_) => false,
    }
}
