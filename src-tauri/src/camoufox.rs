use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
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

pub async fn get_app_dir(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().unwrap();
    path.push("camoufox");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

pub async fn get_camoufox_binary(app: &AppHandle) -> Option<PathBuf> {
    let mut exe_path = get_app_dir(app).await;
    exe_path.push("camoufox.exe");
    if exe_path.exists() {
        Some(exe_path)
    } else {
        None
    }
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

async fn download_release_zip(
    app: &AppHandle,
    release_info: &ReleaseInfo,
    zip_path: &Path,
) -> Result<(String, u64, Option<u64>), String> {
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

    Ok((
        file_name,
        downloaded,
        (total_size > 0).then_some(total_size),
    ))
}

pub async fn download_and_extract(app: &AppHandle) -> Result<(), String> {
    if get_camoufox_binary(app).await.is_some() {
        return Err("Camoufox is already installed".to_string());
    }

    let app_dir = get_app_dir(app).await;
    let release_info = get_latest_release_info().await.map_err(|e| e.to_string())?;
    let zip_path = app_dir.join("camoufox.zip");

    let (file_name, downloaded, total) =
        download_release_zip(app, &release_info, &zip_path).await?;

    let _ = app.emit(
        "install_progress",
        InstallStatus {
            status: format!("Extracting {}...", file_name),
            progress: 95,
            downloaded: Some(downloaded),
            total,
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

pub async fn update_camoufox(app: &AppHandle) -> Result<UpdateOutcome, String> {
    if crate::process_manager::has_running_instances() {
        return Err("Stop all running browser instances before updating the engine.".to_string());
    }

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

    let zip_path = app_dir.join("camoufox.update.zip");
    let download_result = download_release_zip(app, &release_info, &zip_path).await;
    if let Err(err) = download_result {
        let _ = std::fs::remove_file(&zip_path);
        return Err(err);
    }

    let swap_result = replace_engine_from_zip(
        &app_dir,
        &zip_path,
        &release_info.version,
        crate::process_manager::has_running_instances(),
    );
    let _ = std::fs::remove_file(&zip_path);
    swap_result?;

    Ok(UpdateOutcome {
        updated: true,
        previous_version,
        current_version: release_info.version,
    })
}

const UPDATE_STAGING_DIR: &str = ".camoufox-update-staging";
const UPDATE_BACKUP_DIR: &str = ".camoufox-old-backup";
const PRESERVED_USER_ENTRIES: &[&str] = &["profiles", "settings.json", "geo_cache.json"];

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct UpdateOutcome {
    pub updated: bool,
    pub previous_version: String,
    pub current_version: String,
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

fn rollback_swap(app_dir: &Path, backup_dir: &Path, installed_entries: &[std::ffi::OsString]) {
    for name in installed_entries {
        let path = app_dir.join(name);
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        } else if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }

    if let Ok(entries) = std::fs::read_dir(backup_dir) {
        for entry in entries.flatten() {
            let _ = std::fs::rename(entry.path(), app_dir.join(entry.file_name()));
        }
    }
}

pub(crate) fn replace_engine_from_zip(
    app_dir: &Path,
    zip_path: &Path,
    new_version: &str,
    has_running_instances: bool,
) -> Result<(), String> {
    if has_running_instances {
        return Err("Stop all running browser instances before updating the engine.".to_string());
    }

    let staging_dir = app_dir.join(UPDATE_STAGING_DIR);
    let backup_dir = app_dir.join(UPDATE_BACKUP_DIR);

    if staging_dir.exists() {
        std::fs::remove_dir_all(&staging_dir)
            .map_err(|e| format!("Failed to clean stale staging directory: {}", e))?;
    }
    if backup_dir.exists() && app_dir.join("camoufox.exe").exists() {
        std::fs::remove_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to clean stale backup directory: {}", e))?;
    }

    std::fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("Failed to create staging directory: {}", e))?;

    let extract_result = (|| -> Result<(), String> {
        let file = File::open(zip_path).map_err(|e| format!("Failed to open archive: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read archive: {}", e))?;
        archive
            .extract(&staging_dir)
            .map_err(|e| format!("Failed to extract archive: {}", e))?;
        if !staging_dir.join("camoufox.exe").is_file() {
            return Err("Downloaded archive does not contain camoufox.exe".to_string());
        }
        Ok(())
    })();

    if let Err(err) = extract_result {
        let _ = std::fs::remove_dir_all(&staging_dir);
        return Err(err);
    }

    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let zip_name_in_app_dir = (zip_path.parent() == Some(app_dir))
        .then(|| zip_path.file_name())
        .flatten();

    let entries = std::fs::read_dir(app_dir)
        .map_err(|e| format!("Failed to read engine directory: {}", e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                rollback_swap(app_dir, &backup_dir, &[]);
                let _ = std::fs::remove_dir_all(&staging_dir);
                let _ = std::fs::remove_dir_all(&backup_dir);
                return Err(format!("Failed to inspect engine directory: {}", e));
            }
        };
        let name = entry.file_name();
        if is_preserved_entry(&name) || zip_name_in_app_dir == Some(name.as_os_str()) {
            continue;
        }
        if let Err(e) = std::fs::rename(entry.path(), backup_dir.join(&name)) {
            rollback_swap(app_dir, &backup_dir, &[]);
            let _ = std::fs::remove_dir_all(&staging_dir);
            let _ = std::fs::remove_dir_all(&backup_dir);
            return Err(format!(
                "Failed to move old engine file '{}' (close any open Camoufox windows and retry): {}",
                name.to_string_lossy(),
                e
            ));
        }
    }

    let mut installed_entries = Vec::new();
    let staged_entries = match std::fs::read_dir(&staging_dir) {
        Ok(e) => e,
        Err(e) => {
            rollback_swap(app_dir, &backup_dir, &installed_entries);
            let _ = std::fs::remove_dir_all(&staging_dir);
            let _ = std::fs::remove_dir_all(&backup_dir);
            return Err(format!("Failed to read staged engine files: {}", e));
        }
    };

    for entry in staged_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                rollback_swap(app_dir, &backup_dir, &installed_entries);
                let _ = std::fs::remove_dir_all(&staging_dir);
                let _ = std::fs::remove_dir_all(&backup_dir);
                return Err(format!("Failed to inspect staged engine files: {}", e));
            }
        };
        let name = entry.file_name();
        if is_preserved_entry(&name) {
            continue;
        }
        if let Err(e) = std::fs::rename(entry.path(), app_dir.join(&name)) {
            rollback_swap(app_dir, &backup_dir, &installed_entries);
            let _ = std::fs::remove_dir_all(&staging_dir);
            let _ = std::fs::remove_dir_all(&backup_dir);
            return Err(format!(
                "Failed to install new engine file '{}': {}",
                name.to_string_lossy(),
                e
            ));
        }
        installed_entries.push(name);
    }

    if let Err(e) = std::fs::write(app_dir.join("version.txt"), new_version) {
        rollback_swap(app_dir, &backup_dir, &installed_entries);
        let _ = std::fs::remove_dir_all(&staging_dir);
        let _ = std::fs::remove_dir_all(&backup_dir);
        return Err(format!("Failed to write version.txt: {}", e));
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

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", true)
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

        let zip_path = app_dir.join("camoufox.update.zip");
        write_zip(
            &zip_path,
            &[
                ("camoufox.exe", b"new-binary"),
                ("new.dll", b"new-dll"),
                ("browser/new.ja", b"new-omni"),
            ],
        );

        replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false)
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

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false)
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

        replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false)
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

        let err = replace_engine_from_zip(&app_dir, &zip_path, "v159.0-beta.43", false)
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
}
