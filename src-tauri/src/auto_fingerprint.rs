//! Deterministic Camoufox v159 AUTO mode fingerprint and startup-pref generation.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::fingerprint_presets;

pub const CANVAS_NOISE_OVERRIDES: &str = "+CanvasImageExtractionPrompt,+CanvasExtractionBeforeUserInputIsBlocked,+CanvasExtractionFromThirdPartiesIsBlocked";

static FONTS: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static VOICES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static MEDIA_DEVICES: OnceLock<MediaDevicesCatalog> = OnceLock::new();

fn load_fonts() -> &'static HashMap<String, Vec<String>> {
    FONTS.get_or_init(|| {
        serde_json::from_str(include_str!("fonts.json")).expect("Failed to parse fonts.json")
    })
}

fn load_voices() -> &'static HashMap<String, Vec<String>> {
    VOICES.get_or_init(|| {
        serde_json::from_str(include_str!("voices.json")).expect("Failed to parse voices.json")
    })
}

#[derive(Deserialize)]
struct MediaDevicesCatalog {
    win: OsMediaPool,
    mac: OsMediaPool,
    lin: OsMediaPool,
}

#[derive(Deserialize)]
struct OsMediaPool {
    cards: Vec<MediaCard>,
    #[serde(default)]
    headsets: Vec<MediaCard>,
    #[serde(default)]
    displays: Vec<MediaCard>,
    #[serde(default)]
    cameras: Vec<MediaCamera>,
    #[serde(default)]
    p_headset: f64,
    #[serde(default)]
    p_display: f64,
    #[serde(default)]
    p_extra_camera: f64,
    #[serde(default)]
    monitor_sources: bool,
}

#[derive(Deserialize)]
struct MediaCard {
    w: u32,
    #[serde(default)]
    mics: Vec<String>,
    #[serde(default)]
    outs: Vec<String>,
    #[serde(default)]
    cam: Option<String>,
}

#[derive(Deserialize)]
struct MediaCamera {
    w: u32,
    cam: Option<String>,
    mic: Option<String>,
}

fn load_media_devices() -> &'static MediaDevicesCatalog {
    MEDIA_DEVICES.get_or_init(|| {
        serde_json::from_str(include_str!("media-devices.json"))
            .expect("Failed to parse media-devices.json")
    })
}

const ESSENTIAL_FONTS_MACOS: &[&str] = &[
    "Arial",
    "Helvetica",
    "Times New Roman",
    "Courier New",
    "Verdana",
    "Georgia",
    "Trebuchet MS",
    "Tahoma",
    "Helvetica Neue",
    "Lucida Grande",
    "Menlo",
    "Monaco",
    "Geneva",
    "PingFang HK",
    "PingFang SC",
    "PingFang TC",
];
const ESSENTIAL_FONTS_WINDOWS: &[&str] = &[
    "Arial",
    "Times New Roman",
    "Courier New",
    "Verdana",
    "Georgia",
    "Trebuchet MS",
    "Tahoma",
    "Segoe UI",
    "Calibri",
    "Cambria Math",
    "Nirmala UI",
    "Consolas",
];
const ESSENTIAL_FONTS_LINUX: &[&str] = &[
    "Noto Sans",
    "Noto Serif",
    "DejaVu Sans Mono",
    "Arimo",
    "Cousine",
    "Tinos",
    "Twemoji Mozilla",
    "Noto Sans Devanagari",
    "Noto Sans JP",
    "Noto Sans KR",
    "Noto Sans SC",
    "Noto Sans TC",
];

const MARKER_FONTS_MACOS: &[&str] = &["Helvetica Neue"];
const MARKER_FONTS_WINDOWS: &[&str] = &["Segoe UI", "Tahoma", "Cambria Math", "Nirmala UI"];
const MARKER_FONTS_LINUX: &[&str] = &[
    "Noto Sans",
    "Noto Serif",
    "DejaVu Sans Mono",
    "Arimo",
    "Cousine",
    "Tinos",
    "Twemoji Mozilla",
];

