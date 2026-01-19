//! Unified error handling for the Tauri application.
//!
//! All internal functions should return `Result<T, AppError>`.
//! At the Tauri command boundary (lib.rs), errors are converted to
//! `IpcResponse` and serialized to JSON for the frontend.
//!
//! ## Error Flow
//! ```text
//! Internal Function → Result<T, AppError>
//!         ↓
//! Tauri Command (lib.rs) → Convert to IpcResponse
//!         ↓
//! Frontend ← JSON with {status, body}
//! ```
//!
//! ## Special Case: IpcError
//! In rare cases where the status code must be explicitly set
//! (e.g., treating a 401 as success with None), use `IpcError`.
//! This prevents double-wrapping at the command boundary.

use crate::ipc::pub_ipc_response::IpcStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // Authentication errors
    #[error("OAuth configuration not found. Please run login first.")]
    OAuthConfigNotFound,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Device code expired. Please try again.")]
    DeviceCodeExpired,

    #[error("Not authenticated. Please log in.")]
    NotAuthenticated,

    // API errors
    #[error("API request failed: {status} - {message}")]
    ApiRequest { status: u16, message: String },

    #[error("API not initialized. Please authenticate first.")]
    ApiNotInitialized,

    #[error("Failed to parse API response: {0}")]
    ApiParseFailed(String),

    // Cache errors
    #[error("Failed to read cache file '{file}': {source}")]
    CacheRead {
        file: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write cache file '{file}': {source}")]
    CacheWrite {
        file: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Cache file not found: {0}")]
    CacheNotFound(String),

    // Camera errors
    #[error("No camera found")]
    CameraNotFound,

    #[error("Camera unavailable (possibly claimed by another app)")]
    CameraUnavailable,

    #[error("Camera operation failed: {0}")]
    CameraOperation(String),

    #[error("Unsupported OS: {0}")]
    UnsupportedOS(String),

    // Upload errors
    #[error("Upload failed: {0}")]
    UploadFailed(String),

    // Network errors
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Network timeout")]
    NetworkTimeout,

    // Serialization errors
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    // I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // Generic errors with context
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Resource conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Maps AppError variants to appropriate IpcStatus codes for the frontend.
    ///
    /// This method determines how errors are presented to the user by
    /// classifying them into meaningful status codes.
    pub fn to_ipc_status(&self) -> IpcStatus {
        match self {
            // Authentication/Authorization
            Self::NotAuthenticated
            | Self::OAuthConfigNotFound
            | Self::AuthenticationFailed(_)
            | Self::DeviceCodeExpired => IpcStatus::NotAuthenticated,

            // API errors with status codes
            Self::ApiRequest { status, .. } => match *status {
                401 => IpcStatus::NotAuthenticated,
                403 => IpcStatus::NotAuthorized,
                404 => IpcStatus::NotFound,
                409 => IpcStatus::Conflict,
                418 => IpcStatus::ImATeapot,
                503 => IpcStatus::Unavailable,
                400..=499 => IpcStatus::InvalidArgument,
                _ => IpcStatus::InternalError,
            },

            // Not found errors
            Self::CacheNotFound(_) | Self::CameraNotFound => IpcStatus::NotFound,

            // Validation errors
            Self::InvalidArgument(_) => IpcStatus::InvalidArgument,

            // Conflict errors
            Self::Conflict(_) => IpcStatus::Conflict,

            // Unavailable errors
            Self::ApiNotInitialized | Self::CameraUnavailable | Self::NetworkTimeout => {
                IpcStatus::Unavailable
            }

            // Network errors - check error type
            Self::Network(e) => {
                if e.is_timeout() {
                    IpcStatus::Unavailable
                } else if e.is_connect() {
                    IpcStatus::Unavailable
                } else {
                    IpcStatus::InternalError
                }
            }

            // I/O errors - delegate to existing From<io::ErrorKind>
            Self::Io(e) => IpcStatus::from(e.kind()),
            Self::CacheRead { source, .. } | Self::CacheWrite { source, .. } => {
                IpcStatus::from(source.kind())
            }

            // Everything else is internal
            _ => IpcStatus::InternalError,
        }
    }
}
