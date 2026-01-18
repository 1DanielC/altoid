use crate::api::openspace::tictac::{GetOrCreateUploadResponse, TicTacUploadRequest};
use crate::cache::file_cache::{add_skipped_file, is_file_skipped};
use crate::camera::camera_finder::scan_for_camera_fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use walkdir::WalkDir;
use crate::api::http::client::create_http_client;
use crate::cache::pub_user_config::ApiConfig;

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
) -> Result<(), Box<dyn std::error::Error>> {
    let api_config = ApiConfig::default();
    let api_host = api_config.host();

    let camera_info = match scan_for_camera_fs() {
        Some(info) => info,
        None => {
            println!("No camera volume found");
            return Ok(()); // exit the function cleanly
        }
    };

    println!("Found camera volume: {:?}", camera_info.mount_point);
    println!("Device ID: {}", camera_info.device_id);

    // Notify UI that camera was found
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(UploadEvent::CameraFound {
            device_id: camera_info.device_id.clone(),
        });
    }

    // Step 1: Find all .insv files (filtered by skipped files cache)
    let insv_files: Vec<PathBuf> = collect_insv_files(
        camera_info.mount_point,
        &camera_info.device_id,
        progress_tx.as_ref(),
    )?;
    println!("Found {} .insv files to upload", insv_files.len());

    if insv_files.is_empty() {
        println!("No files to upload");
        return Ok(());
    }

    // Create a Tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Step 2: First pass - check upload state for all files and populate UI
    let mut files_to_upload: Vec<FileUploadState> = Vec::new();

    for file in &insv_files {
        let filename = file.file_name().unwrap().to_str().unwrap().to_string();
        let file_size = file.metadata()?.len() as i64;

        let request = TicTacUploadRequest::new(
            camera_info.device_id.clone(),
            filename.clone(),
            "video/insv".to_string(),
            file_size,
            1,
        );

        // Check upload state with server
        match runtime.block_on(check_upload_state(&request, api_host)) {
            Ok(upload_id) => {
                if upload_id.is_some() {
                    // File needs to be uploaded - notify UI with pending status
                    if let Some(ref tx) = progress_tx {
                        let _ = tx.send(UploadEvent::FilePending {
                            filename: filename.clone(),
                            total_bytes: file_size,
                        });
                    }
                    files_to_upload.push(FileUploadState {
                        path: file.clone(),
                        filename,
                        size: file_size,
                        upload_id,
                    });
                } else {
                    // File already exists on server - notify UI
                    println!("File already exists on server, skipping: {:?}", filename);
                    if let Some(ref tx) = progress_tx {
                        let _ = tx.send(UploadEvent::FileSkipped {
                            filename: filename.clone(),
                        });
                    }
                    // Add to skipped files cache
                    add_skipped_file(&filename, file_size, &camera_info.device_id.clone())
                        .expect("Something went wrong adding to skipped cache");
                }
            }
            Err(e) => {
                eprintln!("Failed to check upload state for {:?}: {}", filename, e);
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(UploadEvent::FileFailed {
                        filename,
                        error: e.to_string(),
                    });
                }
            }
        }
    }

    // Step 3: Second pass - actually upload the files that need uploading
    for file_state in files_to_upload {
        // Notify UI that file upload is starting
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(UploadEvent::FileStarted {
                filename: file_state.filename.clone(),
                total_bytes: file_state.size,
            });
        }

        match runtime.block_on(upload_file_chunks(
            &file_state,
            progress_tx.clone(),
            api_host,
        )) {
            Ok(()) => {
                println!("Successfully uploaded: {:?}", file_state.filename);
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(UploadEvent::FileCompleted {
                        filename: file_state.filename,
                    });
                }
            }
            Err(e) => {
                eprintln!("Failed to upload {:?}: {}", file_state.filename, e);
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(UploadEvent::FileFailed {
                        filename: file_state.filename,
                        error: e.to_string(),
                    });
                }
            }
        }
    }

    println!("Upload process completed");
    Ok(())
}

