use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

use crate::camoufox::get_app_dir;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FingerprintConfig {
    // Navigator
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub oscpu: Option<String>,
    pub app_code_name: Option<String>,
    pub app_name: Option<String>,
    pub app_version: Option<String>,
    pub product: Option<String>,
    pub product_sub: Option<String>,
    pub build_id: Option<String>,
    pub hardware_concurrency: Option<u32>,
    pub max_touch_points: Option<u32>,
    pub do_not_track: Option<String>,
    pub language: Option<String>,
    pub languages: Option<String>,
    pub cookie_enabled: Option<bool>,
    pub global_privacy_control: Option<bool>,
    pub online: Option<bool>,

    // Screen & Display
    pub screen_height: Option<u32>,
    pub screen_width: Option<u32>,
    pub screen_avail_height: Option<u32>,
    pub screen_avail_width: Option<u32>,
    pub screen_avail_top: Option<u32>,
    pub screen_avail_left: Option<u32>,
    pub color_depth: Option<u32>,
    pub pixel_depth: Option<u32>,
    pub device_pixel_ratio: Option<f64>,

    // Window
    pub outer_height: Option<u32>,
    pub outer_width: Option<u32>,
    pub inner_height: Option<u32>,
    pub inner_width: Option<u32>,
    pub screen_x: Option<i32>,
    pub screen_y: Option<i32>,

    // WebGL
    pub webgl_renderer: Option<String>,
    pub webgl_vendor: Option<String>,
    pub webgl_block_if_not_defined: Option<bool>,

    // Canvas & Audio Seeds
    pub canvas_seed: Option<u32>,
    pub audio_seed: Option<u32>,

    // AudioContext
    pub audio_sample_rate: Option<u32>,
    pub audio_output_latency: Option<f64>,
    pub audio_max_channel_count: Option<u32>,

    // Fonts
    pub fonts_spacing_seed: Option<u32>,

    // Geolocation, Timezone & Locale
    pub geo_latitude: Option<f64>,
    pub geo_longitude: Option<f64>,
    pub geo_accuracy: Option<f64>,
    pub timezone: Option<String>,
    pub locale_language: Option<String>,
    pub locale_region: Option<String>,

    // WebRTC
    pub webrtc_ipv4: Option<String>,
    pub webrtc_ipv6: Option<String>,
    pub webrtc_local_ipv4: Option<String>,
    pub webrtc_local_ipv6: Option<String>,

    // HTTP Headers
    pub header_user_agent: Option<String>,
    pub header_accept_language: Option<String>,
    pub header_accept_encoding: Option<String>,

    // Battery
    pub battery_charging: Option<bool>,
    pub battery_charging_time: Option<f64>,
    pub battery_discharging_time: Option<f64>,
    pub battery_level: Option<f64>,

    // Media Devices
    pub media_micros: Option<u32>,
    pub media_webcams: Option<u32>,
    pub media_speakers: Option<u32>,

    // Speech Voices
    pub speech_voices: Option<Vec<String>>,

    // Behavior
    pub humanize: Option<bool>,
    pub showcursor: Option<bool>,
    pub pdf_viewer_enabled: Option<bool>,

    // Advanced
    pub allow_main_world: Option<bool>,
    pub force_scope_access: Option<bool>,
    pub memory_saver: Option<bool>,

    // Global Preset Selection (persisted so it survives restart)
    pub global_category: Option<String>,
    pub global_preset_index: Option<i32>,

    // AUTO mode: let camoufox's built-in browserforge handle all fingerprinting
    pub auto_fingerprint: Option<bool>,
}

