use crate::camera::device_type::{CameraInfo, CAMERAS};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri_plugin_log::log::{error, info};

pub static PTP_MOUNT_POINT: &str = "PTP";
pub static GPHOTO2_CMD: &str = "gphoto2";
pub static DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

#[derive(Debug, Clone, Serialize)]
pub struct DetectedCamera {
    pub info: &'static CameraInfo,
    pub serial_number: Option<String>,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CameraFile {
    pub path: PathBuf,
    pub filename: String,
    pub size: i64,
    pub content_type: String,
}

#[derive(Debug, Serialize)]
pub struct CameraWithFiles {
    pub camera: DetectedCamera,
    pub mount_point: Option<PathBuf>,
    pub files: Vec<CameraFile>,
    pub access_error: Option<String>,
}

/// Fast USB scan to detect a known camera. Does not list files.
pub fn detect_camera() -> Option<DetectedCamera> {
    let devices = match rusb::devices() {
        Ok(devices) => devices,
        Err(e) => {
            error!("Failed to enumerate USB devices: {}", e);
            return None;
        }
    };

    for device in devices.iter() {
        if let Ok(desc) = device.device_descriptor() {
            if let Some(camera_info) = CAMERAS.get(&desc.vendor_id()) {
                info!("Found camera: {} (Vendor ID: {})", camera_info.device, desc.vendor_id());

                let serial_number = device
                    .open()
                    .ok()
                    .and_then(|handle| {
                        handle
                            .read_serial_number_string_ascii(&desc)
                            .ok()
                    });

                let device_id = format!(
                    "{}:sn:{}",
                    camera_info.device,
                    serial_number.as_deref().unwrap_or("unknown")
                );

                info!("Camera device ID: {}", device_id);

                return Some(DetectedCamera {
                    info: camera_info,
                    serial_number,
                    device_id,
                });
            }
        }
    }

    info!("No supported camera found connected via USB");
    None
}

/// Full camera detection: find camera then list its files (can be slow).
pub fn find_camera() -> Option<CameraWithFiles> {
    let detected = detect_camera()?;
    let (mount_point, files, access_error) = find_camera_files();

    Some(CameraWithFiles {
        camera: detected,
        mount_point,
        files,
        access_error,
    })
}

fn find_camera_files_ptp() -> (Option<PathBuf>, Vec<CameraFile>, Option<String>) {
    info!("Attempting PTP camera access via gphoto2 CLI...");

    // First, check if gphoto2 is available
    let check_gphoto2 = Command::new("which")
        .arg("gphoto2")
        .output();

    if check_gphoto2.is_err() || !check_gphoto2.as_ref().unwrap().status.success() {
        let error_msg = "gphoto2 CLI not found. Please install it with: brew install gphoto2".to_string();
        error!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    // Try to detect cameras
    let detect_output = Command::new(GPHOTO2_CMD)
        .arg("--auto-detect")
        .output();

    if let Err(e) = detect_output {
        let error_msg = format!("Failed to run gphoto2 --auto-detect: {}", e);
        error!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    let detect_output = detect_output.unwrap();
    let detect_stdout = String::from_utf8_lossy(&detect_output.stdout);

    // Check if any camera was detected
    if !detect_stdout.contains("usb:") {
        let error_msg = "No PTP camera detected by gphoto2".to_string();
        error!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    info!("Camera detected via gphoto2: {}", detect_stdout.trim());

    // List files on the camera
    let list_output = Command::new(GPHOTO2_CMD)
        .arg("--list-files")
        .output();

    if let Err(e) = list_output {
        let error_msg = format!("Failed to run gphoto2 --list-files: {}", e);
        error!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    let list_output = list_output.unwrap();
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);

    // Parse file list from gphoto2 output
    let files = parse_gphoto2_file_list(&list_stdout);

    if files.is_empty() {
        let error_msg = "Camera connected via PTP but no files found".to_string();
        error!("{}", error_msg);
        (Some(PathBuf::from(PTP_MOUNT_POINT)), Vec::new(), Some(error_msg))
    } else {
        info!("Found {} files via PTP", files.len());
        (Some(PathBuf::from(PTP_MOUNT_POINT)), files, None)
    }
}

pub fn parse_gphoto2_file_list(output: &str) -> Vec<CameraFile> {
    let mut files = Vec::new();
    let mut current_folder = String::new();

    for line in output.lines() {
        // Look for folder lines like "There are N files in folder '/path'."
        if line.starts_with("There") && line.contains("files in folder") {
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    current_folder = line[start + 1..start + 1 + end].to_string();
                }
            }
        }
        // Look for file lines starting with #N
        // Format: #1     R0010001.JPG               rd  8367 KB ...
        else if line.starts_with('#') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let filename = parts[1].to_string();
                let file_path = if current_folder.is_empty() {
                    PathBuf::from(&filename)
                } else {
                    PathBuf::from(format!("{}/{}", current_folder, filename))
                };

                // Parse size: e.g. "8367 KB"
                let size_kb = parts[3].parse::<i64>().unwrap_or(0);
                let size_unit = parts.get(4).unwrap_or(&"KB");
                let size = match *size_unit {
                    "MB" => size_kb * 1024 * 1024,
                    "GB" => size_kb * 1024 * 1024 * 1024,
                    _ => size_kb * 1024, // KB default
                };

                let content_type = guess_content_type(&file_path);

                files.push(CameraFile {
                    path: file_path,
                    filename,
                    size,
                    content_type,
                });
            }
        }
    }

