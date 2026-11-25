use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

use crate::camoufox::{get_data_dir, write_json_atomic};
use crate::process_manager;

pub(crate) const ANON_INTERNAL_ENTRIES: &[&str] =
    &["anon_config.json", "user.js", "proxy_auth_ext"];

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct FingerprintConfig {
    // Navigator (active Camoufox v159 properties only)
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub oscpu: Option<String>,
    pub app_version: Option<String>,
    pub hardware_concurrency: Option<u32>,
    pub max_touch_points: Option<u32>,
    pub do_not_track: Option<String>,
    pub language: Option<String>,
    pub global_privacy_control: Option<bool>,

    // CPU identity (Camoufox v159)
    pub cpu_arch: Option<String>,
    pub cpu_manufacturer: Option<String>,

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

    // WebGL & Canvas
    pub webgl_renderer: Option<String>,
    pub webgl_vendor: Option<String>,
    pub webgl_block_if_not_defined: Option<bool>,
    pub block_webgl: Option<bool>,
    pub canvas_noise: Option<bool>,

    // AudioContext
    pub audio_sample_rate: Option<u32>,
    pub audio_output_latency: Option<f64>,
    pub audio_max_channel_count: Option<u32>,

    // Fonts
    pub fonts: Option<Vec<String>>,

    // Geolocation, Timezone & Locale
    pub geo_latitude: Option<f64>,
    pub geo_longitude: Option<f64>,
    pub geo_accuracy: Option<f64>,
    pub timezone: Option<String>,
    pub locale_language: Option<String>,
    pub locale_region: Option<String>,
    pub locale_script: Option<String>,

    // WebRTC & Network
    pub block_webrtc: Option<bool>,
    pub webrtc_ipv4: Option<String>,
    pub webrtc_ipv6: Option<String>,
    pub network_ipv4_only: Option<bool>,

    // HTTP Headers
    pub header_user_agent: Option<String>,
    pub header_accept_language: Option<String>,
    pub header_accept_encoding: Option<String>,

    // Media Devices & Codecs
    pub media_micros: Option<u32>,
    pub media_webcams: Option<u32>,
    pub media_speakers: Option<u32>,
    pub media_spoof_codecs: Option<bool>,

    // Speech Voices
    pub speech_voices: Option<Vec<String>>,
    pub voices_block_if_not_defined: Option<bool>,

    // Behavior & Privacy (Camoufox v159)
    pub showcursor: Option<bool>,
    pub enable_ublock: Option<bool>,
    pub block_images: Option<bool>,
    pub disable_coop: Option<bool>,

    // Advanced (active Camoufox v159 flags)
    pub allow_main_world: Option<bool>,
    pub disable_world_isolation: Option<bool>,
    pub allow_addon_new_tab: Option<bool>,
    pub force_scope_access: Option<bool>,
    pub instant_animations: Option<bool>,
    pub debug: Option<bool>,

    // Global Preset Selection (UI state)
    pub global_category: Option<String>,
    pub global_preset_index: Option<i32>,

    // AUTO mode
    pub auto_fingerprint: Option<bool>,
    pub auto_os: Option<String>,
    pub auto_language: Option<String>,
    pub auto_change_window_size: Option<bool>,
    pub identity_nonce: Option<u64>,
}

impl FingerprintConfig {
    pub fn auto_default() -> Self {
        Self {
            auto_fingerprint: Some(true),
            auto_os: None,
            auto_change_window_size: Some(true),
            identity_nonce: Some(0),
            ..Self::default()
        }
    }
}

pub fn extract_major_version(v: &str) -> Option<&str> {
    for word in v.split(|c: char| !c.is_ascii_alphanumeric() && c != '.') {
        let clean = word.trim_start_matches('v');
        if let Some(dot_idx) = clean.find('.') {
            let major = &clean[..dot_idx];
            if major.len() == 3 && major.chars().all(|c| c.is_ascii_digit()) {
                return Some(major);
            }
        } else if clean.len() == 3 && clean.chars().all(|c| c.is_ascii_digit()) {
            return Some(clean);
        }
    }
    None
}

pub fn harmonize_firefox_user_agent(ua: &str, ff_version: &str) -> String {
    if !ua.contains("Firefox/") && !ua.contains("rv:") {
        return ua.to_string();
    }
    let mut res = String::with_capacity(ua.len());
    let mut i = 0;
    let bytes = ua.as_bytes();
    while i < bytes.len() {
        if ua[i..].starts_with("Firefox/") {
            res.push_str("Firefox/");
            let start = i + 8;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
                end += 1;
            }
            res.push_str(ff_version);
            res.push_str(".0");
            i = end;
        } else if ua[i..].starts_with("rv:") {
            res.push_str("rv:");
            let start = i + 3;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
                end += 1;
            }
            res.push_str(ff_version);
            res.push_str(".0");
            i = end;
        } else {
            let ch = ua[i..].chars().next().unwrap();
            res.push(ch);
            i += ch.len_utf8();
        }
    }
    res
}

