use crate::instances::ProxyConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProxyTestResult {
    pub success: bool,
    pub ip: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub latency_ms: Option<u64>,
    pub dns_leak: Option<bool>,
    pub error: Option<String>,
}

pub fn percent_encode_userinfo(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            b => {
                out.push('%');
                out.push(
                    char::from_digit((b >> 4) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                out.push(
                    char::from_digit((b & 0x0F) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
            }
        }
    }
    out
}

/// Canonical proxy URL with percent-encoded credentials when present.
pub fn proxy_url(proxy_config: &ProxyConfig) -> Result<String, String> {
    let proxy_type = proxy_config
        .proxy_type
        .as_deref()
        .ok_or("No proxy type specified")?;
    let host = proxy_config.host.as_deref().ok_or("No host specified")?;
    let port = proxy_config.port.ok_or("No port specified")?;

    if host.is_empty() || port == 0 {
        return Err("Invalid proxy configuration".to_string());
    }

    Ok(
        match (
            proxy_config.username.as_deref(),
            proxy_config.password.as_deref(),
        ) {
            (Some(user), Some(pass)) if !user.is_empty() && !pass.is_empty() => format!(
                "{}://{}:{}@{}:{}",
                proxy_type,
                percent_encode_userinfo(user),
                percent_encode_userinfo(pass),
                host,
                port
            ),
            (Some(user), _) if !user.is_empty() => format!(
                "{}://{}@{}:{}",
                proxy_type,
                percent_encode_userinfo(user),
                host,
                port
            ),
            _ => format!("{}://{}:{}", proxy_type, host, port),
        },
    )
}

pub fn reqwest_proxy(proxy_config: &ProxyConfig) -> Result<reqwest::Proxy, String> {
    let url = proxy_url(proxy_config)?;
    let proxy_type = proxy_config.proxy_type.as_deref().unwrap_or_default();

    // reqwest speaks socks5 for both socks4 and socks5
    let url = match proxy_type {
        "http" | "socks5" => url,
        "socks4" => url.replacen("socks4://", "socks5://", 1),
        other => return Err(format!("Unsupported proxy type: {}", other)),
    };

    reqwest::Proxy::all(&url).map_err(|e| format!("Invalid proxy URL: {}", e))
}

pub(crate) fn is_direct_proxy(proxy_config: &ProxyConfig) -> bool {
    match proxy_config.proxy_type.as_deref().map(str::trim) {
        None | Some("") => true,
        Some(t) => t.eq_ignore_ascii_case("direct") || t.eq_ignore_ascii_case("none"),
    }
}

pub async fn test_proxy(proxy_config: ProxyConfig) -> Result<ProxyTestResult, String> {
    let direct = is_direct_proxy(&proxy_config);
    let proxy_ref = if direct { None } else { Some(&proxy_config) };
    if let Some(pc) = proxy_ref {
        // Validate proxy configuration upfront so invalid URLs fail fast.
        let _ = reqwest_proxy(pc)?;
    }

    match crate::geo::fetch_live_geo_with_rtt(proxy_ref).await {
        Ok((data, rtt_ms)) => Ok(ProxyTestResult {
            success: true,
            ip: data.ip,
            country: data.country_name.or(data.country_code),
            city: data.city,
            latency_ms: Some(rtt_ms),
            dns_leak: if direct { None } else { Some(false) },
            error: None,
        }),
        Err(e) => Ok(ProxyTestResult {
            success: false,
            ip: None,
            country: None,
            city: None,
            latency_ms: None,
            dns_leak: None,
            error: Some(e),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_url_percent_encodes_special_credential_characters() {
        let cfg = ProxyConfig {
            proxy_type: Some("http".into()),
            host: Some("proxy.example.com".into()),
            port: Some(8080),
            username: Some("user@realm".into()),
            password: Some("p@ss:w#rd/1?%20".into()),
        };
        let url = proxy_url(&cfg).unwrap();
        assert_eq!(
            url,
            "http://user%40realm:p%40ss%3Aw%23rd%2F1%3F%2520@proxy.example.com:8080"
        );
        assert!(reqwest_proxy(&cfg).is_ok());
    }

    #[test]
    fn direct_proxy_detection_handles_none_empty_and_direct() {
        assert!(is_direct_proxy(&ProxyConfig::default()));
        assert!(is_direct_proxy(&ProxyConfig {
            proxy_type: Some("direct".into()),
            ..Default::default()
        }));
        assert!(is_direct_proxy(&ProxyConfig {
            proxy_type: Some("".into()),
            ..Default::default()
        }));
        assert!(!is_direct_proxy(&ProxyConfig {
            proxy_type: Some("socks5".into()),
            host: Some("127.0.0.1".into()),
            port: Some(1080),
            ..Default::default()
        }));
    }
}
