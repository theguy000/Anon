use crate::instances::{get_profiles_dir, ANON_INTERNAL_ENTRIES};
use crate::process_manager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use tauri::AppHandle;

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
            let entry_path = entry.path();
            if entry_path.is_dir() {
                size += dir_size(&entry_path);
            } else if let Ok(meta) = entry.metadata() {
                size += meta.len();
            }
        }
    }
    size
}

fn profile_browsing_size(instance_dir: &Path) -> u64 {
    let mut size = 0u64;
    if let Ok(entries) = fs::read_dir(instance_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if ANON_INTERNAL_ENTRIES.contains(&name_str) {
                    continue;
                }
            }
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

fn path_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_dir() {
        dir_size(path)
    } else {
        path.metadata().map(|m| m.len()).unwrap_or(0)
    }
}

fn has_dir_entries(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .and_then(|mut it| it.next())
        .is_some()
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to remove {}: {}", path.display(), e)),
    }
}

fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to remove {}: {}", path.display(), e)),
    }
}

fn remove_sqlite_triplet(dir: &Path, stem: &str) -> Result<(), String> {
    for suffix in ["", "-wal", "-shm"] {
        remove_file_if_exists(&dir.join(format!("{}{}", stem, suffix)))?;
    }
    Ok(())
}

pub(crate) fn inspect_profile_session(instance_dir: &Path) -> Result<SessionInfo, String> {
    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let ls_bytes = [
        "storage/default",
        "storage/temporary",
        "storage/permanent",
        "storage.sqlite",
        "storage.sqlite-wal",
        "webappsstore.sqlite",
        "webappsstore.sqlite-wal",
    ]
    .iter()
    .map(|rel| path_size(&instance_dir.join(rel)))
    .sum::<u64>();

    let cache_bytes = ["cache2", "startupCache", "shader-cache"]
        .iter()
        .map(|rel| path_size(&instance_dir.join(rel)))
        .sum::<u64>();

    let session_store_exists = instance_dir.join("sessionstore.jsonlz4").exists()
        || has_dir_entries(&instance_dir.join("sessionstore-backups"));

    let has_history = ["places.sqlite", "favicons.sqlite", "formhistory.sqlite"]
        .iter()
        .any(|f| instance_dir.join(f).exists());

    Ok(SessionInfo {
        profile_size_bytes: profile_browsing_size(instance_dir),
        cookies_exists: instance_dir.join("cookies.sqlite").exists(),
        local_storage_size: (ls_bytes > 0).then_some(ls_bytes),
        cache_size: (cache_bytes > 0).then_some(cache_bytes),
        session_store_exists,
        has_history,
    })
}

pub(crate) fn clear_profile_session(instance_dir: &Path, types: &[String]) -> Result<(), String> {
    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let clear_all = types.iter().any(|t| t == "all");

    if clear_all || types.iter().any(|t| t == "cookies") {
        remove_sqlite_triplet(instance_dir, "cookies.sqlite")?;
        remove_sqlite_triplet(instance_dir, "bounce-tracking-protection.sqlite")?;
        remove_sqlite_triplet(instance_dir, "permissions.sqlite")?;
        remove_sqlite_triplet(instance_dir, "content-prefs.sqlite")?;
        remove_file_if_exists(&instance_dir.join("SiteSecurityServiceState.txt"))?;
    }

    if clear_all || types.iter().any(|t| t == "localStorage") {
        remove_sqlite_triplet(instance_dir, "webappsstore.sqlite")?;
        remove_sqlite_triplet(instance_dir, "storage.sqlite")?;
        remove_sqlite_triplet(&instance_dir.join("storage"), "ls-archive.sqlite")?;
        remove_dir_if_exists(&instance_dir.join("storage").join("default"))?;
        remove_dir_if_exists(&instance_dir.join("storage").join("temporary"))?;
        remove_dir_if_exists(&instance_dir.join("storage").join("permanent"))?;
    }

    if clear_all || types.iter().any(|t| t == "cache") {
        remove_dir_if_exists(&instance_dir.join("cache2"))?;
        remove_dir_if_exists(&instance_dir.join("startupCache"))?;
        remove_dir_if_exists(&instance_dir.join("shader-cache"))?;
        remove_file_if_exists(&instance_dir.join("AlternateServices.bin"))?;
    }

    if clear_all || types.iter().any(|t| t == "sessions") {
        remove_file_if_exists(&instance_dir.join("sessionstore.jsonlz4"))?;
        remove_dir_if_exists(&instance_dir.join("sessionstore-backups"))?;
    }

    if clear_all || types.iter().any(|t| t == "history") {
        remove_sqlite_triplet(instance_dir, "places.sqlite")?;
        remove_sqlite_triplet(instance_dir, "favicons.sqlite")?;
        remove_sqlite_triplet(instance_dir, "formhistory.sqlite")?;
    }

    Ok(())
}

pub async fn get_session_info(app: &AppHandle, id: String) -> Result<SessionInfo, String> {
    let profiles_dir = get_profiles_dir(app).await;
    inspect_profile_session(&profiles_dir.join(&id))
}

pub async fn clear_session_data(
    app: &AppHandle,
    id: String,
    types: Vec<String>,
) -> Result<(), String> {
    if process_manager::is_running(&id) {
        return Err("Cannot clear data while instance is running".to_string());
    }

    let profiles_dir = get_profiles_dir(app).await;
    clear_profile_session(&profiles_dir.join(&id), &types)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_profile(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "anon_session_test_{}_{}",
            name,
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn inspects_and_clears_modern_lsng_storage_and_sessionstore_backups_dir() {
        let dir = temp_profile("modern");
        fs::write(dir.join("anon_config.json"), b"{\"id\":\"x\"}").unwrap();
        fs::write(dir.join("cookies.sqlite"), vec![1u8; 64]).unwrap();

        let lsng = dir
            .join("storage")
            .join("default")
            .join("https+++example.com")
            .join("ls");
        fs::create_dir_all(&lsng).unwrap();
        fs::write(lsng.join("data.sqlite"), vec![2u8; 512]).unwrap();

        let backups = dir.join("sessionstore-backups");
        fs::create_dir_all(&backups).unwrap();
        fs::write(backups.join("recovery.jsonlz4"), vec![3u8; 128]).unwrap();

        let cache = dir.join("cache2").join("entries");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("abc"), vec![4u8; 256]).unwrap();

        fs::write(dir.join("places.sqlite"), vec![5u8; 100]).unwrap();
        fs::write(dir.join("favicons.sqlite"), vec![6u8; 50]).unwrap();

        let info = inspect_profile_session(&dir).unwrap();
        assert!(info.cookies_exists);
        assert_eq!(info.local_storage_size, Some(512));
        assert_eq!(info.cache_size, Some(256));
        assert!(info.session_store_exists);
        assert!(info.has_history);
        assert_eq!(info.profile_size_bytes, 64 + 512 + 128 + 256 + 100 + 50);

        clear_profile_session(&dir, &["localStorage".into(), "sessions".into()]).unwrap();
        assert!(!dir.join("storage").join("default").exists());
        assert!(!dir.join("sessionstore-backups").exists());
        assert!(dir.join("cookies.sqlite").exists());
        assert!(dir.join("anon_config.json").exists());

        clear_profile_session(&dir, &["all".into()]).unwrap();
        let after = inspect_profile_session(&dir).unwrap();
        assert!(!after.cookies_exists);
        assert_eq!(after.local_storage_size, None);
        assert_eq!(after.cache_size, None);
        assert!(!after.session_store_exists);
        assert!(!after.has_history);
        assert!(dir.join("anon_config.json").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
