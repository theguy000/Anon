use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone, Default)]
pub struct InstallStatus {
    pub status: String,
    pub progress: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    #[allow(dead_code)]
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

static LEGACY_MIGRATION_DONE: AtomicBool = AtomicBool::new(false);

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn move_entry(src: &Path, dst: &Path) -> io::Result<()> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    if src.is_dir() {
        copy_dir_recursive(src, dst)?;
        std::fs::remove_dir_all(src)
    } else {
        std::fs::copy(src, dst)?;
        std::fs::remove_file(src)
    }
}

/// Move legacy user data (`profiles/`, `settings.json`, `geo_cache.json`) out of
/// the `camoufox/` engine directory into its sibling app-data root without
/// overwriting any existing destination entries.
pub fn migrate_legacy_user_data(legacy_engine_dir: &Path, data_dir: &Path) -> io::Result<()> {
    if !legacy_engine_dir.exists() || legacy_engine_dir == data_dir {
        return Ok(());
    }
    std::fs::create_dir_all(data_dir)?;

    for file_name in ["settings.json", "geo_cache.json"] {
        let src = legacy_engine_dir.join(file_name);
        let dst = data_dir.join(file_name);
        if src.exists() && !dst.exists() {
            move_entry(&src, &dst)?;
        }
    }

    let legacy_profiles = legacy_engine_dir.join("profiles");
    let target_profiles = data_dir.join("profiles");
    if legacy_profiles.is_dir() {
        if !target_profiles.exists() {
            move_entry(&legacy_profiles, &target_profiles)?;
        } else {
            for entry in std::fs::read_dir(&legacy_profiles)? {
                let entry = entry?;
                let dst = target_profiles.join(entry.file_name());
                if !dst.exists() {
                    move_entry(&entry.path(), &dst)?;
                }
            }
            if std::fs::read_dir(&legacy_profiles)?.next().is_none() {
                let _ = std::fs::remove_dir(&legacy_profiles);
            }
        }
    }

    Ok(())
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Invalid target path: {}", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("data.json");
    let tmp_path = parent.join(format!(".{}.{}.tmp", file_name, uuid::Uuid::new_v4()));

    std::fs::write(&tmp_path, json).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("Failed to persist {}: {}", path.display(), e));
    }
    Ok(())
}

/// Parent user-data directory (`%APPDATA%\Anon`), sibling to `camoufox/`.
pub async fn get_data_dir(app: &AppHandle) -> PathBuf {
    let data_dir = app.path().app_data_dir().unwrap();
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).unwrap();
    }
    if !LEGACY_MIGRATION_DONE.swap(true, Ordering::AcqRel) {
        let legacy_engine_dir = data_dir.join("camoufox");
        if let Err(e) = migrate_legacy_user_data(&legacy_engine_dir, &data_dir) {
            eprintln!("Legacy user data migration warning: {}", e);
        }
    }
    data_dir
}

