use crate::instances::FingerprintConfig;
use serde::Serialize;

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Os {
    MacOS,
    Windows,
    Linux,
}

#[derive(Debug, Clone, Serialize)]
pub struct FingerprintConflict {
    pub message: String,
}

// ── OS detection helpers ─────────────────────────────────────────────────────

fn os_from_platform(platform: &str) -> Option<Os> {
    let p = platform.to_lowercase();
    if p == "macintel" {
        Some(Os::MacOS)
    } else if p == "win32" {
        Some(Os::Windows)
    } else if p.starts_with("linux") {
        Some(Os::Linux)
    } else {
        None
    }
}

fn os_from_ua(ua: &str) -> Option<Os> {
    let u = ua.to_lowercase();
    if u.contains("macintosh") || u.contains("mac os x") {
        Some(Os::MacOS)
    } else if u.contains("windows nt") {
        Some(Os::Windows)
    } else if u.contains("linux") || u.contains("x11") {
        Some(Os::Linux)
    } else {
        None
    }
}

fn os_label(os: &Os) -> &'static str {
    match os {
        Os::MacOS => "macOS",
        Os::Windows => "Windows",
        Os::Linux => "Linux",
    }
}

// ── Individual rule checks ───────────────────────────────────────────────────

// Rule 1: platform ↔ user-agent OS mismatch
fn check_platform_ua(fp: &FingerprintConfig) -> Option<FingerprintConflict> {
    let platform = fp.platform.as_deref()?;
    let ua = fp.user_agent.as_deref()?;
    let os_p = os_from_platform(platform)?;
    let os_u = os_from_ua(ua)?;
    if os_p != os_u {
        Some(FingerprintConflict {
            message: format!(
                "Platform \"{}\" indicates {} but User-Agent indicates {}.",
                platform,
                os_label(&os_p),
                os_label(&os_u)
            ),
        })
    } else {
        None
    }
}

// Rule 2: platform ↔ oscpu mismatch
fn check_platform_oscpu(fp: &FingerprintConfig) -> Option<FingerprintConflict> {
    let platform = fp.platform.as_deref()?;
    let oscpu = fp.oscpu.as_deref()?;
    let os_p = os_from_platform(platform)?;
    let osc = oscpu.to_lowercase();

    let mismatch = match os_p {
        Os::MacOS => osc.contains("windows") || osc.starts_with("linux"),
        Os::Windows => osc.contains("mac os x") || osc.starts_with("linux"),
        Os::Linux => osc.contains("windows") || osc.contains("mac os x"),
    };

    if mismatch {
        Some(FingerprintConflict {
            message: format!(
                "Platform \"{}\" ({}) conflicts with OSCPU \"{}\".",
                platform,
                os_label(&os_p),
                oscpu
            ),
        })
    } else {
        None
    }
}

// Rule 3: OS ↔ WebGL renderer mismatch
fn check_os_webgl(fp: &FingerprintConfig, detected_os: &Os) -> Vec<FingerprintConflict> {
    let mut conflicts = Vec::new();
    let renderer = match fp.webgl_renderer.as_deref() {
        Some(r) if !r.is_empty() => r,
        _ => return conflicts,
    };
    let rl = renderer.to_lowercase();

    match detected_os {
        Os::MacOS => {
            if rl.contains("angle") || rl.contains("direct3d") {
                conflicts.push(FingerprintConflict {
                    message: format!(
                        "Renderer \"{}\" (ANGLE/Direct3D) is Windows-only, conflicts with macOS.",
                        renderer
                    ),
                });
            }
        }
        Os::Windows => {
            if rl.starts_with("apple m") {
                conflicts.push(FingerprintConflict {
                    message: format!(
                        "Renderer \"{}\" is Apple Silicon, not available on Windows.",
                        renderer
                    ),
                });
            }
            if !rl.contains("angle") && !rl.starts_with("apple") {
                conflicts.push(FingerprintConflict {
                    message: format!(
                        "Renderer \"{}\" missing ANGLE prefix expected on Windows.",
                        renderer
                    ),
                });
            }
        }
        Os::Linux => {
            if rl.contains("angle") || rl.contains("direct3d") {
                conflicts.push(FingerprintConflict {
                    message: format!(
                        "Renderer \"{}\" (ANGLE/Direct3D) is not used on Linux.",
                        renderer
                    ),
                });
            }
            if rl.starts_with("apple m") {
                conflicts.push(FingerprintConflict {
                    message: format!(
                        "Renderer \"{}\" is Apple Silicon, not available on Linux.",
                        renderer
                    ),
                });
            }
        }
    }

    conflicts
}