const WINDOWS_11_MARKER_FONTS: &[&str] = &[
    "Sans Serif Collection",
    "Segoe Fluent Icons",
    "Segoe UI Variable",
    "Segoe UI Variable Display",
    "Segoe UI Variable Small",
    "Segoe UI Variable Text",
];

const ESSENTIAL_VOICES_MACOS: &[&str] =
    &["Samantha", "Alex", "Fred", "Victoria", "Karen", "Daniel"];

pub fn host_os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

pub fn cpu_identity_pref(arch: Option<&str>, manufacturer: Option<&str>) -> Option<i32> {
    match (arch, manufacturer) {
        (Some("arm"), _) => Some(3),
        (Some("x86") | None, Some("intel")) => Some(1),
        (Some("x86") | None, Some("amd")) => Some(2),
        _ => None,
    }
}

pub fn infer_cpu_identity(
    target_os: &str,
    webgl_renderer: Option<&str>,
    rng: &mut StdRng,
) -> (&'static str, &'static str) {
    let rl = webgl_renderer.unwrap_or("").to_lowercase();
    if rl.contains("apple") {
        return ("arm", "apple");
    }
    if rl.contains("intel") {
        return ("x86", "intel");
    }
    if rl.contains("amd") || rl.contains("radeon") {
        return ("x86", "amd");
    }
    if target_os == "macos" {
        return ("x86", "intel");
    }
    if rng.gen_bool(0.8136) {
        ("x86", "intel")
    } else {
        ("x86", "amd")
    }
}

fn pick_weighted<'a, T>(items: &'a [T], weight: impl Fn(&T) -> u32, rng: &mut StdRng) -> Option<&'a T> {
    let total: u32 = items.iter().map(&weight).sum();
    if total == 0 {
        return items.first();
    }
    let mut pick = rng.gen_range(0..total);
    for item in items {
        let w = weight(item);
        if pick < w {
            return Some(item);
        }
        pick -= w;
    }
    items.last()
}

