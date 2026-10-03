use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresetNavigator {
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub hardware_concurrency: Option<u32>,
    pub max_touch_points: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresetScreen {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub color_depth: Option<u32>,
    pub avail_width: Option<u32>,
    pub avail_height: Option<u32>,
    pub device_pixel_ratio: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresetWebgl {
    pub unmasked_vendor: Option<String>,
    pub unmasked_renderer: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub navigator: Option<PresetNavigator>,
    pub screen: Option<PresetScreen>,
    pub webgl: Option<PresetWebgl>,
    pub speech_voices: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FingerprintPresets {
    pub version: u32,
    pub generated_at: String,
    pub presets: HashMap<String, Vec<Preset>>,
}

pub static FINGERPRINT_DICT: OnceLock<HashMap<String, Vec<Preset>>> = OnceLock::new();

fn parse_pool() -> Result<HashMap<String, Vec<Preset>>, serde_json::Error> {
    serde_json::from_str(include_str!("fingerprint-presets.json"))
        .map(|p: FingerprintPresets| p.presets)
}

/// The preset pool with every embedded user agent rewritten to the version of
/// the browser engine that is actually installed.
///
/// The presets carry a Firefox version because they were captured from a real
/// browser, but that number goes stale the moment the paired Camoufox build
/// moves — a preset captured from 148 launching on 152 is exactly the mismatch
/// this removes. Launch already rewrote the version via
/// `instances::harmonize_firefox_user_agent`, so this does not change what the
/// browser reports; it makes the settings dropdowns tell the truth, and makes
/// the JSON's literal Firefox version irrelevant rather than load-bearing.
///
/// A malformed file degrades to an empty pool instead of panicking: this feeds
/// the settings UI, and a bad hand-edit should not take the app down.
pub fn get_presets_for(engine_major: &str) -> HashMap<String, Vec<Preset>> {
    let Ok(pool) = parse_pool() else {
        eprintln!("fingerprint-presets.json is malformed; continuing with no presets");
        return HashMap::new();
    };

    pool.into_iter()
        .map(|(os, mut list)| {
            for preset in &mut list {
                if let Some(ua) = preset
                    .navigator
                    .as_mut()
                    .and_then(|n| n.user_agent.as_mut())
                {
                    *ua = crate::instances::harmonize_firefox_user_agent(ua, engine_major);
                }
            }
            (os, list)
        })
        .collect()
}

pub fn get_presets() -> &'static HashMap<String, Vec<Preset>> {
    FINGERPRINT_DICT.get_or_init(|| parse_pool().expect("Failed to parse fingerprint-presets.json"))
}

#[allow(dead_code)]
pub fn get_preset(os: &str, index: usize) -> Option<&'static Preset> {
    get_presets().get(os).and_then(|presets| presets.get(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_dict_loads() {
        let dict = get_presets();
        assert!(!dict.is_empty());
        let macos_presets = dict.get("macos").expect("Missing macos presets");
        assert!(!macos_presets.is_empty());
        let first_preset = get_preset("macos", 0).unwrap();
        assert!(first_preset.navigator.is_some());
    }
}
