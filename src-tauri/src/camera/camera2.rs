use crate::camera::device_type::{CameraInfo, CAMERAS};
use serde::Serialize;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::ptr;

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

                // Try to find the camera using gphoto2
                let (mount_point, files, access_error) = find_camera_files_gphoto2();

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

fn find_camera_files_gphoto2() -> (Option<PathBuf>, Vec<PathBuf>, Option<String>) {
    println!("Attempting PTP camera access via gphoto2-sys...");

    unsafe {
        // Initialize context
        let context = gphoto2_sys::gp_context_new();
        if context.is_null() {
            let error_msg = "Failed to create gphoto2 context".to_string();
            eprintln!("{}", error_msg);
            return (None, Vec::new(), Some(error_msg));
        }

        // Initialize camera
        let mut camera: *mut gphoto2_sys::Camera = ptr::null_mut();
        let ret = gphoto2_sys::gp_camera_new(&mut camera);
        if ret != gphoto2_sys::GP_OK {
            let error_msg = format!("Failed to create camera object: {}", get_error_string(ret));
            eprintln!("{}", error_msg);
            gphoto2_sys::gp_context_unref(context);
            return (None, Vec::new(), Some(error_msg));
        }

        // Initialize camera (autodetect and connect)
        let ret = gphoto2_sys::gp_camera_init(camera, context);
        if ret != gphoto2_sys::GP_OK {
            let error_msg = format!("Failed to initialize camera: {}", get_error_string(ret));
            eprintln!("{}", error_msg);
            gphoto2_sys::gp_camera_unref(camera);
            gphoto2_sys::gp_context_unref(context);
            return (None, Vec::new(), Some(error_msg));
        }

        println!("Successfully connected to camera via gphoto2-sys");

        // List files on the camera
        let files = match list_files_recursive(camera, context, "/") {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Error listing files: {}", e);
                gphoto2_sys::gp_camera_exit(camera, context);
                gphoto2_sys::gp_camera_unref(camera);
                gphoto2_sys::gp_context_unref(context);
                return (Some(PathBuf::from("PTP")), Vec::new(), Some(e));
            }
        };

        // Clean up
        gphoto2_sys::gp_camera_exit(camera, context);
        gphoto2_sys::gp_camera_unref(camera);
        gphoto2_sys::gp_context_unref(context);

        if files.is_empty() {
            let error_msg = "Camera connected via PTP but no files found".to_string();
            eprintln!("{}", error_msg);
            (Some(PathBuf::from("PTP")), Vec::new(), Some(error_msg))
        } else {
            println!("Found {} files via PTP", files.len());
            (Some(PathBuf::from("PTP")), files, None)
        }
    }
}

unsafe fn list_files_recursive(
    camera: *mut gphoto2_sys::Camera,
    context: *mut gphoto2_sys::GPContext,
    folder: &str,
) -> Result<Vec<PathBuf>, String> {
    let mut all_files = Vec::new();

    let folder_cstr = CString::new(folder).map_err(|e| format!("Invalid folder path: {}", e))?;

    // Create a file list for files
    let mut file_list: *mut gphoto2_sys::CameraList = ptr::null_mut();
    let ret = gphoto2_sys::gp_list_new(&mut file_list);
    if ret != gphoto2_sys::GP_OK {
        return Err(format!("Failed to create file list: {}", get_error_string(ret)));
    }

    // List files in the current folder
    let ret = gphoto2_sys::gp_camera_folder_list_files(
        camera,
        folder_cstr.as_ptr(),
        file_list,
        context,
    );

    if ret == gphoto2_sys::GP_OK {
        // Get the number of files
        let file_count = gphoto2_sys::gp_list_count(file_list);

        for i in 0..file_count {
            let mut name_ptr: *const libc::c_char = ptr::null();
            let ret = gphoto2_sys::gp_list_get_name(file_list, i, &mut name_ptr);

            if ret == gphoto2_sys::GP_OK && !name_ptr.is_null() {
                if let Ok(name) = CStr::from_ptr(name_ptr).to_str() {
                    let file_path = if folder == "/" {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", folder, name)
                    };
                    all_files.push(PathBuf::from(file_path));
                }
            }
        }
    }

    gphoto2_sys::gp_list_unref(file_list);

    // Create a folder list for subdirectories
    let mut folder_list: *mut gphoto2_sys::CameraList = ptr::null_mut();
    let ret = gphoto2_sys::gp_list_new(&mut folder_list);
    if ret != gphoto2_sys::GP_OK {
        return Err(format!("Failed to create folder list: {}", get_error_string(ret)));
    }

    // List folders in the current folder
    let ret = gphoto2_sys::gp_camera_folder_list_folders(
        camera,
        folder_cstr.as_ptr(),
        folder_list,
        context,
    );

    if ret == gphoto2_sys::GP_OK {
        // Get the number of folders
        let folder_count = gphoto2_sys::gp_list_count(folder_list);

        for i in 0..folder_count {
            let mut name_ptr: *const libc::c_char = ptr::null();
            let ret = gphoto2_sys::gp_list_get_name(folder_list, i, &mut name_ptr);

            if ret == gphoto2_sys::GP_OK && !name_ptr.is_null() {
                if let Ok(name) = CStr::from_ptr(name_ptr).to_str() {
                    let subfolder_path = if folder == "/" {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", folder, name)
                    };

                    // Recursively list files in the subfolder
                    match list_files_recursive(camera, context, &subfolder_path) {
                        Ok(mut subfiles) => all_files.append(&mut subfiles),
                        Err(e) => {
                            eprintln!("Warning: Could not list files in subfolder {}: {}", subfolder_path, e);
                        }
                    }
                }
            }
        }
    }

    gphoto2_sys::gp_list_unref(folder_list);

    Ok(all_files)
}

unsafe fn get_error_string(error_code: i32) -> String {
    let error_ptr = gphoto2_sys::gp_result_as_string(error_code);
    if error_ptr.is_null() {
        return format!("Unknown error ({})", error_code);
    }

    match CStr::from_ptr(error_ptr).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => format!("Error code: {}", error_code),
    }
}