pub async fn get_app_dir(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().unwrap();
    path.push("camoufox");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

pub(crate) fn resolve_camoufox_binary(
    custom_path: Option<&str>,
    engine_dir: &Path,
) -> Option<PathBuf> {
    if let Some(raw) = custom_path.map(str::trim).filter(|s| !s.is_empty()) {
        let candidate = PathBuf::from(raw);
        if candidate.is_file() {
            return Some(candidate);
        }
        if candidate.is_dir() {
            for name in ["camoufox.exe", "camoufox"] {
                let nested = candidate.join(name);
                if nested.is_file() {
                    return Some(nested);
                }
            }
        }
        return None;
    }

    for name in ["camoufox.exe", "camoufox"] {
        let exe = engine_dir.join(name);
        if exe.is_file() {
            return Some(exe);
        }
    }
    None
}

pub async fn get_camoufox_binary(app: &AppHandle) -> Option<PathBuf> {
    let engine_dir = get_app_dir(app).await;
    let settings = crate::settings::load_settings(app).await;
    resolve_camoufox_binary(settings.camoufox_path.as_deref(), &engine_dir)
}

pub struct ReleaseInfo {
    pub download_url: String,
    pub version: String,
}

pub async fn get_camoufox_version(app: &AppHandle) -> String {
    let app_dir = get_app_dir(app).await;
    let version_file = app_dir.join("version.txt");
    if let Ok(ver) = std::fs::read_to_string(version_file) {
        let trimmed = ver.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(bin) = get_camoufox_binary(app).await {
        if let Ok(output) = std::process::Command::new(bin).arg("--version").output() {
            let out_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !out_str.is_empty() {
                return out_str;
            }
        }
    }

    "Camoufox".to_string()
}

struct DownloadedArchive {
    file_name: String,
    downloaded: u64,
    total: Option<u64>,
}

async fn download_release_zip(
    app: &AppHandle,
    release_info: &ReleaseInfo,
    zip_path: &Path,
) -> Result<DownloadedArchive, String> {
    let file_name = release_info
        .download_url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("camoufox.zip")
        .to_string();

    let _ = app.emit(
        "install_progress",
        InstallStatus {
            status: format!("Downloading {}", file_name),
            progress: 10,
            ..Default::default()
        },
    );

    let client = reqwest::Client::builder()
        .user_agent("Anon-Instance-Manager")
        .build()
        .map_err(|e| e.to_string())?;

    let mut response = client
        .get(&release_info.download_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    let mut file = File::create(zip_path).map_err(|e| e.to_string())?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut last_pct: u8 = 0;

    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        io::Write::write_all(&mut file, &chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let pct = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
            if pct != last_pct {
                last_pct = pct;
                let _ = app.emit(
                    "install_progress",
                    InstallStatus {
                        status: format!("Downloading {}", file_name),
                        progress: ((pct as f64 / 100.0) * 80.0) as u8 + 10,
                        downloaded: Some(downloaded),
                        total: Some(total_size),
                    },
                );
            }
        }
    }

    Ok(DownloadedArchive {
        file_name,
        downloaded,
        total: (total_size > 0).then_some(total_size),
    })
}

pub async fn download_and_extract(app: &AppHandle) -> Result<(), String> {
    if get_camoufox_binary(app).await.is_some() {
        return Err("Camoufox is already installed".to_string());
    }

    let app_dir = get_app_dir(app).await;
    let release_info = get_latest_release_info().await.map_err(|e| e.to_string())?;
    let zip_path = app_dir.join("camoufox.zip");

    let archive_info = download_release_zip(app, &release_info, &zip_path).await?;

    let _ = app.emit(
        "install_progress",
        InstallStatus {
            status: format!("Extracting {}...", archive_info.file_name),
            progress: 95,
            downloaded: Some(archive_info.downloaded),
            total: archive_info.total,
        },
    );

    let file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    archive.extract(&app_dir).map_err(|e| e.to_string())?;

    let _ = std::fs::remove_file(zip_path);
    let _ = std::fs::write(app_dir.join("version.txt"), &release_info.version);

    let _ = app.emit(
        "install_progress",
        InstallStatus {
            status: "Done!".to_string(),
            progress: 100,
            ..Default::default()
        },
    );

    Ok(())
}

const RUNNING_INSTANCES_ERR: &str =
    "Stop all running browser instances before updating the engine.";
const UPDATE_STAGING_DIR: &str = ".camoufox-update-staging";
const UPDATE_BACKUP_DIR: &str = ".camoufox-old-backup";
const PRESERVED_USER_ENTRIES: &[&str] = &["profiles", "settings.json", "geo_cache.json"];

static ENGINE_UPDATE_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

struct UpdateGuard(&'static AtomicBool);

impl UpdateGuard {
    fn acquire(flag: &'static AtomicBool) -> Result<Self, String> {
        flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self(flag))
            .map_err(|_| "An engine update is already in progress.".to_string())
    }
}

impl Drop for UpdateGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReplaceStage {
    Extracting,
    Replacing,
    Done,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct UpdateOutcome {
    pub updated: bool,
    pub previous_version: String,
    pub current_version: String,
}

fn prepare_stale_update_dirs(app_dir: &Path, zip_in_staging: bool) -> Result<(), String> {
    let staging_dir = app_dir.join(UPDATE_STAGING_DIR);
    let backup_dir = app_dir.join(UPDATE_BACKUP_DIR);
    let has_valid_binary = app_dir.join("camoufox.exe").is_file();

    if backup_dir.exists() {
        if has_valid_binary {
            std::fs::remove_dir_all(&backup_dir)
                .map_err(|e| format!("Failed to clean stale backup directory: {}", e))?;
        } else {
            return Err(
                "Found backup from an interrupted update while camoufox.exe is missing. Restore or remove .camoufox-old-backup before retrying."
                    .to_string(),
            );
        }
    }

    if !zip_in_staging && staging_dir.exists() {
        if has_valid_binary {
            std::fs::remove_dir_all(&staging_dir)
                .map_err(|e| format!("Failed to clean stale staging directory: {}", e))?;
        } else {
            return Err(
                "Found staging files from an interrupted update while camoufox.exe is missing. Remove .camoufox-update-staging before retrying."
                    .to_string(),
            );
        }
    }

    Ok(())
}

pub async fn update_camoufox(app: &AppHandle) -> Result<UpdateOutcome, String> {
    let _guard = UpdateGuard::acquire(&ENGINE_UPDATE_IN_FLIGHT)?;

    if crate::process_manager::has_running_instances() {
        return Err(RUNNING_INSTANCES_ERR.to_string());
    }

    let _ = app.emit(
        "install_progress",
        InstallStatus {
            status: "Checking latest release...".to_string(),
            progress: 5,
            ..Default::default()
        },
    );

    let app_dir = get_app_dir(app).await;
    let previous_version = get_camoufox_version(app).await;
    let release_info = get_latest_release_info().await.map_err(|e| e.to_string())?;

    if get_camoufox_binary(app).await.is_some()
        && is_up_to_date(&previous_version, &release_info.version)
    {
        return Ok(UpdateOutcome {
            updated: false,
            previous_version: previous_version.clone(),
            current_version: previous_version,
        });
    }

    prepare_stale_update_dirs(&app_dir, false)?;
    let staging_dir = app_dir.join(UPDATE_STAGING_DIR);
    std::fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("Failed to create staging directory: {}", e))?;

    let zip_path = staging_dir.join("camoufox.update.zip");
    let archive_info = match download_release_zip(app, &release_info, &zip_path).await {
        Ok(info) => info,
        Err(err) => {
            let _ = std::fs::remove_dir_all(&staging_dir);
            return Err(err);
        }
    };

    let swap_result = replace_engine_from_zip(
        &app_dir,
        &zip_path,
        &release_info.version,
        crate::process_manager::has_running_instances(),
        |stage| {
            let status = match stage {
                ReplaceStage::Extracting => InstallStatus {
                    status: format!("Extracting {}...", archive_info.file_name),
                    progress: 92,
                    downloaded: Some(archive_info.downloaded),
                    total: archive_info.total,
                },
                ReplaceStage::Replacing => InstallStatus {
                    status: "Replacing old engine...".to_string(),
                    progress: 96,
                    downloaded: Some(archive_info.downloaded),
                    total: archive_info.total,
                },
                ReplaceStage::Done => InstallStatus {
                    status: "Done!".to_string(),
                    progress: 100,
                    ..Default::default()
                },
            };
            let _ = app.emit("install_progress", status);
        },
    );
    if staging_dir.exists() {
        let _ = std::fs::remove_dir_all(&staging_dir);
    }
    swap_result?;

    Ok(UpdateOutcome {
        updated: true,
        previous_version,
        current_version: release_info.version,
    })
}

fn is_preserved_entry(name: &std::ffi::OsStr) -> bool {
    let Some(s) = name.to_str() else {
        return false;
    };
    s.eq_ignore_ascii_case(UPDATE_STAGING_DIR)
        || s.eq_ignore_ascii_case(UPDATE_BACKUP_DIR)
        || PRESERVED_USER_ENTRIES
            .iter()
            .any(|preserved| s.eq_ignore_ascii_case(preserved))
}

fn remove_path(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| format!("remove dir '{}': {}", path.display(), e))
    } else if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("remove file '{}': {}", path.display(), e))
    } else {
        Ok(())
    }
}

