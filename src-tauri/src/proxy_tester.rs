use crate::instances::ProxyConfig;
use serde::{Deserialize, Serialize};
use std::time::Instant;

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

#[derive(Deserialize)]
struct IpApiResponse {
    ip: Option<String>,
    country_name: Option<String>,
    city: Option<String>,
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

pub async fn test_proxy(proxy_config: ProxyConfig) -> Result<ProxyTestResult, String> {
    let proxy = reqwest_proxy(&proxy_config)?;

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let start = Instant::now();

    match client.get("https://ipapi.co/json/").send().await {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;

            if !response.status().is_success() {
                return Ok(ProxyTestResult {
                    success: false,
                    ip: None,
                    country: None,
                    city: None,
                    latency_ms: Some(latency),
                    dns_leak: None,
                    error: Some(format!("HTTP {}", response.status())),
                });
            }

            match response.json::<IpApiResponse>().await {
                Ok(data) => Ok(ProxyTestResult {
                    success: true,
                    ip: data.ip,
                    country: data.country_name,
                    city: data.city,
                    latency_ms: Some(latency),
                    dns_leak: Some(false),
                    error: None,
                }),
                Err(_) => Ok(ProxyTestResult {
                    success: true,
                    ip: None,
                    country: None,
                    city: None,
                    latency_ms: Some(latency),
                    dns_leak: None,
                    error: Some("Connected but failed to parse IP info".to_string()),
                }),
            }
        }
        Err(e) => Ok(ProxyTestResult {
            success: false,
            ip: None,
            country: None,
            city: None,
            latency_ms: None,
            dns_leak: None,
            error: Some(format!("Connection failed: {}", e)),
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
}
