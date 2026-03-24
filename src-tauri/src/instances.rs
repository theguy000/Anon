use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

use crate::camoufox::get_app_dir;
use crate::process_manager;

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
    pub auto_change_window_size: Option<bool>,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProxyConfig {
    pub proxy_type: Option<String>, // "http", "socks4", "socks5", or null/none
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
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
    #[serde(default)]
    pub proxy_config: Option<ProxyConfig>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub proxy_pool: Option<Vec<ProxyConfig>>,
    #[serde(default)]
    pub proxy_rotation_mode: Option<String>,
    #[serde(default)]
    pub proxy_rotation_index: Option<usize>,
    #[serde(default)]
    pub fingerprint_pool: Option<Vec<FingerprintConfig>>,
    #[serde(default)]
    pub fingerprint_rotation_mode: Option<String>,
    #[serde(default)]
    pub fingerprint_rotation_index: Option<usize>,
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
    proxy_config: &Option<ProxyConfig>,
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

    // Proxy settings — prefer structured proxy_config, fall back to legacy proxy string
    if let Some(pc) = proxy_config {
        if let Some(ref ptype) = pc.proxy_type {
            let host = pc.host.as_deref().unwrap_or("");
            let port = pc.port.unwrap_or(0);

            if !host.is_empty() && port > 0 {
                // Enable manual proxy configuration
                user_js_content.push_str("user_pref(\"network.proxy.type\", 1);\n");

                match ptype.as_str() {
                    "http" => {
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.http\", \"{}\");\n",
                            host
                        ));
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.http_port\", {});\n",
                            port
                        ));
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.ssl\", \"{}\");\n",
                            host
                        ));
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.ssl_port\", {});\n",
                            port
                        ));
                    }
                    "socks4" => {
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.socks\", \"{}\");\n",
                            host
                        ));
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.socks_port\", {});\n",
                            port
                        ));
                        user_js_content
                            .push_str("user_pref(\"network.proxy.socks_version\", 4);\n");
                        user_js_content
                            .push_str("user_pref(\"network.proxy.socks_remote_dns\", false);\n");
                    }
                    "socks5" => {
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.socks\", \"{}\");\n",
                            host
                        ));
                        user_js_content.push_str(&format!(
                            "user_pref(\"network.proxy.socks_port\", {});\n",
                            port
                        ));
                        user_js_content
                            .push_str("user_pref(\"network.proxy.socks_version\", 5);\n");
                        user_js_content
                            .push_str("user_pref(\"network.proxy.socks_remote_dns\", true);\n");
                    }
                    _ => {}
                }
            }
        }
    } else if proxy.is_some() {
        // Legacy fallback: old proxy string
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
        proxy_config: None,
        tags: None,
        notes: None,
        proxy_pool: None,
        proxy_rotation_mode: None,
        proxy_rotation_index: None,
        fingerprint_pool: None,
        fingerprint_rotation_mode: None,
        fingerprint_rotation_index: None,
    };

    // Save anon config
    let config_path = instance_dir.join("anon_config.json");
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(config_path, config_json).map_err(|e| e.to_string())?;

    // Generate user.js for persistence and proxy settings
    let _ = ensure_user_js(
        &instance_dir,
        &config.proxy,
        &config.proxy_config,
        config.persist_data,
    );

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

