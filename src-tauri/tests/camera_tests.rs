use std::path::PathBuf;
use altoid_lib::camera::camera::{guess_content_type, parse_gphoto2_file_list};
use altoid_lib::camera::device_type::*;

// --- content type detection ---

#[test]
fn guess_content_type_images() {
    assert_eq!(guess_content_type(&PathBuf::from("photo.jpg")), "image/jpeg");
    assert_eq!(guess_content_type(&PathBuf::from("photo.JPG")), "image/jpeg");
    assert_eq!(guess_content_type(&PathBuf::from("photo.jpeg")), "image/jpeg");
    assert_eq!(guess_content_type(&PathBuf::from("photo.png")), "image/png");
    assert_eq!(guess_content_type(&PathBuf::from("photo.dng")), "image/x-adobe-dng");
}

#[test]
fn guess_content_type_videos() {
    assert_eq!(guess_content_type(&PathBuf::from("video.mp4")), "video/mp4");
    assert_eq!(guess_content_type(&PathBuf::from("video.mov")), "video/quicktime");
}

#[test]
fn guess_content_type_camera_specific() {
    assert_eq!(guess_content_type(&PathBuf::from("IMG_001.insp")), "image/jpeg");
    assert_eq!(guess_content_type(&PathBuf::from("VID_001.insv")), "video/mp4");
}

#[test]
fn guess_content_type_unknown() {
    assert_eq!(guess_content_type(&PathBuf::from("file.xyz")), "application/octet-stream");
    assert_eq!(guess_content_type(&PathBuf::from("file.raw")), "application/octet-stream");
    assert_eq!(guess_content_type(&PathBuf::from("noext")), "application/octet-stream");
}

// --- gphoto2 output parsing ---

#[test]
fn parse_gphoto2_empty_output() {
    let files = parse_gphoto2_file_list("");
    assert!(files.is_empty());
}

#[test]
fn parse_gphoto2_no_files_message() {
    let output = "There are 0 files in folder '/store_00020001/DCIM/100RICOH'.";
    let files = parse_gphoto2_file_list(output);
    assert!(files.is_empty());
}

#[test]
fn parse_gphoto2_single_file() {
    let output = "\
There are 1 files in folder '/store_00020001/DCIM/100RICOH'.
#1     R0010001.JPG               rd  8367 KB  image/jpeg";
    let files = parse_gphoto2_file_list(output);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "R0010001.JPG");
    assert_eq!(files[0].size, 8367 * 1024);
    assert_eq!(files[0].path, PathBuf::from("/store_00020001/DCIM/100RICOH/R0010001.JPG"));
    assert_eq!(files[0].content_type, "image/jpeg");
}

#[test]
fn parse_gphoto2_multiple_files_across_folders() {
    let output = "\
There are 2 files in folder '/store_00020001/DCIM/100RICOH'.
#1     R0010001.JPG               rd  8367 KB  image/jpeg
#2     R0010002.MP4               rd  150 MB  video/mp4
There are 1 files in folder '/store_00020001/DCIM/101RICOH'.
#3     R0010003.DNG               rd  25600 KB  image/x-adobe-dng";
    let files = parse_gphoto2_file_list(output);
    assert_eq!(files.len(), 3);

    assert_eq!(files[0].path, PathBuf::from("/store_00020001/DCIM/100RICOH/R0010001.JPG"));
    assert_eq!(files[0].size, 8367 * 1024);

    assert_eq!(files[1].filename, "R0010002.MP4");
    assert_eq!(files[1].size, 150 * 1024 * 1024);

    assert_eq!(files[2].path, PathBuf::from("/store_00020001/DCIM/101RICOH/R0010003.DNG"));
}

#[test]
fn parse_gphoto2_file_without_folder() {
    let output = "#1     standalone.jpg             rd  1024 KB  image/jpeg";
    let files = parse_gphoto2_file_list(output);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, PathBuf::from("standalone.jpg"));
}

// --- device type lookups ---

#[test]
fn camera_lookup_insta360() {
    let cam = CAMERAS.get(&1802).expect("Insta360 should be in CAMERAS map");
    assert_eq!(cam.vendor, VendorType::Insta);
    assert_eq!(cam.vendor_id, 1802);
    assert_eq!(cam.device, DeviceType::Insta360OneX2);
}

#[test]
fn camera_lookup_theta_z1() {
    let cam = CAMERAS.get(&1482).expect("THETA Z1 should be in CAMERAS map");
    assert_eq!(cam.vendor, VendorType::Theta);
    assert_eq!(cam.vendor_id, 1482);
    assert_eq!(cam.device, DeviceType::ThetaZ1);
}

#[test]
fn camera_lookup_unknown_vendor() {
    assert!(CAMERAS.get(&9999).is_none());
}

#[test]
fn vendor_type_display() {
    assert_eq!(format!("{}", VendorType::Insta), "Insta");
    assert_eq!(format!("{}", VendorType::Theta), "Theta");
}

#[test]
fn device_type_display() {
    assert_eq!(format!("{}", DeviceType::Insta360OneX2), "Insta360 One X2");
    assert_eq!(format!("{}", DeviceType::ThetaZ1), "RICOH THETA Z1");
}
