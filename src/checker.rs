use chrono::Utc;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub ok: bool,
    pub latency_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub checked_at: String,
}

pub async fn check_via_proxy(proxy_url: &str, check_url: &str) -> Result<CheckResult, String> {
    let checked_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if proxy_url.is_empty() {
        return Ok(CheckResult {
            ok: false,
            latency_ms: None,
            status_code: None,
            error: Some("No proxy URL configured".into()),
            checked_at,
        });
    }

    let proxy = reqwest::Proxy::all(proxy_url)
        .map_err(|e| format!("Failed to configure proxy: {e}"))?;

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(TIMEOUT)
        .connect_timeout(TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let start = std::time::Instant::now();

    match client.get(check_url).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let _ = response.text().await;
            let latency = start.elapsed().as_millis() as u64;
            Ok(CheckResult {
                ok: status >= 200 && status < 400,
                latency_ms: Some(latency),
                status_code: Some(status),
                error: None,
                checked_at,
            })
        }
        Err(e) => Ok(CheckResult {
            ok: false,
            latency_ms: None,
            status_code: None,
            error: Some(e.to_string()),
            checked_at,
        }),
    }
}

pub async fn check_direct(check_url: &str) -> Result<CheckResult, String> {
    let checked_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .connect_timeout(TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let start = std::time::Instant::now();

    match client.get(check_url).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let _ = response.text().await;
            let latency = start.elapsed().as_millis() as u64;
            Ok(CheckResult {
                ok: status >= 200 && status < 400,
                latency_ms: Some(latency),
                status_code: Some(status),
                error: None,
                checked_at,
            })
        }
        Err(e) => Ok(CheckResult {
            ok: false,
            latency_ms: None,
            status_code: None,
            error: Some(e.to_string()),
            checked_at,
        }),
    }
}