pub async fn launch_instance(app: &AppHandle, id: String, startup_url: Option<String>) -> Result<u32, String> {
    // Check if already running
    if process_manager::is_running(&id) {
        return Err("Instance is already running".to_string());
    }

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
    let proxy_config = config.as_ref().and_then(|c| c.proxy_config.clone());
    let persist_data = config.as_ref().is_none_or(|c| c.persist_data);

    // Proxy rotation
    let effective_proxy;
    let effective_proxy_config;
    if let Some(ref pool) = config.as_ref().and_then(|c| c.proxy_pool.as_ref()) {
        if !pool.is_empty() {
            let mode = config.as_ref().and_then(|c| c.proxy_rotation_mode.as_deref()).unwrap_or("sequential");
            let idx = if mode == "random" {
                use rand::Rng;
                rand::thread_rng().gen_range(0..pool.len())
            } else {
                let current = config.as_ref().and_then(|c| c.proxy_rotation_index).unwrap_or(0);
                let idx = current % pool.len();
                // Update index for next launch
                if let Ok(contents) = fs::read_to_string(&config_path) {
                    if let Ok(mut cfg) = serde_json::from_str::<InstanceConfig>(&contents) {
                        cfg.proxy_rotation_index = Some(idx + 1);
                        let _ = fs::write(&config_path, serde_json::to_string_pretty(&cfg).unwrap());
                    }
                }
                idx
            };
            effective_proxy_config = Some(pool[idx].clone());
            // Build legacy proxy string from pool entry
            effective_proxy = effective_proxy_config.as_ref().and_then(|pc| {
                let ptype = pc.proxy_type.as_deref()?;
                let host = pc.host.as_deref()?;
                let port = pc.port?;
                Some(format!("{}://{}:{}", ptype, host, port))
            });
        } else {
            effective_proxy = proxy.clone();
            effective_proxy_config = proxy_config.clone();
        }
    } else {
        effective_proxy = proxy.clone();
        effective_proxy_config = proxy_config.clone();
    }

    // Update user.js on every launch to ensure preferences are applied
    let _ = ensure_user_js(&instance_dir, &effective_proxy, &effective_proxy_config, persist_data);

    let bin_path = crate::camoufox::get_camoufox_binary(app)
        .await
        .ok_or_else(|| "Camoufox binary not downloaded".to_string())?;

    // Fingerprint rotation
    let effective_fp = if let Some(ref pool) = config.as_ref().and_then(|c| c.fingerprint_pool.as_ref()) {
        if !pool.is_empty() {
            let mode = config.as_ref().and_then(|c| c.fingerprint_rotation_mode.as_deref()).unwrap_or("sequential");
            let idx = if mode == "random" {
                use rand::Rng;
                rand::thread_rng().gen_range(0..pool.len())
            } else {
                let current = config.as_ref().and_then(|c| c.fingerprint_rotation_index).unwrap_or(0);
                let idx = current % pool.len();
                if let Ok(contents) = fs::read_to_string(&config_path) {
                    if let Ok(mut cfg) = serde_json::from_str::<InstanceConfig>(&contents) {
                        cfg.fingerprint_rotation_index = Some(idx + 1);
                        let _ = fs::write(&config_path, serde_json::to_string_pretty(&cfg).unwrap());
                    }
                }
                idx
            };
            Some(&pool[idx])
        } else {
            config.as_ref().and_then(|c| c.fingerprint.as_ref())
        }
    } else {
        config.as_ref().and_then(|c| c.fingerprint.as_ref())
    };

    // Build the CAMOU_CONFIG JSON from fingerprint settings
    let camou_config_json = if let Some(fp) = effective_fp {
        if fp.auto_fingerprint == Some(true) {
            crate::auto_fingerprint::generate_auto_config(
                fp.auto_change_window_size.unwrap_or(true),
                fp.outer_width,
                fp.outer_height,
            )
            .to_string()
        } else {
            build_camou_config(fp).to_string()
        }
    } else {
        r#"{"showcursor":false}"#.to_string()
    };

    // Spawn detached process with CAMOU_CONFIG env var
    let mut cmd = std::process::Command::new(bin_path);
    cmd.arg("--profile").arg(&instance_dir).env("CAMOU_CONFIG", &camou_config_json);
    if let Some(ref url) = startup_url {
        cmd.arg(url);
    }
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to launch instance: {}", e))?;

    let pid = child.id();

    // Register in process manager
    process_manager::register(&id, pid);

    // Spawn a lightweight async task to track the real browser process.
    //
    // On Windows, camoufox uses Firefox's launcher-process pattern:
    //   1. The spawned child (stub) exits quickly after launching the real browser.
    //   2. We wait for the stub to exit, then scan for the real browser PID by
    //      matching --profile <instance_dir> in process command-line arguments.
    //   3. We update the registry with the real PID and poll until it exits.
    //   4. We emit `instance-stopped` so the frontend can update its state.
    let app_handle = app.clone();
    let instance_id = id.clone();
    let profile_dir = instance_dir.clone();
    tokio::spawn(async move {
        // Step 1: reap the stub on the blocking pool — returns quickly.
        let _ = tokio::task::spawn_blocking(move || child.wait()).await;

        // Step 2: give the real browser a moment to start, then find its PID.
        // Retry a few times since Firefox may still be initialising.
        let mut real_pid: Option<u32> = None;
        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if let Some(found) = process_manager::find_browser_pid(&profile_dir) {
                real_pid = Some(found);
                break;
            }
        }

        if let Some(browser_pid) = real_pid {
            // Step 3: update registry with the real browser PID and poll until it exits.
            process_manager::update_pid(&instance_id, browser_pid);
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if !process_manager::is_pid_alive(browser_pid) {
                    break;
                }
            }
        }
        // If we never found the real PID the browser either never started or
        // exited before we could find it — either way fall through to cleanup.

        // Step 4: clean up and notify frontend.
        process_manager::unregister(&instance_id);
        let _ = app_handle.emit("instance-stopped", &instance_id);
    });

    Ok(pid)
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
            let _ = ensure_user_js(
                &instance_dir,
                &config.proxy,
                &config.proxy_config,
                config.persist_data,
            );

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
) -> Result<Vec<crate::fingerprint_validator::FingerprintConflict>, String> {
    // Validate fingerprint coherence (skipped in AUTO mode)
    let conflicts = crate::fingerprint_validator::validate_fingerprint(&fingerprint);

    // Return conflicts without persisting so the UI can ask the user to fix them first
    if !conflicts.is_empty() {
        return Ok(conflicts);
    }

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
            return Ok(conflicts);
        }
    }

    Err("Failed to update instance settings".to_string())
}

