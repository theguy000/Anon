//! Per-instance stable identity: geo binding and stable seeds.
//!
//! Camoufox's binary does NOT fill unset CAMOU_CONFIG keys — every accessor in
//! `additions/camoucfg/MaskConfig.hpp` returns `std::nullopt` for a missing key
//! and the caller falls through to stock Firefox. The fpgen/BrowserForge fill,
//! the coherence check, and the proxy→geo derivation all live in the Python and
//! TypeScript libraries, which this app bypasses by spawning `camoufox.exe`
//! directly. So anything not set here leaks the real host machine's value, and
//! every instance run on this host correlates on it.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use tauri::AppHandle;

use crate::instances::ProxyConfig;

// ── Bundled country table ────────────────────────────────────────────────────

static GEO_TABLE: OnceLock<HashMap<String, CountryProfile>> = OnceLock::new();

#[derive(Deserialize)]
struct GeoTable {
    profiles: HashMap<String, CountryProfile>,
}

#[derive(Deserialize, Clone)]
struct CountryProfile {
    timezones: Vec<String>,
    /// Full locale tag, e.g. "en-US". `locale:language` wants the bare language,
    /// so that is split off at resolve time rather than duplicated in the data.
    language: String,
    region: String,
    lat: f64,
    lon: f64,
    #[serde(rename = "acceptLanguage")]
    accept_language: String,
}

fn table() -> &'static HashMap<String, CountryProfile> {
    GEO_TABLE.get_or_init(|| {
        let parsed: GeoTable =
            serde_json::from_str(include_str!("geo.json")).expect("Failed to parse geo.json");
        parsed.profiles
    })
}

// ── Stable hashing ───────────────────────────────────────────────────────────

/// FNV-1a 64. Deliberately not `std::hash::DefaultHasher`, which is documented
/// as unstable across Rust releases — these values have to stay put for the
/// life of an instance.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[derive(Clone, Copy, Debug)]
pub struct Seeds {
    pub canvas: u32,
    pub audio: u32,
    pub fonts_spacing: u32,
}

/// Canvas / audio / font-spacing hashes must be STABLE for the life of an
/// instance. A seed that changes every launch is a stronger tell than a fixed
/// one: real hardware is stable, so a churning hash sitting next to a frozen
/// user agent reads as automation.
///
/// Derived from the instance id — a UUID that never changes — so there is
/// nothing to persist and no write that can silently fail and hand the instance
/// a fresh identity on the next launch.
pub fn stable_seeds(instance_id: &str) -> Seeds {
    let base = fnv1a64(instance_id.as_bytes());
    Seeds {
        canvas: nonzero(base ^ 0x9e37_79b9_7f4a_7c15),
        audio: nonzero(base.rotate_left(17) ^ 0xbf58_476d_1ce4_e5b9),
        fonts_spacing: nonzero(base.rotate_left(31) ^ 0x94d0_49bb_1331_11eb),
    }
}

/// Camoufox accepts a seed of 0, but a zero seed is a fixed point some
/// detectors probe for. Keep every seed in 1..=u32::MAX.
fn nonzero(h: u64) -> u32 {
    let v = (h >> 32) as u32;
    if v == 0 {
        1
    } else {
        v
    }
}

// ── Geo ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct GeoContext {
    /// `timezone` — must be a valid TZ identifier.
    pub timezone: String,
    /// `locale:language` — bare language, e.g. "en".
    pub language: String,
    /// `locale:region` — e.g. "US".
    pub region: String,
    /// `locale:all` — "en-US, en".
    pub locale_all: String,
    /// `headers.Accept-Language`.
    pub accept_language: String,
    /// `geolocation:latitude`/`longitude`. `None` when the country is unknown:
    /// asserting a position we cannot justify is worse than letting the browser
    /// deny geolocation, which is a common and non-leaking state.
    pub coordinates: Option<(f64, f64)>,
}

/// Resolve a country to a geo identity for one instance. `salt` is the instance
/// id: it selects the timezone within the country and jitters the coordinates,
/// so two instances behind the same country do not report an identical location.
pub fn resolve(country: &str, salt: &str) -> GeoContext {
    let h = fnv1a64(salt.as_bytes());

    let Some(profile) = table().get(country) else {
        return unknown();
    };
    if profile.timezones.is_empty() {
        return unknown();
    }

    let timezone = profile.timezones[(h % profile.timezones.len() as u64) as usize].clone();
    let bare = profile
        .language
        .split('-')
        .next()
        .unwrap_or("en")
        .to_string();

    // ±3° keeps the point inside the country's rough footprint while differing
    // per instance. Coarse on purpose: city-level precision is not claimable
    // from a country lookup.
    let jlat = ((h >> 8) % 6000) as f64 / 1000.0 - 3.0;
    let jlon = ((h >> 28) % 6000) as f64 / 1000.0 - 3.0;

    GeoContext {
        timezone,
        locale_all: format!("{}, {}", profile.language, bare),
        language: bare,
        region: profile.region.clone(),
        accept_language: profile.accept_language.clone(),
        coordinates: Some((
            (profile.lat + jlat).clamp(-90.0, 90.0),
            (profile.lon + jlon).clamp(-180.0, 180.0),
        )),
    }
}