fn rollback_swap(
    app_dir: &Path,
    backup_dir: &Path,
    installed_entries: &[std::ffi::OsString],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for name in installed_entries {
        if let Err(e) = remove_path(&app_dir.join(name)) {
            errors.push(e);
        }
    }

    match std::fs::read_dir(backup_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let dest = app_dir.join(entry.file_name());
                        // Windows fs::rename fails if destination exists (e.g. partial version.txt).
                        if let Err(e) = remove_path(&dest) {
                            errors.push(e);
                            continue;
                        }
                        if let Err(e) = std::fs::rename(entry.path(), &dest) {
                            errors.push(format!(
                                "restore '{}': {}",
                                entry.file_name().to_string_lossy(),
                                e
                            ));
                        }
                    }
                    Err(e) => errors.push(format!("inspect backup entry: {}", e)),
                }
            }
        }
        Err(e) => errors.push(format!("read backup directory: {}", e)),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(crate) fn replace_engine_from_zip(
    app_dir: &Path,
    zip_path: &Path,
    new_version: &str,
    has_running_instances: bool,
    mut on_stage: impl FnMut(ReplaceStage),
) -> Result<(), String> {
    if has_running_instances {
        return Err(RUNNING_INSTANCES_ERR.to_string());
    }

    let staging_dir = app_dir.join(UPDATE_STAGING_DIR);
    let backup_dir = app_dir.join(UPDATE_BACKUP_DIR);
    let zip_in_staging = zip_path.parent() == Some(staging_dir.as_path());

    prepare_stale_update_dirs(app_dir, zip_in_staging)?;

    let extract_dir = staging_dir.join("extracted");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)
            .map_err(|e| format!("Failed to clean extract directory: {}", e))?;
    }
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create staging directory: {}", e))?;

    on_stage(ReplaceStage::Extracting);
    let extract_result = (|| -> Result<(), String> {
        let file = File::open(zip_path).map_err(|e| format!("Failed to open archive: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read archive: {}", e))?;
        archive
            .extract(&extract_dir)
            .map_err(|e| format!("Failed to extract archive: {}", e))?;
        if !extract_dir.join("camoufox.exe").is_file() {
            return Err("Downloaded archive does not contain camoufox.exe".to_string());
        }
        Ok(())
    })();

    if let Err(err) = extract_result {
        let _ = std::fs::remove_dir_all(&staging_dir);
        return Err(err);
    }

    on_stage(ReplaceStage::Replacing);
    if let Err(e) = std::fs::create_dir_all(&backup_dir) {
        let _ = std::fs::remove_dir_all(&staging_dir);
        return Err(format!("Failed to create backup directory: {}", e));
    }

    let mut installed_entries = Vec::new();
    let swap_result = (|| -> Result<(), String> {
        let entries = std::fs::read_dir(app_dir)
            .map_err(|e| format!("Failed to read engine directory: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to inspect engine directory: {}", e))?;
            let name = entry.file_name();
            if is_preserved_entry(&name) {
                continue;
            }
            std::fs::rename(entry.path(), backup_dir.join(&name)).map_err(|e| {
                format!(
                    "Failed to move old engine file '{}' (close any open Camoufox windows and retry): {}",
                    name.to_string_lossy(),
                    e
                )
            })?;
        }

        let staged_entries = std::fs::read_dir(&extract_dir)
            .map_err(|e| format!("Failed to read staged engine files: {}", e))?;
        for entry in staged_entries {
            let entry =
                entry.map_err(|e| format!("Failed to inspect staged engine files: {}", e))?;
            let name = entry.file_name();
            if is_preserved_entry(&name) {
                continue;
            }
            std::fs::rename(entry.path(), app_dir.join(&name)).map_err(|e| {
                format!(
                    "Failed to install new engine file '{}': {}",
                    name.to_string_lossy(),
                    e
                )
            })?;
            installed_entries.push(name);
        }

        std::fs::write(app_dir.join("version.txt"), new_version)
            .map_err(|e| format!("Failed to write version.txt: {}", e))?;
        Ok(())
    })();

    if let Err(err) = swap_result {
        let rollback_res = rollback_swap(app_dir, &backup_dir, &installed_entries);
        let _ = std::fs::remove_dir_all(&staging_dir);
        return match rollback_res {
            Ok(()) => {
                let _ = std::fs::remove_dir_all(&backup_dir);
                Err(err)
            }
            Err(rollback_errors) => Err(format!(
                "{} (rollback incomplete; previous files kept in {}: {})",
                err,
                UPDATE_BACKUP_DIR,
                rollback_errors.join("; ")
            )),
        };
    }

    std::fs::remove_dir_all(&backup_dir).map_err(|e| {
        format!(
            "Updated engine but failed to delete old engine backup: {}",
            e
        )
    })?;
    std::fs::remove_dir_all(&staging_dir).map_err(|e| {
        format!(
            "Updated engine but failed to clean staging directory: {}",
            e
        )
    })?;

    on_stage(ReplaceStage::Done);
    Ok(())
}