// Rule 4: OS ↔ speech voices mismatch
fn check_os_voices(fp: &FingerprintConfig, detected_os: &Os) -> Vec<FingerprintConflict> {
    let voices = match fp.speech_voices.as_ref() {
        Some(v) if !v.is_empty() => v,
        _ => return Vec::new(),
    };

    // Distinctive OS-exclusive voice prefixes/names (from voices.json)
    const WINDOWS_VOICE_MARKERS: &[&str] = &[
        "microsoft david",
        "microsoft mark",
        "microsoft zira",
        "microsoft george",
        "microsoft hazel",
        "microsoft susan",
        "microsoft james",
    ];

    const MACOS_VOICE_MARKERS: &[&str] = &[
        "samantha", "alex", "fred", "victoria", "karen", "daniel", "fiona", "moira", "tessa",
        "veena", "rishi", "kyoko", "yuna", "tingting", "meijia", "sinji",
    ];

    let mut conflicts = Vec::new();

    for voice_entry in voices {
        // Voice entries can be "Name:Locale:Type" — extract the name part
        let name = voice_entry.split(':').next().unwrap_or(voice_entry);
        let nl = name.to_lowercase();

        match detected_os {
            Os::MacOS => {
                if WINDOWS_VOICE_MARKERS.iter().any(|m| nl.starts_with(m)) {
                    conflicts.push(FingerprintConflict {
                        message: format!(
                            "Voice \"{}\" is Windows-exclusive, conflicts with macOS.",
                            name
                        ),
                    });
                    break;
                }
            }
            Os::Windows => {
                if MACOS_VOICE_MARKERS.iter().any(|m| nl == *m) {
                    conflicts.push(FingerprintConflict {
                        message: format!(
                            "Voice \"{}\" is macOS-exclusive, conflicts with Windows.",
                            name
                        ),
                    });
                    break;
                }
            }
            Os::Linux => {
                if WINDOWS_VOICE_MARKERS.iter().any(|m| nl.starts_with(m))
                    || MACOS_VOICE_MARKERS.iter().any(|m| nl == *m)
                {
                    let foreign = if WINDOWS_VOICE_MARKERS.iter().any(|m| nl.starts_with(m)) {
                        "Windows"
                    } else {
                        "macOS"
                    };
                    conflicts.push(FingerprintConflict {
                        message: format!(
                            "Voice \"{}\" is {}-exclusive, conflicts with Linux.",
                            name, foreign
                        ),
                    });
                    break;
                }
            }
        }
    }

    conflicts
}

// Rule 5: header user-agent ↔ navigator user-agent mismatch
fn check_header_ua(fp: &FingerprintConfig) -> Option<FingerprintConflict> {
    let nav_ua = fp.user_agent.as_deref()?;
    let hdr_ua = fp.header_user_agent.as_deref()?;
    if nav_ua != hdr_ua {
        Some(FingerprintConflict {
            message: "HTTP header User-Agent and navigator User-Agent do not match.".into(),
        })
    } else {
        None
    }
}

// Rule 6: OS ↔ screen property coherence
fn check_os_screen(fp: &FingerprintConfig, detected_os: &Os) -> Vec<FingerprintConflict> {
    let mut conflicts = Vec::new();

    match detected_os {
        Os::MacOS => {
            if let Some(dpr) = fp.device_pixel_ratio {
                if dpr != 1.0 && dpr != 2.0 {
                    conflicts.push(FingerprintConflict {
                        message: format!("DPR {} is unusual for macOS (expected 1.0 or 2.0).", dpr),
                    });
                }
            }
            if let Some(tp) = fp.max_touch_points {
                if tp > 0 {
                    conflicts.push(FingerprintConflict {
                        message: format!("Touch points {} invalid for macOS (no touchscreen).", tp),
                    });
                }
            }
        }
        Os::Windows => {
            if let Some(cd) = fp.color_depth {
                if cd == 30 {
                    conflicts.push(FingerprintConflict {
                        message: "Color depth 30 is macOS-specific, Windows uses 24.".into(),
                    });
                }
            }
        }
        Os::Linux => {
            if let Some(cd) = fp.color_depth {
                if cd == 30 {
                    conflicts.push(FingerprintConflict {
                        message: "Color depth 30 is macOS-specific, Linux uses 24.".into(),
                    });
                }
            }
        }
    }

    conflicts
}

// ── Main entry point ─────────────────────────────────────────────────────────