fn draw_media_devices(target_os: &str, rng: &mut StdRng) -> serde_json::Map<String, serde_json::Value> {
    let catalog = load_media_devices();
    let pool = match target_os {
        "windows" => &catalog.win,
        "macos" => &catalog.mac,
        _ => &catalog.lin,
    };

    let mut mic_labels: Vec<String> = Vec::new();
    let mut mic_groups: Vec<String> = Vec::new();
    let mut cam_labels: Vec<String> = Vec::new();
    let mut cam_groups: Vec<String> = Vec::new();
    let mut out_labels: Vec<String> = Vec::new();
    let mut out_groups: Vec<String> = Vec::new();
    let mut group_idx = 0u32;

    let next_group = |idx: &mut u32| -> String {
        *idx += 1;
        format!("grp-{}", *idx)
    };

    if let Some(card) = pick_weighted(&pool.cards, |c| c.w, rng) {
        let g = next_group(&mut group_idx);
        for m in &card.mics {
            mic_labels.push(m.clone());
            mic_groups.push(g.clone());
        }
        for o in &card.outs {
            out_labels.push(o.clone());
            out_groups.push(g.clone());
            if pool.monitor_sources {
                mic_labels.push(format!("Monitor of {}", o));
                mic_groups.push(g.clone());
            }
        }
        if let Some(ref cam) = card.cam {
            cam_labels.push(cam.clone());
            cam_groups.push(g);
        }
    }

    if cam_labels.is_empty() && !pool.cameras.is_empty() {
        if let Some(cam) = pick_weighted(&pool.cameras, |c| c.w, rng) {
            let g = next_group(&mut group_idx);
            if let Some(ref label) = cam.cam {
                cam_labels.push(label.clone());
                cam_groups.push(g.clone());
            }
            if let Some(ref mic) = cam.mic {
                mic_labels.push(mic.clone());
                mic_groups.push(g);
            }
        }
    }

    if !pool.headsets.is_empty() && rng.gen_bool(pool.p_headset.clamp(0.0, 1.0)) {
        if let Some(hs) = pick_weighted(&pool.headsets, |c| c.w, rng) {
            let g = next_group(&mut group_idx);
            for m in &hs.mics {
                mic_labels.push(m.clone());
                mic_groups.push(g.clone());
            }
            for o in &hs.outs {
                out_labels.push(o.clone());
                out_groups.push(g.clone());
                if pool.monitor_sources {
                    mic_labels.push(format!("Monitor of {}", o));
                    mic_groups.push(g.clone());
                }
            }
        }
    }

    if !pool.displays.is_empty() && rng.gen_bool(pool.p_display.clamp(0.0, 1.0)) {
        if let Some(disp) = pick_weighted(&pool.displays, |c| c.w, rng) {
            let g = next_group(&mut group_idx);
            for m in &disp.mics {
                mic_labels.push(m.clone());
                mic_groups.push(g.clone());
            }
            for o in &disp.outs {
                out_labels.push(o.clone());
                out_groups.push(g.clone());
            }
            if let Some(ref cam) = disp.cam {
                cam_labels.push(cam.clone());
                cam_groups.push(g);
            }
        }
    }

    if !pool.cameras.is_empty() && rng.gen_bool(pool.p_extra_camera.clamp(0.0, 1.0)) {
        if let Some(cam) = pick_weighted(&pool.cameras, |c| c.w, rng) {
            if let Some(ref label) = cam.cam {
                if !cam_labels.contains(label) {
                    let g = next_group(&mut group_idx);
                    cam_labels.push(label.clone());
                    cam_groups.push(g.clone());
                    if let Some(ref mic) = cam.mic {
                        mic_labels.push(mic.clone());
                        mic_groups.push(g);
                    }
                }
            }
        }
    }

    let mut out = serde_json::Map::new();
    out.insert("mediaDevices:enabled".into(), serde_json::json!(true));
    out.insert("mediaDevices:micros".into(), serde_json::json!(mic_labels.len()));
    out.insert("mediaDevices:webcams".into(), serde_json::json!(cam_labels.len()));
    out.insert("mediaDevices:speakers".into(), serde_json::json!(out_labels.len()));
    out.insert("mediaDevices:microphoneLabels".into(), serde_json::json!(mic_labels));
    out.insert("mediaDevices:microphoneGroups".into(), serde_json::json!(mic_groups));
    out.insert("mediaDevices:webcamLabels".into(), serde_json::json!(cam_labels));
    out.insert("mediaDevices:webcamGroups".into(), serde_json::json!(cam_groups));
    out.insert("mediaDevices:speakerLabels".into(), serde_json::json!(out_labels));
    out.insert("mediaDevices:speakerGroups".into(), serde_json::json!(out_groups));
    out
}

fn generate_font_subset(target_os: &str, rng: &mut StdRng) -> Vec<String> {
    let fonts_data = load_fonts();

    let os_key = match target_os {
        "macos" => "mac",
        "windows" => "win",
        "linux" => "lin",
        other => other,
    };

    let full_list = match fonts_data.get(os_key) {
        Some(list) => list,
        None => return Vec::new(),
    };

    let (essential_set, markers): (HashSet<&str>, &[&str]) = match target_os {
        "windows" => (
            ESSENTIAL_FONTS_WINDOWS.iter().copied().collect(),
            MARKER_FONTS_WINDOWS,
        ),
        "linux" => (
            ESSENTIAL_FONTS_LINUX.iter().copied().collect(),
            MARKER_FONTS_LINUX,
        ),
        _ => (
            ESSENTIAL_FONTS_MACOS.iter().copied().collect(),
            MARKER_FONTS_MACOS,
        ),
    };

    let mut result: Vec<String> = full_list
        .iter()
        .filter(|f| essential_set.contains(f.as_str()))
        .cloned()
        .collect();
    let non_essential: Vec<&String> = full_list
        .iter()
        .filter(|f| !essential_set.contains(f.as_str()))
        .collect();

    let pct = rng.gen_range(30..=78);
    let count = (pct as f64 / 100.0 * non_essential.len() as f64).round() as usize;

    let mut indices: Vec<usize> = (0..non_essential.len()).collect();
    indices.shuffle(rng);
    for &idx in indices.iter().take(count) {
        result.push(non_essential[idx].clone());
    }

    let existing: HashSet<String> = result.iter().cloned().collect();
    for &marker in markers {
        if !existing.contains(marker) {
            result.push(marker.to_string());
        }
    }

    result
}

