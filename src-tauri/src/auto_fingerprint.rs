//! AUTO mode fingerprint generation.

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::fingerprint_presets;

// ── Bundled data ────────────────────────────────────────────────────────────

static FONTS: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static VOICES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

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

// ── Essential / marker fonts per OS ─────────────────────────────────────────

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

const MARKER_FONTS_MACOS: &[&str] = &[
    "Helvetica Neue",
    "PingFang HK",
    "PingFang SC",
    "PingFang TC",
];
const MARKER_FONTS_WINDOWS: &[&str] = &["Segoe UI", "Tahoma", "Cambria Math", "Nirmala UI"];
const MARKER_FONTS_LINUX: &[&str] = &["Arimo", "Cousine", "Tinos", "Twemoji Mozilla"];

// ── Essential voices per OS ─────────────────────────────────────────────────

const ESSENTIAL_VOICES_MACOS: &[&str] =
    &["Samantha", "Alex", "Fred", "Victoria", "Karen", "Daniel"];

// ── Font subset generation ──────────────────────────────────────────────────

fn generate_random_font_subset(target_os: &str) -> Vec<String> {
    let fonts_data = load_fonts();
    let mut rng = rand::thread_rng();

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

    // Split into essential and non-essential
    let mut result: Vec<String> = full_list
        .iter()
        .filter(|f| essential_set.contains(f.as_str()))
        .cloned()
        .collect();
    let non_essential: Vec<&String> = full_list
        .iter()
        .filter(|f| !essential_set.contains(f.as_str()))
        .collect();

    // Random percentage between 30-78%
    let pct = rng.gen_range(30..=78);
    let count = (pct as f64 / 100.0 * non_essential.len() as f64).round() as usize;

    // Randomly select non-essential fonts
    let mut indices: Vec<usize> = (0..non_essential.len()).collect();
    indices.shuffle(&mut rng);
    for &idx in indices.iter().take(count) {
        result.push(non_essential[idx].clone());
    }

    // Ensure marker fonts are present
    let existing: HashSet<String> = result.iter().cloned().collect();
    for &marker in markers {
        if !existing.contains(marker) {
            result.push(marker.to_string());
        }
    }

    result
}

// ── Voice subset generation ─────────────────────────────────────────────────

fn generate_random_voice_subset(target_os: &str) -> Vec<String> {
    let voices_data = load_voices();
    let mut rng = rand::thread_rng();

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

    // Extract voice names from "Name:locale:type" format
    let voice_names: Vec<String> = full_list
        .iter()
        .filter_map(|entry| entry.split(':').next().map(String::from))
        .collect();

    // Windows has too few voices to subset — return all
    if target_os == "windows" {
        return voice_names;
    }

    // macOS: random 40-80% subset
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
    indices.shuffle(&mut rng);
    for &idx in indices.iter().take(count) {
        result.push(non_essential[idx].clone());
    }

    result
}

// ── Main generation function ────────────────────────────────────────────────

