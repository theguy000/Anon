use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use crate::camoufox::get_app_dir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub skip_wipe_confirmation: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            skip_wipe_confirmation: false,
        }
    }
}

pub async fn get_settings_path(app: &AppHandle) -> PathBuf {
    let mut path = get_app_dir(app).await;
    path.push("settings.json");
    path
}

pub async fn load_settings(app: &AppHandle) -> AppSettings {
    let path = get_settings_path(app).await;
    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str::<AppSettings>(&contents) {
            return settings;
        }
    }
    AppSettings::default()
}

pub async fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app).await;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}