fn generate_voice_subset(target_os: &str, rng: &mut StdRng) -> Vec<String> {
    let voices_data = load_voices();
    let os_key = match target_os {
        "macos" => "mac",
        "windows" => "win",
        "linux" => "lin",
        other => other,
    };

    let full_list = match voices_data.get(os_key) {
        Some(list) => list,
        None => return Vec::new(),
    };

    if full_list.is_empty() {
        return Vec::new();
    }

    let voice_names: Vec<String> = full_list
        .iter()
        .filter_map(|entry| entry.split(':').next().map(String::from))
        .collect();

    if target_os == "windows" {
        return voice_names;
    }

    let essential: HashSet<&str> = ESSENTIAL_VOICES_MACOS.iter().copied().collect();
    let mut result: Vec<String> = voice_names
        .iter()
        .filter(|v| essential.contains(v.as_str()))
        .cloned()
        .collect();
    let non_essential: Vec<&String> = voice_names
        .iter()
        .filter(|v| !essential.contains(v.as_str()))
        .collect();

    let pct = rng.gen_range(40..=80);
    let count = (pct as f64 / 100.0 * non_essential.len() as f64).round() as usize;

    let mut indices: Vec<usize> = (0..non_essential.len()).collect();
    indices.shuffle(rng);
    for &idx in indices.iter().take(count) {
        result.push(non_essential[idx].clone());
    }

    result
}

pub struct AutoFingerprintOptions<'a> {
    pub auto_os: Option<&'a str>,
    pub change_window_size: bool,
    pub default_outer_width: Option<u32>,
    pub default_outer_height: Option<u32>,
    pub target_ff_version: Option<&'a str>,
    pub identity_seed: u64,
    pub block_webrtc: bool,
    pub network_ipv4_only: Option<bool>,
    pub canvas_noise: Option<bool>,
}

pub struct AutoFingerprintOutput {
    pub config: serde_json::Map<String, serde_json::Value>,
    pub prefs: serde_json::Map<String, serde_json::Value>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub target_os: String,
}

fn select_preset_categories(auto_os: Option<&str>) -> Vec<&'static str> {
    match auto_os.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("any") => vec!["windows", "macos", "linux"],
        Some("windows") => vec!["windows"],
        Some("macos") => vec!["macos"],
        Some("linux") => vec!["linux"],
        _ => vec![host_os_name()],
    }
}