fn apply_shared_v159_flags(
    fp: &FingerprintConfig,
    config: &mut serde_json::Map<String, serde_json::Value>,
    prefs: &mut serde_json::Map<String, serde_json::Value>,
) {
    if let Some(v) = fp.showcursor {
        config.insert("showcursor".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.media_spoof_codecs {
        config.insert("media:spoof_codecs".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.network_ipv4_only {
        config.insert("network:ipv4Only".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.allow_main_world {
        config.insert("allowMainWorld".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.disable_world_isolation {
        config.insert("disableWorldIsolation".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.allow_addon_new_tab {
        config.insert("allowAddonNewtab".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.force_scope_access {
        config.insert("forceScopeAccess".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.instant_animations {
        config.insert("instantAnimations".into(), serde_json::json!(v));
    }
    if let Some(v) = fp.debug {
        config.insert("debug".into(), serde_json::json!(v));
    }

    if fp.block_images == Some(true) {
        prefs.insert("permissions.default.image".into(), serde_json::json!(2));
    }
    if fp.block_webgl == Some(true) {
        prefs.insert("webgl.disabled".into(), serde_json::json!(true));
    }
    if fp.disable_coop == Some(true) {
        prefs.insert(
            "browser.tabs.remote.useCrossOriginOpenerPolicy".into(),
            serde_json::json!(false),
        );
    }
    if fp.canvas_noise == Some(true) {
        prefs.insert(
            "privacy.baselineFingerprintingProtection.overrides".into(),
            serde_json::json!(crate::auto_fingerprint::CANVAS_NOISE_OVERRIDES),
        );
        prefs.insert("gfx.blacklist.webgpu".into(), serde_json::json!(4));
    }
}

/// Convert a manual `FingerprintConfig` into active Camoufox v159 `CAMOU_CONFIG`
/// and `CAMOU_PREFS` maps, emitting zero removed properties.
pub(crate) fn build_camou_config_and_prefs(
    fp: &FingerprintConfig,
    target_ff_version: Option<&str>,
) -> (
    serde_json::Map<String, serde_json::Value>,
    serde_json::Map<String, serde_json::Value>,
) {
    let mut m = serde_json::Map::new();
    let mut prefs = serde_json::Map::new();

    macro_rules! set_str {
        ($key:expr, $field:expr) => {
            if let Some(ref v) = $field {
                if !v.trim().is_empty() {
                    m.insert($key.to_string(), serde_json::Value::String(v.clone()));
                }
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

    let effective_ua = fp.user_agent.as_ref().map(|ua| {
        if let Some(ver) = target_ff_version {
            harmonize_firefox_user_agent(ua, ver)
        } else {
            ua.clone()
        }
    });
    set_str!("navigator.userAgent", effective_ua);
    set_str!("navigator.platform", fp.platform);
    set_str!("navigator.oscpu", fp.oscpu);
    let effective_app_ver = fp.app_version.as_ref().map(|v| {
        if let Some(ver) = target_ff_version {
            harmonize_firefox_user_agent(v, ver)
        } else {
            v.clone()
        }
    });
    set_str!("navigator.appVersion", effective_app_ver);
    set_u32!("navigator.hardwareConcurrency", fp.hardware_concurrency);
    set_u32!("navigator.maxTouchPoints", fp.max_touch_points);
    set_str!("navigator.doNotTrack", fp.do_not_track);
    set_str!("navigator.language", fp.language);
    set_bool!("navigator.globalPrivacyControl", fp.global_privacy_control);

    if let Some(ref dnt) = fp.do_not_track {
        prefs.insert(
            "privacy.donottrackheader.enabled".into(),
            serde_json::json!(dnt == "1"),
        );
    }
    if let Some(gpc) = fp.global_privacy_control {
        prefs.insert(
            "privacy.globalprivacycontrol.enabled".into(),
            serde_json::json!(gpc),
        );
    }

    // CPU identity
    set_str!("cpu:arch", fp.cpu_arch);
    set_str!("cpu:manufacturer", fp.cpu_manufacturer);
    if let Some(pref_val) = crate::auto_fingerprint::cpu_identity_pref(
        fp.cpu_arch.as_deref(),
        fp.cpu_manufacturer.as_deref(),
    ) {
        prefs.insert("general.cpu_identity".into(), serde_json::json!(pref_val));
    }

    // Screen & Display
    set_u32!("screen.height", fp.screen_height);
    set_u32!("screen.width", fp.screen_width);
    set_u32!("screen.availHeight", fp.screen_avail_height);
    set_u32!("screen.availWidth", fp.screen_avail_width);
    set_u32!("screen.availTop", fp.screen_avail_top);
    set_u32!("screen.availLeft", fp.screen_avail_left);
    set_u32!("screen.colorDepth", fp.color_depth);
    set_u32!("screen.pixelDepth", fp.pixel_depth);
    set_f64!("window.devicePixelRatio", fp.device_pixel_ratio);
    if let Some(dpr) = fp.device_pixel_ratio {
        if dpr > 0.0 {
            prefs.insert(
                "layout.css.devPixelsPerPx".into(),
                serde_json::json!(format!("{}", dpr)),
            );
        }
    }

    // Window
    set_u32!("window.outerHeight", fp.outer_height);
    set_u32!("window.outerWidth", fp.outer_width);
    set_u32!("window.innerHeight", fp.inner_height);
    set_u32!("window.innerWidth", fp.inner_width);
    set_i32!("window.screenX", fp.screen_x);
    set_i32!("window.screenY", fp.screen_y);

    // WebGL (only shaderPrecisionFormats:blockIfNotDefined remains active in v159)
    set_str!("webGl:renderer", fp.webgl_renderer);
    set_str!("webGl:vendor", fp.webgl_vendor);
    if let Some(v) = fp.webgl_block_if_not_defined {
        m.insert(
            "webGl:shaderPrecisionFormats:blockIfNotDefined".into(),
            serde_json::json!(v),
        );
        m.insert(
            "webGl2:shaderPrecisionFormats:blockIfNotDefined".into(),
            serde_json::json!(v),
        );
    }

    // AudioContext
    set_u32!("AudioContext:sampleRate", fp.audio_sample_rate);
    set_f64!("AudioContext:outputLatency", fp.audio_output_latency);
    set_u32!("AudioContext:maxChannelCount", fp.audio_max_channel_count);

    // Fonts
    if let Some(ref fonts) = fp.fonts {
        if !fonts.is_empty() {
            m.insert("fonts".into(), serde_json::json!(fonts));
        }
    }

    // Geolocation, Timezone & Locale
    set_f64!("geolocation:latitude", fp.geo_latitude);
    set_f64!("geolocation:longitude", fp.geo_longitude);
    set_f64!("geolocation:accuracy", fp.geo_accuracy);
    if fp.geo_latitude.is_some() && fp.geo_longitude.is_some() {
        prefs.insert("permissions.default.geo".into(), serde_json::json!(1));
    }
    set_str!("timezone", fp.timezone);
    set_str!("locale:language", fp.locale_language);
    set_str!("locale:region", fp.locale_region);
    set_str!("locale:script", fp.locale_script);
    let requested_locale = fp
        .locale_language
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|lang| {
            let mut parts = vec![lang];
            if let Some(sc) = fp
                .locale_script
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                parts.push(sc);
            }
            if let Some(reg) = fp
                .locale_region
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                parts.push(reg);
            }
            parts.join("-")
        })
        .unwrap_or_else(|| "en-US".to_string());
    prefs.insert(
        "intl.locale.requested".into(),
        serde_json::json!(requested_locale),
    );

    // WebRTC
    if fp.block_webrtc != Some(true) {
        set_str!("webrtc:ipv4", fp.webrtc_ipv4);
        set_str!("webrtc:ipv6", fp.webrtc_ipv6);
    }

    // HTTP Headers
    let effective_header_ua = fp
        .header_user_agent
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|ua| {
            if let Some(ver) = target_ff_version {
                harmonize_firefox_user_agent(ua, ver)
            } else {
                ua.clone()
            }
        })
        .or_else(|| effective_ua.clone());
    set_str!("headers.User-Agent", effective_header_ua);
    set_str!("headers.Accept-Language", fp.header_accept_language);
    set_str!("headers.Accept-Encoding", fp.header_accept_encoding);

    // Media Devices
    if fp.media_micros.is_some() || fp.media_webcams.is_some() || fp.media_speakers.is_some() {
        m.insert("mediaDevices:enabled".into(), serde_json::json!(true));
    }
    set_u32!("mediaDevices:micros", fp.media_micros);
    set_u32!("mediaDevices:webcams", fp.media_webcams);
    set_u32!("mediaDevices:speakers", fp.media_speakers);

    // Speech Voices
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
        m.insert("voices".into(), serde_json::Value::Array(arr));
        m.insert(
            "voices:blockIfNotDefined".into(),
            serde_json::json!(fp.voices_block_if_not_defined.unwrap_or(true)),
        );
    } else if let Some(block) = fp.voices_block_if_not_defined {
        m.insert("voices:blockIfNotDefined".into(), serde_json::json!(block));
    }

    m.insert(
        "showcursor".into(),
        serde_json::json!(fp.showcursor.unwrap_or(false)),
    );
    apply_shared_v159_flags(fp, &mut m, &mut prefs);

    (m, prefs)
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
    let mut path = get_data_dir(app).await;
    path.push("profiles");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    out.push(val);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn parse_legacy_proxy_string(raw: &str) -> Option<ProxyConfig> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (scheme_raw, rest) = trimmed.split_once("://")?;
    let proxy_type = match scheme_raw.to_ascii_lowercase().as_str() {
        "http" | "https" => "http",
        "socks4" => "socks4",
        "socks5" => "socks5",
        _ => return None,
    };

    let rest = rest.split('/').next().unwrap_or(rest);
    let (userinfo, hostport) = match rest.rsplit_once('@') {
        Some((u, hp)) => (Some(u), hp),
        None => (None, rest),
    };

    let (host, port_str) = hostport.rsplit_once(':')?;
    let host = host.trim_matches(|c| c == '[' || c == ']');
    let port: u16 = port_str.parse().ok()?;
    if host.is_empty() || port == 0 {
        return None;
    }

    let (username, password) = match userinfo {
        Some(u) => match u.split_once(':') {
            Some((user, pass)) => (Some(percent_decode(user)), Some(percent_decode(pass))),
            None => (Some(percent_decode(u)), None),
        },
        None => (None, None),
    };

    Some(ProxyConfig {
        proxy_type: Some(proxy_type.to_string()),
        host: Some(host.to_string()),
        port: Some(port),
        username,
        password,
    })
}

pub(crate) fn resolve_effective_proxy(
    proxy_config: &Option<ProxyConfig>,
    legacy_proxy: &Option<String>,
) -> Option<ProxyConfig> {
    if let Some(pc) = proxy_config {
        let valid = pc.proxy_type.as_deref().is_some_and(|t| !t.is_empty())
            && pc.host.as_deref().is_some_and(|h| !h.is_empty())
            && pc.port.unwrap_or(0) > 0;
        if valid {
            return Some(pc.clone());
        }
    }
    legacy_proxy.as_deref().and_then(parse_legacy_proxy_string)
}

pub(crate) fn format_legacy_proxy(pc: &ProxyConfig) -> Option<String> {
    crate::proxy_tester::proxy_url(pc).ok()
}

fn push_user_pref(out: &mut String, key: &str, value: &serde_json::Value) {
    let encoded_key = serde_json::to_string(key).unwrap();
    let encoded_val = serde_json::to_string(value).unwrap();
    out.push_str(&format!("user_pref({}, {});\n", encoded_key, encoded_val));
}

pub(crate) fn build_profile_user_prefs(
    resolved_proxy: Option<&ProxyConfig>,
    persist_data: bool,
    block_webrtc: bool,
) -> Vec<(&'static str, serde_json::Value)> {
    let mut prefs: Vec<(&'static str, serde_json::Value)> = Vec::new();

    // Keep stock 50: setting 0 forces window.history.length === 0 (bot tell per camoufox.cfg).
    prefs.push(("browser.sessionhistory.max_entries", serde_json::json!(50)));

    if persist_data {
        prefs.push((
            "browser.sessionhistory.max_total_viewers",
            serde_json::json!(-1),
        ));
        prefs.push(("browser.startup.page", serde_json::json!(3)));
        prefs.push((
            "browser.sessionstore.resume_from_crash",
            serde_json::json!(true),
        ));
        prefs.push(("browser.sessionstore.privacy_level", serde_json::json!(0)));
        prefs.push(("browser.formfill.enable", serde_json::json!(true)));
        prefs.push((
            "browser.places.interactions.enabled",
            serde_json::json!(true),
        ));
        prefs.push(("browser.urlbar.suggest.history", serde_json::json!(true)));
        prefs.push(("signon.rememberSignons", serde_json::json!(true)));
        prefs.push((
            "privacy.sanitize.sanitizeOnShutdown",
            serde_json::json!(false),
        ));
        prefs.push((
            "privacy.clearOnShutdown.openWindows",
            serde_json::json!(false),
        ));
    } else {
        prefs.push((
            "browser.sessionhistory.max_total_viewers",
            serde_json::json!(0),
        ));
        prefs.push(("browser.startup.page", serde_json::json!(1)));
        prefs.push((
            "browser.sessionstore.resume_from_crash",
            serde_json::json!(false),
        ));
        prefs.push(("browser.sessionstore.privacy_level", serde_json::json!(2)));
        prefs.push(("browser.formfill.enable", serde_json::json!(false)));
        prefs.push((
            "browser.places.interactions.enabled",
            serde_json::json!(false),
        ));
        prefs.push(("browser.urlbar.suggest.history", serde_json::json!(false)));
        prefs.push(("signon.rememberSignons", serde_json::json!(false)));
        prefs.push((
            "privacy.sanitize.sanitizeOnShutdown",
            serde_json::json!(true),
        ));
        prefs.push(("privacy.clearOnShutdown.cookies", serde_json::json!(true)));
        prefs.push(("privacy.clearOnShutdown.cache", serde_json::json!(true)));
        prefs.push(("privacy.clearOnShutdown.history", serde_json::json!(true)));
        prefs.push((
            "privacy.clearOnShutdown.offlineApps",
            serde_json::json!(true),
        ));
        prefs.push(("privacy.clearOnShutdown.sessions", serde_json::json!(true)));
        prefs.push((
            "privacy.clearOnShutdown.openWindows",
            serde_json::json!(true),
        ));
        prefs.push((
            "privacy.clearOnShutdown_v2.cookiesAndStorage",
            serde_json::json!(true),
        ));
        prefs.push(("privacy.clearOnShutdown_v2.cache", serde_json::json!(true)));
        prefs.push((
            "privacy.clearOnShutdown_v2.browsingHistoryAndDownloads",
            serde_json::json!(true),
        ));
        prefs.push((
            "privacy.clearOnShutdown_v2.formdata",
            serde_json::json!(true),
        ));
    }

    // Explicitly reset all proxy fields first so switching proxy types or to Direct
    // clears stale values previously persisted into prefs.js.
    let mut proxy_type = 0i32;
    let mut http_host = "";
    let mut http_port = 0u16;
    let mut socks_host = "";
    let mut socks_port = 0u16;
    let mut socks_version = 5i32;
    let mut socks_remote_dns = false;

    if let Some(pc) = resolved_proxy {
        if let (Some(ptype), Some(host), Some(port)) =
            (pc.proxy_type.as_deref(), pc.host.as_deref(), pc.port)
        {
            if !host.is_empty() && port > 0 {
                match ptype {
                    "http" => {
                        proxy_type = 1;
                        http_host = host;
                        http_port = port;
                    }
                    "socks4" => {
                        proxy_type = 1;
                        socks_host = host;
                        socks_port = port;
                        socks_version = 4;
                    }
                    "socks5" => {
                        proxy_type = 1;
                        socks_host = host;
                        socks_port = port;
                        socks_version = 5;
                        socks_remote_dns = true;
                    }
                    _ => {}
                }
            }
        }
    }

    prefs.push(("network.proxy.type", serde_json::json!(proxy_type)));
    prefs.push(("network.proxy.http", serde_json::json!(http_host)));
    prefs.push(("network.proxy.http_port", serde_json::json!(http_port)));
    prefs.push(("network.proxy.ssl", serde_json::json!(http_host)));
    prefs.push(("network.proxy.ssl_port", serde_json::json!(http_port)));
    prefs.push(("network.proxy.socks", serde_json::json!(socks_host)));
    prefs.push(("network.proxy.socks_port", serde_json::json!(socks_port)));
    prefs.push((
        "network.proxy.socks_version",
        serde_json::json!(socks_version),
    ));
    prefs.push((
        "network.proxy.socks_remote_dns",
        serde_json::json!(socks_remote_dns),
    ));
    prefs.push((
        "network.proxy.allow_hijacking_localhost",
        serde_json::json!(false),
    ));
    prefs.push((
        "network.proxy.no_proxies_on",
        serde_json::json!(if proxy_type == 1 {
            "localhost, 127.0.0.1, ::1"
        } else {
            ""
        }),
    ));

    // Camoufox v159 enforces ICE proxy_only_if_behind_proxy + mDNS obfuscation in camoufox.cfg;
    // only disable PeerConnection outright when explicitly blocking WebRTC.
    prefs.push((
        "media.peerconnection.enabled",
        serde_json::json!(!block_webrtc),
    ));

    prefs.push(("extensions.autoDisableScopes", serde_json::json!(0)));
    prefs.push(("xpinstall.signatures.required", serde_json::json!(false)));
    prefs
}

fn ensure_user_js(
    instance_dir: &Path,
    resolved_proxy: Option<&ProxyConfig>,
    persist_data: bool,
    block_webrtc: bool,
) -> io::Result<()> {
    let user_js_path = instance_dir.join("user.js");
    let mut user_js_content = String::from("// Anon Instance Preferences\n");
    for (key, value) in build_profile_user_prefs(resolved_proxy, persist_data, block_webrtc) {
        push_user_pref(&mut user_js_content, key, &value);
    }
    fs::write(user_js_path, user_js_content)
}

/// Remove Anon-managed startup pref keys from `prefs.js` so user-branch copies
/// cannot shadow updated `CAMOU_PREFS_1..N` default-branch values on relaunch.
pub(crate) fn scrub_managed_prefs_from_prefs_js(
    instance_dir: &Path,
    managed_keys: &[&str],
) -> io::Result<()> {
    let prefs_path = instance_dir.join("prefs.js");
    if !prefs_path.is_file() || managed_keys.is_empty() {
        return Ok(());
    }
    let content = fs::read_to_string(&prefs_path)?;
    let mut changed = false;
    let mut kept = String::with_capacity(content.len());
    for line in content.lines() {
        let should_drop = managed_keys.iter().any(|k| {
            line.trim_start()
                .starts_with(&format!("user_pref(\"{}\",", k))
        });
        if should_drop {
            changed = true;
        } else {
            kept.push_str(line);
            kept.push('\n');
        }
    }
    if changed {
        fs::write(&prefs_path, kept)?;
    }
    Ok(())
}

/// Serialize JSON with non-ASCII characters escaped as \uXXXX so camoufox.cfg
/// reading CAMOU_PREFS_* via Windows ANSI getenv() never corrupts UTF-8.
pub(crate) fn encode_ascii_json(value: &serde_json::Value) -> String {
    let raw = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii() {
            out.push(ch);
        } else {
            let mut buf = [0u16; 2];
            for unit in ch.encode_utf16(&mut buf) {
                out.push_str(&format!("\\u{:04x}", unit));
            }
        }
    }
    out
}

pub(crate) fn chunk_env_value(
    prefix: &str,
    data: &str,
    chunk_size: usize,
) -> Vec<(String, String)> {
    let chars: Vec<char> = data.chars().collect();
    chars
        .chunks(chunk_size)
        .enumerate()
        .map(|(i, chunk)| (format!("{}_{}", prefix, i + 1), chunk.iter().collect()))
        .collect()
}

pub(crate) fn apply_camou_prefs_env(
    cmd: &mut std::process::Command,
    prefs: &serde_json::Map<String, serde_json::Value>,
) {
    if prefs.is_empty() {
        return;
    }
    let chunk_size = if cfg!(windows) { 2047 } else { 32767 };
    let encoded = encode_ascii_json(&serde_json::Value::Object(prefs.clone()));
    for (k, v) in chunk_env_value("CAMOU_PREFS", &encoded, chunk_size) {
        cmd.env(k, v);
    }
}

fn ensure_proxy_auth_extension(
    instance_dir: &Path,
    proxy_config: Option<&ProxyConfig>,
) -> io::Result<Option<PathBuf>> {
    let ext_dir = instance_dir.join("proxy_auth_ext");
    // Firefox webRequest.onAuthRequired only supports HTTP/HTTPS proxies;
    // SOCKS5 auth is handled by socks5_relay instead.
    if let Some(pc) = proxy_config {
        if pc.proxy_type.as_deref() == Some("http") {
            if let (Some(ref u), Some(ref p)) = (&pc.username, &pc.password) {
                if !u.is_empty() {
                    fs::create_dir_all(&ext_dir)?;

                    let manifest = serde_json::json!({
                        "manifest_version": 2,
                        "name": "Anon Proxy Auth",
                        "version": "1.0",
                        "description": "Proxy authentication for Anon Browser",
                        "permissions": [
                            "webRequest",
                            "webRequestBlocking",
                            "<all_urls>"
                        ],
                        "background": {
                            "scripts": ["background.js"]
                        }
                    });
                    fs::write(
                        ext_dir.join("manifest.json"),
                        serde_json::to_string_pretty(&manifest)?,
                    )?;

                    let bg_js = format!(
                        r#"browser.webRequest.onAuthRequired.addListener(
    function(details) {{
        if (details.isProxy) {{
            return {{
                authCredentials: {{
                    username: {},
                    password: {}
                }}
            }};
        }}
    }},
    {{ urls: ["<all_urls>"] }},
    ["blocking"]
);"#,
                        serde_json::to_string(u).unwrap_or_default(),
                        serde_json::to_string(p).unwrap_or_default()
                    );
                    fs::write(ext_dir.join("background.js"), bg_js)?;
                    return Ok(Some(ext_dir));
                }
            }
        }
    }

    if ext_dir.exists() {
        let _ = fs::remove_dir_all(&ext_dir);
    }
    Ok(None)
}

fn apply_camou_config_env(cmd: &mut std::process::Command, config_json: &str) {
    let chunk_size = if cfg!(windows) { 2047 } else { 32767 };
    cmd.env_remove("CAMOU_CONFIG");
    for (k, v) in chunk_env_value("CAMOU_CONFIG", config_json, chunk_size) {
        cmd.env(k, v);
    }
}

pub(crate) fn cleanup_instance_data(instance_dir: &Path) -> io::Result<()> {
    if !instance_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(instance_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str());

        if let Some(name) = file_name {
            if ANON_INTERNAL_ENTRIES.contains(&name) {
                continue;
            }
        }

        let res = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        if let Err(e) = res {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e);
            }
        }
    }
    Ok(())
}

async fn cleanup_instance_data_with_retry(instance_dir: &Path) -> io::Result<()> {
    let mut last_err = None;
    for attempt in 0..8 {
        match cleanup_instance_data(instance_dir) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
                if attempt < 7 {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
            }
        }
    }
    Err(last_err.unwrap())
}

pub async fn cleanup_if_ephemeral(app: &AppHandle, id: &str) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(id);
    let config_path = instance_dir.join("anon_config.json");
    let persist_data = fs::read_to_string(&config_path)
        .ok()
        .and_then(|c| serde_json::from_str::<InstanceConfig>(&c).ok())
        .is_none_or(|cfg| cfg.persist_data);
    if !persist_data {
        cleanup_instance_data_with_retry(&instance_dir)
            .await
            .map_err(|e| format!("Failed to wipe ephemeral profile {}: {}", id, e))?;
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
    instances.sort_by_key(|i| std::cmp::Reverse(i.created_at));
    Ok(instances)
}

pub(crate) fn build_new_instance_config(
    id: String,
    name: String,
    proxy: Option<String>,
    persist_data: bool,
    app_settings: &crate::settings::AppSettings,
) -> InstanceConfig {
    let parsed_proxy = proxy.as_deref().and_then(parse_legacy_proxy_string);
    let initial_proxy_config = parsed_proxy.or_else(|| app_settings.default_proxy_template.clone());
    let initial_proxy_str = initial_proxy_config
        .as_ref()
        .and_then(format_legacy_proxy)
        .or_else(|| proxy.filter(|s| !s.trim().is_empty()));

    InstanceConfig {
        id,
        name,
        proxy: initial_proxy_str,
        persist_data,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        fingerprint: Some(
            app_settings
                .default_fingerprint_template
                .clone()
                .unwrap_or_else(FingerprintConfig::auto_default),
        ),
        proxy_config: initial_proxy_config,
        tags: None,
        notes: None,
        proxy_pool: None,
        proxy_rotation_mode: None,
        proxy_rotation_index: None,
        fingerprint_pool: None,
        fingerprint_rotation_mode: None,
        fingerprint_rotation_index: None,
    }
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

    let app_settings = crate::settings::load_settings(app).await;
    let profiles_dir = get_profiles_dir(app).await;
    let id = uuid::Uuid::new_v4().to_string();

    let instance_dir = profiles_dir.join(&id);
    fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;

    let config = build_new_instance_config(id, name, proxy, persist_data, &app_settings);

    let config_path = instance_dir.join("anon_config.json");
    write_json_atomic(&config_path, &config)?;

    let resolved_proxy = resolve_effective_proxy(&config.proxy_config, &config.proxy);
    let block_webrtc = config
        .fingerprint
        .as_ref()
        .is_some_and(|f| f.block_webrtc == Some(true));
    ensure_user_js(
        &instance_dir,
        resolved_proxy.as_ref(),
        config.persist_data,
        block_webrtc,
    )
    .map_err(|e| format!("Failed to write user.js: {}", e))?;

    Ok(config)
}

pub(crate) fn select_proxy_for_launch(
    config: Option<&InstanceConfig>,
) -> (Option<ProxyConfig>, Option<usize>) {
    if let Some(pool) = config.and_then(|c| c.proxy_pool.as_ref()) {
        if !pool.is_empty() {
            let mode = config
                .and_then(|c| c.proxy_rotation_mode.as_deref())
                .unwrap_or("sequential");
            return if mode == "random" {
                use rand::Rng;
                let idx = rand::thread_rng().gen_range(0..pool.len());
                (Some(pool[idx].clone()), None)
            } else {
                let current = config.and_then(|c| c.proxy_rotation_index).unwrap_or(0);
                let idx = current % pool.len();
                (Some(pool[idx].clone()), Some(idx + 1))
            };
        }
    }
    (
        resolve_effective_proxy(
            &config.and_then(|c| c.proxy_config.clone()),
            &config.and_then(|c| c.proxy.clone()),
        ),
        None,
    )
}

pub async fn delete_instance(app: &AppHandle, id: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(id);

    if instance_dir.exists() {
        fs::remove_dir_all(instance_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn launch_instance(
    app: &AppHandle,
    id: String,
    startup_url: Option<String>,
) -> Result<u32, String> {
    if process_manager::is_running(&id) {
        return Err("Instance is already running".to_string());
    }

    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);

    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let config_path = instance_dir.join("anon_config.json");
    let config: Option<InstanceConfig> = if config_path.exists() {
        fs::read_to_string(&config_path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
    } else {
        None
    };

    let persist_data = config.as_ref().is_none_or(|c| c.persist_data);

    // Guarantee ephemeral profiles start clean even after an earlier crash or hard kill.
    if !persist_data {
        cleanup_instance_data_with_retry(&instance_dir)
            .await
            .map_err(|e| format!("Failed to wipe ephemeral profile before launch: {}", e))?;
    }

    struct RelayGuard {
        id: String,
        active: bool,
    }
    impl Drop for RelayGuard {
        fn drop(&mut self) {
            if self.active {
                crate::socks5_relay::stop_for_instance(&self.id);
            }
        }
    }
    let mut relay_guard = RelayGuard {
        id: id.clone(),
        active: false,
    };

    let bin_path = crate::camoufox::get_camoufox_binary(app)
        .await
        .ok_or_else(|| "Camoufox binary not downloaded".to_string())?;

    let (effective_proxy_config, next_proxy_idx) = select_proxy_for_launch(config.as_ref());
    if let Some(next_idx) = next_proxy_idx {
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(mut cfg) = serde_json::from_str::<InstanceConfig>(&contents) {
                cfg.proxy_rotation_index = Some(next_idx);
                let _ = write_json_atomic(&config_path, &cfg);
            }
        }
    }

    // For authenticated SOCKS5 proxies, route Firefox through a per-instance loopback relay
    // that handles RFC 1929 upstream auth (Firefox WebExtensions only auth HTTP proxies).
    let mut browser_proxy_config = effective_proxy_config.clone();
    if let Some(ref pc) = effective_proxy_config {
        let is_socks5_auth = pc.proxy_type.as_deref() == Some("socks5")
            && pc.username.as_deref().is_some_and(|u| !u.is_empty());
        if is_socks5_auth {
            let upstream_host = pc.host.clone().unwrap_or_default();
            let upstream_port = pc.port.unwrap_or(0);
            let username = pc.username.clone().unwrap_or_default();
            let password = pc.password.clone().unwrap_or_default();
            let local_port = crate::socks5_relay::start_for_instance(
                &id,
                upstream_host,
                upstream_port,
                username,
                password,
            )
            .await?;
            relay_guard.active = true;
            browser_proxy_config = Some(ProxyConfig {
                proxy_type: Some("socks5".to_string()),
                host: Some("127.0.0.1".to_string()),
                port: Some(local_port),
                username: None,
                password: None,
            });
        }
    }

    let default_fp = FingerprintConfig::auto_default();
    let effective_fp: &FingerprintConfig =
        if let Some(pool) = config.as_ref().and_then(|c| c.fingerprint_pool.as_ref()) {
            if !pool.is_empty() {
                let mode = config
                    .as_ref()
                    .and_then(|c| c.fingerprint_rotation_mode.as_deref())
                    .unwrap_or("sequential");
                let idx = if mode == "random" {
                    use rand::Rng;
                    rand::thread_rng().gen_range(0..pool.len())
                } else {
                    let current = config
                        .as_ref()
                        .and_then(|c| c.fingerprint_rotation_index)
                        .unwrap_or(0);
                    let idx = current % pool.len();
                    if let Ok(contents) = fs::read_to_string(&config_path) {
                        if let Ok(mut cfg) = serde_json::from_str::<InstanceConfig>(&contents) {
                            cfg.fingerprint_rotation_index = Some(idx + 1);
                            let _ = write_json_atomic(&config_path, &cfg);
                        }
                    }
                    idx
                };
                &pool[idx]
            } else {
                config
                    .as_ref()
                    .and_then(|c| c.fingerprint.as_ref())
                    .unwrap_or(&default_fp)
            }
        } else {
            config
                .as_ref()
                .and_then(|c| c.fingerprint.as_ref())
                .unwrap_or(&default_fp)
        };

    let block_webrtc = effective_fp.block_webrtc == Some(true);
    ensure_user_js(
        &instance_dir,
        browser_proxy_config.as_ref(),
        persist_data,
        block_webrtc,
    )
    .map_err(|e| format!("Failed to write user.js: {}", e))?;

    let engine_ver_str = crate::camoufox::get_camoufox_version(app).await;
    let target_ff_ver = extract_major_version(&engine_ver_str).unwrap_or("159");

    let proxy_ext = ensure_proxy_auth_extension(&instance_dir, effective_proxy_config.as_ref())
        .map_err(|e| format!("Failed to prepare proxy auth extension: {}", e))?;

    // Missing auto_fingerprint defaults to Auto so new and legacy-empty instances are protected.
    let use_auto = effective_fp.auto_fingerprint != Some(false);
    let (mut camou_map, camou_prefs) = if use_auto {
        let geo = crate::geo::for_instance(app, &id, effective_proxy_config.as_ref()).await;
        let nonce = effective_fp.identity_nonce.unwrap_or(0);
        let opts = crate::auto_fingerprint::AutoFingerprintOptions {
            auto_os: effective_fp.auto_os.as_deref(),
            auto_language: effective_fp.auto_language.as_deref(),
            change_window_size: effective_fp.auto_change_window_size.unwrap_or(true),
            default_outer_width: effective_fp.outer_width,
            default_outer_height: effective_fp.outer_height,
            target_ff_version: Some(target_ff_ver),
            identity_seed: crate::geo::identity_seed(&id, nonce),
            block_webrtc,
            canvas_noise: effective_fp.canvas_noise,
        };
        let mut bundle = crate::auto_fingerprint::generate_auto_bundle(&opts, &geo);
        apply_shared_v159_flags(effective_fp, &mut bundle.config, &mut bundle.prefs);
        (bundle.config, bundle.prefs)
    } else {
        build_camou_config_and_prefs(effective_fp, Some(target_ff_ver))
    };

    let mut addons: Vec<String> = Vec::new();
    if effective_fp.enable_ublock == Some(true) {
        let ubo_dir = bin_path
            .parent()
            .map(|d| d.join("addons").join("UBO"))
            .filter(|d| d.join("manifest.json").is_file())
            .ok_or_else(|| {
                "Bundled uBlock Origin (addons/UBO) not found beside Camoufox binary. Update the engine or disable uBlock Origin in instance settings.".to_string()
            })?;
        addons.push(ubo_dir.to_string_lossy().to_string());
    }
    if let Some(ext_path) = proxy_ext {
        addons.push(ext_path.to_string_lossy().to_string());
    }
    if !addons.is_empty() {
        camou_map.insert("addons".into(), serde_json::json!(addons));
    }

    const MANAGED_CAMOU_PREFS: &[&str] = &[
        "intl.locale.requested",
        "intl.accept_languages",
        "permissions.default.geo",
        "general.cpu_identity",
        "ui.useOverlayScrollbars",
        "mousewheel.system_scroll_override.enabled",
        "javascript.options.use_ucrt_math",
        "widget.non-native-theme.scrollbar.style",
        "privacy.baselineFingerprintingProtection.overrides",
        "gfx.blacklist.webgpu",
        "permissions.default.image",
        "webgl.disabled",
        "browser.tabs.remote.useCrossOriginOpenerPolicy",
        "privacy.donottrackheader.enabled",
        "privacy.globalprivacycontrol.enabled",
        "layout.css.devPixelsPerPx",
    ];
    scrub_managed_prefs_from_prefs_js(&instance_dir, MANAGED_CAMOU_PREFS)
        .map_err(|e| format!("Failed to clean stale startup prefs: {}", e))?;
    let camou_config_json = serde_json::Value::Object(camou_map).to_string();

    let mut cmd = std::process::Command::new(bin_path);
    cmd.arg("--no-remote")
        .arg("--new-instance")
        .arg("--profile")
        .arg(&instance_dir);
    cmd.env("MOZ_NO_REMOTE", "1");
    apply_camou_config_env(&mut cmd, &camou_config_json);
    apply_camou_prefs_env(&mut cmd, &camou_prefs);
    if let Some(ref url) = startup_url {
        cmd.arg(url);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to launch instance: {}", e))?;
    relay_guard.active = false;

    let pid = child.id();
    process_manager::register(&id, pid);

    let app_handle = app.clone();
    let instance_id = id.clone();
    let profile_dir = instance_dir.clone();
    tokio::spawn(async move {
        let _ = tokio::task::spawn_blocking(move || child.wait()).await;

        let mut real_pid: Option<u32> = None;
        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if let Some(found) = process_manager::find_browser_pid(&profile_dir) {
                real_pid = Some(found);
                break;
            }
        }

        if let Some(browser_pid) = real_pid {
            process_manager::update_pid(&instance_id, browser_pid);
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if !process_manager::is_pid_alive(browser_pid) {
                    break;
                }
            }
        }

        crate::socks5_relay::stop_for_instance(&instance_id);
        process_manager::unregister(&instance_id);

        if let Err(e) = cleanup_if_ephemeral(&app_handle, &instance_id).await {
            eprintln!("{}", e);
        }

        let _ = app_handle.emit("instance-stopped", &instance_id);
    });

    Ok(pid)
}

pub async fn toggle_persistence(app: &AppHandle, id: String, enabled: bool) -> Result<(), String> {
    if !enabled && process_manager::is_running(&id) {
        return Err("Stop the instance before switching to wipe mode".to_string());
    }

    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);

    if !instance_dir.exists() {
        return Err("Instance profile not found".to_string());
    }

    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.persist_data = enabled;
            write_json_atomic(&config_path, &config)?;

            let resolved_proxy = resolve_effective_proxy(&config.proxy_config, &config.proxy);
            let block_webrtc = config
                .fingerprint
                .as_ref()
                .is_some_and(|f| f.block_webrtc == Some(true));
            ensure_user_js(
                &instance_dir,
                resolved_proxy.as_ref(),
                config.persist_data,
                block_webrtc,
            )
            .map_err(|e| format!("Failed to update user.js: {}", e))?;

            if !enabled {
                cleanup_instance_data_with_retry(&instance_dir)
                    .await
                    .map_err(|e| format!("Failed to wipe profile data: {}", e))?;
            }

            return Ok(());
        }
    }

    Err("Failed to update instance config".to_string())
}

pub async fn regenerate_identity(app: &AppHandle, id: String) -> Result<u64, String> {
    if process_manager::is_running(&id) {
        return Err("Stop the instance before regenerating its identity".to_string());
    }

    let profiles_dir = get_profiles_dir(app).await;
    let config_path = profiles_dir.join(&id).join("anon_config.json");
    let contents =
        fs::read_to_string(&config_path).map_err(|_| "Instance profile not found".to_string())?;
    let mut config = serde_json::from_str::<InstanceConfig>(&contents)
        .map_err(|e| format!("Failed to parse instance config: {}", e))?;

    let mut fp = config
        .fingerprint
        .unwrap_or_else(FingerprintConfig::auto_default);
    let next_nonce = fp.identity_nonce.unwrap_or(0).wrapping_add(1);
    fp.identity_nonce = Some(next_nonce);
    fp.auto_fingerprint = Some(true);
    config.fingerprint = Some(fp);

    write_json_atomic(&config_path, &config)?;
    Ok(next_nonce)
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
            write_json_atomic(&config_path, &config)?;
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

            config.proxy = proxy_config.as_ref().and_then(format_legacy_proxy);

            write_json_atomic(&config_path, &config)?;

            let resolved_proxy = resolve_effective_proxy(&config.proxy_config, &config.proxy);
            let block_webrtc = config
                .fingerprint
                .as_ref()
                .is_some_and(|f| f.block_webrtc == Some(true));
            ensure_user_js(
                &instance_dir,
                resolved_proxy.as_ref(),
                config.persist_data,
                block_webrtc,
            )
            .map_err(|e| format!("Failed to update user.js: {}", e))?;

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
    if instances
        .iter()
        .any(|i| i.id != id && i.name.eq_ignore_ascii_case(&new_name))
    {
        return Err("An instance with this name already exists".to_string());
    }
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.name = new_name;
            write_json_atomic(&config_path, &config)?;
            return Ok(());
        }
    }
    Err("Failed to rename instance".to_string())
}

pub async fn update_instance_tags(
    app: &AppHandle,
    id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.tags = if tags.is_empty() { None } else { Some(tags) };
            write_json_atomic(&config_path, &config)?;
            return Ok(());
        }
    }
    Err("Failed to update instance tags".to_string())
}

pub async fn update_instance_notes(
    app: &AppHandle,
    id: String,
    notes: Option<String>,
) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let instance_dir = profiles_dir.join(&id);
    let config_path = instance_dir.join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.notes = notes.filter(|n| !n.trim().is_empty());
            write_json_atomic(&config_path, &config)?;
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

pub async fn import_instances(
    app: &AppHandle,
    json: String,
) -> Result<Vec<InstanceConfig>, String> {
    let existing = list_instances(app).await?;
    let profiles_dir = get_profiles_dir(app).await;

    // Try parsing as array of instances or as export bundle
    let configs_to_import: Vec<InstanceConfig> =
        if let Ok(bundle) = serde_json::from_str::<serde_json::Value>(&json) {
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
        while existing
            .iter()
            .any(|i| i.name.eq_ignore_ascii_case(&new_name))
            || imported
                .iter()
                .any(|i: &InstanceConfig| i.name.eq_ignore_ascii_case(&new_name))
        {
            counter += 1;
            new_name = format!(
                "{} (imported{})",
                config.name,
                if counter > 1 {
                    format!(" {}", counter)
                } else {
                    String::new()
                }
            );
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
        write_json_atomic(&config_path, &new_config)?;
        let resolved_proxy = resolve_effective_proxy(&new_config.proxy_config, &new_config.proxy);
        let block_webrtc = new_config
            .fingerprint
            .as_ref()
            .is_some_and(|f| f.block_webrtc == Some(true));
        ensure_user_js(
            &instance_dir,
            resolved_proxy.as_ref(),
            new_config.persist_data,
            block_webrtc,
        )
        .map_err(|e| format!("Failed to write user.js: {}", e))?;
        imported.push(new_config);
    }
    Ok(imported)
}

pub async fn update_proxy_pool(
    app: &AppHandle,
    id: String,
    pool: Vec<ProxyConfig>,
    mode: Option<String>,
) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let config_path = profiles_dir.join(&id).join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.proxy_pool = if pool.is_empty() { None } else { Some(pool) };
            config.proxy_rotation_mode = mode;
            write_json_atomic(&config_path, &config)?;
            return Ok(());
        }
    }
    Err("Failed to update proxy pool".to_string())
}

