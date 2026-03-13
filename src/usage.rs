use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

const CACHE_PATH: &str = "/tmp/claude/statusline-usage-cache.json";
const CACHE_TTL_SECS: u64 = 60;
const API_URL: &str = "https://api.anthropic.com/api/oauth/usage";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageResponse {
    pub five_hour: Option<UsagePeriod>,
    pub seven_day: Option<UsagePeriod>,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsagePeriod {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtraUsage {
    pub is_enabled: Option<bool>,
    pub utilization: Option<f64>,
    pub used_credits: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub resets_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    timestamp: u64,
    data: UsageResponse,
}

pub fn fetch_usage() -> Option<UsageResponse> {
    // Try fresh cache first
    if let Some(cached) = read_cache(false) {
        return Some(cached);
    }

    // Try API call
    let token = resolve_token()?;
    match call_api(&token) {
        Some(response) => {
            write_cache(&response);
            Some(response)
        }
        None => {
            // Fall back to stale cache
            read_cache(true)
        }
    }
}

fn resolve_token() -> Option<String> {
    // 1. Environment variable
    if let Ok(token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN")
        && !token.is_empty()
    {
        return Some(token);
    }

    // 2. Platform keychain
    if let Some(token) = resolve_keychain_token() {
        return Some(token);
    }

    // 3. Credentials file
    resolve_file_token()
}

fn resolve_keychain_token() -> Option<String> {
    let output = if cfg!(target_os = "macos") {
        Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "Claude Code-credentials",
                "-w",
            ])
            .output()
            .ok()?
    } else {
        Command::new("secret-tool")
            .args(["lookup", "service", "Claude Code-credentials"])
            .output()
            .ok()?
    };

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    extract_access_token(&raw)
}

fn resolve_file_token() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = format!("{}/.claude/.credentials.json", home);
    let content = fs::read_to_string(path).ok()?;
    extract_access_token(&content)
}

pub(crate) fn extract_access_token(json_str: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
    val.get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

fn call_api(token: &str) -> Option<UsageResponse> {
    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(3)))
            .build(),
    );
    let response = agent
        .get(API_URL)
        .header("Authorization", &format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "claude-code/2.1.34")
        .call()
        .ok()?;

    let body = response.into_body().read_to_string().ok()?;
    serde_json::from_str(&body).ok()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_cache(allow_stale: bool) -> Option<UsageResponse> {
    let content = fs::read_to_string(CACHE_PATH).ok()?;
    let entry: CacheEntry = serde_json::from_str(&content).ok()?;
    let age = now_secs().saturating_sub(entry.timestamp);
    if allow_stale || age < CACHE_TTL_SECS {
        Some(entry.data)
    } else {
        None
    }
}

fn write_cache(data: &UsageResponse) {
    let entry = CacheEntry {
        timestamp: now_secs(),
        data: data.clone(),
    };
    if let Ok(json) = serde_json::to_string(&entry) {
        let dir = Path::new(CACHE_PATH).parent().unwrap();
        let _ = fs::create_dir_all(dir);
        let _ = fs::write(CACHE_PATH, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_access_token_valid() {
        let json = r#"{"claudeAiOauth":{"accessToken":"test-token-123"}}"#;
        assert_eq!(
            extract_access_token(json),
            Some("test-token-123".to_string())
        );
    }

    #[test]
    fn test_extract_access_token_missing_oauth() {
        let json = r#"{"other":"value"}"#;
        assert_eq!(extract_access_token(json), None);
    }

    #[test]
    fn test_extract_access_token_missing_token() {
        let json = r#"{"claudeAiOauth":{"other":"value"}}"#;
        assert_eq!(extract_access_token(json), None);
    }

    #[test]
    fn test_extract_access_token_invalid_json() {
        assert_eq!(extract_access_token("not json"), None);
    }

    #[test]
    fn test_extract_access_token_empty() {
        assert_eq!(extract_access_token(""), None);
    }

    #[test]
    fn test_usage_response_deserialization() {
        let json = r#"{
            "five_hour": {"utilization": 23.5, "resets_at": "2026-03-12T15:30:00Z"},
            "seven_day": {"utilization": 45.2, "resets_at": "2026-03-19T00:00:00Z"},
            "extra_usage": {"is_enabled": true, "utilization": 10.0, "used_credits": 500, "monthly_limit": 5000, "resets_at": "2026-04-01T00:00:00Z"}
        }"#;
        let resp: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.five_hour.as_ref().unwrap().utilization, Some(23.5));
        assert_eq!(resp.seven_day.as_ref().unwrap().utilization, Some(45.2));
        let extra = resp.extra_usage.as_ref().unwrap();
        assert_eq!(extra.is_enabled, Some(true));
        assert_eq!(extra.used_credits, Some(500.0));
        assert_eq!(extra.monthly_limit, Some(5000.0));
        assert_eq!(extra.resets_at.as_deref(), Some("2026-04-01T00:00:00Z"));
    }

    #[test]
    fn test_usage_response_minimal() {
        let json = r#"{}"#;
        let resp: UsageResponse = serde_json::from_str(json).unwrap();
        assert!(resp.five_hour.is_none());
        assert!(resp.seven_day.is_none());
        assert!(resp.extra_usage.is_none());
    }

    #[test]
    fn test_usage_response_partial() {
        let json = r#"{"five_hour": {"utilization": 10.0}}"#;
        let resp: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.five_hour.as_ref().unwrap().utilization, Some(10.0));
        assert!(resp.five_hour.as_ref().unwrap().resets_at.is_none());
        assert!(resp.seven_day.is_none());
    }

    #[test]
    fn test_cache_entry_roundtrip() {
        let data = UsageResponse {
            five_hour: Some(UsagePeriod {
                utilization: Some(25.0),
                resets_at: Some("2026-03-12T15:00:00Z".to_string()),
            }),
            seven_day: Some(UsagePeriod {
                utilization: Some(40.0),
                resets_at: None,
            }),
            extra_usage: None,
        };
        let entry = CacheEntry {
            timestamp: 1000,
            data,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: CacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.timestamp, 1000);
        assert_eq!(parsed.data.five_hour.unwrap().utilization, Some(25.0));
    }
}
