use serde::{Deserialize, Deserializer, Serialize};
use regex::Regex;

fn parse_hex_u16<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        let bit_regex = Regex::new(r"0x[0-9a-fA-F]+").unwrap();

        // Keep only first token (e.g. "0x070a" from "0x070a  (Vendor...)")
        let trimmed = s.split_whitespace().next().unwrap_or(&s);

        // Try regex first, fallback to raw trimmed string
        let num_str = bit_regex
            .find(trimmed)
            .map(|m| m.as_str())
            .unwrap_or(trimmed)
            .trim_start_matches("0x")
            .trim_start_matches("0X");

        match u16::from_str_radix(num_str, 16) {
            Ok(n) => Ok(Some(n)),
            Err(_) => Ok(None), // ← gracefully return None on parse failure
        }
    } else {
        Ok(None)
    }
}

/**
 * Data model for the output of the sys_profiler_usb tool for MacOS
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct UsbRoot {
    #[serde(rename = "SPUSBDataType")]
    pub spusb_data_type: Vec<UsbBus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsbBus {
    #[serde(rename = "_items", default)]
    pub items: Vec<UsbNode>,
    #[serde(rename = "_name")]
    pub name: String,
    #[serde(default)]
    pub host_controller: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsbNode {
    // Common fields present for hubs and devices
    #[serde(rename = "_name")]
    pub name: String,

    #[serde(default)]
    pub bcd_device: Option<String>,
    #[serde(default)]
    pub location_id: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,

    #[serde(default, deserialize_with = "parse_hex_u16")]
    pub product_id: Option<u16>,
    #[serde(default)]
    pub serial_num: Option<String>,

    #[serde(default, deserialize_with = "parse_hex_u16")]
    pub vendor_id: Option<u16>,

    // If this node is a hub, it may have nested children
    #[serde(rename = "_items", default)]
    pub items: Option<Vec<UsbNode>>,

    // If this node represents a device with media
    #[serde(rename = "Media", default)]
    pub media: Option<Vec<UsbMedia>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsbMedia {
    #[serde(rename = "_name")]
    pub name: String,

    #[serde(default)]
    pub bsd_name: Option<String>,

    #[serde(rename = "USB Interface", default)]
    pub usb_interface: Option<u64>,

    #[serde(default)]
    pub volumes: Option<Vec<UsbVolume>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsbVolume {
    #[serde(rename = "_name")]
    pub name: String,

    #[serde(default)]
    pub bsd_name: Option<String>,
    #[serde(rename = "file_system", default)]
    pub file_system: Option<String>,

    #[serde(default)]
    pub iocontent: Option<String>,

    #[serde(rename = "mount_point", default)]
    pub mount_point: Option<String>,

    // Human-readable size like "255.84 GB"
    #[serde(rename = "size", default)]
    pub size: Option<String>,

    #[serde(rename = "size_in_bytes", default)]
    pub size_in_bytes: Option<u64>,

    #[serde(rename = "volume_uuid", default)]
    pub volume_uuid: Option<String>,
}
