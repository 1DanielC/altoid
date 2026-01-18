use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorType {
    Insta,
    Theta,
}

impl VendorType {
    pub fn from_vendor_id(vendor_id: u16) -> Option<Self> {
        match vendor_id {
            1802 => Some(VendorType::Insta),
            1482 => Some(VendorType::Theta),
            _ => None,
        }
    }
}

impl fmt::Display for VendorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VendorType::Insta => write!(f, "Insta"),
            VendorType::Theta => write!(f, "Theta"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Insta360OneX2,
    ThetaZ1,
}

impl DeviceType {
    pub fn from_product_id(product_id: u16) -> Option<Self> {
        match product_id {
            16422 => Some(DeviceType::Insta360OneX2),
            16423 => Some(DeviceType::Insta360OneX2),
            877 => Some(DeviceType::ThetaZ1),
            _ => None,
        }
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Insta360OneX2 => write!(f, "Insta360 One X2"),
            DeviceType::ThetaZ1 => write!(f, "Theta Z1"),
        }
    }
}