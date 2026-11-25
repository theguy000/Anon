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

fn pick_weighted<'a, T>(
    items: &'a [T],
    weight: impl Fn(&T) -> u32,
    rng: &mut StdRng,
) -> Option<&'a T> {
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

fn draw_media_devices(
    target_os: &str,
    rng: &mut StdRng,
) -> serde_json::Map<String, serde_json::Value> {
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
    out.insert(
        "mediaDevices:micros".into(),
        serde_json::json!(mic_labels.len()),
    );
    out.insert(
        "mediaDevices:webcams".into(),
        serde_json::json!(cam_labels.len()),
    );
    out.insert(
        "mediaDevices:speakers".into(),
        serde_json::json!(out_labels.len()),
    );
    out.insert(
        "mediaDevices:microphoneLabels".into(),
        serde_json::json!(mic_labels),
    );
    out.insert(
        "mediaDevices:microphoneGroups".into(),
        serde_json::json!(mic_groups),
    );
    out.insert(
        "mediaDevices:webcamLabels".into(),
        serde_json::json!(cam_labels),
    );
    out.insert(
        "mediaDevices:webcamGroups".into(),
        serde_json::json!(cam_groups),
    );
    out.insert(
        "mediaDevices:speakerLabels".into(),
        serde_json::json!(out_labels),
    );
    out.insert(
        "mediaDevices:speakerGroups".into(),
        serde_json::json!(out_groups),
    );
    out
}

fn host_has_win11_fonts() -> bool {
    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_string());
        std::path::Path::new(&windir)
            .join("Fonts")
            .join("SegUIVar.ttf")
            .is_file()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

fn generate_font_subset(
    target_os: &str,
    native_host: bool,
    host_win11: bool,
    rng: &mut StdRng,
) -> Vec<String> {
    // Camoufox v159 disables bundled fonts on native Windows/macOS identities
    // so only OS-shipped base fonts should be claimed; uninstalled optional
    // additions would fall back when measured by a page.
    if native_host && target_os == "windows" {
        let mut base: Vec<String> = ESSENTIAL_FONTS_WINDOWS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        if host_win11 {
            for &f in WINDOWS_11_MARKER_FONTS {
                if !base.iter().any(|existing| existing == f) {
                    base.push(f.to_string());
                }
            }
        }
        return base;
    }
    if native_host && target_os == "macos" {
        return ESSENTIAL_FONTS_MACOS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
    }

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

struct ParsedVoice<'a> {
    name: &'a str,
    lang: &'a str,
    is_local: bool,
}

fn parse_voice_entry(entry: &str) -> Option<ParsedVoice<'_>> {
    let mut parts = entry.splitn(3, ':');
    let name = parts.next()?.trim();
    let lang = parts.next()?.trim();
    if name.is_empty() || lang.is_empty() {
        return None;
    }
    let is_local = parts.next().is_none_or(|t| t.trim() == "local");
    Some(ParsedVoice {
        name,
        lang,
        is_local,
    })
}

fn voice_to_json(v: &ParsedVoice<'_>, is_default: bool) -> serde_json::Value {
    serde_json::json!({
        "name": v.name,
        "lang": v.lang,
        "voiceUri": v.name,
        "isDefault": is_default,
        "isLocalService": v.is_local,
    })
}

