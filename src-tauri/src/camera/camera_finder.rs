use gphoto::{Camera, Context};
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct CameraInfo {
    pub mount_point: PathBuf,
    pub device_id: String,
}

pub fn find_camera() -> Option<()> {
    let os = std::env::consts::OS;

    match os {
        "linux" => {
            println!("Linux");
            scan_for_camera_fs_linux();
            Some(())
        }
        "windows" => {
            println!("Windows");
            scan_for_camera_fs_windows();
            Some(())
        }
        "macos" => {
            println!("MacOS");
            let res = scan_for_camera_fs_macos();
            if let Err(e) = res {
                eprintln!("Error: {}", e);
            }
            Some(())
        }
        _ => panic!("Unsupported OS"),
    }
}

fn scan_for_camera_fs_linux() -> Option<CameraInfo> {
    eprintln!("Linux not supported yet");
    None
}

fn scan_for_camera_fs_windows() -> Option<CameraInfo> {
    eprintln!("Windows not supported yet");
    None
}

fn scan_for_camera_fs_macos() -> Result<(), String> {
    let mut context = Context::new().map_err(|e| e.to_string())?;

    // Use a Result to avoid crashing if initialization fails
    let camera_result = Camera::autodetect(&mut context);

    match camera_result {
        Ok(mut camera) => {
            let storages = camera.storage(&mut context).map_err(|e| e.to_string())?;
            let capture = camera.capture_image(&mut context).unwrap();
            let file = gphoto::FileMedia::create(Path::new(&*capture.basename())).unwrap();
            storages.iter().for_each(|storage| {
                storage.description().map(|d| println!("{}", d));
                println!("POOP")
            });
        }
        Err(e) => {
            eprintln!("Could not open camera: {:?}", e);
            return Err("Camera unavailable (possibly claimed by another app)".into());
        }
    }

    Ok(())
}