fn collect_insv_files(
    volume: PathBuf,
    device_id: &str,
    progress_tx: Option<&Sender<UploadEvent>>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut insv_files = Vec::new();

    for entry in WalkDir::new(volume) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext.eq_ignore_ascii_case("insv") {
                            let path = entry.path().to_path_buf();

                            // Check if this file is in the skipped cache
                            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                                if let Ok(metadata) = entry.metadata() {
                                    let size = metadata.len() as i64;

                                    // Skip if already in cache
                                    if is_file_skipped(filename, size, device_id) {
                                        println!("Skipping cached file: {}", filename);

                                        // Send event for cached skipped file
                                        if let Some(tx) = progress_tx {
                                            let _ = tx.send(UploadEvent::FileSkipped {
                                                filename: filename.to_string(),
                                            });
                                        }
                                        continue;
                                    }
                                }
                            }

                            insv_files.push(path);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading directory entry: {}", e),
        }
    }

    Ok(insv_files)
}

// Check upload state with server (does file need to be uploaded?)
async fn check_upload_state(
    req: &TicTacUploadRequest,
    api_host: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = &create_http_client();
    let create_url = format!("{}/api/tictac/uploads", api_host);

    let response = client.post(&create_url).json(&req).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to create upload: {}", response.status()).into());
    }

    let create_response: GetOrCreateUploadResponse = response.json().await?;
    Ok(create_response.upload_id)
}

// Upload the actual file chunks
async fn upload_file_chunks(
    file_state: &FileUploadState,
    progress_tx: Option<Sender<UploadEvent>>,
    api_host: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let upload_id = file_state
        .upload_id
        .as_ref()
        .ok_or("No upload ID provided")?;

    let client = &create_http_client();
    let mut file_handle = File::open(&file_state.path)?;
    let file_size = file_state.size;
    let filename = file_state.filename.clone();

    // TODO Upload file into chunks of CHUNK_SIZE bytes
    let num_parts = 1;
    let chunk_size = file_size;

    for part in 0..num_parts {
        let start = part as i64 * chunk_size;
        let end = ((part + 1) as i64 * chunk_size).min(file_size) - 1;
        let chunk_len = (end - start + 1) as usize;

        // Read chunk from file
        file_handle.seek(SeekFrom::Start(start as u64))?;
        let mut buffer = vec![0u8; chunk_len];
        file_handle.read_exact(&mut buffer)?;

        // Upload chunk with Content-Range header
        let upload_url = format!("{}/api/tictac/uploads/{}", api_host, upload_id);
        let content_range = format!("bytes {}-{}/{}", start, end, file_size);

        let response = client
            .put(&upload_url)
            .header("Content-Range", content_range)
            .header("Content-Type", "application/octet-stream")
            .body(buffer)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to upload chunk {}: {}", part, response.status()).into());
        }

        println!(
            "Uploaded chunk {}/{} (bytes {}-{})",
            part + 1,
            num_parts,
            start,
            end
        );

        // Send progress update
        if let Some(ref tx) = progress_tx {
            let bytes_uploaded = end + 1;
            let _ = tx.send(UploadEvent::FileProgress {
                filename: filename.clone(),
                bytes_uploaded,
                total_bytes: file_size,
            });
        }
    }

    Ok(())
}

// Public API for React to get list of files to upload
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub path: String,
    pub filename: String,
    pub size: i64,
    pub content_type: String,
}

pub fn get_uploadable_files() -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
    // Scan for camera
    let camera_info = match scan_for_camera_fs() {
        Some(info) => info,
        None => {
            return Ok(Vec::new()); // No camera found, return empty list
        }
    };

    println!("Found camera volume: {:?}", camera_info.mount_point);
    println!("Device ID: {}", camera_info.device_id);

    // Collect .insv files (without progress callback)
    let insv_files = collect_insv_files(camera_info.mount_point, &camera_info.device_id, None)?;

    // Convert to FileInfo structs
    let mut file_list = Vec::new();
    for file_path in insv_files {
        if let Some(filename) = file_path.file_name().and_then(|f| f.to_str()) {
            if let Ok(metadata) = file_path.metadata() {
                let size = metadata.len() as i64;
                file_list.push(FileInfo {
                    path: file_path.to_string_lossy().to_string(),
                    filename: filename.to_string(),
                    size,
                    content_type: "video/insv".to_string(),
                });
            }
        }
    }

    Ok(file_list)
}

// Public API for React to read file chunks
pub fn read_chunk(
    path: &str,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0u8; length];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    Ok(buffer)
}
