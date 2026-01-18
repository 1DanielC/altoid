use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TicTacUploadRequest {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceFilename")]
    pub device_filename: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "size")]
    pub size: i64,
    #[serde(rename = "numParts")]
    pub num_parts: i32,
}

impl TicTacUploadRequest {
    pub fn new(
        device_id: String,
        device_filename: String,
        content_type: String,
        size: i64,
        num_parts: i32,
    ) -> Self {
        Self {
            device_id,
            device_filename,
            content_type,
            size,
            num_parts,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrCreateUploadResponse {
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
}