/// Build deterministic Camoufox v159 `CAMOU_CONFIG` and `CAMOU_PREFS` maps from a
/// per-instance identity seed and resolved geo context.
pub fn generate_auto_bundle(
    opts: &AutoFingerprintOptions<'_>,
    geo: &crate::geo::GeoContext,
) -> AutoFingerprintOutput {
    let presets = fingerprint_presets::get_presets();
    let mut rng = StdRng::seed_from_u64(opts.identity_seed);

    let categories = select_preset_categories(opts.auto_os);
    let mut candidates: Vec<&fingerprint_presets::Preset> = Vec::new();
    for cat in &categories {
        if let Some(list) = presets.get(*cat) {
            candidates.extend(list.iter());
        }
    }
    if candidates.is_empty() {
        for cat in ["windows", "macos", "linux"] {
            if let Some(list) = presets.get(cat) {
                candidates.extend(list.iter());
            }
        }
    }

    if !opts.change_window_size {
        if let (Some(req_w), Some(req_h)) = (opts.default_outer_width, opts.default_outer_height) {
            let filtered: Vec<&fingerprint_presets::Preset> = candidates
                .iter()
                .copied()
                .filter(|p| {
                    p.screen
                        .as_ref()
                        .map(|s| {
                            let aw = s.avail_width.or(s.width).unwrap_or(0);
                            let ah = s.avail_height.or(s.height).unwrap_or(0);
                            aw >= req_w && ah >= req_h
                        })
                        .unwrap_or(false)
                })
                .collect();
            if !filtered.is_empty() {
                candidates = filtered;
            }
        }
    }

    let mut config = serde_json::Map::new();
    let mut prefs = serde_json::Map::new();
    if candidates.is_empty() {
        return AutoFingerprintOutput {
            config,
            prefs,
            target_os: host_os_name().to_string(),
        };
    }

    let preset = candidates[rng.gen_range(0..candidates.len())];
    config.insert("showcursor".into(), serde_json::json!(false));

    let mut target_os = host_os_name();

    if let Some(ref nav) = preset.navigator {
        if let Some(ref ua) = nav.user_agent {
            let eff_ua = if let Some(ver) = opts.target_ff_version {
                crate::instances::harmonize_firefox_user_agent(ua, ver)
            } else {
                ua.clone()
            };
            config.insert("navigator.userAgent".into(), serde_json::Value::String(eff_ua.clone()));
            config.insert("headers.User-Agent".into(), serde_json::Value::String(eff_ua));
        }
        if let Some(ref plat) = nav.platform {
            config.insert("navigator.platform".into(), serde_json::Value::String(plat.clone()));
            match plat.as_str() {
                "MacIntel" => {
                    config.insert(
                        "navigator.oscpu".into(),
                        serde_json::Value::String("Intel Mac OS X 10.15".into()),
                    );
                    target_os = "macos";
                }
                "Win32" => {
                    config.insert(
                        "navigator.oscpu".into(),
                        serde_json::Value::String("Windows NT 10.0; Win64; x64".into()),
                    );
                    target_os = "windows";
                }
                p if p.contains("Linux") || p.contains("linux") => {
                    config.insert(
                        "navigator.oscpu".into(),
                        serde_json::Value::String("Linux x86_64".into()),
                    );
                    target_os = "linux";
                }
                _ => {}
            }
        }
        if let Some(hc) = nav.hardware_concurrency {
            config.insert("navigator.hardwareConcurrency".into(), serde_json::json!(hc));
        }
        if let Some(mtp) = nav.max_touch_points {
            config.insert("navigator.maxTouchPoints".into(), serde_json::json!(mtp));
        }
    }

    if let Some(ref screen) = preset.screen {
        if let Some(w) = screen.width {
            config.insert("screen.width".into(), serde_json::json!(w));
        }
        if let Some(h) = screen.height {
            config.insert("screen.height".into(), serde_json::json!(h));
        }
        if let Some(cd) = screen.color_depth {
            config.insert("screen.colorDepth".into(), serde_json::json!(cd));
            config.insert("screen.pixelDepth".into(), serde_json::json!(cd));
        }
        if let Some(aw) = screen.avail_width {
            config.insert("screen.availWidth".into(), serde_json::json!(aw));
        }
        if let Some(ah) = screen.avail_height {
            config.insert("screen.availHeight".into(), serde_json::json!(ah));
        }

        let avail_w = screen.avail_width.or(screen.width).unwrap_or(1920);
        let avail_h = screen.avail_height.or(screen.height).unwrap_or(1080);

        // When change_window_size is true, vary window size per launch using a non-seeded RNG
        // while keeping the underlying hardware/screen/GPU/CPU/font identity deterministic.
        let mut launch_rng = rand::thread_rng();
        let (outer_w, outer_h) = if opts.change_window_size {
            let pct_w = launch_rng.gen_range(78..=96) as f64 / 100.0;
            let pct_h = launch_rng.gen_range(78..=96) as f64 / 100.0;
            (
                ((avail_w as f64 * pct_w) as u32).max(800),
                ((avail_h as f64 * pct_h) as u32).max(600),
            )
        } else if let (Some(w), Some(h)) = (opts.default_outer_width, opts.default_outer_height) {
            (w.min(avail_w), h.min(avail_h))
        } else {
            (
                ((avail_w as f64 * 0.9) as u32).max(800),
                ((avail_h as f64 * 0.9) as u32).max(600),
            )
        };

        config.insert("window.outerWidth".into(), serde_json::json!(outer_w));
        config.insert("window.outerHeight".into(), serde_json::json!(outer_h));
        config.insert("window.innerWidth".into(), serde_json::json!(outer_w.saturating_sub(16)));
        config.insert("window.innerHeight".into(), serde_json::json!(outer_h.saturating_sub(86)));

        let max_x = avail_w.saturating_sub(outer_w);
        let max_y = avail_h.saturating_sub(outer_h);
        let (screen_x, screen_y) = if opts.change_window_size {
            (
                if max_x > 0 { launch_rng.gen_range(0..=max_x) as i32 } else { 0 },
                if max_y > 0 { launch_rng.gen_range(0..=max_y) as i32 } else { 0 },
            )
        } else {
            ((max_x / 2) as i32, (max_y / 2) as i32)
        };
        config.insert("window.screenX".into(), serde_json::json!(screen_x));
        config.insert("window.screenY".into(), serde_json::json!(screen_y));
    }

    let webgl_renderer = preset
        .webgl
        .as_ref()
        .and_then(|w| w.unmasked_renderer.as_deref());
    if let Some(ref webgl) = preset.webgl {
        if let Some(ref vendor) = webgl.unmasked_vendor {
            config.insert("webGl:vendor".into(), serde_json::Value::String(vendor.clone()));
        }
        if let Some(ref renderer) = webgl.unmasked_renderer {
            config.insert("webGl:renderer".into(), serde_json::Value::String(renderer.clone()));
        }
    }

    // Camoufox v159 CPU identity (drives canvas/WebAudio/Opus/WASM arithmetic parity)
    let (cpu_arch, cpu_maker) = infer_cpu_identity(target_os, webgl_renderer, &mut rng);
    config.insert("cpu:arch".into(), serde_json::json!(cpu_arch));
    config.insert("cpu:manufacturer".into(), serde_json::json!(cpu_maker));
    if let Some(pref_val) = cpu_identity_pref(Some(cpu_arch), Some(cpu_maker)) {
        prefs.insert("general.cpu_identity".into(), serde_json::json!(pref_val));
    }

    // Geo, Locale & WebRTC
    config.insert("timezone".into(), serde_json::json!(geo.timezone));
    config.insert("locale:language".into(), serde_json::json!(geo.language));
    config.insert("locale:region".into(), serde_json::json!(geo.region));
    config.insert("locale:all".into(), serde_json::json!(geo.locale_all));
    config.insert("headers.Accept-Language".into(), serde_json::json!(geo.accept_language));

    let requested_locale = format!("{}-{}", geo.language, geo.region);
    prefs.insert("intl.locale.requested".into(), serde_json::json!(requested_locale));

    if let Some((lat, lon)) = geo.coordinates {
        config.insert("geolocation:latitude".into(), serde_json::json!(lat));
        config.insert("geolocation:longitude".into(), serde_json::json!(lon));
        prefs.insert("permissions.default.geo".into(), serde_json::json!(1));
    }

    if !opts.block_webrtc {
        if let Some(ref ip) = geo.exit_ip {
            if ip.contains(':') {
                config.insert("webrtc:ipv6".into(), serde_json::json!(ip));
            } else if ip.contains('.') {
                config.insert("webrtc:ipv4".into(), serde_json::json!(ip));
            }
        }
    }

    if let Some(ipv4_only) = opts.network_ipv4_only {
        if ipv4_only {
            config.insert("network:ipv4Only".into(), serde_json::json!(true));
        }
    }

    // Deterministic font subset from v159 bundle
    let fonts = generate_font_subset(target_os, &mut rng);
    let is_win11_fonts = target_os == "windows"
        && (fonts.is_empty()
            || fonts
                .iter()
                .any(|f| WINDOWS_11_MARKER_FONTS.contains(&f.as_str())));
    if !fonts.is_empty() {
        config.insert("fonts".into(), serde_json::json!(fonts));
    }

    // Deterministic speech voices + fail-closed guard so host SAPI/NSSpeech voices never leak
    if let Some(ref speech_voices) = preset.speech_voices {
        let voice_subset = generate_voice_subset(target_os, &mut rng);
        let voice_arr: Vec<serde_json::Value> = speech_voices
            .iter()
            .filter(|entry| {
                let name = entry.split(':').next().unwrap_or("");
                voice_subset.iter().any(|v| v == name)
            })
            .enumerate()
            .map(|(filtered_idx, entry)| {
                let parts: Vec<&str> = entry.splitn(3, ':').collect();
                let name = parts.first().unwrap_or(&"").to_string();
                let lang = parts.get(1).unwrap_or(&"").to_string();
                let voice_uri = name.clone();
                serde_json::json!({
                    "name": name,
                    "lang": lang,
                    "voiceUri": voice_uri,
                    "isDefault": filtered_idx == 0,
                    "isLocalService": parts.get(2).is_none_or(|t| *t == "local")
                })
            })
            .collect();
        config.insert("voices".into(), serde_json::Value::Array(voice_arr));
    } else {
        config.insert("voices".into(), serde_json::Value::Array(Vec::new()));
    }
    config.insert("voices:blockIfNotDefined".into(), serde_json::json!(true));

    // Deterministic OS-matched media devices
    for (k, v) in draw_media_devices(target_os, &mut rng) {
        config.insert(k, v);
    }

    // Scrollbars & wheel parity
    let overlay_scrollbars = if target_os == "windows" {
        if is_win11_fonts { 1 } else { 0 }
    } else {
        1
    };
    prefs.insert("ui.useOverlayScrollbars".into(), serde_json::json!(overlay_scrollbars));
    prefs.insert(
        "mousewheel.system_scroll_override.enabled".into(),
        serde_json::json!(target_os != "macos"),
    );

    if target_os == "windows" && host_os_name() != "windows" {
        prefs.insert("javascript.options.use_ucrt_math".into(), serde_json::json!(true));
        config.insert("input:windowsKeyboard".into(), serde_json::json!(true));
        prefs.insert(
            "widget.non-native-theme.scrollbar.style".into(),
            serde_json::json!(if overlay_scrollbars == 1 { 5 } else { 4 }),
        );
    }

    // Cover canvas readback + block WebGPU when claiming a different OS than the host
    // (or when explicitly enabled by the user).
    let claiming_foreign_os = target_os != host_os_name();
    let cover_canvas = opts.canvas_noise.unwrap_or(claiming_foreign_os);
    if cover_canvas {
        prefs.insert(
            "privacy.baselineFingerprintingProtection.overrides".into(),
            serde_json::json!(CANVAS_NOISE_OVERRIDES),
        );
        prefs.insert("gfx.blacklist.webgpu".into(), serde_json::json!(4));
    }

    AutoFingerprintOutput {
        config,
        prefs,
        target_os: target_os.to_string(),
    }
}

