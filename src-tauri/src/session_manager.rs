use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use crate::instances::get_profiles_dir;
use crate::process_manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionInfo {
    pub profile_size_bytes: u64,
    pub cookies_exists: bool,
    pub local_storage_size: Option<u64>,
    pub cache_size: Option<u64>,
    pub session_store_exists: bool,
    pub has_history: bool,
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut size = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path);
            } else if let Ok(meta) = entry.metadata() {
                size += meta.len();
            }
        }
    }
    size
}

pub async fn get_session_info(app: &AppHandle, id: String) -> Result<SessionInfo, String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);

    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let profile_size = dir_size(&instance_dir);
    let cookies_path = instance_dir.join("cookies.sqlite");
    let local_storage_path = instance_dir.join("webappsstore.sqlite");
    let cache_path = instance_dir.join("cache2");
    let session_path = instance_dir.join("sessionstore.jsonlz4");
    let history_path = instance_dir.join("places.sqlite");

    Ok(SessionInfo {
        profile_size_bytes: profile_size,
        cookies_exists: cookies_path.exists(),
        local_storage_size: if local_storage_path.exists() {
            local_storage_path.metadata().ok().map(|m| m.len())
        } else {
            None
        },
        cache_size: if cache_path.exists() {
            Some(dir_size(&cache_path))
        } else {
            None
        },
        session_store_exists: session_path.exists(),
        has_history: history_path.exists(),
    })
}

pub async fn clear_session_data(app: &AppHandle, id: String, types: Vec<String>) -> Result<(), String> {
    if process_manager::is_running(&id) {
        return Err("Cannot clear data while instance is running".to_string());
    }

    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);

    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let clear_all = types.contains(&"all".to_string());

    if clear_all || types.contains(&"cookies".to_string()) {
        let _ = fs::remove_file(instance_dir.join("cookies.sqlite"));
        let _ = fs::remove_file(instance_dir.join("cookies.sqlite-wal"));
        let _ = fs::remove_file(instance_dir.join("cookies.sqlite-shm"));
    }

    if clear_all || types.contains(&"localStorage".to_string()) {
        let _ = fs::remove_file(instance_dir.join("webappsstore.sqlite"));
        let _ = fs::remove_file(instance_dir.join("webappsstore.sqlite-wal"));
        let _ = fs::remove_file(instance_dir.join("webappsstore.sqlite-shm"));
    }

    if clear_all || types.contains(&"cache".to_string()) {
        let _ = fs::remove_dir_all(instance_dir.join("cache2"));
    }

    if clear_all || types.contains(&"sessions".to_string()) {
        let _ = fs::remove_file(instance_dir.join("sessionstore.jsonlz4"));
        let _ = fs::remove_file(instance_dir.join("sessionstore-backups"));
    }

    if clear_all || types.contains(&"history".to_string()) {
        let _ = fs::remove_file(instance_dir.join("places.sqlite"));
        let _ = fs::remove_file(instance_dir.join("places.sqlite-wal"));
        let _ = fs::remove_file(instance_dir.join("places.sqlite-shm"));
    }

    Ok(())
}
