use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Emitter};

#[derive(Serialize, Clone)]
pub struct InstallStatus {
    pub status: String,
    pub progress: u8,
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

pub async fn download_and_extract(app: &AppHandle) -> Result<(), String> {
    let app_dir = get_app_dir(app).await;
    let url = get_latest_release_url().await.map_err(|e| e.to_string())?;
    
    app.emit("install_progress", InstallStatus { status: "Downloading engine...".to_string(), progress: 10 }).unwrap();
    
    let client = reqwest::Client::builder()
        .user_agent("Anon-Instance-Manager")
        .build()
        .map_err(|e| e.to_string())?;
        
    let mut response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
    let zip_path = app_dir.join("camoufox.zip");
    let mut file = File::create(&zip_path).map_err(|e| e.to_string())?;
    
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        io::Write::write_all(&mut file, &chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let progress = ((downloaded as f64 / total_size as f64) * 80.0) as u8 + 10;
            let _ = app.emit("install_progress", InstallStatus { status: "Downloading...".to_string(), progress });
        }
    }
    
    app.emit("install_progress", InstallStatus { status: "Extracting...".to_string(), progress: 95 }).unwrap();
    
    // Extract zip
    let file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    archive.extract(&app_dir).map_err(|e| e.to_string())?;
    
    // Clean up zip
    let _ = std::fs::remove_file(zip_path);
    
    app.emit("install_progress", InstallStatus { status: "Done!".to_string(), progress: 100 }).unwrap();
    
    Ok(())
}

async fn get_latest_release_url() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Anon-Instance-Manager")
        .build()?;
        
    let res: GitHubRelease = client
        .get("https://api.github.com/repos/daijro/camoufox/releases/latest")
        .send()
        .await?
        .json()
        .await?;
        
    // Find Windows x86_64 zip first, then fall back to any Windows zip
    let mut win_fallback: Option<String> = None;
    for asset in &res.assets {
        if asset.name.contains("win") && asset.name.ends_with(".zip") {
            if asset.name.contains("x86_64") {
                return Ok(asset.browser_download_url.clone());
            }
            if win_fallback.is_none() {
                win_fallback = Some(asset.browser_download_url.clone());
            }
        }
    }
    
    win_fallback.ok_or_else(|| "No Windows release found".into())
}