pub async fn update_fingerprint_pool(
    app: &AppHandle,
    id: String,
    pool: Vec<FingerprintConfig>,
    mode: Option<String>,
) -> Result<(), String> {
    let profiles_dir = get_profiles_dir(app).await;
    let config_path = profiles_dir.join(&id).join("anon_config.json");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<InstanceConfig>(&contents) {
            config.fingerprint_pool = if pool.is_empty() { None } else { Some(pool) };
            config.fingerprint_rotation_mode = mode;
            write_json_atomic(&config_path, &config)?;
            return Ok(());
        }
    }
    Err("Failed to update fingerprint pool".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "anon_instances_test_{}_{}",
            name,
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn user_js_never_sets_sessionhistory_zero_and_resets_proxy_on_direct() {
        let dir = temp_dir("userjs");
        let proxy = ProxyConfig {
            proxy_type: Some("socks5".into()),
            host: Some("10.0.0.2".into()),
            port: Some(1080),
            username: None,
            password: None,
        };
        ensure_user_js(&dir, Some(&proxy), false, false).unwrap();
        let with_proxy = fs::read_to_string(dir.join("user.js")).unwrap();
        assert!(with_proxy.contains("user_pref(\"browser.sessionhistory.max_entries\", 50);"));
        assert!(with_proxy.contains("user_pref(\"privacy.sanitize.sanitizeOnShutdown\", true);"));
        assert!(with_proxy
            .contains("user_pref(\"privacy.clearOnShutdown_v2.cookiesAndStorage\", true);"));
        assert!(with_proxy.contains("user_pref(\"network.proxy.type\", 1);"));
        assert!(with_proxy.contains("user_pref(\"network.proxy.socks\", \"10.0.0.2\");"));
        assert!(with_proxy.contains("user_pref(\"media.peerconnection.enabled\", true);"));

        ensure_user_js(&dir, None, true, false).unwrap();
        let direct = fs::read_to_string(dir.join("user.js")).unwrap();
        assert!(direct.contains("user_pref(\"network.proxy.type\", 0);"));
        assert!(direct.contains("user_pref(\"network.proxy.socks\", \"\");"));
        assert!(direct.contains("user_pref(\"network.proxy.socks_port\", 0);"));
        assert!(direct.contains("user_pref(\"network.proxy.http\", \"\");"));
        assert!(direct.contains("user_pref(\"privacy.sanitize.sanitizeOnShutdown\", false);"));
        assert!(direct.contains("user_pref(\"browser.sessionstore.privacy_level\", 0);"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_instance_data_wipes_browsing_files_and_preserves_config() {
        let dir = temp_dir("cleanup");
        fs::write(dir.join("anon_config.json"), b"{}").unwrap();
        fs::write(dir.join("user.js"), b"// prefs").unwrap();
        fs::create_dir_all(dir.join("proxy_auth_ext")).unwrap();
        fs::write(dir.join("proxy_auth_ext").join("manifest.json"), b"{}").unwrap();

        fs::write(dir.join("cookies.sqlite"), b"cookie").unwrap();
        fs::write(dir.join("places.sqlite"), b"history").unwrap();
        fs::create_dir_all(dir.join("storage").join("default")).unwrap();
        fs::write(dir.join("storage").join("default").join("data"), b"ls").unwrap();
        fs::create_dir_all(dir.join("sessionstore-backups")).unwrap();
        fs::write(
            dir.join("sessionstore-backups").join("recovery.jsonlz4"),
            b"s",
        )
        .unwrap();

        cleanup_instance_data(&dir).unwrap();

        assert!(dir.join("anon_config.json").exists());
        assert!(dir.join("user.js").exists());
        assert!(dir.join("proxy_auth_ext").exists());
        assert!(!dir.join("cookies.sqlite").exists());
        assert!(!dir.join("places.sqlite").exists());
        assert!(!dir.join("storage").exists());
        assert!(!dir.join("sessionstore-backups").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_proxy_parser_extracts_decoded_credentials_and_host() {
        let parsed =
            parse_legacy_proxy_string("socks5://alice%40org:p%40ss%3A1@proxy.test:1080").unwrap();
        assert_eq!(parsed.proxy_type.as_deref(), Some("socks5"));
        assert_eq!(parsed.host.as_deref(), Some("proxy.test"));
        assert_eq!(parsed.port, Some(1080));
        assert_eq!(parsed.username.as_deref(), Some("alice@org"));
        assert_eq!(parsed.password.as_deref(), Some("p@ss:1"));
    }

    #[test]
    fn proxy_auth_extension_only_created_for_authenticated_http_proxy() {
        let dir = temp_dir("ext");
        let socks = ProxyConfig {
            proxy_type: Some("socks5".into()),
            host: Some("1.2.3.4".into()),
            port: Some(1080),
            username: Some("u".into()),
            password: Some("p".into()),
        };
        assert!(ensure_proxy_auth_extension(&dir, Some(&socks))
            .unwrap()
            .is_none());

        let http = ProxyConfig {
            proxy_type: Some("http".into()),
            ..socks
        };
        let ext = ensure_proxy_auth_extension(&dir, Some(&http)).unwrap();
        assert!(ext.is_some());
        assert!(dir.join("proxy_auth_ext").join("background.js").is_file());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn encode_ascii_json_escapes_non_ascii_for_windows_camou_prefs() {
        let val = serde_json::json!({"intl.locale.requested": "fr-FR", "label": "caf\u{00e9}"});
        let encoded = encode_ascii_json(&val);
        assert!(encoded.is_ascii());
        assert!(encoded.contains("\\u00e9"));
        let decoded: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn scrub_managed_prefs_drops_stale_user_branch_entries_only() {
        let dir = temp_dir("scrub");
        fs::write(
            dir.join("prefs.js"),
            "user_pref(\"intl.locale.requested\", \"de-DE\");\nuser_pref(\"intl.accept_languages\", \"bn-BD, bn\");\nuser_pref(\"browser.custom\", 42);\n",
        )
        .unwrap();
        scrub_managed_prefs_from_prefs_js(&dir, &["intl.locale.requested", "intl.accept_languages"])
            .unwrap();
        let after = fs::read_to_string(dir.join("prefs.js")).unwrap();
        assert!(!after.contains("intl.locale.requested"));
        assert!(!after.contains("intl.accept_languages"));
        assert!(after.contains("browser.custom"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn manual_mode_defaults_intl_locale_requested_to_en_us_when_unset() {
        let fp = FingerprintConfig {
            auto_fingerprint: Some(false),
            ..Default::default()
        };
        let (_cfg, prefs) = build_camou_config_and_prefs(&fp, Some("159"));
        assert_eq!(
            prefs.get("intl.locale.requested"),
            Some(&serde_json::json!("en-US"))
        );
    }

    #[test]
    fn explicit_ipv4_only_false_overrides_auto_ipv6_mismatch_guard() {
        let mut cfg = serde_json::Map::new();
        cfg.insert("network:ipv4Only".into(), serde_json::json!(true));
        let mut prefs = serde_json::Map::new();
        let fp = FingerprintConfig {
            network_ipv4_only: Some(false),
            ..Default::default()
        };
        apply_shared_v159_flags(&fp, &mut cfg, &mut prefs);
        assert_eq!(cfg.get("network:ipv4Only"), Some(&serde_json::json!(false)));
    }

    #[test]
    fn manual_v159_config_emits_active_keys_and_startup_prefs() {
        let fp = FingerprintConfig {
            auto_fingerprint: Some(false),
            platform: Some("Win32".into()),
            cpu_arch: Some("x86".into()),
            cpu_manufacturer: Some("amd".into()),
            locale_language: Some("fr".into()),
            locale_region: Some("FR".into()),
            geo_latitude: Some(48.8566),
            geo_longitude: Some(2.3522),
            network_ipv4_only: Some(true),
            media_spoof_codecs: Some(true),
            canvas_noise: Some(true),
            block_images: Some(true),
            disable_coop: Some(true),
            webgl_block_if_not_defined: Some(true),
            ..Default::default()
        };

        let (cfg, prefs) = build_camou_config_and_prefs(&fp, Some("159"));
        assert_eq!(cfg.get("cpu:arch"), Some(&serde_json::json!("x86")));
        assert_eq!(cfg.get("cpu:manufacturer"), Some(&serde_json::json!("amd")));
        assert_eq!(cfg.get("network:ipv4Only"), Some(&serde_json::json!(true)));
        assert_eq!(
            cfg.get("media:spoof_codecs"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            cfg.get("webGl:shaderPrecisionFormats:blockIfNotDefined"),
            Some(&serde_json::json!(true))
        );
        assert!(!cfg.contains_key("webGl:parameters:blockIfNotDefined"));
        assert!(!cfg.contains_key("canvas:seed"));
        assert!(!cfg.contains_key("humanize"));

        assert_eq!(
            prefs.get("general.cpu_identity"),
            Some(&serde_json::json!(2))
        );
        assert_eq!(
            prefs.get("intl.locale.requested"),
            Some(&serde_json::json!("fr-FR"))
        );
        assert_eq!(
            prefs.get("permissions.default.geo"),
            Some(&serde_json::json!(1))
        );
        assert_eq!(
            prefs.get("permissions.default.image"),
            Some(&serde_json::json!(2))
        );
        assert_eq!(
            prefs.get("browser.tabs.remote.useCrossOriginOpenerPolicy"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            prefs.get("gfx.blacklist.webgpu"),
            Some(&serde_json::json!(4))
        );
        assert!(prefs.contains_key("privacy.baselineFingerprintingProtection.overrides"));

        // Verify legacy JSON with removed fields still deserializes cleanly
        let legacy_json = r#"{"user_agent":"Firefox/148.0","canvas_seed":123,"audio_seed":456,"battery_level":0.9,"humanize":true}"#;
        let parsed: FingerprintConfig = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(parsed.user_agent.as_deref(), Some("Firefox/148.0"));
    }

    #[test]
    fn new_instance_applies_default_templates_or_auto_default() {
        let defaults = crate::settings::AppSettings::default();
        let cfg = build_new_instance_config("id-1".into(), "One".into(), None, true, &defaults);
        assert_eq!(
            cfg.fingerprint.as_ref().and_then(|f| f.auto_fingerprint),
            Some(true)
        );
        assert_eq!(
            cfg.fingerprint.as_ref().and_then(|f| f.auto_os.as_deref()),
            None
        );

        let templated = crate::settings::AppSettings {
            default_proxy_template: Some(ProxyConfig {
                proxy_type: Some("http".into()),
                host: Some("tmpl.proxy".into()),
                port: Some(8080),
                username: Some("u@1".into()),
                password: Some("p:2".into()),
            }),
            default_fingerprint_template: Some(FingerprintConfig {
                auto_fingerprint: Some(true),
                auto_os: Some("linux".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let cfg2 = build_new_instance_config("id-2".into(), "Two".into(), None, false, &templated);
        assert_eq!(
            cfg2.proxy_config.as_ref().and_then(|p| p.host.as_deref()),
            Some("tmpl.proxy")
        );
        assert_eq!(
            cfg2.fingerprint.as_ref().and_then(|f| f.auto_os.as_deref()),
            Some("linux")
        );
        let reparsed = parse_legacy_proxy_string(cfg2.proxy.as_deref().unwrap()).unwrap();
        assert_eq!(reparsed.username.as_deref(), Some("u@1"));
        assert_eq!(reparsed.password.as_deref(), Some("p:2"));
    }

    #[test]
    fn proxy_rotation_selects_same_entry_for_routing_and_auth() {
        let p0 = ProxyConfig {
            proxy_type: Some("http".into()),
            host: Some("p0.test".into()),
            port: Some(8000),
            username: Some("u0".into()),
            password: Some("k0".into()),
        };
        let p1 = ProxyConfig {
            proxy_type: Some("http".into()),
            host: Some("p1.test".into()),
            port: Some(8001),
            username: Some("u1".into()),
            password: Some("k1".into()),
        };
        let mut cfg = build_new_instance_config(
            "id-rot".into(),
            "Rot".into(),
            None,
            true,
            &crate::settings::AppSettings::default(),
        );
        cfg.proxy_pool = Some(vec![p0.clone(), p1.clone()]);
        cfg.proxy_rotation_mode = Some("sequential".into());
        cfg.proxy_rotation_index = Some(0);

        let (sel0, next0) = select_proxy_for_launch(Some(&cfg));
        assert_eq!(
            sel0.as_ref().and_then(|p| p.host.as_deref()),
            Some("p0.test")
        );
        assert_eq!(
            sel0.as_ref().and_then(|p| p.username.as_deref()),
            Some("u0")
        );
        assert_eq!(next0, Some(1));

        cfg.proxy_rotation_index = next0;
        let (sel1, next1) = select_proxy_for_launch(Some(&cfg));
        assert_eq!(
            sel1.as_ref().and_then(|p| p.host.as_deref()),
            Some("p1.test")
        );
        assert_eq!(
            sel1.as_ref().and_then(|p| p.username.as_deref()),
            Some("u1")
        );
        assert_eq!(next1, Some(2));
    }
}