pub fn validate_fingerprint(fp: &FingerprintConfig) -> Vec<FingerprintConflict> {
    // Skip validation in AUTO mode — presets are already curated
    if fp.auto_fingerprint == Some(true) {
        return Vec::new();
    }

    let mut conflicts = Vec::new();

    // Rules 1 & 2: direct field cross-checks (no OS detection needed)
    if let Some(c) = check_platform_ua(fp) {
        conflicts.push(c);
    }
    if let Some(c) = check_platform_oscpu(fp) {
        conflicts.push(c);
    }

    // Rule 5: header UA vs nav UA
    if let Some(c) = check_header_ua(fp) {
        conflicts.push(c);
    }

    // Determine OS for OS-dependent rules (3, 4, 6)
    let detected_os = fp
        .platform
        .as_deref()
        .and_then(os_from_platform)
        .or_else(|| fp.user_agent.as_deref().and_then(os_from_ua));

    if let Some(ref os) = detected_os {
        // Rule 3: OS ↔ WebGL
        conflicts.extend(check_os_webgl(fp, os));
        // Rule 4: OS ↔ Voices
        conflicts.extend(check_os_voices(fp, os));
        // Rule 6: OS ↔ Screen
        conflicts.extend(check_os_screen(fp, os));
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::FingerprintConfig;

    fn base() -> FingerprintConfig {
        FingerprintConfig::default()
    }

    #[test]
    fn no_conflicts_when_empty() {
        assert!(validate_fingerprint(&base()).is_empty());
    }

    #[test]
    fn no_conflicts_in_auto_mode() {
        let mut fp = base();
        fp.auto_fingerprint = Some(true);
        fp.platform = Some("MacIntel".into());
        fp.webgl_renderer = Some("ANGLE (NVIDIA)".into());
        assert!(validate_fingerprint(&fp).is_empty());
    }

    #[test]
    fn rule1_platform_ua_mismatch() {
        let mut fp = base();
        fp.platform = Some("MacIntel".into());
        fp.user_agent = Some(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:148.0) Gecko/20100101 Firefox/148.0"
                .into(),
        );
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule2_platform_oscpu_mismatch() {
        let mut fp = base();
        fp.platform = Some("Win32".into());
        fp.oscpu = Some("Intel Mac OS X 10.15".into());
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule3_macos_angle_renderer() {
        let mut fp = base();
        fp.platform = Some("MacIntel".into());
        fp.webgl_renderer = Some("ANGLE (NVIDIA, NVIDIA GeForce GTX 980 Direct3D11)".into());
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule3_windows_apple_renderer() {
        let mut fp = base();
        fp.platform = Some("Win32".into());
        fp.webgl_renderer = Some("Apple M1".into());
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule3_windows_missing_angle() {
        let mut fp = base();
        fp.platform = Some("Win32".into());
        fp.webgl_renderer = Some("NVIDIA GeForce GTX 980, or similar".into());
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule4_macos_windows_voice() {
        let mut fp = base();
        fp.platform = Some("MacIntel".into());
        fp.speech_voices = Some(vec!["Microsoft David:en-US:local".into()]);
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule4_windows_macos_voice() {
        let mut fp = base();
        fp.platform = Some("Win32".into());
        fp.speech_voices = Some(vec!["Samantha:en-US:local".into()]);
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule5_header_ua_mismatch() {
        let mut fp = base();
        fp.user_agent =
            Some("Mozilla/5.0 (Macintosh; rv:148.0) Gecko/20100101 Firefox/148.0".into());
        fp.header_user_agent =
            Some("Mozilla/5.0 (Windows NT 10.0; rv:148.0) Gecko/20100101 Firefox/148.0".into());
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule6_macos_fractional_dpr() {
        let mut fp = base();
        fp.platform = Some("MacIntel".into());
        fp.device_pixel_ratio = Some(1.25);
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule6_macos_touch_points() {
        let mut fp = base();
        fp.platform = Some("MacIntel".into());
        fp.max_touch_points = Some(5);
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn rule6_windows_color_depth_30() {
        let mut fp = base();
        fp.platform = Some("Win32".into());
        fp.color_depth = Some(30);
        let c = validate_fingerprint(&fp);
        assert!(!c.is_empty());
    }

    #[test]
    fn coherent_macos_fingerprint_passes() {
        let mut fp = base();
        fp.platform = Some("MacIntel".into());
        fp.user_agent = Some(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:148.0) Gecko/20100101 Firefox/148.0"
                .into(),
        );
        fp.oscpu = Some("Intel Mac OS X 10.15".into());
        fp.webgl_renderer = Some("Apple M1".into());
        fp.device_pixel_ratio = Some(2.0);
        fp.max_touch_points = Some(0);
        fp.speech_voices = Some(vec!["Samantha:en-US:local".into()]);
        assert!(validate_fingerprint(&fp).is_empty());
    }

    #[test]
    fn coherent_windows_fingerprint_passes() {
        let mut fp = base();
        fp.platform = Some("Win32".into());
        fp.user_agent = Some(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:148.0) Gecko/20100101 Firefox/148.0"
                .into(),
        );
        fp.oscpu = Some("Windows NT 10.0; Win64; x64".into());
        fp.webgl_renderer =
            Some("ANGLE (NVIDIA, NVIDIA GeForce GTX 980 Direct3D11 vs_5_0 ps_5_0)".into());
        fp.color_depth = Some(24);
        fp.speech_voices = Some(vec!["Microsoft David:en-US:local".into()]);
        assert!(validate_fingerprint(&fp).is_empty());
    }

    #[test]
    fn coherent_linux_fingerprint_passes() {
        let mut fp = base();
        fp.platform = Some("Linux x86_64".into());
        fp.user_agent =
            Some("Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0".into());
        fp.webgl_renderer = Some("NVIDIA GeForce GTX 980, or similar".into());
        fp.color_depth = Some(24);
        assert!(validate_fingerprint(&fp).is_empty());
    }

    #[test]
    fn partial_config_no_false_positives() {
        // Only WebGL set, no platform or UA — can't determine OS, should pass
        let mut fp = base();
        fp.webgl_renderer = Some("ANGLE (NVIDIA)".into());
        assert!(validate_fingerprint(&fp).is_empty());
    }
}
