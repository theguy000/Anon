//! Per-instance deterministic identity seeding and exit-IP geolocation binding.

use crate::camoufox::{get_data_dir, write_json_atomic};
use crate::instances::ProxyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

const PROXY_CACHE_TTL_SECS: u64 = 6 * 3600;
const DIRECT_CACHE_TTL_SECS: u64 = 3600;

static GEO_TABLE: OnceLock<HashMap<String, CountryProfile>> = OnceLock::new();
static GEO_CACHE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Deserialize)]
struct GeoTable {
    profiles: HashMap<String, CountryProfile>,
}

#[derive(Deserialize, Clone)]
struct CountryProfile {
    timezones: Vec<String>,
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

/// FNV-1a 64-bit hash (stable across Rust releases and platforms).
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Deterministic 64-bit seed derived from an instance ID and optional regeneration nonce.
pub fn identity_seed(instance_id: &str, nonce: u64) -> u64 {
    let input = format!("{}:{}", instance_id, nonce);
    let h = fnv1a64(input.as_bytes());
    if h == 0 {
        1
    } else {
        h
    }
}

/// Credential-free cache key: hashes the proxy URL so passwords are never persisted to disk.
pub fn geo_cache_key(proxy: Option<&ProxyConfig>) -> Result<String, String> {
    match proxy {
        Some(pc) => {
            let url = crate::proxy_tester::proxy_url(pc)?;
            Ok(format!("proxy:{:016x}", fnv1a64(url.as_bytes())))
        }
        None => Ok("direct".to_string()),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoLookup {
    pub ip: Option<String>,
    pub country_name: Option<String>,
    pub country_code: Option<String>,
    pub timezone: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GeoCacheEntry {
    lookup: GeoLookup,
    cached_at: u64,
}

#[derive(Clone, Debug)]
pub struct GeoContext {
    pub exit_ip: Option<String>,
    pub timezone: String,
    pub language: String,
    pub region: String,
    pub locale_all: String,
    pub accept_language: String,
    pub coordinates: Option<(f64, f64)>,
}

fn find_country_profile(
    country_name: Option<&str>,
    country_code: Option<&str>,
) -> Option<&'static CountryProfile> {
    let tbl = table();
    if let Some(name) = country_name {
        if let Some(p) = tbl.get(name) {
            return Some(p);
        }
    }
    if let Some(code) = country_code {
        let upper = code.to_ascii_uppercase();
        if let Some(p) = tbl.values().find(|p| p.region.eq_ignore_ascii_case(&upper)) {
            return Some(p);
        }
    }
    None
}

/// Build a coherent GeoContext from a real IP lookup when available, falling back
/// to the bundled country table only for missing fields.
pub fn resolve_from_lookup(lookup: &GeoLookup, salt: &str) -> GeoContext {
    let h = fnv1a64(salt.as_bytes());
    let profile = find_country_profile(
        lookup.country_name.as_deref(),
        lookup.country_code.as_deref(),
    );

    let (language, region, locale_all, accept_language) = if let Some(p) = profile {
        let bare = p.language.split('-').next().unwrap_or("en").to_string();
        (
            bare.clone(),
            p.region.clone(),
            format!("{}, {}", p.language, bare),
            p.accept_language.clone(),
        )
    } else if let Some(code) = lookup
        .country_code
        .as_deref()
        .filter(|c| c.len() == 2 && c.chars().all(|ch| ch.is_ascii_alphabetic()))
    {
        let upper = code.to_ascii_uppercase();
        (
            "en".to_string(),
            upper.clone(),
            format!("en-{}, en", upper),
            format!("en-{},en;q=0.9", upper),
        )
    } else {
        (
            "en".to_string(),
            "US".to_string(),
            "en-US, en".to_string(),
            "en-US,en;q=0.9".to_string(),
        )
    };

    let timezone = lookup
        .timezone
        .as_deref()
        .filter(|tz| !tz.trim().is_empty())
        .map(str::to_string)
        .or_else(|| {
            profile
                .filter(|p| !p.timezones.is_empty())
                .map(|p| p.timezones[(h % p.timezones.len() as u64) as usize].clone())
        })
        .unwrap_or_else(|| "Etc/UTC".to_string());

    // Small ±0.03° (~3 km) deterministic jitter around observed exit coords so two
    // instances on the same exit IP don't report identical decimal coordinates while
    // staying inside the real city/region.
    let coordinates = match (lookup.latitude, lookup.longitude) {
        (Some(lat), Some(lon)) => {
            let jlat = (((h >> 8) % 600) as f64 / 10000.0) - 0.03;
            let jlon = (((h >> 28) % 600) as f64 / 10000.0) - 0.03;
            Some((
                (lat + jlat).clamp(-90.0, 90.0),
                (lon + jlon).clamp(-180.0, 180.0),
            ))
        }
        _ => profile.map(|p| {
            let jlat = (((h >> 8) % 600) as f64 / 10000.0) - 0.03;
            let jlon = (((h >> 28) % 600) as f64 / 10000.0) - 0.03;
            (
                (p.lat + jlat).clamp(-90.0, 90.0),
                (p.lon + jlon).clamp(-180.0, 180.0),
            )
        }),
    };

    GeoContext {
        exit_ip: lookup.ip.clone(),
        timezone,
        language,
        region,
        locale_all,
        accept_language,
        coordinates,
    }
}

#[cfg(test)]
pub fn resolve(country: &str, salt: &str) -> GeoContext {
    let lookup = GeoLookup {
        ip: None,
        country_name: Some(country.to_string()),
        country_code: None,
        timezone: None,
        latitude: None,
        longitude: None,
    };
    resolve_from_lookup(&lookup, salt)
}

fn unknown() -> GeoContext {
    GeoContext {
        exit_ip: None,
        timezone: "Etc/UTC".into(),
        language: "en".into(),
        region: "US".into(),
        locale_all: "en-US, en".into(),
        accept_language: "en-US,en;q=0.9".into(),
        coordinates: None,
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Deserialize)]
struct IpApiCoResponse {
    ip: Option<String>,
    country_name: Option<String>,
    country_code: Option<String>,
    timezone: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct IpApiComResponse {
    status: Option<String>,
    query: Option<String>,
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    timezone: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
}

async fn cache_path(app: &AppHandle) -> std::path::PathBuf {
    get_data_dir(app).await.join("geo_cache.json")
}

pub(crate) fn read_cached_lookup(
    path: &std::path::Path,
    key: &str,
    ttl_secs: u64,
    now: u64,
) -> Option<GeoLookup> {
    let text = std::fs::read_to_string(path).ok()?;
    let map = serde_json::from_str::<HashMap<String, GeoCacheEntry>>(&text).ok()?;
    let entry = map.get(key)?;
    if now.saturating_sub(entry.cached_at) <= ttl_secs {
        Some(entry.lookup.clone())
    } else {
        None
    }
}

pub(crate) fn write_cached_lookup(
    path: &std::path::Path,
    key: &str,
    lookup: &GeoLookup,
    now: u64,
) -> Result<(), String> {
    let _guard = GEO_CACHE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    let mut map = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<HashMap<String, GeoCacheEntry>>(&t).ok())
        .unwrap_or_default();
    map.insert(
        key.to_string(),
        GeoCacheEntry {
            lookup: lookup.clone(),
            cached_at: now,
        },
    );
    write_json_atomic(path, &map)
}

/// Resolve geo identity through the instance's proxy (or direct egress when unproxied).
pub async fn for_instance(
    app: &AppHandle,
    instance_id: &str,
    proxy: Option<&ProxyConfig>,
) -> GeoContext {
    match fetch_geo(app, proxy).await {
        Some(lookup) => resolve_from_lookup(&lookup, instance_id),
        None => unknown(),
    }
}

async fn fetch_geo(app: &AppHandle, proxy: Option<&ProxyConfig>) -> Option<GeoLookup> {
    let key = geo_cache_key(proxy).ok()?;
    let ttl = if proxy.is_some() {
        PROXY_CACHE_TTL_SECS
    } else {
        DIRECT_CACHE_TTL_SECS
    };
    let path = cache_path(app).await;
    let now = now_secs();

    if let Some(cached) = read_cached_lookup(&path, &key, ttl, now) {
        return Some(cached);
    }

    let mut builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10));
    if let Some(pc) = proxy {
        builder = builder.proxy(crate::proxy_tester::reqwest_proxy(pc).ok()?);
    }
    let client = builder.build().ok()?;

    let lookup = match fetch_from_ipapi_co(&client).await {
        Some(found) => found,
        None => fetch_from_ip_api_com(&client).await?,
    };

    let _ = write_cached_lookup(&path, &key, &lookup, now);
    Some(lookup)
}

async fn fetch_from_ipapi_co(client: &reqwest::Client) -> Option<GeoLookup> {
    let resp = client.get("https://ipapi.co/json/").send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let parsed = resp.json::<IpApiCoResponse>().await.ok()?;
    if parsed.country_name.is_none() && parsed.country_code.is_none() && parsed.timezone.is_none() {
        return None;
    }
    Some(GeoLookup {
        ip: parsed.ip,
        country_name: parsed.country_name,
        country_code: parsed.country_code,
        timezone: parsed.timezone,
        latitude: parsed.latitude,
        longitude: parsed.longitude,
    })
}

async fn fetch_from_ip_api_com(client: &reqwest::Client) -> Option<GeoLookup> {
    let resp = client
        .get("http://ip-api.com/json?fields=status,query,country,countryCode,timezone,lat,lon")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let parsed = resp.json::<IpApiComResponse>().await.ok()?;
    if parsed.status.as_deref() != Some("success") {
        return None;
    }
    Some(GeoLookup {
        ip: parsed.query,
        country_name: parsed.country,
        country_code: parsed.country_code,
        timezone: parsed.timezone,
        latitude: parsed.lat,
        longitude: parsed.lon,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_seed_is_deterministic_and_changes_with_nonce() {
        let id = "11111111-1111-4111-8111-111111111111";
        assert_eq!(identity_seed(id, 0), identity_seed(id, 0));
        assert_ne!(identity_seed(id, 0), identity_seed(id, 1));
        assert_ne!(
            identity_seed(id, 0),
            identity_seed("22222222-2222-4222-8222-222222222222", 0)
        );
    }

    #[test]
    fn geo_cache_key_hashes_proxy_credentials_and_respects_ttl() {
        let proxy = ProxyConfig {
            proxy_type: Some("socks5".into()),
            host: Some("proxy.secret.net".into()),
            port: Some(1080),
            username: Some("myuser".into()),
            password: Some("supersecretpass".into()),
        };
        let key = geo_cache_key(Some(&proxy)).unwrap();
        assert!(key.starts_with("proxy:"));
        assert!(!key.contains("myuser"));
        assert!(!key.contains("supersecretpass"));
        assert!(!key.contains("proxy.secret.net"));

        let tmp =
            std::env::temp_dir().join(format!("anon_geo_cache_{}.json", uuid::Uuid::new_v4()));
        let lookup = GeoLookup {
            ip: Some("203.0.113.10".into()),
            country_name: Some("Germany".into()),
            country_code: Some("DE".into()),
            timezone: Some("Europe/Berlin".into()),
            latitude: Some(52.52),
            longitude: Some(13.405),
        };
        write_cached_lookup(&tmp, &key, &lookup, 1000).unwrap();
        let disk_text = std::fs::read_to_string(&tmp).unwrap();
        assert!(!disk_text.contains("supersecretpass"));

        assert!(read_cached_lookup(&tmp, &key, 600, 1500).is_some());
        assert!(read_cached_lookup(&tmp, &key, 600, 1601).is_none());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn resolve_from_lookup_preserves_real_exit_timezone_ip_and_city_coords() {
        let lookup = GeoLookup {
            ip: Some("198.51.100.7".into()),
            country_name: Some("United States".into()),
            country_code: Some("US".into()),
            timezone: Some("America/Chicago".into()),
            latitude: Some(41.8781),
            longitude: Some(-87.6298),
        };
        let g = resolve_from_lookup(&lookup, "instance-1");
        assert_eq!(g.exit_ip.as_deref(), Some("198.51.100.7"));
        assert_eq!(g.timezone, "America/Chicago");
        assert_eq!(g.language, "en");
        assert_eq!(g.region, "US");
        let (lat, lon) = g.coordinates.unwrap();
        assert!((lat - 41.8781).abs() <= 0.031);
        assert!((lon - (-87.6298)).abs() <= 0.031);
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
        let g = resolve("Atlantis", "some-instance");
        assert!(!g.timezone.is_empty());
        assert!(!g.language.is_empty());
        assert!(!g.region.is_empty());
        assert!(!g.accept_language.is_empty());
        assert!(g.coordinates.is_none());
    }
}
