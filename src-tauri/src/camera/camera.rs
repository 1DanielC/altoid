use crate::camera::device_type::{CameraInfo, CAMERAS};

pub fn find_camera() -> Option<&'static CameraInfo> {
    Some(CAMERAS.get(&1482).unwrap())
}