pub async fn update_instance_proxy(
    app: &AppHandle,
    id: String,
    proxy_config: Option<ProxyConfig>,
) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);

    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.proxy_config = proxy_config.clone();

            // Also update the legacy proxy field for display purposes
            config.proxy = proxy_config.as_ref().and_then(|pc| {
                let ptype = pc.proxy_type.as_deref()?;
                let host = pc.host.as_deref()?;
                let port = pc.port?;
                if host.is_empty() || port == 0 {
                    return None;
                }
                match (pc.username.as_deref(), pc.password.as_deref()) {
                    (Some(user), Some(pass)) if !user.is_empty() && !pass.is_empty() => {
                        Some(format!("{}://{}:{}@{}:{}", ptype, user, pass, host, port))
                    }
                    (Some(user), _) if !user.is_empty() => {
                        Some(format!("{}://{}@{}:{}", ptype, user, host, port))
                    }
                    _ => Some(format!("{}://{}:{}", ptype, host, port)),
                }
            });

            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;

            // Regenerate user.js with updated proxy settings
            let _ = ensure_user_js(
                &instance_dir,
                &config.proxy,
                &config.proxy_config,
                config.persist_data,
            );

            return Ok(());
        }
    }

    Err("Failed to update instance proxy settings".to_string())
}

pub async fn rename_instance(app: &AppHandle, id: String, new_name: String) -> Result<(), String> {
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        return Err("Instance name cannot be empty".to_string());
    }
    if process_manager::is_running(&id) {
        return Err("Cannot rename a running instance".to_string());
    }
    let instances = list_instances(app).await?;
    if instances.iter().any(|i| i.id != id && i.name.eq_ignore_ascii_case(&new_name)) {
        return Err("An instance with this name already exists".to_string());
    }
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.name = new_name;
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err("Failed to rename instance".to_string())
}

pub async fn update_instance_tags(app: &AppHandle, id: String, tags: Vec<String>) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.tags = if tags.is_empty() { None } else { Some(tags) };
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err("Failed to update instance tags".to_string())
}

pub async fn update_instance_notes(app: &AppHandle, id: String, notes: Option<String>) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.notes = notes.filter(|n| !n.trim().is_empty());
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err("Failed to update instance notes".to_string())
}

