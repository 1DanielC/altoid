use crate::error::AppError;
use std::sync::{Mutex, MutexGuard};
use tauri_plugin_log::log::error;

/// Extension trait for Option to convert to Result<AppError> + log
pub trait OptionExt<T> {
    fn ok_or_app_err(self, context: &'static str) -> Result<T, AppError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_app_err(self, context: &'static str) -> Result<T, AppError> {
        self.ok_or({
            error!("{}: None encountered", context);
            AppError::internal(context)
        })
    }
}

pub trait ResultExt<T> {
    fn map_app_err(self, context: &'static str) -> Result<T, AppError>;
}

impl<T> ResultExt<T> for Result<T, AppError> {
    fn map_app_err(self, context: &'static str) -> Result<T, AppError> {
        self.map_err(|e| {
            error!("{context}");
            e
        })
    }
}

pub trait LockExt<T> {
    fn lock_or_err(&self) -> Result<MutexGuard<'_, Option<T>>, AppError>;
}

impl<T> LockExt<T> for Mutex<Option<T>> {
    fn lock_or_err(&self) -> Result<MutexGuard<'_, Option<T>>, AppError> {
        self.lock().map_err(|_| AppError::LockingError)
    }
}
