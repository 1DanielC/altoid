use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum VendorType {
    Insta,
    Theta,
}

impl fmt::Display for VendorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VendorType::Insta => write!(f, "Insta"),
            VendorType::Theta => write!(f, "Theta"),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Insta360OneX2,
    ThetaZ1,
}
impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Insta360OneX2 => write!(f, "Insta360 One X2"),
            DeviceType::ThetaZ1 => write!(f, "RICOH THETA Z1"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct CameraInfo {
    pub vendor: VendorType,
    pub vendor_id: u16,
    pub device: DeviceType,
}

pub static CAMERAS: Lazy<HashMap<u16, CameraInfo>> = Lazy::new(|| {
    HashMap::from([
        (
            1802,
            CameraInfo {
                vendor: VendorType::Insta,
                vendor_id: 1802,
                device: DeviceType::Insta360OneX2,
            },
        ),
        (
            1482,
            CameraInfo {
                vendor: VendorType::Theta,
                vendor_id: 1482,
                device: DeviceType::ThetaZ1,
            },
        ),
    ])
});
