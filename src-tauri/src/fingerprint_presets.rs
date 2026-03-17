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

pub fn get_presets() -> &'static HashMap<String, Vec<Preset>> {
    FINGERPRINT_DICT.get_or_init(|| {
        let json_str = include_str!("fingerprint-presets.json");
        let parsed: FingerprintPresets =
            serde_json::from_str(json_str).expect("Failed to parse fingerprint-presets.json");
        parsed.presets
    })
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
