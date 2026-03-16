use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::camoufox::get_app_dir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    pub proxy: Option<String>,
    pub created_at: i64,
}

pub async fn get_profiles_dir(app: &AppHandle) -> PathBuf {
    let mut path = get_app_dir(app).await;
    path.push("profiles");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

pub async fn list_instances(app: &AppHandle) -> Result<Vec<InstanceConfig>, String> {
    let profiles_dir = get_profiles_dir(app).await;
    let mut instances = Vec::new();

    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let config_path = entry.path().join("anon_config.json");
            if config_path.exists() {
                if let Ok(contents) = fs::read_to_string(config_path) {
                    if let Ok(config) = serde_json::from_str::<InstanceConfig>(&contents) {
                        instances.push(config);
                    }
                }
            }
        }
    }

    // Sort by creation time, newest first
    instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(instances)
}

pub async fn create_instance(app: &AppHandle, name: String, proxy: Option<String>) -> Result<InstanceConfig, String> {
    let profiles_dir = get_profiles_dir(app).await;
    let id = uuid::Uuid::new_v4().to_string();
    
    let instance_dir = profiles_dir.join(&id);
    fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;

    let config = InstanceConfig {
        id: id.clone(),
        name,
        proxy,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };

    // Save anon config
    let config_path = instance_dir.join("anon_config.json");
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(config_path, config_json).map_err(|e| e.to_string())?;

    // Generate basic prefs.js for Camoufox proxy if needed
    if let Some(ref p) = config.proxy {
        let prefs_path = instance_dir.join("prefs.js");
        // Note: Basic template. For real proxy config in Firefox, we'd parse the URL
        // and set network.proxy.type, network.proxy.http, etc.
        let prefs_content = format!("// Anon Instance Prefs\nuser_pref(\"network.proxy.type\", 1);\n");
        let _ = fs::write(prefs_path, prefs_content);
    }

    Ok(config)
}

pub async fn delete_instance(app: &AppHandle, id: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(id);
    
    if instance_dir.exists() {
        fs::remove_dir_all(instance_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn launch_instance(app: &AppHandle, id: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    
    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let bin_path = crate::camoufox::get_camoufox_binary(app).await
        .ok_or_else(|| "Camoufox binary not downloaded".to_string())?;

    // Spawn detached process
    std::process::Command::new(bin_path)
        .arg("--profile")
        .arg(&instance_dir)
        .spawn()
        .map_err(|e| format!("Failed to launch instance: {}", e))?;

    Ok(())
}