pub async fn export_instance(app: &AppHandle, id: String) -> Result<String, String> {
    let profiles_dir = get_profiles_dir(app).await;
    let config_path = profiles_dir.join(&id).join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<InstanceConfig>(&contents) {
            return serde_json::to_string_pretty(&config).map_err(|e| e.to_string());
        }
    }
    Err("Instance not found".to_string())
}

pub async fn export_all_instances(app: &AppHandle) -> Result<String, String> {
    let instances = list_instances(app).await?;
    let settings = crate::settings::load_settings(app).await;
    let export = serde_json::json!({
        "version": "1.0",
        "instances": instances,
        "settings": settings,
    });
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

pub async fn import_instances(app: &AppHandle, json: String) -> Result<Vec<InstanceConfig>, String> {
    let existing = list_instances(app).await?;
    let profiles_dir = get_profiles_dir(app).await;

    // Try parsing as array of instances or as export bundle
    let configs_to_import: Vec<InstanceConfig> = if let Ok(bundle) = serde_json::from_str::<serde_json::Value>(&json) {
        if let Some(arr) = bundle.get("instances").and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|v| serde_json::from_value::<InstanceConfig>(v.clone()).ok())
                .collect()
        } else if let Ok(single) = serde_json::from_str::<InstanceConfig>(&json) {
            vec![single]
        } else if let Ok(arr) = serde_json::from_str::<Vec<InstanceConfig>>(&json) {
            arr
        } else {
            return Err("Invalid import format".to_string());
        }
    } else {
        return Err("Invalid JSON".to_string());
    };

    let mut imported = Vec::new();
    for config in configs_to_import {
        let new_id = uuid::Uuid::new_v4().to_string();
        let mut new_name = config.name.clone();

        // Handle name conflicts
        let mut counter = 0;
        while existing.iter().any(|i| i.name.eq_ignore_ascii_case(&new_name))
            || imported.iter().any(|i: &InstanceConfig| i.name.eq_ignore_ascii_case(&new_name)) {
            counter += 1;
            new_name = format!("{} (imported{})", config.name, if counter > 1 { format!(" {}", counter) } else { String::new() });
        }

        let new_config = InstanceConfig {
            id: new_id.clone(),
            name: new_name,
            proxy: config.proxy,
            persist_data: config.persist_data,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            fingerprint: config.fingerprint,
            proxy_config: config.proxy_config,
            tags: config.tags,
            notes: config.notes,
            proxy_pool: config.proxy_pool,
            proxy_rotation_mode: config.proxy_rotation_mode,
            proxy_rotation_index: None,
            fingerprint_pool: config.fingerprint_pool,
            fingerprint_rotation_mode: config.fingerprint_rotation_mode,
            fingerprint_rotation_index: None,
        };

        let instance_dir = profiles_dir.join(&new_id);
        fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;
        let config_path = instance_dir.join("anon_config.json");
        let config_json = serde_json::to_string_pretty(&new_config).unwrap();
        fs::write(config_path, config_json).map_err(|e| e.to_string())?;
        let _ = ensure_user_js(&instance_dir, &new_config.proxy, &new_config.proxy_config, new_config.persist_data);
        imported.push(new_config);
    }
    Ok(imported)
}

pub async fn update_proxy_pool(app: &AppHandle, id: String, pool: Vec<ProxyConfig>, mode: Option<String>) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let config_path = profiles_dir.join(&id).join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.proxy_pool = if pool.is_empty() { None } else { Some(pool) };
            config.proxy_rotation_mode = mode;
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err("Failed to update proxy pool".to_string())
}

pub async fn update_fingerprint_pool(app: &AppHandle, id: String, pool: Vec<FingerprintConfig>, mode: Option<String>) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let config_path = profiles_dir.join(&id).join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.fingerprint_pool = if pool.is_empty() { None } else { Some(pool) };
            config.fingerprint_rotation_mode = mode;
            let config_json = serde_json::to_string_pretty(&config).unwrap();
            fs::write(&config_path, config_json).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err("Failed to update fingerprint pool".to_string())
}
