use crate::camera::device_type::{CameraInfo, CAMERAS};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct CameraWithFiles {
    pub info: &'static CameraInfo,
    pub mount_point: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub access_error: Option<String>,
}

pub fn find_camera() -> Option<CameraWithFiles> {
    // Attempt to enumerate USB devices
    let devices = match rusb::devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("Failed to enumerate USB devices: {}", e);
            return None;
        }
    };

    // Iterate through all connected USB devices
    for device in devices.iter() {
        if let Ok(desc) = device.device_descriptor() {
            let vendor_id = desc.vendor_id();

            // Check if this vendor ID matches any camera in our CAMERAS map
            if let Some(camera_info) = CAMERAS.get(&vendor_id) {
                println!("Found camera: {} (Vendor ID: {})", camera_info.device, vendor_id);

                // Try to find the mounted storage device and list files
                let (mount_point, files, access_error) = find_camera_files();

                return Some(CameraWithFiles {
                    info: camera_info,
                    mount_point,
                    files,
                    access_error,
                });
            }
        }
    }

    // No matching camera found
    eprintln!("No supported camera found connected via USB");
    None
}

fn find_camera_files_ptp() -> (Option<PathBuf>, Vec<PathBuf>, Option<String>) {
    println!("Attempting PTP camera access via gphoto2 CLI...");

    // First, check if gphoto2 is available
    let check_gphoto2 = Command::new("which")
        .arg("gphoto2")
        .output();

    if check_gphoto2.is_err() || !check_gphoto2.as_ref().unwrap().status.success() {
        let error_msg = "gphoto2 CLI not found. Please install it with: brew install gphoto2".to_string();
        eprintln!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    // Try to detect cameras
    let detect_output = Command::new("gphoto2")
        .arg("--auto-detect")
        .output();

    if let Err(e) = detect_output {
        let error_msg = format!("Failed to run gphoto2 --auto-detect: {}", e);
        eprintln!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    let detect_output = detect_output.unwrap();
    let detect_stdout = String::from_utf8_lossy(&detect_output.stdout);

    // Check if any camera was detected
    if !detect_stdout.contains("usb:") {
        let error_msg = "No PTP camera detected by gphoto2".to_string();
        eprintln!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    println!("Camera detected via gphoto2: {}", detect_stdout.trim());

    // List files on the camera
    let list_output = Command::new("gphoto2")
        .arg("--list-files")
        .output();

    if let Err(e) = list_output {
        let error_msg = format!("Failed to run gphoto2 --list-files: {}", e);
        eprintln!("{}", error_msg);
        return (None, Vec::new(), Some(error_msg));
    }

    let list_output = list_output.unwrap();
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);

    // Parse file list from gphoto2 output
    let files = parse_gphoto2_file_list(&list_stdout);

    if files.is_empty() {
        let error_msg = "Camera connected via PTP but no files found".to_string();
        eprintln!("{}", error_msg);
        (Some(PathBuf::from("PTP")), Vec::new(), Some(error_msg))
    } else {
        println!("Found {} files via PTP", files.len());
        (Some(PathBuf::from("PTP")), files, None)
    }
}

fn parse_gphoto2_file_list(output: &str) -> Vec<PathBuf> {
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
        else if line.starts_with('#') {
            // Parse file line format: #1     R0010001.JPG               rd  8367 KB ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let filename = parts[1];
                let file_path = if current_folder.is_empty() {
                    PathBuf::from(filename)
                } else {
                    PathBuf::from(format!("{}/{}", current_folder, filename))
                };
                files.push(file_path);
            }
        }
    }

    files
}

fn find_camera_files() -> (Option<PathBuf>, Vec<PathBuf>, Option<String>) {
    // First, try PTP access via gphoto2
    let (mount_point, files, error) = find_camera_files_ptp();
    if mount_point.is_some() && !files.is_empty() {
        return (mount_point, files, error);
    }

    // If PTP didn't work, fall back to mass storage detection
    println!("PTP access failed or no files found, trying mass storage detection...");

    // Try to enumerate mounted drives
    let drives = match bb_drivelist::drive_list() {
        Ok(drives) => drives,
        Err(e) => {
            let error_msg = format!("Failed to enumerate drives: {}", e);
            eprintln!("{}", error_msg);
            return (None, Vec::new(), Some(error_msg));
        }
    };

    println!("Total drives detected: {}", drives.len());

    // Look for removable drives (cameras typically mount as removable storage)
    for drive in &drives {
        println!("Drive: device={}, is_removable={}, mountpoints={}",
                 drive.device, drive.is_removable, drive.mountpoints.len());

        for mp in &drive.mountpoints {
            println!("  Mountpoint: {}", mp.path);
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
            println!("Checking removable drive at: {}", path.display());

            match list_files_recursive(&path, &path) {
                Ok(files) if !files.is_empty() => {
                    println!("Found {} files on device at {}", files.len(), path.display());
                    return (Some(path.clone()), files, None);
                }
                Ok(_) => {
                    println!("No files found at {}", path.display());
                }
                Err(e) => {
                    let error_msg = format!(
                        "Cannot access files at {}. Error: {}. \
                        On macOS, you may need to grant 'Full Disk Access' permission to your terminal or app in System Settings > Privacy & Security > Full Disk Access.",
                        path.display(),
                        e
                    );
                    eprintln!("{}", error_msg);
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
            println!("Found potential RICOH mount at: {}", path.display());
            match list_files_recursive(&path, &path) {
                Ok(files) if !files.is_empty() => {
                    println!("Found {} files on device at {}", files.len(), path.display());
                    return (Some(path.clone()), files, None);
                }
                Ok(_) => {
                    println!("No files found at {}", path.display());
                }
                Err(e) => {
                    eprintln!("Error accessing {}: {}", path.display(), e);
                }
            }
        }
    }

    let error_msg = "Camera found but no mounted storage device detected. Please ensure the camera is in the correct USB mode (usually 'Mass Storage' or 'File Transfer' mode).".to_string();
    eprintln!("{}", error_msg);
    (None, Vec::new(), Some(error_msg))
}

fn list_files_recursive(base_path: &PathBuf, current_path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let entries = fs::read_dir(current_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively list files in subdirectories
            match list_files_recursive(base_path, &path) {
                Ok(mut subfiles) => files.append(&mut subfiles),
                Err(e) => {
                    eprintln!("Warning: Could not read directory {}: {}", path.display(), e);
                    // Continue with other directories
                }
            }
        } else if path.is_file() {
            // Store relative path from base mount point
            if let Ok(relative_path) = path.strip_prefix(base_path) {
                files.push(relative_path.to_path_buf());
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}