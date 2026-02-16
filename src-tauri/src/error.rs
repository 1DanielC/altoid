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
//!
//! ## Error Chaining
//! Most error variants capture their source error via `#[source]`, enabling
//! full error chain traversal and stack traces when logging with `{:?}`.

use crate::ipc::pub_ipc_response::IpcStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // Authentication errors
    #[error("OAuth configuration not found. Please run login first.")]
    OAuthConfigNotFound,

    #[error("Authentication failed: {message}")]
    AuthenticationFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Device code expired. Please try again.")]
    DeviceCodeExpired,

    #[error("Not authenticated. Please log in.")]
    NotAuthenticated,

    // API errors
    #[error("API request failed: {status} - {message}")]
    ApiRequest { status: u16, message: String },

    #[error("API not initialized. Please authenticate first.")]
    ApiNotInitialized,

    #[error("Failed to parse API response: {message}")]
    ApiParseFailed {
        message: String,
        #[source]
        source: reqwest::Error,
    },

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

    #[error("Camera operation failed: {message}")]
    CameraOperation {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Resource conflict: {message}")]
    Conflict {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // Network errors
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Network timeout")]
    NetworkTimeout,

    // I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // URL parsing errors
    #[error("URL parse error: {message}")]
    UrlParse {
        message: String,
        #[source]
        source: url::ParseError,
    },

    // Generic errors with context
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Could not acquire lock for resource")]
    LockingError,

    // Serialization errors
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("Unsupported OS: {0}")]
    UnsupportedOS(String),

    // Upload errors
    #[error("Upload failed: {message}")]
    UploadFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

// Helper constructors for ergonomic error creation
impl AppError {
    /// Create an AuthenticationFailed error without a source
    pub fn auth_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create an AuthenticationFailed error with a source
    pub fn auth_failed_with<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::AuthenticationFailed {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an Internal error without a source
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create an Internal error with a source
    pub fn internal_with<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an InvalidArgument error without a source
    pub fn invalid_arg(message: impl Into<String>) -> Self {
        Self::InvalidArgument {
            message: message.into(),
            source: None,
        }
    }

    /// Create an InvalidArgument error with a source
    pub fn invalid_arg_with<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::InvalidArgument {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a UrlParse error
    pub fn url_parse(message: impl Into<String>, source: url::ParseError) -> Self {
        Self::UrlParse {
            message: message.into(),
            source,
        }
    }

    /// Create an ApiParseFailed error
    pub fn api_parse_failed(message: impl Into<String>, source: reqwest::Error) -> Self {
        Self::ApiParseFailed {
            message: message.into(),
            source,
        }
    }

    /// Create a CameraOperation error without a source
    pub fn camera_op(message: impl Into<String>) -> Self {
        Self::CameraOperation {
            message: message.into(),
            source: None,
        }
    }

    /// Create a CameraOperation error with a source
    pub fn camera_op_with<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::CameraOperation {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an UploadFailed error without a source
    pub fn upload_failed(message: impl Into<String>) -> Self {
        Self::UploadFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create an UploadFailed error with a source
    pub fn upload_failed_with<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::UploadFailed {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a Conflict error without a source
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
            source: None,
        }
    }

    /// Create a Conflict error with a source
    pub fn conflict_with<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Conflict {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
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
            | Self::AuthenticationFailed { .. }
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
            Self::InvalidArgument { .. } => IpcStatus::InvalidArgument,

            // Conflict errors
            Self::Conflict { .. } => IpcStatus::Conflict,

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