#[cfg(test)]
mod audit {
    use super::*;
    use crate::geo::{identity_seed, resolve};
    use std::collections::{BTreeMap, HashSet};

    const COUNTRIES: &[&str] = &[
        "United States",
        "Germany",
        "Japan",
        "Brazil",
        "United Kingdom",
        "India",
        "Netherlands",
        "Australia",
        "Canada",
        "France",
        "Singapore",
        "Mexico",
    ];

    const REMOVED_V159_KEYS: &[&str] = &[
        "canvas:seed",
        "audio:seed",
        "fonts:spacing_seed",
        "battery:charging",
        "battery:chargingTime",
        "battery:dischargingTime",
        "battery:level",
        "humanize",
        "memorysaver",
        "pdfViewerEnabled",
        "navigator.appCodeName",
        "navigator.appName",
        "navigator.buildID",
        "navigator.cookieEnabled",
        "navigator.languages",
        "navigator.onLine",
        "navigator.product",
        "navigator.productSub",
        "webGl:parameters:blockIfNotDefined",
        "webGl2:parameters:blockIfNotDefined",
    ];

    const REQUIRED: &[&str] = &[
        "navigator.userAgent",
        "navigator.platform",
        "navigator.oscpu",
        "cpu:arch",
        "cpu:manufacturer",
        "timezone",
        "locale:language",
        "locale:region",
        "locale:all",
        "headers.User-Agent",
        "headers.Accept-Language",
        "geolocation:latitude",
        "geolocation:longitude",
        "screen.width",
        "screen.height",
        "webGl:vendor",
        "webGl:renderer",
        "fonts",
        "voices:blockIfNotDefined",
        "mediaDevices:enabled",
        "mediaDevices:microphoneLabels",
    ];

