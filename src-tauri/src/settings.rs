use crate::camoufox::get_app_dir;
use crate::instances::{ProxyConfig, FingerprintConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagDefinition {
    pub label: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub skip_wipe_confirmation: bool,
    #[serde(default)]
    pub skip_delete_confirmation: bool,
    #[serde(default)]
    pub default_view_mode: Option<String>,
    #[serde(default)]
    pub default_sort_field: Option<String>,
    #[serde(default)]
    pub default_sort_dir: Option<String>,
    #[serde(default)]
    pub default_proxy_template: Option<ProxyConfig>,
    #[serde(default)]
    pub default_fingerprint_template: Option<FingerprintConfig>,
    #[serde(default)]
    pub camoufox_path: Option<String>,
    #[serde(default)]
    pub tag_definitions: Option<Vec<TagDefinition>>,
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
