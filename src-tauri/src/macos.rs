use crate::CefResult;
use std::error::Error;
use tauri::ipc::Channel;

pub async fn command(channel: Channel<CefResult>) -> Result<(), Box<dyn Error + Send + Sync>> {
    Ok(())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    Ok(())
}