/// Convert a FingerprintConfig into a JSON object that camoufox understands
/// (property keys from camoufox's properties.json).
/// Only non-None fields are included so camoufox falls back to its defaults.
fn build_camou_config(fp: &FingerprintConfig) -> serde_json::Value {
    let mut m = serde_json::Map::new();

    // ── Helper macros ────────────────────────────────────────────────────
    macro_rules! set_str {
        ($key:expr, $field:expr) => {
            if let Some(ref v) = $field {
                m.insert($key.to_string(), serde_json::Value::String(v.clone()));
            }
        };
    }
    macro_rules! set_u32 {
        ($key:expr, $field:expr) => {
            if let Some(v) = $field {
                m.insert($key.to_string(), serde_json::json!(v));
            }
        };
    }
    macro_rules! set_i32 {
        ($key:expr, $field:expr) => {
            if let Some(v) = $field {
                m.insert($key.to_string(), serde_json::json!(v));
            }
        };
    }
    macro_rules! set_f64 {
        ($key:expr, $field:expr) => {
            if let Some(v) = $field {
                m.insert($key.to_string(), serde_json::json!(v));
            }
        };
    }
    macro_rules! set_bool {
        ($key:expr, $field:expr) => {
            if let Some(v) = $field {
                m.insert($key.to_string(), serde_json::json!(v));
            }
        };
    }

    // ── Navigator ────────────────────────────────────────────────────────
    set_str!("navigator.userAgent", fp.user_agent);
    set_str!("navigator.platform", fp.platform);
    set_str!("navigator.oscpu", fp.oscpu);
    set_str!("navigator.appCodeName", fp.app_code_name);
    set_str!("navigator.appName", fp.app_name);
    set_str!("navigator.appVersion", fp.app_version);
    set_str!("navigator.product", fp.product);
    set_str!("navigator.productSub", fp.product_sub);
    set_str!("navigator.buildID", fp.build_id);
    set_u32!("navigator.hardwareConcurrency", fp.hardware_concurrency);
    set_u32!("navigator.maxTouchPoints", fp.max_touch_points);
    set_str!("navigator.doNotTrack", fp.do_not_track);
    set_str!("navigator.language", fp.language);
    set_bool!("navigator.cookieEnabled", fp.cookie_enabled);
    set_bool!("navigator.globalPrivacyControl", fp.global_privacy_control);
    set_bool!("navigator.onLine", fp.online);

    // languages: comma-separated string → JSON array
    if let Some(ref langs) = fp.languages {
        let arr: Vec<serde_json::Value> = langs
            .split(',')
            .map(|s| serde_json::Value::String(s.trim().to_string()))
            .collect();
        m.insert(
            "navigator.languages".to_string(),
            serde_json::Value::Array(arr),
        );
    }

    // ── Screen & Display ─────────────────────────────────────────────────
    set_u32!("screen.height", fp.screen_height);
    set_u32!("screen.width", fp.screen_width);
    set_u32!("screen.availHeight", fp.screen_avail_height);
    set_u32!("screen.availWidth", fp.screen_avail_width);
    set_u32!("screen.availTop", fp.screen_avail_top);
    set_u32!("screen.availLeft", fp.screen_avail_left);
    set_u32!("screen.colorDepth", fp.color_depth);
    set_u32!("screen.pixelDepth", fp.pixel_depth);
    set_f64!("window.devicePixelRatio", fp.device_pixel_ratio);

    // ── Window ────────────────────────────────────────────────────────
    set_u32!("window.outerHeight", fp.outer_height);
    set_u32!("window.outerWidth", fp.outer_width);
    set_u32!("window.innerHeight", fp.inner_height);
    set_u32!("window.innerWidth", fp.inner_width);
    set_i32!("window.screenX", fp.screen_x);
    set_i32!("window.screenY", fp.screen_y);

    // ── WebGL ────────────────────────────────────────────────────────
    set_str!("webGl:renderer", fp.webgl_renderer);
    set_str!("webGl:vendor", fp.webgl_vendor);
    set_bool!(
        "webGl:parameters:blockIfNotDefined",
        fp.webgl_block_if_not_defined
    );

    // ── Canvas & Audio Seeds ─────────────────────────────────────────
    set_u32!("canvas:seed", fp.canvas_seed);
    set_u32!("audio:seed", fp.audio_seed);

    // ── AudioContext ─────────────────────────────────────────────────
    set_u32!("AudioContext:sampleRate", fp.audio_sample_rate);
    set_f64!("AudioContext:outputLatency", fp.audio_output_latency);
    set_u32!("AudioContext:maxChannelCount", fp.audio_max_channel_count);

    // ── Fonts ────────────────────────────────────────────────────────
    set_u32!("fonts:spacing_seed", fp.fonts_spacing_seed);

    // ── Geolocation, Timezone & Locale ───────────────────────────────
    set_f64!("geolocation:latitude", fp.geo_latitude);
    set_f64!("geolocation:longitude", fp.geo_longitude);
    set_f64!("geolocation:accuracy", fp.geo_accuracy);
    set_str!("timezone", fp.timezone);
    set_str!("locale:language", fp.locale_language);
    set_str!("locale:region", fp.locale_region);

    // ── WebRTC ───────────────────────────────────────────────────────
    set_str!("webrtc:ipv4", fp.webrtc_ipv4);
    set_str!("webrtc:ipv6", fp.webrtc_ipv6);
    set_str!("webrtc:localipv4", fp.webrtc_local_ipv4);
    set_str!("webrtc:localipv6", fp.webrtc_local_ipv6);

    // ── HTTP Headers ────────────────────────────────────────────────
    set_str!("headers.User-Agent", fp.header_user_agent);
    set_str!("headers.Accept-Language", fp.header_accept_language);
    set_str!("headers.Accept-Encoding", fp.header_accept_encoding);

    // ── Battery ──────────────────────────────────────────────────────
    set_bool!("battery:charging", fp.battery_charging);
    set_f64!("battery:chargingTime", fp.battery_charging_time);
    set_f64!("battery:dischargingTime", fp.battery_discharging_time);
    set_f64!("battery:level", fp.battery_level);

    // ── Media Devices ───────────────────────────────────────────────
    set_u32!("mediaDevices:micros", fp.media_micros);
    set_u32!("mediaDevices:webcams", fp.media_webcams);
    set_u32!("mediaDevices:speakers", fp.media_speakers);

    // ── Speech Voices ───────────────────────────────────────────────
    // Stored as ["Name:lang:type", ...], camoufox expects
    // [{name, lang, voiceUri, isDefault, isLocalService}, ...]
    if let Some(ref voices) = fp.speech_voices {
        let arr: Vec<serde_json::Value> = voices
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let parts: Vec<&str> = entry.splitn(3, ':').collect();
                let name = parts.first().unwrap_or(&"").to_string();
                let lang = parts.get(1).unwrap_or(&"").to_string();
                let voice_uri = name.clone();
                serde_json::json!({
                    "name": name,
                    "lang": lang,
                    "voiceUri": voice_uri,
                    "isDefault": i == 0,
                    "isLocalService": parts.get(2).is_none_or(|t| *t == "local")
                })
            })
            .collect();
        m.insert("voices".to_string(), serde_json::Value::Array(arr));
    }

    // ── Behavior ─────────────────────────────────────────────────────
    set_bool!("humanize", fp.humanize);
    // Default showcursor to false when not explicitly set
    m.insert(
        "showcursor".to_string(),
        serde_json::json!(fp.showcursor.unwrap_or(false)),
    );
    set_bool!("pdfViewerEnabled", fp.pdf_viewer_enabled);

    // ── Advanced ─────────────────────────────────────────────────────
    set_bool!("allowMainWorld", fp.allow_main_world);
    set_bool!("forceScopeAccess", fp.force_scope_access);
    set_bool!("memorysaver", fp.memory_saver);

    // global_category / global_preset_index are UI-only, not sent.

    serde_json::Value::Object(m)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    pub proxy: Option<String>,
    pub persist_data: bool,
    pub created_at: i64,
    #[serde(default)]
    pub fingerprint: Option<FingerprintConfig>,
}

