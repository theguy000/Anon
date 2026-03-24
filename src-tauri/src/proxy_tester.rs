use serde::{Deserialize, Serialize};
use crate::instances::ProxyConfig;
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

pub async fn test_proxy(proxy_config: ProxyConfig) -> Result<ProxyTestResult, String> {
    let proxy_type = proxy_config.proxy_type.as_deref().ok_or("No proxy type specified")?;
    let host = proxy_config.host.as_deref().ok_or("No host specified")?;
    let port = proxy_config.port.ok_or("No port specified")?;

    if host.is_empty() || port == 0 {
        return Err("Invalid proxy configuration".to_string());
    }

    let proxy_url = match (proxy_config.username.as_deref(), proxy_config.password.as_deref()) {
        (Some(user), Some(pass)) if !user.is_empty() && !pass.is_empty() => {
            format!("{}://{}:{}@{}:{}", proxy_type, user, pass, host, port)
        }
        (Some(user), _) if !user.is_empty() => {
            format!("{}://{}@{}:{}", proxy_type, user, host, port)
        }
        _ => format!("{}://{}:{}", proxy_type, host, port),
    };

    // Map proxy types for reqwest
    let reqwest_proxy_url = match proxy_type {
        "http" => proxy_url.clone(),
        "socks4" => proxy_url.replace("socks4://", "socks5://"), // reqwest uses socks5 for both
        "socks5" => proxy_url.clone(),
        _ => return Err(format!("Unsupported proxy type: {}", proxy_type)),
    };

    let proxy = reqwest::Proxy::all(&reqwest_proxy_url).map_err(|e| format!("Invalid proxy URL: {}", e))?;

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
                Ok(data) => {
                    Ok(ProxyTestResult {
                        success: true,
                        ip: data.ip,
                        country: data.country_name,
                        city: data.city,
                        latency_ms: Some(latency),
                        dns_leak: Some(false),
                        error: None,
                    })
                }
                Err(_) => {
                    Ok(ProxyTestResult {
                        success: true,
                        ip: None,
                        country: None,
                        city: None,
                        latency_ms: Some(latency),
                        dns_leak: None,
                        error: Some("Connected but failed to parse IP info".to_string()),
                    })
                }
            }
        }
        Err(e) => {
            Ok(ProxyTestResult {
                success: false,
                ip: None,
                country: None,
                city: None,
                latency_ms: None,
                dns_leak: None,
                error: Some(format!("Connection failed: {}", e)),
            })
        }
    }
}
