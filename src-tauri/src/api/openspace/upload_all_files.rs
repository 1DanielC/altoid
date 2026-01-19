use crate::api::http::client::create_http_client;
use crate::api::openspace::tictac::{GetOrCreateUploadResponse, TicTacUploadRequest};
use crate::cache::file_cache::{add_skipped_file, is_file_skipped};
use crate::cache::pub_user_config::ApiConfig;
use crate::camera::camera_finder::find_camera;
use crate::error::AppError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use walkdir::WalkDir;

const CHUNK_SIZE: i64 = 8 * 1024 * 1024; // 8MB chunks

#[derive(Debug)]
struct FileToUpload {
    path: PathBuf,
    size: i64,
}

#[derive(Debug)]
struct FileUploadState {
    path: PathBuf,
    filename: String,
    size: i64,
    upload_id: Option<String>,
}

#[derive(Debug)]
enum UploadResult {
    Completed,
    Skipped,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum UploadEvent {
    CameraFound {
        device_id: String,
    },
    FilePending {
        filename: String,
        total_bytes: i64,
    },
    FileStarted {
        filename: String,
        total_bytes: i64,
    },
    FileProgress {
        filename: String,
        bytes_uploaded: i64,
        total_bytes: i64,
    },
    FileSkipped {
        filename: String,
    },
    FileCompleted {
        filename: String,
    },
    FileFailed {
        filename: String,
        error: String,
    },
}

pub fn upload_all_files(
    progress_tx: Option<Sender<UploadEvent>>,
) -> Result<(), AppError> {
    // TODO: Implement actual upload logic
    Ok(())
}