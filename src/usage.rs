use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CACHE_DIR: &str = "/tmp/csl";
const CACHE_PATH: &str = "/tmp/csl/usage-cache.json";
const CACHE_TTL: Duration = Duration::from_secs(60);
const API_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const API_TIMEOUT: Duration = Duration::from_secs(3);

// ---------- public types ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub rate_limits: Vec<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub window_label: String,
    pub usage_percentage: f64,
    pub reset_info: String,
}

// ---------- intermediate API response types ----------

/// Mirror of the expected JSON envelope from the API.
#[derive(Deserialize)]
struct ApiResponse {
    #[serde(default, rename = "rateLimits")]
    rate_limits: Vec<ApiRateLimit>,
}

#[derive(Deserialize)]
struct ApiRateLimit {
    #[serde(default, rename = "windowLabel")]
    window_label: String,
    #[serde(default, rename = "usagePercentage")]
    usage_percentage: f64,
    #[serde(default, rename = "resetInfo")]
    reset_info: String,
}

// ---------- token resolution ----------

/// Resolve an OAuth token by trying, in order:
/// 1. `CLAUDE_OAUTH_TOKEN` environment variable
/// 2. System keyring (service "claude-api", user "oauth-token")
/// 3. `~/.claude/.credentials.json` (`oauthToken` or `token` field)
pub fn resolve_token() -> Option<String> {
    // 1. Environment variable
    if let Ok(val) = std::env::var("CLAUDE_OAUTH_TOKEN")
        && !val.is_empty()
    {
        return Some(val);
    }

    // 2. Keyring
    if let Some(tok) = resolve_token_keyring() {
        return Some(tok);
    }

    // 3. Credentials file
    if let Some(tok) = resolve_token_credentials_file() {
        return Some(tok);
    }

    None
}

fn resolve_token_keyring() -> Option<String> {
    let entry = keyring::Entry::new("claude-api", "oauth-token").ok()?;
    let password = entry.get_password().ok()?;
    if password.is_empty() {
        None
    } else {
        Some(password)
    }
}

fn resolve_token_credentials_file() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(".claude").join(".credentials.json");
    let contents = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let obj = value.as_object()?;

    // Try oauthToken first, then token.
    for key in &["oauthToken", "token"] {
        if let Some(tok) = obj.get(*key).and_then(|v| v.as_str())
            && !tok.is_empty()
        {
            return Some(tok.to_owned());
        }
    }

    None
}

// ---------- cache management ----------

fn cache_path() -> PathBuf {
    PathBuf::from(CACHE_PATH)
}

/// Read cached usage data. Returns `Some` only when the cache file exists,
/// is younger than `CACHE_TTL`, and contains valid JSON.
pub fn read_cache() -> Option<UsageData> {
    read_cache_inner(true)
}

/// Read the cache ignoring staleness (used as a fallback when the API fails).
fn read_stale_cache() -> Option<UsageData> {
    read_cache_inner(false)
}

fn read_cache_inner(enforce_ttl: bool) -> Option<UsageData> {
    let path = cache_path();

    let metadata = fs::metadata(&path).ok()?;

    if enforce_ttl {
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(CACHE_TTL);
        if age >= CACHE_TTL {
            return None;
        }
    }

    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            // Unreadable — delete and bail.
            let _ = fs::remove_file(&path);
            return None;
        }
    };

    match serde_json::from_str::<UsageData>(&contents) {
        Ok(data) => Some(data),
        Err(_) => {
            // Corrupt cache — delete and bail.
            let _ = fs::remove_file(&path);
            None
        }
    }
}

pub fn write_cache(data: &UsageData) {
    let _ = fs::create_dir_all(CACHE_DIR);
    let json = match serde_json::to_string(data) {
        Ok(j) => j,
        Err(_) => return,
    };
    let _ = fs::write(cache_path(), json);
}

// ---------- API fetch ----------

/// Fetch usage data from the Anthropic API.
/// Returns `None` on any error (network, timeout, non-200, parse failure).
pub fn fetch_usage(token: &str) -> Option<UsageData> {
    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(API_TIMEOUT))
            .build(),
    );

    let mut response = agent
        .get(API_URL)
        .header("Authorization", &format!("Bearer {}", token))
        .call()
        .ok()?;

    let body: String = response.body_mut().read_to_string().ok()?;

    // Try the expected shape first.
    if let Ok(api) = serde_json::from_str::<ApiResponse>(&body) {
        let limits: Vec<RateLimit> = api
            .rate_limits
            .into_iter()
            .map(|r| RateLimit {
                window_label: r.window_label,
                usage_percentage: r.usage_percentage,
                reset_info: r.reset_info,
            })
            .collect();

        return Some(UsageData {
            rate_limits: limits,
        });
    }

    // Fallback: if the JSON is valid but shaped differently, return empty
    // rate_limits so callers at least know the fetch succeeded.
    if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
        return Some(UsageData {
            rate_limits: Vec::new(),
        });
    }

    None
}

// ---------- public orchestrator ----------

/// Get current API usage data.
///
/// Resolution order:
/// 1. Fresh cache (< 60 s old) — return immediately.
/// 2. Resolve an OAuth token; if unavailable return `None`.
/// 3. Fetch from the API; on success update cache and return.
/// 4. On API failure fall back to stale cache.
/// 5. If everything fails return `None`.
pub fn get_usage() -> Option<UsageData> {
    // 1. Fresh cache
    if let Some(cached) = read_cache() {
        return Some(cached);
    }

    // 2. Resolve token
    let token = resolve_token()?;

    // 3. Fetch from API
    if let Some(data) = fetch_usage(&token) {
        write_cache(&data);
        return Some(data);
    }

    // 4. Stale cache fallback
    if let Some(stale) = read_stale_cache() {
        return Some(stale);
    }

    // 5. Nothing worked
    None
}