fn build_locale_voices(
    target_os: &str,
    requested_locale: &str,
    rng: &mut StdRng,
) -> Vec<serde_json::Value> {
    let os_key = match target_os {
        "macos" => "mac",
        "windows" => "win",
        _ => return Vec::new(),
    };
    let Some(raw_list) = load_voices().get(os_key) else {
        return Vec::new();
    };
    let parsed: Vec<ParsedVoice<'_>> = raw_list
        .iter()
        .filter_map(|e| parse_voice_entry(e))
        .collect();
    if parsed.is_empty() {
        return Vec::new();
    }

    let primary_lang = requested_locale
        .split('-')
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();

    if target_os == "windows" {
        let mut matched: Vec<&ParsedVoice<'_>> = parsed
            .iter()
            .filter(|v| v.lang.eq_ignore_ascii_case(requested_locale))
            .collect();
        if matched.is_empty() {
            if let Some(first_region_lang) = parsed
                .iter()
                .find(|v| {
                    v.lang
                        .split('-')
                        .next()
                        .is_some_and(|l| l.eq_ignore_ascii_case(&primary_lang))
                })
                .map(|v| v.lang)
            {
                matched = parsed
                    .iter()
                    .filter(|v| v.lang.eq_ignore_ascii_case(first_region_lang))
                    .collect();
            }
        }
        if matched.is_empty() {
            matched = parsed
                .iter()
                .filter(|v| v.lang.eq_ignore_ascii_case("en-US"))
                .collect();
        }

        // Prefer modern OneCore voices over legacy Desktop/Anna voices when both exist.
        let modern: Vec<&ParsedVoice<'_>> = matched
            .iter()
            .copied()
            .filter(|v| !v.name.contains("Desktop") && !v.name.contains("Anna"))
            .collect();
        let chosen = if !modern.is_empty() { modern } else { matched };

        return chosen
            .into_iter()
            .enumerate()
            .map(|(idx, v)| voice_to_json(v, idx == 0))
            .collect();
    }

    // macOS ships a multi-language system voice set whose default voice matches the UI locale.
    let mut chosen: Vec<&ParsedVoice<'_>> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();

    for v in parsed
        .iter()
        .filter(|v| v.lang.eq_ignore_ascii_case(requested_locale))
    {
        if seen.insert(v.name) {
            chosen.push(v);
        }
    }
    if chosen.is_empty() {
        for v in parsed.iter().filter(|v| {
            v.lang
                .split('-')
                .next()
                .is_some_and(|l| l.eq_ignore_ascii_case(&primary_lang))
        }) {
            if seen.insert(v.name) {
                chosen.push(v);
            }
        }
    }
    if chosen.is_empty() {
        for v in parsed
            .iter()
            .filter(|v| v.lang.eq_ignore_ascii_case("en-US"))
        {
            if seen.insert(v.name) {
                chosen.push(v);
            }
        }
    }

    let essential: HashSet<&str> = ESSENTIAL_VOICES_MACOS.iter().copied().collect();
    for v in parsed.iter().filter(|v| essential.contains(v.name)) {
        if seen.insert(v.name) {
            chosen.push(v);
        }
    }

    let remaining: Vec<&ParsedVoice<'_>> =
        parsed.iter().filter(|v| !seen.contains(v.name)).collect();
    let pct = rng.gen_range(40..=70);
    let count = (pct as f64 / 100.0 * remaining.len() as f64).round() as usize;
    let mut indices: Vec<usize> = (0..remaining.len()).collect();
    indices.shuffle(rng);
    for &idx in indices.iter().take(count) {
        let v = remaining[idx];
        if seen.insert(v.name) {
            chosen.push(v);
        }
    }

    chosen
        .into_iter()
        .enumerate()
        .map(|(idx, v)| voice_to_json(v, idx == 0))
        .collect()
}

fn is_valid_region_subtag(region: &str) -> bool {
    (region.len() == 2 && region.chars().all(|c| c.is_ascii_alphabetic()))
        || (region.len() == 3 && region.chars().all(|c| c.is_ascii_digit()))
}

fn format_script_subtag(script: &str) -> String {
    let mut chars = script.chars();
    match chars.next() {
        Some(first) => format!(
            "{}{}",
            first.to_ascii_uppercase(),
            chars.as_str().to_ascii_lowercase()
        ),
        None => String::new(),
    }
}

/// Resolve Auto-mode `(language, region, script, locale_all, requested_locale)`:
/// - `None` or blank -> `"en-US"` (readable English default)
/// - `"geo"` -> live exit IP country's locale
/// - `"lang-REGION"` or `"lang-Script-REGION"` -> explicit locale tag
pub fn resolve_auto_locale(
    auto_language: Option<&str>,
    geo: &crate::geo::GeoContext,
) -> (String, String, Option<String>, String, String) {
    let raw = auto_language.map(str::trim).unwrap_or("");
    if raw.is_empty() {
        return (
            "en".into(),
            "US".into(),
            None,
            "en-US, en".into(),
            "en-US".into(),
        );
    }
    if raw.eq_ignore_ascii_case("geo") {
        let primary = geo
            .locale_all
            .split(',')
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("{}-{}", geo.language, geo.region));
        return (
            geo.language.clone(),
            geo.region.clone(),
            None,
            geo.locale_all.clone(),
            primary,
        );
    }

    let normalized = raw.replace('_', "-");
    let parts: Vec<&str> = normalized.split('-').collect();
    match parts.as_slice() {
        [lang, region] => {
            let lang_ok =
                (2..=3).contains(&lang.len()) && lang.chars().all(|c| c.is_ascii_alphabetic());
            if lang_ok && is_valid_region_subtag(region) {
                let l = lang.to_ascii_lowercase();
                let r = region.to_ascii_uppercase();
                let requested = format!("{}-{}", l, r);
                let locale_all = format!("{}, {}", requested, l);
                return (l, r, None, locale_all, requested);
            }
        }
        [lang, script, region] => {
            let lang_ok =
                (2..=3).contains(&lang.len()) && lang.chars().all(|c| c.is_ascii_alphabetic());
            let script_ok = script.len() == 4 && script.chars().all(|c| c.is_ascii_alphabetic());
            if lang_ok && script_ok && is_valid_region_subtag(region) {
                let l = lang.to_ascii_lowercase();
                let s = format_script_subtag(script);
                let r = region.to_ascii_uppercase();
                let requested = format!("{}-{}-{}", l, s, r);
                let locale_all = format!("{}, {}-{}, {}", requested, l, r, l);
                return (l, r, Some(s), locale_all, requested);
            }
        }
        _ => {}
    }

    (
        "en".into(),
        "US".into(),
        None,
        "en-US, en".into(),
        "en-US".into(),
    )
}

