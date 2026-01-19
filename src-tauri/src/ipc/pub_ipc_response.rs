use crate::error::AppError;
use crate::traits::traits::ToJson;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq, EnumString)]
pub enum IpcStatus {
    Ok,
    Accepted,
    InvalidArgument,
    Conflict,
    NotAuthenticated,
    NotAuthorized,
    NotFound,
    ImATeapot,
    Unavailable,
    InternalError,
}

impl IpcStatus {
    /// Convert any error into an IpcStatus by checking its concrete type
    /// This handles downcasting from trait objects
    pub fn from_error(error: &(dyn std::error::Error + 'static)) -> Self {
        // Try to downcast to known error types
        if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
            return Self::from(io_err.kind());
        }

        // Add more error type checks here as you encounter them
        // if let Some(custom_err) = error.downcast_ref::<YourCustomError>() {
        //     return match custom_err {
        //         YourCustomError::NotFound => IpcStatus::NotFound,
        //         ...
        //     };
        // }

        // Check the error chain (source errors)
        if let Some(source) = error.source() {
            if let Some(io_err) = source.downcast_ref::<std::io::Error>() {
                return Self::from(io_err.kind());
            }
        }

        // Default fallback
        IpcStatus::InternalError
    }

    pub fn default_message(&self) -> &'static str {
        match self {
            IpcStatus::Ok => "Ok",
            IpcStatus::Accepted => "Accepted",
            IpcStatus::InvalidArgument => "Invalid argument",
            IpcStatus::Conflict => "Conflict",
            IpcStatus::NotAuthenticated => "User not logged in",
            IpcStatus::NotAuthorized => "User not authorized",
            IpcStatus::NotFound => "Resource not found",
            IpcStatus::ImATeapot => "🫖",
            IpcStatus::Unavailable => "Resource Unavailable",
            IpcStatus::InternalError => "Internal Error. Please Contact OpenSpace",
        }
    }
}

impl From<std::io::ErrorKind> for IpcStatus {
    fn from(kind: std::io::ErrorKind) -> Self {
        match kind {
            std::io::ErrorKind::NotFound => IpcStatus::NotFound,
            std::io::ErrorKind::PermissionDenied => IpcStatus::NotAuthorized,
            std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::ConnectionReset => {
                IpcStatus::Unavailable
            }
            _ => IpcStatus::InternalError,
        }
    }
}

impl From<std::io::Error> for IpcStatus {
    fn from(error: std::io::Error) -> Self {
        Self::from(error.kind())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub status: IpcStatus,
    pub body: Value,
}

impl IpcResponse {
    pub fn new(status: IpcStatus, body: Value) -> Self {
        Self {
            status,
            body,
        }
    }

    pub fn new_message(status: IpcStatus, message: String) -> Self {
        Self::new(status, ErrorMessageWrapper::from(message).to_json().unwrap())
    }

    /// Create from AppError - primary conversion path.
    ///
    /// This is the standard way to convert errors to IpcResponse.
    /// It uses the error's `to_ipc_status()` method to determine the appropriate status.
    pub fn from_app_error(error: AppError) -> Self {
        let status = error.to_ipc_status();
        let message = error.to_string();

        Self {
            status,
            body: ErrorMessageWrapper::from(message)
                .to_json()
                .expect("Failed to serialize error message"),
        }
    }

    /// Create from IpcError - for special cases where status is pre-determined.
    ///
    /// Use this when you have an IpcError (rare case) that already has
    /// its status explicitly set.
    pub fn from_ipc_error(error: crate::ipc::ipc_error::IpcError) -> Self {
        error.to_response()
    }

    /// Create from any error type.
    ///
    /// **Deprecated**: Use `from_app_error()` or `from_ipc_error()` instead.
    /// This method is kept for backward compatibility during the transition.
    #[deprecated(note = "Use from_app_error or from_ipc_error instead")]
    pub fn from_error<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        // Classify the error type
        let error_type = IpcStatus::from_error(&error);

        // Build the struct
        Self {
            status: error_type,
            body: ErrorMessageWrapper::from(error.to_string())
                .to_json()
                .expect("Failed to serialize error message"),
        }
    }

    /// Create with error type and default message
    pub fn from_type(status: IpcStatus) -> Self {
        Self {
            status,
            body: serde_json::to_value(ErrorMessageWrapper::from(
                status.default_message().to_string(),
            ))
            .expect("Failed to serialize error message"),
        }
    }
}

/// Helper trait for clean conversion at the lib.rs boundary.
///
/// This trait provides a uniform way to convert errors to IpcResponse
/// regardless of whether they're AppError or IpcError.
pub trait ToIpcResponse {
    fn to_ipc_response(self) -> IpcResponse;
}

impl ToIpcResponse for AppError {
    fn to_ipc_response(self) -> IpcResponse {
        IpcResponse::from_app_error(self)
    }
}

impl ToIpcResponse for crate::ipc::ipc_error::IpcError {
    fn to_ipc_response(self) -> IpcResponse {
        IpcResponse::from_ipc_error(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMessageWrapper {
    pub message: String,
}

impl From<String> for ErrorMessageWrapper {
    fn from(msg: String) -> Self {
        ErrorMessageWrapper { message: msg }
    }
}
