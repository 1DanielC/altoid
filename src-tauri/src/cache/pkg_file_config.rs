use serde::{Deserialize, Serialize};

pub const SKIPPED_FILES_FILE: &str = "skipped_files.json";

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