/// No proxy, or the geo lookup failed. Timezone and locale are still set —
/// omitting those is what leaks the host machine's values, which is the whole
/// problem this module exists to solve.
fn unknown() -> GeoContext {
    GeoContext {
        timezone: "Etc/UTC".into(),
        language: "en".into(),
        region: "US".into(),
        locale_all: "en-US, en".into(),
        accept_language: "en-US,en;q=0.9".into(),
        coordinates: None,
    }
}

// ── Country lookup ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IpApiCountry {
    country_name: Option<String>,
}

async fn cache_path(app: &AppHandle) -> std::path::PathBuf {
    crate::camoufox::get_app_dir(app)
        .await
        .join("geo_cache.json")
}

/// Resolve the geo identity for an instance, looking the proxy's country up
/// through that proxy. Results are cached per proxy URL on disk: ipapi.co
/// rate-limits aggressively, and a throttled lookup would silently degrade every
/// instance to the unknown-country fallback.
pub async fn for_instance(
    app: &AppHandle,
    instance_id: &str,
    proxy: Option<&ProxyConfig>,
) -> GeoContext {
    match fetch_country(app, proxy).await {
        Some(country) => resolve(&country, instance_id),
        None => unknown(),
    }
}

async fn fetch_country(app: &AppHandle, proxy: Option<&ProxyConfig>) -> Option<String> {
    let proxy = proxy?;
    let key = crate::proxy_tester::proxy_url(proxy).ok()?;
    let path = cache_path(app).await;

    let read_cache = || {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<HashMap<String, String>>(&t).ok())
            .unwrap_or_default()
    };

    if let Some(cached) = read_cache().get(&key) {
        return Some(cached.clone());
    }

    let client = reqwest::Client::builder()
        .proxy(crate::proxy_tester::reqwest_proxy(proxy).ok()?)
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;

    let body = client
        .get("https://ipapi.co/json/")
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let country = serde_json::from_str::<IpApiCountry>(&body)
        .ok()?
        .country_name?;

    let mut cache = read_cache();
    cache.insert(key, country.clone());
    if let Ok(json) = serde_json::to_string(&cache) {
        let _ = std::fs::write(&path, json);
    }
    Some(country)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_are_stable_per_instance_and_distinct_across_instances() {
        let a = stable_seeds("11111111-1111-4111-8111-111111111111");
        let b = stable_seeds("22222222-2222-4222-8222-222222222222");

        assert_eq!(
            a.canvas,
            stable_seeds("11111111-1111-4111-8111-111111111111").canvas
        );
        assert_eq!(
            a.audio,
            stable_seeds("11111111-1111-4111-8111-111111111111").audio
        );
        assert_ne!(a.canvas, b.canvas);

        // 0 is a fixed point some detectors probe for
        for s in [a.canvas, a.audio, a.fonts_spacing] {
            assert_ne!(s, 0);
        }
    }

    #[test]
    fn known_country_resolves_to_coherent_locale_parts() {
        let g = resolve("Germany", "some-instance");
        assert_eq!(g.language, "de");
        assert_eq!(g.region, "DE");
        assert_eq!(g.locale_all, "de-DE, de");
        assert_eq!(g.timezone, "Europe/Berlin");
        assert!(g.coordinates.is_some());
    }

    #[test]
    fn unknown_country_still_sets_timezone_and_locale() {
        // The whole point: omitting these leaks the host's real values.
        let g = resolve("Atlantis", "some-instance");
        assert!(!g.timezone.is_empty());
        assert!(!g.language.is_empty());
        assert!(!g.region.is_empty());
        assert!(!g.accept_language.is_empty());
        assert!(g.coordinates.is_none());
    }

    #[test]
    fn same_country_different_instances_do_not_collide() {
        let a = resolve("United States", "instance-a");
        let b = resolve("United States", "instance-b");
        let differs = a.timezone != b.timezone || a.coordinates != b.coordinates;
        assert!(
            differs,
            "two instances in one country reported identical geo"
        );
    }
}