pub struct AutoFingerprintOptions<'a> {
    pub auto_os: Option<&'a str>,
    pub auto_language: Option<&'a str>,
    pub change_window_size: bool,
    pub default_outer_width: Option<u32>,
    pub default_outer_height: Option<u32>,
    pub target_ff_version: Option<&'a str>,
    pub identity_seed: u64,
    pub block_webrtc: bool,
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
            config.insert(
                "navigator.userAgent".into(),
                serde_json::Value::String(eff_ua.clone()),
            );
            config.insert(
                "headers.User-Agent".into(),
                serde_json::Value::String(eff_ua),
            );
        }
        if let Some(ref plat) = nav.platform {
            config.insert(
                "navigator.platform".into(),
                serde_json::Value::String(plat.clone()),
            );
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
            config.insert(
                "navigator.hardwareConcurrency".into(),
                serde_json::json!(hc),
            );
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
        config.insert(
            "window.innerWidth".into(),
            serde_json::json!(outer_w.saturating_sub(16)),
        );
        config.insert(
            "window.innerHeight".into(),
            serde_json::json!(outer_h.saturating_sub(86)),
        );

        let max_x = avail_w.saturating_sub(outer_w);
        let max_y = avail_h.saturating_sub(outer_h);
        let (screen_x, screen_y) = if opts.change_window_size {
            (
                if max_x > 0 {
                    launch_rng.gen_range(0..=max_x) as i32
                } else {
                    0
                },
                if max_y > 0 {
                    launch_rng.gen_range(0..=max_y) as i32
                } else {
                    0
                },
            )
        } else {
            ((max_x / 2) as i32, (max_y / 2) as i32)
        };
        config.insert("window.screenX".into(), serde_json::json!(screen_x));
        config.insert("window.screenY".into(), serde_json::json!(screen_y));

        let dpr = screen
            .device_pixel_ratio
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(if target_os == "macos" { 2.0 } else { 1.0 });
        config.insert("window.devicePixelRatio".into(), serde_json::json!(dpr));
        prefs.insert(
            "layout.css.devPixelsPerPx".into(),
            serde_json::json!(format!("{}", dpr)),
        );
    }

    let webgl_renderer = preset
        .webgl
        .as_ref()
        .and_then(|w| w.unmasked_renderer.as_deref());
    if let Some(ref webgl) = preset.webgl {
        if let Some(ref vendor) = webgl.unmasked_vendor {
            config.insert(
                "webGl:vendor".into(),
                serde_json::Value::String(vendor.clone()),
            );
        }
        if let Some(ref renderer) = webgl.unmasked_renderer {
            config.insert(
                "webGl:renderer".into(),
                serde_json::Value::String(renderer.clone()),
            );
        }
    }

    let (cpu_arch, cpu_maker) = infer_cpu_identity(target_os, webgl_renderer, &mut rng);
    config.insert("cpu:arch".into(), serde_json::json!(cpu_arch));
    config.insert("cpu:manufacturer".into(), serde_json::json!(cpu_maker));
    if let Some(pref_val) = cpu_identity_pref(Some(cpu_arch), Some(cpu_maker)) {
        prefs.insert("general.cpu_identity".into(), serde_json::json!(pref_val));
    }

    config.insert("timezone".into(), serde_json::json!(geo.timezone));
    let (locale_lang, locale_region, locale_script, locale_all, requested_locale) =
        resolve_auto_locale(opts.auto_language, geo);
    config.insert("locale:language".into(), serde_json::json!(locale_lang));
    config.insert("locale:region".into(), serde_json::json!(locale_region));
    if let Some(ref script) = locale_script {
        config.insert("locale:script".into(), serde_json::json!(script));
    }
    config.insert("locale:all".into(), serde_json::json!(locale_all));
    prefs.insert(
        "intl.locale.requested".into(),
        serde_json::json!(requested_locale),
    );

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

    if geo.ipv6_mismatch {
        config.insert("network:ipv4Only".into(), serde_json::json!(true));
    }

    let native_host = target_os == host_os_name();
    let host_win11 = native_host && target_os == "windows" && host_has_win11_fonts();
    let fonts = generate_font_subset(target_os, native_host, host_win11, &mut rng);
    let is_win11_fonts = target_os == "windows"
        && (fonts.is_empty()
            || fonts
                .iter()
                .any(|f| WINDOWS_11_MARKER_FONTS.contains(&f.as_str())));
    if !fonts.is_empty() {
        config.insert("fonts".into(), serde_json::json!(fonts));
    }

    // Locale-matched speech voices + fail-closed guard so host SAPI/NSSpeech voices never leak
    let voice_arr = build_locale_voices(target_os, &requested_locale, &mut rng);
    config.insert("voices".into(), serde_json::Value::Array(voice_arr));
    config.insert("voices:blockIfNotDefined".into(), serde_json::json!(true));

    for (k, v) in draw_media_devices(target_os, &mut rng) {
        config.insert(k, v);
    }

    let overlay_scrollbars = if target_os == "windows" {
        if is_win11_fonts {
            1
        } else {
            0
        }
    } else {
        1
    };
    prefs.insert(
        "ui.useOverlayScrollbars".into(),
        serde_json::json!(overlay_scrollbars),
    );
    prefs.insert(
        "mousewheel.system_scroll_override.enabled".into(),
        serde_json::json!(target_os != "macos"),
    );

    if target_os == "windows" && host_os_name() != "windows" {
        prefs.insert(
            "javascript.options.use_ucrt_math".into(),
            serde_json::json!(true),
        );
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
        "geolocation:latitude",
        "geolocation:longitude",
        "screen.width",
        "screen.height",
        "window.devicePixelRatio",
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
                auto_language: Some("geo"),
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(&id, 0),
                block_webrtc: false,
                canvas_noise: None,
            };
            let out = generate_auto_bundle(&opts, &resolve(country, &id));

            for removed in REMOVED_V159_KEYS {
                assert!(
                    !out.config.contains_key(*removed),
                    "auto config emitted removed Camoufox v159 key: {removed}"
                );
            }
            assert!(
                !out.config.contains_key("headers.Accept-Language"),
                "auto mode must let Firefox derive Accept-Language from locale:all"
            );
            assert!(out.prefs.contains_key("intl.locale.requested"));
            assert!(out.prefs.contains_key("general.cpu_identity"));
            assert!(out.prefs.contains_key("layout.css.devPixelsPerPx"));
            assert_eq!(
                out.prefs.get("permissions.default.geo"),
                Some(&serde_json::json!(1))
            );

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
                auto_language: None,
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, nonce),
                block_webrtc: false,
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
        assert!(
            any_diff,
            "regenerating identity nonce did not change identity"
        );
    }

    #[test]
    fn host_os_default_matches_host_and_foreign_os_enables_canvas_noise() {
        let id = "host-check-instance";
        let geo = resolve("United States", id);

        let host_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: None,
                auto_language: None,
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                canvas_noise: None,
            },
            &geo,
        );
        assert_eq!(host_out.target_os, host_os_name());
        assert!(!host_out
            .prefs
            .contains_key("privacy.baselineFingerprintingProtection.overrides"));

        let foreign_os = if host_os_name() == "windows" {
            "macos"
        } else {
            "windows"
        };
        let foreign_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: Some(foreign_os),
                auto_language: None,
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                canvas_noise: None,
            },
            &geo,
        );
        assert_eq!(foreign_out.target_os, foreign_os);
        assert!(foreign_out
            .prefs
            .contains_key("privacy.baselineFingerprintingProtection.overrides"));
        assert_eq!(
            foreign_out.prefs.get("gfx.blacklist.webgpu"),
            Some(&serde_json::json!(4))
        );
    }

    #[test]
    fn auto_language_defaults_to_en_us_and_supports_geo_and_explicit_locales() {
        let id = "lang-test-instance";
        let japan_geo = resolve("Japan", id);

        let default_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: Some("windows"),
                auto_language: None,
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                canvas_noise: None,
            },
            &japan_geo,
        );
        assert_eq!(default_out.config.get("timezone"), Some(&serde_json::json!("Asia/Tokyo")));
        assert_eq!(default_out.config.get("locale:language"), Some(&serde_json::json!("en")));
        assert_eq!(default_out.config.get("locale:region"), Some(&serde_json::json!("US")));
        assert_eq!(default_out.config.get("locale:all"), Some(&serde_json::json!("en-US, en")));
        assert_eq!(
            default_out.prefs.get("intl.locale.requested"),
            Some(&serde_json::json!("en-US"))
        );
        assert!(!default_out.config.contains_key("headers.Accept-Language"));

        let geo_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: Some("macos"),
                auto_language: Some("geo"),
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                canvas_noise: None,
            },
            &japan_geo,
        );
        assert_eq!(geo_out.config.get("locale:language"), Some(&serde_json::json!("ja")));
        assert_eq!(geo_out.config.get("locale:region"), Some(&serde_json::json!("JP")));
        assert_eq!(geo_out.config.get("locale:all"), Some(&serde_json::json!("ja-JP, ja")));
        assert_eq!(
            geo_out.prefs.get("intl.locale.requested"),
            Some(&serde_json::json!("ja-JP"))
        );
        let mac_voices = geo_out.config.get("voices").and_then(|v| v.as_array()).unwrap();
        assert_eq!(mac_voices[0].get("lang"), Some(&serde_json::json!("ja-JP")));
        assert_eq!(mac_voices[0].get("isDefault"), Some(&serde_json::json!(true)));

        let explicit_out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: Some("windows"),
                auto_language: Some("de-DE"),
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                canvas_noise: None,
            },
            &japan_geo,
        );
        assert_eq!(explicit_out.config.get("locale:language"), Some(&serde_json::json!("de")));
        assert_eq!(explicit_out.config.get("locale:region"), Some(&serde_json::json!("DE")));
        assert_eq!(
            explicit_out.prefs.get("intl.locale.requested"),
            Some(&serde_json::json!("de-DE"))
        );
        let win_voices = explicit_out.config.get("voices").and_then(|v| v.as_array()).unwrap();
        assert!(!win_voices.is_empty());
        assert!(win_voices.iter().all(|v| v.get("lang") == Some(&serde_json::json!("de-DE"))));
    }

    #[test]
    fn native_host_fonts_stay_bounded_and_ipv6_mismatch_enables_ipv4_only() {
        let id = "native-fonts-ipv6";
        let mut geo = resolve("Japan", id);
        geo.ipv6_mismatch = true;

        let out = generate_auto_bundle(
            &AutoFingerprintOptions {
                auto_os: None,
                auto_language: None,
                change_window_size: false,
                default_outer_width: None,
                default_outer_height: None,
                target_ff_version: Some("159"),
                identity_seed: identity_seed(id, 0),
                block_webrtc: false,
                canvas_noise: None,
            },
            &geo,
        );
        assert_eq!(
            out.config.get("network:ipv4Only"),
            Some(&serde_json::json!(true))
        );
        assert!(out.config.contains_key("window.devicePixelRatio"));
        assert!(out.prefs.contains_key("layout.css.devPixelsPerPx"));

        if host_os_name() == "windows" || host_os_name() == "macos" {
            let fonts = out.config.get("fonts").and_then(|v| v.as_array()).unwrap();
            assert!(
                fonts.len() <= 25,
                "native host should claim OS base fonts only, got {}",
                fonts.len()
            );
        }

        // Cross-platform deterministic checks for Win10 vs Win11 native fonts,
        // 3-part BCP-47 script tags, and macOS en-US default voice fallback.
        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);
        let win10 = generate_font_subset("windows", true, false, &mut rng_a);
        let win11 = generate_font_subset("windows", true, true, &mut rng_b);
        assert!(!win10.iter().any(|f| WINDOWS_11_MARKER_FONTS.contains(&f.as_str())));
        assert!(win11.iter().any(|f| WINDOWS_11_MARKER_FONTS.contains(&f.as_str())));

        let (l, r, sc, _all, req) = resolve_auto_locale(Some("zh-Hans-CN"), &geo);
        assert_eq!((l.as_str(), r.as_str(), sc.as_deref(), req.as_str()), ("zh", "CN", Some("Hans"), "zh-Hans-CN"));

        let mut rng_c = StdRng::seed_from_u64(7);
        let mac_unmatched = build_locale_voices("macos", "xx-ZZ", &mut rng_c);
        assert_eq!(mac_unmatched[0].get("lang"), Some(&serde_json::json!("en-US")));
        assert_eq!(mac_unmatched[0].get("isDefault"), Some(&serde_json::json!(true)));
    }
}
