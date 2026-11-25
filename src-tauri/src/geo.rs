//! Per-instance deterministic identity seeding and live exit-IP geolocation binding.

use crate::camoufox::get_data_dir;
use crate::instances::ProxyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Once, OnceLock};
use std::time::Duration;
use tauri::AppHandle;

const GEO_PROVIDER_TIMEOUT: Duration = Duration::from_secs(5);
const IPV6_PROBE_TIMEOUT: Duration = Duration::from_millis(1500);

static GEO_TABLE: OnceLock<HashMap<String, CountryProfile>> = OnceLock::new();
static CLEANUP_LEGACY_CACHE: Once = Once::new();

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GeoLookup {
    pub ip: Option<String>,
    pub country_name: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct GeoContext {
    pub exit_ip: Option<String>,
    pub timezone: String,
    pub language: String,
    pub region: String,
    pub locale_all: String,
    pub coordinates: Option<(f64, f64)>,
    pub ipv6_mismatch: bool,
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

/// Build a coherent GeoContext from a live IP lookup, falling back to the
/// bundled country table only for missing fields.
pub fn resolve_from_lookup(lookup: &GeoLookup, salt: &str) -> GeoContext {
    let h = fnv1a64(salt.as_bytes());
    let profile = find_country_profile(
        lookup.country_name.as_deref(),
        lookup.country_code.as_deref(),
    );

    let (language, region, locale_all) = if let Some(p) = profile {
        let bare = p.language.split('-').next().unwrap_or("en").to_string();
        (
            bare.clone(),
            p.region.clone(),
            format!("{}, {}", p.language, bare),
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
        )
    } else {
        (
            "en".to_string(),
            "US".to_string(),
            "en-US, en".to_string(),
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
        coordinates,
        ipv6_mismatch: false,
    }
}

/// True when a direct IPv6 exit resolves to a different country or timezone
/// than the direct IPv4 exit (for example, an IPv4-only VPN on a dual-stack ISP).
pub fn ipv6_mismatches_ipv4(ipv4: &GeoContext, ipv6: &GeoLookup) -> bool {
    if let Some(code) = ipv6
        .country_code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !code.eq_ignore_ascii_case(&ipv4.region) {
            return true;
        }
    }
    if let Some(tz) = ipv6
        .timezone
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !ipv4.timezone.trim().is_empty() && !tz.eq_ignore_ascii_case(&ipv4.timezone) {
            return true;
        }
    }
    false
}

#[cfg(test)]
pub fn resolve(country: &str, salt: &str) -> GeoContext {
    let lookup = GeoLookup {
        ip: None,
        country_name: Some(country.to_string()),
        country_code: None,
        city: None,
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
        coordinates: None,
        ipv6_mismatch: false,
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberOrString {
    Num(f64),
    Str(String),
}

impl NumberOrString {
    fn as_f64(&self) -> Option<f64> {
        match self {
            NumberOrString::Num(v) if v.is_finite() => Some(*v),
            NumberOrString::Str(s) => s.trim().parse::<f64>().ok().filter(|v| v.is_finite()),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
struct GeoJsResponse {
    ip: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    timezone: Option<String>,
    latitude: Option<NumberOrString>,
    longitude: Option<NumberOrString>,
}

#[derive(Deserialize)]
struct IpWhoIsTimezone {
    id: Option<String>,
}

#[derive(Deserialize)]
struct IpWhoIsResponse {
    success: Option<bool>,
    ip: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    timezone: Option<IpWhoIsTimezone>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct IpApiCoResponse {
    ip: Option<String>,
    country_name: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    timezone: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

fn non_empty(val: Option<String>) -> Option<String> {
    val.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

pub(crate) fn parse_geojs(body: &str) -> Option<GeoLookup> {
    let parsed: GeoJsResponse = serde_json::from_str(body).ok()?;
    let country_name = non_empty(parsed.country);
    let country_code = non_empty(parsed.country_code);
    let timezone = non_empty(parsed.timezone);
    if country_name.is_none() && country_code.is_none() && timezone.is_none() {
        return None;
    }
    Some(GeoLookup {
        ip: non_empty(parsed.ip),
        country_name,
        country_code,
        city: non_empty(parsed.city),
        timezone,
        latitude: parsed.latitude.as_ref().and_then(NumberOrString::as_f64),
        longitude: parsed.longitude.as_ref().and_then(NumberOrString::as_f64),
    })
}

pub(crate) fn parse_ipwhois(body: &str) -> Option<GeoLookup> {
    let parsed: IpWhoIsResponse = serde_json::from_str(body).ok()?;
    if parsed.success != Some(true) {
        return None;
    }
    let country_name = non_empty(parsed.country);
    let country_code = non_empty(parsed.country_code);
    let timezone = parsed.timezone.and_then(|tz| non_empty(tz.id));
    if country_name.is_none() && country_code.is_none() && timezone.is_none() {
        return None;
    }
    Some(GeoLookup {
        ip: non_empty(parsed.ip),
        country_name,
        country_code,
        city: non_empty(parsed.city),
        timezone,
        latitude: parsed.latitude.filter(|v| v.is_finite()),
        longitude: parsed.longitude.filter(|v| v.is_finite()),
    })
}

pub(crate) fn parse_ipapi_co(body: &str) -> Option<GeoLookup> {
    let parsed: IpApiCoResponse = serde_json::from_str(body).ok()?;
    let country_name = non_empty(parsed.country_name);
    let country_code = non_empty(parsed.country_code);
    let timezone = non_empty(parsed.timezone);
    if country_name.is_none() && country_code.is_none() && timezone.is_none() {
        return None;
    }
    Some(GeoLookup {
        ip: non_empty(parsed.ip),
        country_name,
        country_code,
        city: non_empty(parsed.city),
        timezone,
        latitude: parsed.latitude.filter(|v| v.is_finite()),
        longitude: parsed.longitude.filter(|v| v.is_finite()),
    })
}

async fn fetch_url_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let resp = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Anon/0.1")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

/// Fetch live exit-IP geolocation over HTTPS without caching, so VPN switches
/// and rotating proxies always reflect the active exit.
pub async fn fetch_live_geo(proxy: Option<&ProxyConfig>) -> Result<GeoLookup, String> {
    let mut builder = reqwest::Client::builder().timeout(GEO_PROVIDER_TIMEOUT);
    if let Some(pc) = proxy {
        builder = builder.proxy(crate::proxy_tester::reqwest_proxy(pc)?);
    }
    let client = builder
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut errors: Vec<String> = Vec::new();

    match fetch_url_text(&client, "https://get.geojs.io/v1/ip/geo.json").await {
        Ok(body) => {
            if let Some(lookup) = parse_geojs(&body) {
                return Ok(lookup);
            }
            errors.push("geojs: invalid response".into());
        }
        Err(e) => errors.push(format!("geojs: {}", e)),
    }

    match fetch_url_text(&client, "https://ipwho.is/").await {
        Ok(body) => {
            if let Some(lookup) = parse_ipwhois(&body) {
                return Ok(lookup);
            }
            errors.push("ipwho.is: invalid response".into());
        }
        Err(e) => errors.push(format!("ipwho.is: {}", e)),
    }

    match fetch_url_text(&client, "https://ipapi.co/json/").await {
        Ok(body) => {
            if let Some(lookup) = parse_ipapi_co(&body) {
                return Ok(lookup);
            }
            errors.push("ipapi.co: invalid response".into());
        }
        Err(e) => errors.push(format!("ipapi.co: {}", e)),
    }

    Err(format!("Geo lookup failed ({})", errors.join("; ")))
}

async fn check_direct_ipv6_mismatch(ipv4: &GeoContext) -> bool {
    let is_ipv4_exit = ipv4
        .exit_ip
        .as_deref()
        .is_some_and(|ip| ip.contains('.') && !ip.contains(':'));
    if !is_ipv4_exit {
        return false;
    }

    let Ok(client) = reqwest::Client::builder()
        .timeout(IPV6_PROBE_TIMEOUT)
        .build()
    else {
        return false;
    };

    let Ok(body) = fetch_url_text(&client, "https://ipv6.geojs.io/v1/ip/geo.json").await else {
        return false;
    };

    let Some(ipv6_lookup) = parse_geojs(&body) else {
        return false;
    };
    let is_ipv6_exit = ipv6_lookup
        .ip
        .as_deref()
        .is_some_and(|ip| ip.contains(':'));
    if !is_ipv6_exit {
        return false;
    }

    ipv6_mismatches_ipv4(ipv4, &ipv6_lookup)
}

/// Resolve live geo identity through the instance's proxy (or direct/VPN egress).
pub async fn for_instance(
    app: &AppHandle,
    instance_id: &str,
    proxy: Option<&ProxyConfig>,
) -> GeoContext {
    let data_dir = get_data_dir(app).await;
    CLEANUP_LEGACY_CACHE.call_once(|| {
        let legacy = data_dir.join("geo_cache.json");
        if legacy.exists() {
            let _ = std::fs::remove_file(legacy);
        }
    });

    match fetch_live_geo(proxy).await {
        Ok(lookup) => {
            let mut ctx = resolve_from_lookup(&lookup, instance_id);
            if proxy.is_none() {
                ctx.ipv6_mismatch = check_direct_ipv6_mismatch(&ctx).await;
            }
            ctx
        }
        Err(err) => {
            eprintln!("Live geo lookup failed for instance {}: {}", instance_id, err);
            unknown()
        }
    }
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
    fn parses_geojs_ipwhois_and_ipapi_co_payloads() {
        let geojs = r#"{
            "ip": "180.149.230.197",
            "country": "Japan",
            "country_code": "JP",
            "city": "Tokyo",
            "timezone": "Asia/Tokyo",
            "latitude": "35.6893",
            "longitude": "139.6899"
        }"#;
        let g = parse_geojs(geojs).expect("geojs should parse");
        assert_eq!(g.ip.as_deref(), Some("180.149.230.197"));
        assert_eq!(g.country_name.as_deref(), Some("Japan"));
        assert_eq!(g.country_code.as_deref(), Some("JP"));
        assert_eq!(g.city.as_deref(), Some("Tokyo"));
        assert_eq!(g.timezone.as_deref(), Some("Asia/Tokyo"));
        assert_eq!(g.latitude, Some(35.6893));
        assert_eq!(g.longitude, Some(139.6899));

        let ipwhois = r#"{
            "success": true,
            "ip": "180.149.230.197",
            "country": "Japan",
            "country_code": "JP",
            "city": "Tokyo",
            "timezone": { "id": "Asia/Tokyo", "abbr": "JST" },
            "latitude": 35.6895,
            "longitude": 139.6917
        }"#;
        let w = parse_ipwhois(ipwhois).expect("ipwho.is should parse");
        assert_eq!(w.timezone.as_deref(), Some("Asia/Tokyo"));
        assert_eq!(w.country_code.as_deref(), Some("JP"));
        assert_eq!(w.city.as_deref(), Some("Tokyo"));

        let ipwhois_fail = r#"{"success": false, "message": "rate limited"}"#;
        assert!(parse_ipwhois(ipwhois_fail).is_none());

        let ipapi = r#"{
            "ip": "198.51.100.7",
            "country_name": "Germany",
            "country_code": "DE",
            "city": "Berlin",
            "timezone": "Europe/Berlin",
            "latitude": 52.52,
            "longitude": 13.405
        }"#;
        let a = parse_ipapi_co(ipapi).expect("ipapi.co should parse");
        assert_eq!(a.country_name.as_deref(), Some("Germany"));
        assert_eq!(a.city.as_deref(), Some("Berlin"));
        assert_eq!(a.timezone.as_deref(), Some("Europe/Berlin"));
    }

    #[test]
    fn ipv6_mismatch_detects_country_or_timezone_leak_past_vpn() {
        let japan_vpn = resolve_from_lookup(
            &GeoLookup {
                ip: Some("180.149.230.197".into()),
                country_name: Some("Japan".into()),
                country_code: Some("JP".into()),
                city: Some("Tokyo".into()),
                timezone: Some("Asia/Tokyo".into()),
                latitude: Some(35.6893),
                longitude: Some(139.6899),
            },
            "inst-1",
        );

        let isp_ipv6_bd = GeoLookup {
            ip: Some("2400:1234::1".into()),
            country_name: Some("Bangladesh".into()),
            country_code: Some("BD".into()),
            city: Some("Dhaka".into()),
            timezone: Some("Asia/Dhaka".into()),
            latitude: Some(23.78),
            longitude: Some(90.41),
        };
        assert!(ipv6_mismatches_ipv4(&japan_vpn, &isp_ipv6_bd));

        let vpn_ipv6_jp = GeoLookup {
            ip: Some("2404:6800::1".into()),
            country_name: Some("Japan".into()),
            country_code: Some("JP".into()),
            city: Some("Tokyo".into()),
            timezone: Some("Asia/Tokyo".into()),
            latitude: Some(35.69),
            longitude: Some(139.69),
        };
        assert!(!ipv6_mismatches_ipv4(&japan_vpn, &vpn_ipv6_jp));
    }

    #[test]
    fn resolve_from_lookup_preserves_real_exit_timezone_ip_and_city_coords() {
        let lookup = GeoLookup {
            ip: Some("198.51.100.7".into()),
            country_name: Some("United States".into()),
            country_code: Some("US".into()),
            city: Some("Chicago".into()),
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
        assert!(!g.locale_all.is_empty());
        assert!(g.coordinates.is_none());
    }
}