    files
}

fn find_camera_files() -> (Option<PathBuf>, Vec<CameraFile>, Option<String>) {
    // First, try PTP access via gphoto2
    let (mount_point, files, error) = find_camera_files_ptp();
    if mount_point.is_some() && !files.is_empty() {
        return (mount_point, files, error);
    }

    // If PTP didn't work, fall back to mass storage detection
    info!("PTP access failed or no files found, trying mass storage detection...");

    // Try to enumerate mounted drives
    let drives = match bb_drivelist::drive_list() {
        Ok(drives) => drives,
        Err(e) => {
            let error_msg = format!("Failed to enumerate drives: {}", e);
            error!("{}", error_msg);
            return (None, Vec::new(), Some(error_msg));
        }
    };

    info!("Total drives detected: {}", drives.len());

    // Look for removable drives (cameras typically mount as removable storage)
    for drive in &drives {
        info!("Drive: device={}, is_removable={}, mountpoints={}",
                 drive.device, drive.is_removable, drive.mountpoints.len());

        for mp in &drive.mountpoints {
            info!("  Mountpoint: {}", mp.path);
        }
    }

    // First, try removable drives
    for drive in &drives {
        if !drive.is_removable {
            continue;
        }

        // Try each mount point
        for mount_point in &drive.mountpoints {
            let path = PathBuf::from(&mount_point.path);
            info!("Checking removable drive at: {}", path.display());

            match list_files_recursive(&path, &path) {
                Ok(files) if !files.is_empty() => {
                    info!("Found {} files on device at {}", files.len(), path.display());
                    return (Some(path.clone()), files, None);
                }
                Ok(_) => {
                    info!("No files found at {}", path.display());
                }
                Err(e) => {
                    let error_msg = format!(
                        "Cannot access files at {}. Error: {}. \
                        On macOS, you may need to grant 'Full Disk Access' permission to your terminal or app in System Settings > Privacy & Security > Full Disk Access.",
                        path.display(),
                        e
                    );
                    error!("{}", error_msg);
                    return (Some(path.clone()), Vec::new(), Some(error_msg));
                }
            }
        }
    }

    // On macOS, RICOH THETA cameras might mount at /Volumes/
    // Try common RICOH THETA mount points
    let potential_paths = vec![
        "/Volumes/RICOH THETA",
        "/Volumes/RICOH THETA Z1",
        "/Volumes/NO NAME",
    ];

    for path_str in potential_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            info!("Found potential RICOH mount at: {}", path.display());
            match list_files_recursive(&path, &path) {
                Ok(files) if !files.is_empty() => {
                    info!("Found {} files on device at {}", files.len(), path.display());
                    return (Some(path.clone()), files, None);
                }
                Ok(_) => {
                    info!("No files found at {}", path.display());
                }
                Err(e) => {
                    info!("Error accessing {}: {}", path.display(), e);
                }
            }
        }
    }

    let error_msg = "Camera found but no mounted storage device detected. Please ensure the camera is in the correct USB mode (usually 'Mass Storage' or 'File Transfer' mode).".to_string();
    error!("{}", error_msg);
    (None, Vec::new(), Some(error_msg))
}

pub fn guess_content_type(path: &PathBuf) -> String {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("mov") => "video/quicktime".to_string(),
        Some("dng") => "image/x-adobe-dng".to_string(),
        Some("raw") => DEFAULT_CONTENT_TYPE.to_string(),
        Some("insp") => "image/jpeg".to_string(),
        Some("insv") => "video/mp4".to_string(),
        _ => DEFAULT_CONTENT_TYPE.to_string(),
    }
}

fn list_files_recursive(base_path: &PathBuf, current_path: &PathBuf) -> std::io::Result<Vec<CameraFile>> {
    let mut files = Vec::new();

    let entries = fs::read_dir(current_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            match list_files_recursive(base_path, &path) {
                Ok(mut subfiles) => files.append(&mut subfiles),
                Err(e) => {
                    error!("Warning: Could not read directory {}: {}", path.display(), e);
                }
            }
        } else if path.is_file() {
            let metadata = entry.metadata()?;
            let relative_path = path.strip_prefix(base_path)
                .unwrap_or(&path)
                .to_path_buf();
            let filename = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content_type = guess_content_type(&path);

            files.push(CameraFile {
                path: relative_path,
                filename,
                size: metadata.len() as i64,
                content_type,
            });
        }
    }

    Ok(files)
}
