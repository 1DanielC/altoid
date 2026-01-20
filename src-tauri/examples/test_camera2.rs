// Test program to verify camera2 detection using gphoto2-sys
fn main() {
    println!("Testing camera detection with gphoto2-sys (camera2.rs)...\n");

    match altoid_lib::camera::camera2::find_camera() {
        Some(camera_with_files) => {
            println!("Camera detected:");
            println!("  Device: {}", camera_with_files.info.device);
            println!("  Vendor: {}", camera_with_files.info.vendor);
            println!("  Vendor ID: {}", camera_with_files.info.vendor_id);
            println!("  Mount point: {:?}", camera_with_files.mount_point);
            println!("  Files found: {}", camera_with_files.files.len());

            if let Some(error) = &camera_with_files.access_error {
                println!("  Access error: {}", error);
            }

            if !camera_with_files.files.is_empty() {
                println!("\nFirst 10 files:");
                for (i, file) in camera_with_files.files.iter().take(10).enumerate() {
                    println!("  {}. {}", i + 1, file.display());
                }
            }
        }
        None => {
            println!("No camera detected");
        }
    }
}