    const MUST_VARY: &[&str] = &[
        "navigator.userAgent",
        "navigator.platform",
        "timezone",
        "locale:region",
        "geolocation:latitude",
        "fonts",
        "webGl:renderer",
    ];

    #[test]
    fn audit_identity_entropy_and_zero_removed_v159_keys() {
        let mut seen: BTreeMap<String, HashSet<String>> = BTreeMap::new();
        for i in 0..200 {
            let id = format!("instance-{i:04}");
            let country = COUNTRIES[i % COUNTRIES.len()];
            let opts = AutoFingerprintOptions {
                auto_os: Some("any"),
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(&id, 0),
                block_webrtc: false,
                network_ipv4_only: None,
                canvas_noise: None,
            };
            let out = generate_auto_bundle(&opts, &resolve(country, &id));

            for removed in REMOVED_V159_KEYS {
                assert!(
                    !out.config.contains_key(*removed),
                    "auto config emitted removed Camoufox v159 key: {removed}"
                );
            }
            assert!(out.prefs.contains_key("intl.locale.requested"));
            assert!(out.prefs.contains_key("general.cpu_identity"));
            assert_eq!(out.prefs.get("permissions.default.geo"), Some(&serde_json::json!(1)));

            for (k, v) in &out.config {
                seen.entry(k.clone()).or_default().insert(v.to_string());
            }
        }

        for req in REQUIRED {
            assert!(seen.contains_key(*req), "missing required key: {req}");
        }
        for key in MUST_VARY {
            assert!(
                seen.get(*key).map(|s| s.len()).unwrap_or(0) > 1,
                "key was pinned to one value: {key}"
            );
        }
    }

