use crate::camera::sys_profiler_usb::UsbRoot;
use crate::camera::device_type::{DeviceType, VendorType};
use std::path::PathBuf;
use std::process::Command;

pub struct CameraInfo {
    pub mount_point: PathBuf,
    pub device_id: String,
}

pub fn scan_for_camera_fs() -> Option<CameraInfo> {
    let os = std::env::consts::OS;

    match os {
        "linux" => {
            println!("Linux");
            scan_for_camera_fs_linux()
        }
        "windows" => {
            println!("Windows");
            scan_for_camera_fs_windows()
        }
        "macos" => {
            println!("MacOS");
            scan_for_camera_fs_macos()
        }
        _ => panic!("Unsupported OS"),
    }
}

fn scan_for_camera_fs_linux() -> Option<CameraInfo>{
    eprintln!("Linux not supported yet");
    None
}

fn scan_for_camera_fs_windows() -> Option<CameraInfo> {
    eprintln!("Windows not supported yet");
    None
}

fn scan_for_camera_fs_macos() -> Option<CameraInfo> {
    let out = Command::new("system_profiler")
        .arg("SPUSBDataType")
        .arg("-json")
        .output()
        .expect("Failed to run system_profiler");

    let json_output: String = String::from_utf8_lossy(&out.stdout).to_string();
    let usb_root: UsbRoot = serde_json::from_str(&json_output).unwrap();

    // Get First Camera Node
    let Some(camera_node) = usb_root
        .spusb_data_type
        .iter()
        .flat_map(|bus| bus.items.iter())
        .find(|n| {
            VendorType::from_vendor_id(n.vendor_id.unwrap_or_default()).is_some()
                && DeviceType::from_product_id(n.product_id.unwrap_or_default()).is_some()
        })
    else {
        println!("No camera found");
        return None;
    };

    println!("Found Camera: {}", camera_node.name);

    // Build device ID from device type and serial number
    DeviceType::from_product_id(camera_node.product_id.unwrap_or_default())?;
    let _serial_num = camera_node.serial_num.as_deref().unwrap_or("unknown");

    // TODO: Derive the actual device ID from the camera's serial number
    // The serial number format may not match the expected device ID format
    // For now, using a mock device ID
    let device_id = "Insta360 OneX2:sn:INSXECAFEBEEF".to_string();

    // Get First Volume with a mount point
    if let Some(mount_point) = camera_node
        .media
        .iter()
        .flat_map(|m| m.iter())
        .flat_map(|m| m.volumes.iter())
        .flat_map(|v| v.iter())
        .find(|v| v.mount_point.is_some())
        .map(|v| v.mount_point.as_deref())
    {
        if let Some(m) = mount_point {
            println!("Found Volume: {}", m);
            println!("Device ID: {}", device_id);

            Some(CameraInfo {
                mount_point: PathBuf::from(m),
                device_id,
            })
        } else {
            None
        }
    } else {
        None
    }
}