pub async fn get_profiles_dir(app: &AppHandle) -> PathBuf {
    let mut path = get_app_dir(app).await;
    path.push("profiles");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

fn ensure_user_js(
    instance_dir: &Path,
    proxy: &Option<String>,
    persist_data: bool,
) -> io::Result<()> {
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
    if proxy.is_some() {
        user_js_content.push_str("user_pref(\"network.proxy.type\", 1);\n");
    }

    fs::write(user_js_path, user_js_content)
}

fn cleanup_instance_data(instance_dir: &Path) -> io::Result<()> {
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

pub async fn create_instance(
    app: &AppHandle,
    name: String,
    proxy: Option<String>,
    persist_data: bool,
) -> Result<InstanceConfig, String> {
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
        fingerprint: None,
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

    // Load the full instance config (including fingerprint)
    let config_path = instance_dir.join("anon_config.json");
    let config: Option<InstanceConfig> = if config_path.exists() {
        fs::read_to_string(&config_path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
    } else {
        None
    };

    let proxy = config.as_ref().and_then(|c| c.proxy.clone());
    let persist_data = config.as_ref().is_none_or(|c| c.persist_data);

    // Update user.js on every launch to ensure preferences are applied
    let _ = ensure_user_js(&instance_dir, &proxy, persist_data);

    let bin_path = crate::camoufox::get_camoufox_binary(app)
        .await
        .ok_or_else(|| "Camoufox binary not downloaded".to_string())?;

    // Build the CAMOU_CONFIG JSON from fingerprint settings
    let camou_config_json = if let Some(fp) = config.as_ref().and_then(|c| c.fingerprint.as_ref()) {
        if fp.auto_fingerprint == Some(true) {
            crate::auto_fingerprint::generate_auto_config().to_string()
        } else {
            build_camou_config(fp).to_string()
        }
    } else {
        r#"{"showcursor":false}"#.to_string()
    };

    // Spawn detached process with CAMOU_CONFIG env var
    std::process::Command::new(bin_path)
        .arg("--profile")
        .arg(&instance_dir)
        .env("CAMOU_CONFIG", &camou_config_json)
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

pub async fn update_instance_settings(
    app: &AppHandle,
    id: String,
    fingerprint: FingerprintConfig,
) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);

    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.fingerprint = Some(fingerprint);
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err("Failed to update instance settings".to_string())
}