    #[test]
    fn auto_identity_is_deterministic_across_launches_and_changes_on_regenerate() {
        let id = "aaaaaaaa-1111-4111-8111-111111111111";
        let geo = resolve("Germany", id);
        let mk = |nonce| {
            let opts = AutoFingerprintOptions {
                auto_os: Some("any"),
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, nonce),
                block_webrtc: false,
                network_ipv4_only: None,
                canvas_noise: None,
            };
            generate_auto_bundle(&opts, &geo)
        };

        let first = mk(0);
        let second = mk(0);
        assert_eq!(first.config, second.config);
        assert_eq!(first.prefs, second.prefs);

        // Regenerating nonce should produce a distinct identity across a few rolls
        let mut any_diff = false;
        for n in 1..=5 {
            if mk(n).config != first.config {
                any_diff = true;
                break;
            }
        }
        assert!(any_diff, "regenerating identity nonce did not change identity");
    }

    #[test]
    fn host_os_default_matches_host_and_foreign_os_enables_canvas_noise() {
        let id = "host-check-instance";
        let geo = resolve("United States", id);

        let host_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: None,
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                network_ipv4_only: None,
                canvas_noise: None,
            },
            &geo,
        );
        assert_eq!(host_out.target_os, host_os_name());
        assert!(!host_out.prefs.contains_key("privacy.baselineFingerprintingProtection.overrides"));

        let foreign_os = if host_os_name() == "windows" { "macos" } else { "windows" };
        let foreign_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: Some(foreign_os),
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                network_ipv4_only: None,
                canvas_noise: None,
            },
            &geo,
        );
        assert_eq!(foreign_out.target_os, foreign_os);
        assert!(foreign_out.prefs.contains_key("privacy.baselineFingerprintingProtection.overrides"));
        assert_eq!(foreign_out.prefs.get("gfx.blacklist.webgpu"), Some(&serde_json::json!(4)));
    }
}