/// Build a complete CAMOU_CONFIG JSON from a random preset.
///
/// `seeds` and `geo` come from the caller because both must be stable per
/// instance rather than redrawn per launch. See `crate::geo`.
pub fn generate_auto_config(
    change_window_size: bool,
    default_outer_width: Option<u32>,
    default_outer_height: Option<u32>,
    target_ff_version: Option<&str>,
    seeds: crate::geo::Seeds,
    geo: &crate::geo::GeoContext,
) -> serde_json::Value {
    let presets = fingerprint_presets::get_presets();
    let mut rng = rand::thread_rng();

    // Collect all presets from all OS categories
    let mut candidates: Vec<&fingerprint_presets::Preset> = Vec::new();
    for preset_list in presets.values() {
        for p in preset_list {
            candidates.push(p);
        }
    }

    // When a fixed window-size preset is chosen, prefer fingerprint presets
    // whose screen is large enough to accommodate it.
    if !change_window_size {
        if let (Some(req_w), Some(req_h)) = (default_outer_width, default_outer_height) {
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

    if candidates.is_empty() {
        return serde_json::json!({});
    }

    // Pick a random preset
    let preset = candidates[rng.gen_range(0..candidates.len())];

    let mut config = serde_json::Map::new();

    // ── Show cursor (default off) ────────────────────────────────────────
    config.insert("showcursor".into(), serde_json::json!(false));

    // ── Navigator ────────────────────────────────────────────────────────
    let mut target_os = "macos"; // default

    if let Some(ref nav) = preset.navigator {
        if let Some(ref ua) = nav.user_agent {
            let eff_ua = if let Some(ver) = target_ff_version {
                crate::instances::harmonize_firefox_user_agent(ua, ver)
            } else {
                ua.clone()
            };
            config.insert(
                "navigator.userAgent".into(),
                serde_json::Value::String(eff_ua.clone()),
            );
            // Also set as HTTP header
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

            // Derive oscpu from platform
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

    // ── Screen ───────────────────────────────────────────────────────────
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
        // Note: devicePixelRatio not recommended per camoufox docs
        // (any value other than 1.0 is suspicious)

        // ── Derive window dimensions from screen ────────────────────────
        // Browsers don't usually run maximized. When enabled, vary window
        // geometry per launch; otherwise keep deterministic geometry.
        let avail_w = screen.avail_width.or(screen.width).unwrap_or(1920);
        let avail_h = screen.avail_height.or(screen.height).unwrap_or(1080);

        let (outer_w, outer_h) = if change_window_size {
            let pct_w = rng.gen_range(75..=100) as f64 / 100.0;
            let pct_h = rng.gen_range(75..=100) as f64 / 100.0;
            (
                ((avail_w as f64 * pct_w) as u32).max(800),
                ((avail_h as f64 * pct_h) as u32).max(600),
            )
        } else if let (Some(w), Some(h)) = (default_outer_width, default_outer_height) {
            (w.min(avail_w), h.min(avail_h))
        } else {
            (
                ((avail_w as f64 * 0.9) as u32).max(800),
                ((avail_h as f64 * 0.9) as u32).max(600),
            )
        };

        config.insert("window.outerWidth".into(), serde_json::json!(outer_w));
        config.insert("window.outerHeight".into(), serde_json::json!(outer_h));

        // innerWidth/Height: outer minus typical chrome (borders ~16px, toolbar ~80px)
        let inner_w = outer_w.saturating_sub(16);
        let inner_h = outer_h.saturating_sub(80);
        config.insert("window.innerWidth".into(), serde_json::json!(inner_w));
        config.insert("window.innerHeight".into(), serde_json::json!(inner_h));

        // screenX/screenY: random position or centered fallback
        let max_x = avail_w.saturating_sub(outer_w);
        let max_y = avail_h.saturating_sub(outer_h);
        let (screen_x, screen_y) = if change_window_size {
            (
                if max_x > 0 {
                    rng.gen_range(0..=max_x) as i32
                } else {
                    0
                },
                if max_y > 0 {
                    rng.gen_range(0..=max_y) as i32
                } else {
                    0
                },
            )
        } else {
            ((max_x / 2) as i32, (max_y / 2) as i32)
        };
        config.insert("window.screenX".into(), serde_json::json!(screen_x));
        config.insert("window.screenY".into(), serde_json::json!(screen_y));
    }

    // ── WebGL ────────────────────────────────────────────────────────────
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

    // ── Geo & Locale ────────────────────────────────────────────────────────
    // Bound to the instance's proxy, not to the host. Omitting any of these
    // does not make them unique — the binary falls through to stock Firefox, so
    // every instance would report this machine's real timezone, locale and
    // position, which both correlates your instances and contradicts the proxy.
    //
    // navigator.language / navigator.languages are deliberately NOT set: camoufox
    // derives them from locale:language / locale:region, which keeps them
    // consistent by construction. headers.User-Agent is likewise left to fall
    // back to navigator.userAgent.
    config.insert("timezone".into(), serde_json::json!(geo.timezone));
    config.insert("locale:language".into(), serde_json::json!(geo.language));
    config.insert("locale:region".into(), serde_json::json!(geo.region));
    config.insert("locale:all".into(), serde_json::json!(geo.locale_all));
    config.insert(
        "headers.Accept-Language".into(),
        serde_json::json!(geo.accept_language),
    );
    if let Some((lat, lon)) = geo.coordinates {
        // Accuracy is derived from coordinate precision when unset.
        config.insert("geolocation:latitude".into(), serde_json::json!(lat));
        config.insert("geolocation:longitude".into(), serde_json::json!(lon));
    }

    // ── Stable seeds ─────────────────────────────────────────────────────────
    // Per instance, not per launch — see crate::geo::stable_seeds.
    config.insert(
        "fonts:spacing_seed".into(),
        serde_json::json!(seeds.fonts_spacing),
    );
    config.insert("audio:seed".into(), serde_json::json!(seeds.audio));
    config.insert("canvas:seed".into(), serde_json::json!(seeds.canvas));

    // ── Random font subset ──────────────────────────────────────────────
    let fonts = generate_random_font_subset(target_os);
    if !fonts.is_empty() {
        config.insert(
            "fonts".into(),
            serde_json::Value::Array(fonts.into_iter().map(serde_json::Value::String).collect()),
        );
    }

    // ── Random voice subset ─────────────────────────────────────────────
    // Voices in CAMOU_CONFIG use the structured format
    if let Some(ref speech_voices) = preset.speech_voices {
        let voice_subset = generate_random_voice_subset(target_os);
        if !voice_subset.is_empty() {
            // Build voice objects matching camoufox's expected format
            let voice_arr: Vec<serde_json::Value> = speech_voices
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    let name = entry.split(':').next().unwrap_or("");
                    voice_subset.iter().any(|v| v == name)
                })
                .enumerate()
                .map(|(filtered_idx, (_, entry))| {
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
            if !voice_arr.is_empty() {
                config.insert("voices".into(), serde_json::Value::Array(voice_arr));
            }
        }
    }

    serde_json::Value::Object(config)
}

#[cfg(test)]
mod audit {
    use super::*;
    use crate::geo::{resolve, stable_seeds};
    use std::collections::{BTreeMap, HashSet};

    /// Proxy countries the sample is drawn across, so the entropy being measured
    /// is the entropy real instances actually get.
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

    /// Must never be absent from a generated config. Camoufox's binary returns
    /// `std::nullopt` for a missing key and falls through to stock Firefox, so an
    /// absent key does not mean "neutral" — it means the real host value is
    /// reported, and every instance on this machine correlates on it.
    const REQUIRED: &[&str] = &[
        "navigator.userAgent",
        "navigator.platform",
        "navigator.oscpu",
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
        "canvas:seed",
        "audio:seed",
        "fonts:spacing_seed",
    ];

    /// Must take more than one value across the pool. A single distinct value is
    /// the "every instance is the same user" failure.
    const MUST_VARY: &[&str] = &[
        "navigator.userAgent",
        "navigator.platform",
        "timezone",
        "locale:region",
        "geolocation:latitude",
        "canvas:seed",
        "audio:seed",
        "fonts:spacing_seed",
        "fonts",
    ];

    const SAMPLES: usize = 300;

    fn sample() -> BTreeMap<String, HashSet<String>> {
        let mut out: BTreeMap<String, HashSet<String>> = BTreeMap::new();
        for i in 0..SAMPLES {
            let id = format!("instance-{i:04}");
            let country = COUNTRIES[i % COUNTRIES.len()];
            let cfg = generate_auto_config(
                false,
                None,
                None,
                Some("156"),
                stable_seeds(&id),
                &resolve(country, &id),
            );
            collect("", &cfg, &mut out);
        }
        out
    }

    fn collect(prefix: &str, v: &serde_json::Value, out: &mut BTreeMap<String, HashSet<String>>) {
        match v {
            serde_json::Value::Object(map) => {
                for (k, val) in map {
                    let path = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    collect(&path, val, out);
                }
            }
            serde_json::Value::String(s) => {
                out.entry(prefix.to_string()).or_default().insert(s.clone());
            }
            other => {
                out.entry(prefix.to_string())
                    .or_default()
                    .insert(other.to_string());
            }
        }
    }

    /// Generate a pool of configs and report the two things that matter:
    /// which keys are identical across the whole pool (pinned), and which
    /// required keys never appear at all (host leak).
    #[test]
    fn audit_identity_entropy() {
        let seen = sample();

        let missing: Vec<&str> = REQUIRED
            .iter()
            .copied()
            .filter(|k| !seen.contains_key(*k))
            .collect();

        let mut pinned: Vec<(&str, usize)> = seen
            .iter()
            .filter(|(k, v)| v.len() == 1 && MUST_VARY.contains(&k.as_str()))
            .map(|(k, v)| (k.as_str(), v.len()))
            .collect();
        pinned.sort();

        println!("\n=== {SAMPLES} generated identities ===");
        println!("keys emitted: {}", seen.len());
        println!("\nrequired but ABSENT (host value leaks through):");
        for k in &missing {
            println!("  {k}");
        }
        if missing.is_empty() {
            println!("  (none)");
        }
        println!("\nmust-vary but PINNED (one value across all {SAMPLES}):");
        for (k, _) in &pinned {
            println!("  {k}");
        }
        if pinned.is_empty() {
            println!("  (none)");
        }
        println!("\nspread:");
        for k in MUST_VARY {
            let n = seen.get(*k).map(|s| s.len()).unwrap_or(0);
            println!("  {n:>4}  {k}");
        }

        assert!(
            missing.is_empty(),
            "these keys are absent, so the browser reports the real host value: {missing:?}"
        );
        assert!(
            pinned.is_empty(),
            "these keys are identical across every instance: {pinned:?}"
        );
    }

    /// The other half of uniqueness: an instance must be recognisably ITSELF on
    /// every relaunch. Churning seeds next to a frozen user agent is a stronger
    /// tell than a fixed value.
    #[test]
    fn audit_seeds_are_stable_across_launches() {
        let id = "aaaaaaaa-1111-4111-8111-111111111111";
        let seeds = stable_seeds(id);
        let geo = resolve("Germany", id);

        let mut seen = HashSet::new();
        for _ in 0..25 {
            let cfg = generate_auto_config(true, None, None, Some("156"), seeds, &geo);
            let obj = cfg.as_object().unwrap();
            seen.insert((
                obj["canvas:seed"].clone(),
                obj["audio:seed"].clone(),
                obj["fonts:spacing_seed"].clone(),
                obj["timezone"].clone(),
                obj["locale:region"].clone(),
            ));
        }

        assert_eq!(
            seen.len(),
            1,
            "identity fields churned across launches of one instance: {seen:?}"
        );
    }
}
