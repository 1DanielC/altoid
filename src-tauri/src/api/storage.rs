use tauri::path::BaseDirectory;
use tauri::{AppHandle, Runtime};
use tauri_plugin_fs::{FsExt, OpenOptions};
use crate::error::AppError;
use crate::ipc::ipc_error::IpcError;

#[tauri::command]
pub async fn clear_cache<R: Runtime>(app: AppHandle<R>) -> Result<(), AppError> {
    let fs = app.fs();

    fs.remove_file("cache.json", BaseDirectory::AppData)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(())
}

#[tauri::command]
pub async fn write_cache<R: Runtime>(app: AppHandle<R>, contents: String) -> Result<(), AppError> {
    let fs = app.fs();

    fs.open("cache.json", OpenOptions::write(true))
    .await
    .map_err(|e|AppError::from(e))?;

    Ok(())
}

#[tauri::command]
pub async fn read_cache<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, AppError> {
    let fs = app.fs();

    if !fs
        .exists("cache.json", BaseDirectory::AppData)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(None);
    }

    let bytes = fs
        .read("cache.json")
        .await
        .map_err(|e| e.to_string())?;

    Ok(Some(String::from_utf8(bytes).map_err(|e| e.to_string())?))
}