fn normalize_version(raw: &str) -> Option<String> {
    let token = raw.split_whitespace().last()?;
    let trimmed = token.trim_start_matches(['v', 'V']);
    if trimmed.is_empty() || !trimmed.chars().next()?.is_ascii_digit() {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

pub(crate) fn is_up_to_date(installed: &str, latest: &str) -> bool {
    match (normalize_version(installed), normalize_version(latest)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

fn select_windows_release(releases: &[GitHubRelease]) -> Option<ReleaseInfo> {
    for release in releases {
        let mut win_fallback: Option<&str> = None;
        for asset in &release.assets {
            if asset.name.starts_with("camoufox-")
                && asset.name.contains("win")
                && asset.name.ends_with(".zip")
            {
                if asset.name.contains("x86_64") {
                    return Some(ReleaseInfo {
                        download_url: asset.browser_download_url.clone(),
                        version: release.tag_name.clone(),
                    });
                }
                if win_fallback.is_none() {
                    win_fallback = Some(&asset.browser_download_url);
                }
            }
        }
        if let Some(url) = win_fallback {
            return Some(ReleaseInfo {
                download_url: url.to_string(),
                version: release.tag_name.clone(),
            });
        }
    }
    None
}

async fn get_latest_release_info() -> Result<ReleaseInfo, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Anon-Instance-Manager")
        .build()?;

    // Query /releases so prereleases (e.g. beta builds) are included before falling back to /latest.
    if let Ok(resp) = client
        .get("https://api.github.com/repos/daijro/camoufox/releases?per_page=20")
        .send()
        .await
    {
        if let Ok(releases) = resp.json::<Vec<GitHubRelease>>().await {
            if let Some(info) = select_windows_release(&releases) {
                return Ok(info);
            }
        }
    }

    let latest: GitHubRelease = client
        .get("https://api.github.com/repos/daijro/camoufox/releases/latest")
        .send()
        .await?
        .json()
        .await?;

    select_windows_release(&[latest]).ok_or_else(|| "No Windows Camoufox release found".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str, url: &str) -> GitHubAsset {
        GitHubAsset {
            name: name.to_string(),
            browser_download_url: url.to_string(),
        }
    }

    fn release(tag: &str, assets: Vec<GitHubAsset>) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag.to_string(),
            assets,
        }
    }

    #[test]
    fn select_windows_release_skips_non_windows_and_prefers_x86_64() {
        let releases = vec![
            release(
                "font-bundle-v4",
                vec![asset("fonts.zip", "https://example.com/fonts.zip")],
            ),
            release(
                "v159.0-beta.43",
                vec![
                    asset(
                        "camoufox-159.0-beta.43-win.i686.zip",
                        "https://example.com/i686.zip",
                    ),
                    asset(
                        "camoufox-159.0-beta.43-win.x86_64.zip",
                        "https://example.com/x86_64.zip",
                    ),
                ],
            ),
        ];

        let selected = select_windows_release(&releases).expect("should pick Windows release");
        assert_eq!(selected.version, "v159.0-beta.43");
        assert_eq!(selected.download_url, "https://example.com/x86_64.zip");
    }

    #[test]
    fn select_windows_release_falls_back_to_other_windows_zip() {
        let releases = vec![release(
            "v159.0-beta.43",
            vec![asset(
                "camoufox-159.0-beta.43-win.i686.zip",
                "https://example.com/i686.zip",
            )],
        )];

        let selected = select_windows_release(&releases).expect("should fall back to win zip");
        assert_eq!(selected.version, "v159.0-beta.43");
        assert_eq!(selected.download_url, "https://example.com/i686.zip");
    }

    #[test]
    fn is_up_to_date_normalizes_prefix_and_rejects_unknown() {
        assert!(is_up_to_date("v159.0-beta.43", "v159.0-beta.43"));
        assert!(is_up_to_date("159.0-beta.43\n", "v159.0-beta.43"));
        assert!(is_up_to_date("Camoufox 159.0-beta.43", "v159.0-beta.43"));
        assert!(!is_up_to_date("v156.0.1-beta.33", "v159.0-beta.43"));
        assert!(!is_up_to_date("Camoufox", "v159.0-beta.43"));
        assert!(!is_up_to_date("", "v159.0-beta.43"));
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let path = std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write_zip(zip_path: &std::path::Path, entries: &[(&str, &[u8])]) {
        let file = File::create(zip_path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            io::Write::write_all(&mut writer, content).unwrap();
        }
        writer.finish().unwrap();
    }

    fn seed_existing_install(app_dir: &std::path::Path) {
        std::fs::write(app_dir.join("camoufox.exe"), b"old-binary").unwrap();
        std::fs::write(app_dir.join("legacy.dll"), b"old-dll").unwrap();
        std::fs::create_dir_all(app_dir.join("browser")).unwrap();
        std::fs::write(app_dir.join("browser/old.ja"), b"old-omni").unwrap();
        std::fs::write(app_dir.join("version.txt"), "v156.0.1-beta.33").unwrap();

        std::fs::create_dir_all(app_dir.join("profiles/inst-1")).unwrap();
        std::fs::write(
            app_dir.join("profiles/inst-1/config.json"),
            b"{\"name\":\"p1\"}",
        )
        .unwrap();
        std::fs::write(app_dir.join("settings.json"), b"{\"theme\":\"dark\"}").unwrap();
        std::fs::write(app_dir.join("geo_cache.json"), b"{\"proxy\":\"US\"}").unwrap();
    }

    #[test]
    fn replace_engine_rejects_when_instances_are_running() {
        let tmp = TempDir::new("anon-camoufox-running");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        let zip_path = tmp.path().join("new.zip");
        write_zip(&zip_path, &[("camoufox.exe", b"new-binary")]);

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", true, |_| {})
            .expect_err("update must fail when instances are running");
        assert!(err.to_lowercase().contains("running"));
        assert_eq!(
            std::fs::read(app_dir.join("camoufox.exe")).unwrap(),
            b"old-binary"
        );
        assert_eq!(
            std::fs::read_to_string(app_dir.join("version.txt")).unwrap(),
            "v156.0.1-beta.33"
        );
    }

    #[test]
    fn replace_engine_swaps_binary_deletes_old_files_and_preserves_user_data() {
        let tmp = TempDir::new("anon-camoufox-swap");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        let staging_dir = app_dir.join(UPDATE_STAGING_DIR);
        std::fs::create_dir_all(&staging_dir).unwrap();
        let zip_path = staging_dir.join("camoufox.update.zip");
        write_zip(
            &zip_path,
            &[
                ("camoufox.exe", b"new-binary"),
                ("new.dll", b"new-dll"),
                ("browser/new.ja", b"new-omni"),
            ],
        );

        replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false, |_| {})
            .expect("replace should succeed");

        assert_eq!(
            std::fs::read(app_dir.join("camoufox.exe")).unwrap(),
            b"new-binary"
        );
        assert_eq!(std::fs::read(app_dir.join("new.dll")).unwrap(), b"new-dll");
        assert_eq!(
            std::fs::read(app_dir.join("browser/new.ja")).unwrap(),
            b"new-omni"
        );
        assert_eq!(
            std::fs::read_to_string(app_dir.join("version.txt")).unwrap(),
            "v159.0-beta.43"
        );

        assert!(!app_dir.join("legacy.dll").exists());
        assert!(!app_dir.join("browser/old.ja").exists());
        assert!(!app_dir.join(UPDATE_STAGING_DIR).exists());
        assert!(!app_dir.join(UPDATE_BACKUP_DIR).exists());

        assert_eq!(
            std::fs::read(app_dir.join("profiles/inst-1/config.json")).unwrap(),
            b"{\"name\":\"p1\"}"
        );
        assert_eq!(
            std::fs::read(app_dir.join("settings.json")).unwrap(),
            b"{\"theme\":\"dark\"}"
        );
        assert_eq!(
            std::fs::read(app_dir.join("geo_cache.json")).unwrap(),
            b"{\"proxy\":\"US\"}"
        );
    }

    #[test]
    fn replace_engine_aborts_without_modifying_install_when_archive_lacks_binary() {
        let tmp = TempDir::new("anon-camoufox-missing-bin");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        let zip_path = tmp.path().join("bad.zip");
        write_zip(&zip_path, &[("nested/camoufox.exe", b"wrong-layout")]);

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false, |_| {})
            .expect_err("archive without top-level camoufox.exe must fail");
        assert!(err.contains("camoufox.exe"));

        assert_eq!(
            std::fs::read(app_dir.join("camoufox.exe")).unwrap(),
            b"old-binary"
        );
        assert_eq!(
            std::fs::read(app_dir.join("legacy.dll")).unwrap(),
            b"old-dll"
        );
        assert_eq!(
            std::fs::read_to_string(app_dir.join("version.txt")).unwrap(),
            "v156.0.1-beta.33"
        );
        assert!(!app_dir.join(UPDATE_STAGING_DIR).exists());
        assert!(!app_dir.join(UPDATE_BACKUP_DIR).exists());
    }

    #[test]
    fn replace_engine_never_overwrites_preserved_user_entries_from_archive() {
        let tmp = TempDir::new("anon-camoufox-preserve-archive");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        let zip_path = tmp.path().join("override-attempt.zip");
        write_zip(
            &zip_path,
            &[
                ("camoufox.exe", b"new-binary"),
                ("settings.json", b"{\"theme\":\"wiped\"}"),
                ("profiles/inst-1/config.json", b"wiped"),
            ],
        );

        replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false, |_| {})
            .expect("replace should succeed");

        assert_eq!(
            std::fs::read(app_dir.join("settings.json")).unwrap(),
            b"{\"theme\":\"dark\"}"
        );
        assert_eq!(
            std::fs::read(app_dir.join("profiles/inst-1/config.json")).unwrap(),
            b"{\"name\":\"p1\"}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn replace_engine_rolls_back_when_old_file_is_locked() {
        use std::os::windows::fs::OpenOptionsExt;

        let tmp = TempDir::new("anon-camoufox-locked");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        let locked_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(app_dir.join("legacy.dll"))
            .unwrap();

        let zip_path = tmp.path().join("new.zip");
        write_zip(&zip_path, &[("camoufox.exe", b"new-binary")]);

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false, |_| {})
            .expect_err("locked file must fail swap");
        assert!(err.contains("legacy.dll"));

        drop(locked_file);

        assert_eq!(
            std::fs::read(app_dir.join("camoufox.exe")).unwrap(),
            b"old-binary"
        );
        assert_eq!(
            std::fs::read(app_dir.join("legacy.dll")).unwrap(),
            b"old-dll"
        );
        assert_eq!(
            std::fs::read_to_string(app_dir.join("version.txt")).unwrap(),
            "v156.0.1-beta.33"
        );
        assert!(!app_dir.join(UPDATE_STAGING_DIR).exists());
        assert!(!app_dir.join(UPDATE_BACKUP_DIR).exists());
    }

    #[test]
    fn update_guard_rejects_concurrent_acquire_and_releases_on_drop() {
        static TEST_LOCK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

        let guard = UpdateGuard::acquire(&TEST_LOCK).expect("first acquire should succeed");
        let err = match UpdateGuard::acquire(&TEST_LOCK) {
            Ok(_) => panic!("second acquire while held must fail"),
            Err(e) => e,
        };
        assert!(err.to_lowercase().contains("in progress"));

        drop(guard);

        let _guard2 = UpdateGuard::acquire(&TEST_LOCK).expect("acquire after drop should succeed");
    }

    #[test]
    fn replace_engine_reports_stages_in_order_and_stops_on_invalid_archive() {
        let tmp = TempDir::new("anon-camoufox-stages");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        let bad_zip = tmp.path().join("bad.zip");
        write_zip(&bad_zip, &[("not-camoufox.txt", b"nope")]);
        let mut bad_stages = Vec::new();
        assert!(
            replace_engine_from_zip(&app_dir, &bad_zip, "v159.0-beta.43", false, |stage| {
                bad_stages.push(stage)
            },)
            .is_err()
        );
        assert_eq!(bad_stages, vec![ReplaceStage::Extracting]);

        let good_zip = tmp.path().join("good.zip");
        write_zip(&good_zip, &[("camoufox.exe", b"new-binary")]);
        let mut good_stages = Vec::new();
        replace_engine_from_zip(&app_dir, &good_zip, "v159.0-beta.43", false, |stage| {
            good_stages.push(stage)
        })
        .expect("replace should succeed");
        assert_eq!(
            good_stages,
            vec![
                ReplaceStage::Extracting,
                ReplaceStage::Replacing,
                ReplaceStage::Done,
            ]
        );
    }

    #[test]
    fn replace_engine_cleans_stale_dirs_when_binary_is_present() {
        let tmp = TempDir::new("anon-camoufox-stale-ok");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        seed_existing_install(&app_dir);

        std::fs::create_dir_all(app_dir.join(UPDATE_STAGING_DIR)).unwrap();
        std::fs::write(app_dir.join(UPDATE_STAGING_DIR).join("stale.tmp"), b"x").unwrap();
        std::fs::create_dir_all(app_dir.join(UPDATE_BACKUP_DIR)).unwrap();
        std::fs::write(app_dir.join(UPDATE_BACKUP_DIR).join("old.tmp"), b"y").unwrap();

        let zip_path = tmp.path().join("new.zip");
        write_zip(&zip_path, &[("camoufox.exe", b"new-binary")]);

        replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false, |_| {})
            .expect("stale dirs should be cleaned when camoufox.exe is present");
        assert_eq!(
            std::fs::read(app_dir.join("camoufox.exe")).unwrap(),
            b"new-binary"
        );
        assert!(!app_dir.join(UPDATE_STAGING_DIR).exists());
        assert!(!app_dir.join(UPDATE_BACKUP_DIR).exists());
    }

    #[test]
    fn replace_engine_preserves_backup_and_aborts_when_binary_is_missing() {
        let tmp = TempDir::new("anon-camoufox-stale-missing-bin");
        let app_dir = tmp.path().join("camoufox");
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(app_dir.join(UPDATE_BACKUP_DIR)).unwrap();
        std::fs::write(
            app_dir.join(UPDATE_BACKUP_DIR).join("camoufox.exe"),
            b"rescued-binary",
        )
        .unwrap();

        let zip_path = tmp.path().join("new.zip");
        write_zip(&zip_path, &[("camoufox.exe", b"new-binary")]);

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false, |_| {})
            .expect_err("must not delete backup when camoufox.exe is missing");
        assert!(err.contains(UPDATE_BACKUP_DIR));
        assert_eq!(
            std::fs::read(app_dir.join(UPDATE_BACKUP_DIR).join("camoufox.exe")).unwrap(),
            b"rescued-binary"
        );
    }

    #[cfg(windows)]
    #[test]
    fn rollback_swap_returns_errors_when_restore_target_is_locked() {
        use std::os::windows::fs::OpenOptionsExt;

        let tmp = TempDir::new("anon-camoufox-rollback-err");
        let app_dir = tmp.path().join("camoufox");
        let backup_dir = app_dir.join(UPDATE_BACKUP_DIR);
        std::fs::create_dir_all(&backup_dir).unwrap();
        std::fs::write(backup_dir.join("camoufox.exe"), b"backup-bin").unwrap();
        std::fs::write(app_dir.join("camoufox.exe"), b"locked-dest").unwrap();

        let locked_dest = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(app_dir.join("camoufox.exe"))
            .unwrap();

        let errs = rollback_swap(&app_dir, &backup_dir, &[])
            .expect_err("locked destination should report rollback error");
        assert!(!errs.is_empty());
        assert!(backup_dir.join("camoufox.exe").is_file());

        drop(locked_dest);
    }

    #[test]
    fn migrate_legacy_user_data_moves_entries_and_is_idempotent() {
        let tmp = TempDir::new("anon-migrate");
        let data_dir = tmp.path().join("Anon");
        let engine_dir = data_dir.join("camoufox");
        std::fs::create_dir_all(engine_dir.join("profiles").join("p1")).unwrap();
        std::fs::write(engine_dir.join("profiles").join("p1").join("anon_config.json"), b"{\"id\":\"p1\"}").unwrap();
        std::fs::write(engine_dir.join("settings.json"), b"{\"skip_wipe_confirmation\":true}").unwrap();
        std::fs::write(engine_dir.join("geo_cache.json"), b"{}").unwrap();
        std::fs::write(engine_dir.join("camoufox.exe"), b"bin").unwrap();

        migrate_legacy_user_data(&engine_dir, &data_dir).unwrap();
        assert!(data_dir.join("profiles").join("p1").join("anon_config.json").is_file());
        assert!(data_dir.join("settings.json").is_file());
        assert!(data_dir.join("geo_cache.json").is_file());
        assert!(engine_dir.join("camoufox.exe").is_file());
        assert!(!engine_dir.join("settings.json").exists());

        std::fs::write(data_dir.join("settings.json"), b"{\"skip_wipe_confirmation\":false}").unwrap();
        std::fs::write(engine_dir.join("settings.json"), b"{\"skip_wipe_confirmation\":true}").unwrap();
        migrate_legacy_user_data(&engine_dir, &data_dir).unwrap();
        assert_eq!(
            std::fs::read_to_string(data_dir.join("settings.json")).unwrap(),
            "{\"skip_wipe_confirmation\":false}"
        );
    }

    #[test]
    fn resolve_camoufox_binary_honors_custom_file_and_dir_paths() {
        let tmp = TempDir::new("anon-bin");
        let engine_dir = tmp.path().join("engine");
        let custom_dir = tmp.path().join("custom");
        std::fs::create_dir_all(&engine_dir).unwrap();
        std::fs::create_dir_all(&custom_dir).unwrap();
        let default_bin = engine_dir.join("camoufox.exe");
        let custom_bin = custom_dir.join("camoufox.exe");
        std::fs::write(&default_bin, b"default").unwrap();
        std::fs::write(&custom_bin, b"custom").unwrap();

        assert_eq!(resolve_camoufox_binary(None, &engine_dir), Some(default_bin));
        assert_eq!(
            resolve_camoufox_binary(Some(custom_bin.to_str().unwrap()), &engine_dir),
            Some(custom_bin.clone())
        );
        assert_eq!(
            resolve_camoufox_binary(Some(custom_dir.to_str().unwrap()), &engine_dir),
            Some(custom_bin)
        );
        assert_eq!(
            resolve_camoufox_binary(Some(tmp.path().join("missing.exe").to_str().unwrap()), &engine_dir),
            None
        );
    }

    #[test]
    fn write_json_atomic_replaces_existing_file_cleanly() {
        let tmp = TempDir::new("anon-atomic");
        let path = tmp.path().join("config.json");
        write_json_atomic(&path, &serde_json::json!({"v": 1})).unwrap();
        write_json_atomic(&path, &serde_json::json!({"v": 2})).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"v\": 2"));
        let leftovers: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }
}
