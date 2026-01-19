use crate::error::AppError;
use crate::ipc::pub_ipc_response::{ErrorMessageWrapper, IpcResponse, IpcStatus};
use crate::traits::traits::ToJson;
use std::fmt;

/// Wrapper for errors that already have an IpcStatus determined.
///
/// ## When to use IpcError
/// IpcError should be used ONLY in rare cases where the status code must be
/// explicitly set and cannot be derived from the error type. For example:
/// - Treating a 401 error as `Ok(None)` instead of an error
/// - Custom business logic that requires specific status overrides
///
/// ## Standard usage (99% of cases)
/// Most code should use `AppError` and let the status be derived automatically
/// at the command boundary in lib.rs.
///
/// ## Design
/// This struct does NOT wrap another error - it simply stores a pre-determined
/// status and message. This prevents double-wrapping at the lib.rs boundary.
#[derive(Debug)]
pub struct IpcError {
    status: IpcStatus,
    message: String,
}

impl IpcError {
    /// Creates a new IpcError with explicit status and message.
    ///
    /// Use this when you need to override the automatic status derivation.
    pub fn new(status: IpcStatus, message: String) -> Self {
        Self { status, message }
    }

    /// Creates an IpcError from an AppError, using its derived status.
    ///
    /// This is useful when you want to convert an AppError to IpcError
    /// while preserving the automatically derived status.
    pub fn from_app_error(error: AppError) -> Self {
        Self {
            status: error.to_ipc_status(),
            message: error.to_string(),
        }
    }

    /// Converts this IpcError to an IpcResponse for sending to the frontend.
    pub fn to_response(self) -> IpcResponse {
        IpcResponse::new(
            self.status,
            ErrorMessageWrapper::from(self.message)
                .to_json()
                .expect("Failed to serialize error message"),
        )
    }

    /// Returns the IpcStatus of this error.
    pub fn status(&self) -> IpcStatus {
        self.status
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for IpcError {}

// Allow converting AppError to IpcError
impl From<AppError> for IpcError {
    fn from(error: AppError) -> Self {
        Self::from_app_error(error)
    }
}
