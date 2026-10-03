use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::PathBuf;
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

pub async fn download_and_extract(app: &AppHandle) -> Result<(), String> {
    // Prevent re-download if already installed
    if get_camoufox_binary(app).await.is_some() {
        return Err("Camoufox is already installed".to_string());
    }

    let app_dir = get_app_dir(app).await;
    let release_info = get_latest_release_info().await.map_err(|e| e.to_string())?;

    let file_name = release_info
        .download_url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("camoufox.zip")
        .to_string();

    app.emit(
        "install_progress",
        InstallStatus {
            status: format!("Downloading {}", file_name),
            progress: 10,
            ..Default::default()
        },
    )
    .unwrap();

    let client = reqwest::Client::builder()
        .user_agent("Anon-Instance-Manager")
        .build()
        .map_err(|e| e.to_string())?;

    let mut response = client
        .get(&release_info.download_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let zip_path = app_dir.join("camoufox.zip");
    let mut file = File::create(&zip_path).map_err(|e| e.to_string())?;

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

    app.emit(
        "install_progress",
        InstallStatus {
            status: format!("Extracting {}...", file_name),
            progress: 95,
            downloaded: Some(downloaded),
            total: (total_size > 0).then_some(total_size),
        },
    )
    .unwrap();

    // Extract zip
    let file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    archive.extract(&app_dir).map_err(|e| e.to_string())?;

    // Clean up zip
    let _ = std::fs::remove_file(zip_path);

    // Save version info
    let _ = std::fs::write(app_dir.join("version.txt"), &release_info.version);

    app.emit(
        "install_progress",
        InstallStatus {
            status: "Done!".to_string(),
            progress: 100,
            ..Default::default()
        },
    )
    .unwrap();

    Ok(())
}

async fn get_latest_release_info() -> Result<ReleaseInfo, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Anon-Instance-Manager")
        .build()?;

    // First try querying releases list to include prereleases (such as v156.0.1-beta.33)
    if let Ok(resp) = client
        .get("https://api.github.com/repos/daijro/camoufox/releases?per_page=20")
        .send()
        .await
    {
        if let Ok(releases) = resp.json::<Vec<GitHubRelease>>().await {
            for release in releases {
                let mut win_fallback: Option<String> = None;
                for asset in &release.assets {
                    if asset.name.starts_with("camoufox-")
                        && asset.name.contains("win")
                        && asset.name.ends_with(".zip")
                    {
                        if asset.name.contains("x86_64") {
                            return Ok(ReleaseInfo {
                                download_url: asset.browser_download_url.clone(),
                                version: release.tag_name,
                            });
                        }
                        if win_fallback.is_none() {
                            win_fallback = Some(asset.browser_download_url.clone());
                        }
                    }
                }
                if let Some(url) = win_fallback {
                    return Ok(ReleaseInfo {
                        download_url: url,
                        version: release.tag_name,
                    });
                }
            }
        }
    }

    // Fallback to /releases/latest
    let res: GitHubRelease = client
        .get("https://api.github.com/repos/daijro/camoufox/releases/latest")
        .send()
        .await?
        .json()
        .await?;

    let mut win_fallback: Option<String> = None;
    for asset in &res.assets {
        if asset.name.starts_with("camoufox-")
            && asset.name.contains("win")
            && asset.name.ends_with(".zip")
        {
            if asset.name.contains("x86_64") {
                return Ok(ReleaseInfo {
                    download_url: asset.browser_download_url.clone(),
                    version: res.tag_name,
                });
            }
            if win_fallback.is_none() {
                win_fallback = Some(asset.browser_download_url.clone());
            }
        }
    }

    if let Some(url) = win_fallback {
        return Ok(ReleaseInfo {
            download_url: url,
            version: res.tag_name,
        });
    }

    Err("No Windows Camoufox release found".into())
}
