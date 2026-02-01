use tauri_plugin_log::log::info;

// Test program to verify camera2 detection using gphoto2-sys
fn main() {
    info!("Testing camera detection with gphoto2-sys (camera2.rs)...\n");

    match altoid_lib::camera::camera2::find_camera() {
        Some(camera_with_files) => {
            info!("Camera detected:");
            info!("  Device: {}", camera_with_files.info.device);
            info!("  Vendor: {}", camera_with_files.info.vendor);
            info!("  Vendor ID: {}", camera_with_files.info.vendor_id);
            info!("  Mount point: {:?}", camera_with_files.mount_point);
            info!("  Files found: {}", camera_with_files.files.len());

            if let Some(error) = &camera_with_files.access_error {
                info!("  Access error: {}", error);
            }

            if !camera_with_files.files.is_empty() {
                info!("\nFirst 10 files:");
                for (i, file) in camera_with_files.files.iter().take(10).enumerate() {
                    info!("  {}. {}", i + 1, file.display());
                }
            }
        }
        None => {
            info!("No camera detected");
        }
    }
}
