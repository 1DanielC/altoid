use crate::error::AppError;
use gphoto::{Camera, Context};
use std::path::{Path, PathBuf};

pub struct CameraInfo {
    pub mount_point: PathBuf,
    pub device_id: String,
}

pub fn find_camera() -> Result<Option<Camera>, AppError> {
    let os = std::env::consts::OS;

    match os {
        "linux" => {
            println!("Linux");
            scan_for_camera_fs_linux()?;
            Ok(None)
        }
        "windows" => {
            println!("Windows");
            scan_for_camera_fs_windows()?;
            Ok(None)
        }
        "macos" => {
            println!("MacOS");
            scan_for_camera_fs_macos()
        }
        _ => Err(AppError::UnsupportedOS(os.to_string())),
    }
}

fn scan_for_camera_fs_linux() -> Result<CameraInfo, AppError> {
    Err(AppError::UnsupportedOS("Linux camera support not implemented yet".to_string()))
}

fn scan_for_camera_fs_windows() -> Result<CameraInfo, AppError> {
    Err(AppError::UnsupportedOS("Windows camera support not implemented yet".to_string()))
}

fn scan_for_camera_fs_macos() -> Result<Option<Camera>, AppError> {
    // Try to create the context, return None if it fails
    let mut context = match Context::new() {
        Ok(ctx) => ctx,
        Err(_) => return Ok(None),
    };

    // Try to autodetect a camera, return None if it fails
    match Camera::autodetect(&mut context) {
        Ok(camera) => Ok(Some(camera)),
        Err(_) => Ok(None),
    }
}

pub fn get_camera_info(camera: &Camera) -> CameraInfo {
    unimplemented!()
}

pub fn get_camera_files(mut context: Context, mut camera: Camera) -> Result<(), AppError> {
    let storages = camera
        .storage(&mut context)
        .map_err(|e| AppError::CameraOperation(e.to_string()))?;
    let capture = camera
        .capture_image(&mut context)
        .map_err(|e| AppError::CameraOperation(e.to_string()))?;
    let file = gphoto::FileMedia::create(Path::new(&*capture.basename()))
        .map_err(|e| AppError::CameraOperation(e.to_string()))?;

    storages.iter().for_each(|storage| {
        storage.description().map(|d| println!("{}", d));
    });

    Ok(())
}
