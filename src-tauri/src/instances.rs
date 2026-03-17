use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::camoufox::get_app_dir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    pub proxy: Option<String>,
    pub persist_data: bool,
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

fn ensure_user_js(instance_dir: &PathBuf, proxy: &Option<String>, persist_data: bool) -> io::Result<()> {
    let user_js_path = instance_dir.join("user.js");
    let mut user_js_content = String::from("// Anon Instance Preferences\n");
    
    // Persistence preferences
    if persist_data {
        user_js_content.push_str("user_pref(\"browser.sessionhistory.max_entries\", 50);\n");
        user_js_content.push_str("user_pref(\"browser.sessionhistory.max_total_viewers\", -1);\n");
        user_js_content.push_str("user_pref(\"browser.formfill.enable\", true);\n");
        user_js_content.push_str("user_pref(\"browser.places.interactions.enabled\", true);\n");
        user_js_content.push_str("user_pref(\"browser.urlbar.suggest.history\", true);\n");
        user_js_content.push_str("user_pref(\"signon.rememberSignons\", true);\n");
        user_js_content.push_str("user_pref(\"privacy.clearOnShutdown.openWindows\", false);\n");
        user_js_content.push_str("user_pref(\"browser.sessionstore.resume_from_crash\", true);\n");
    } else {
        // Explicitly disable if turned off
        user_js_content.push_str("user_pref(\"browser.sessionhistory.max_entries\", 0);\n");
        user_js_content.push_str("user_pref(\"browser.sessionhistory.max_total_viewers\", 0);\n");
        user_js_content.push_str("user_pref(\"browser.formfill.enable\", false);\n");
        user_js_content.push_str("user_pref(\"browser.places.interactions.enabled\", false);\n");
        user_js_content.push_str("user_pref(\"browser.urlbar.suggest.history\", false);\n");
        user_js_content.push_str("user_pref(\"signon.rememberSignons\", false);\n");
        user_js_content.push_str("user_pref(\"privacy.clearOnShutdown.openWindows\", true);\n");
        user_js_content.push_str("user_pref(\"browser.sessionstore.resume_from_crash\", false);\n");
    }

    // Proxy settings
    if let Some(_) = proxy {
        user_js_content.push_str("user_pref(\"network.proxy.type\", 1);\n");
    }

    fs::write(user_js_path, user_js_content)
}

fn cleanup_instance_data(instance_dir: &PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(instance_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str());
        
        // Preserve essential config files
        if let Some(name) = file_name {
            if name == "anon_config.json" || name == "user.js" {
                continue;
            }
        }
        
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
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

pub async fn create_instance(app: &AppHandle, name: String, proxy: Option<String>, persist_data: bool) -> Result<InstanceConfig, String> {
    let instances = list_instances(app).await?;
    if instances.iter().any(|i| i.name.eq_ignore_ascii_case(&name)) {
        return Err("An instance with this name already exists".to_string());
    }

    let profiles_dir = get_profiles_dir(app).await;
    let id = uuid::Uuid::new_v4().to_string();
    
    let instance_dir = profiles_dir.join(&id);
    fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;

    let config = InstanceConfig {
        id: id.clone(),
        name,
        proxy,
        persist_data,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };

    // Save anon config
    let config_path = instance_dir.join("anon_config.json");
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(config_path, config_json).map_err(|e| e.to_string())?;

    // Generate user.js for persistence and proxy settings
    let _ = ensure_user_js(&instance_dir, &config.proxy, config.persist_data);

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

    // Load config to get settings
    let config_path = instance_dir.join("anon_config.json");
    let (proxy, persist_data) = if config_path.exists() {
        if let Ok(contents) = fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<InstanceConfig>(&contents) {
                (config.proxy, config.persist_data)
            } else { (None, true) }
        } else { (None, true) }
    } else { (None, true) };

    // Update user.js on every launch to ensure preferences are applied
    let _ = ensure_user_js(&instance_dir, &proxy, persist_data);

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

pub async fn toggle_persistence(app: &AppHandle, id: String, enabled: bool) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    
    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.persist_data = enabled;
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            
            // Immediately update user.js
            let _ = ensure_user_js(&instance_dir, &config.proxy, config.persist_data);

            // If disabling, clean up data
            if !enabled {
                let _ = cleanup_instance_data(&instance_dir);
            }
            
            return Ok(());
        }
    }
    
    Err("Failed to update instance config".to_string())
}
