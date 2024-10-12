use size::fmt::{Base, Style};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::windows::{command, open_folder};
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::{command, open_folder};
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::linux::{command, open_folder};

#[derive(Debug, Clone)]
pub struct CefSize(size::Size);

impl serde::Serialize for CefSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .format()
            .with_base(Base::Base10)
            .with_style(Style::Abbreviated)
            .to_string()
            .serialize(serializer)
    }
}

impl std::ops::Deref for CefSize {
    type Target = size::Size;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "tag", content = "data")]
pub enum CefResult {
    #[serde(rename_all = "camelCase")]
    Path {
        name: String,
        path: String,
        icon: String,
        size: CefSize,
    },
    Count(usize),
    TotalSize(CefSize),
    Sign,
}

#[tauri::command]
async fn start(channel: tauri::ipc::Channel<CefResult>) -> Result<(), String> {
    match command(channel).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let e = e.to_string();
            #[cfg(debug_assertions)]
            eprintln!("{}", e);
            Err(e)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start, open_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
