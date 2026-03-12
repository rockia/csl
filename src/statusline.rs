use serde::Deserialize;
use std::io::Read;
use std::time::SystemTime;

use crate::format;
use crate::git;
use crate::usage;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StdinInput {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub context_window: u64,
    #[serde(default)]
    pub tokens_used: u64,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default)]
    pub session_start_time: String,
}

pub fn run() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let parsed: StdinInput = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return,
    };

    let lines = render(&parsed);
    for line in &lines {
        println!("{line}");
    }
}

fn render(input: &StdinInput) -> Vec<String> {
    let mut lines = Vec::new();

    // --- Session info line ---
    let context_pct = if input.context_window > 0 {
        (input.tokens_used as f64 / input.context_window as f64) * 100.0
    } else {
        0.0
    };
    let bar = format::colorized_progress_bar(context_pct);

    let dir = format::shorten_path(&input.working_directory);

    let git_info = if !input.working_directory.is_empty() {
        git::get_git_info(&input.working_directory)
    } else {
        None
    };

    let dir_branch = match &git_info {
        Some(info) => {
            let branch_str = if info.is_dirty {
                format::colored(&format!("({})", info.branch), format::YELLOW)
            } else {
                format!("({})", info.branch)
            };
            format!("{dir} {branch_str}")
        }
        None => dir,
    };

    let duration = parse_duration(&input.session_start_time);
    let duration_str = match duration {
        Some(secs) => format!("  \u{23F1} {}", format::format_duration(secs)),
        None => String::new(),
    };

    lines.push(format!(
        "{}  {}  {}{duration_str}",
        input.model, bar, dir_branch
    ));

    // --- Rate limit lines ---
    if let Some(usage_data) = usage::get_usage() {
        for limit in &usage_data.rate_limits {
            let limit_bar = format::colorized_progress_bar(limit.usage_percentage);
            lines.push(format!(
                "{}  {}  {}",
                limit.window_label, limit_bar, limit.reset_info
            ));
        }
    }

    lines
}

fn parse_duration(iso_str: &str) -> Option<u64> {
    if iso_str.is_empty() {
        return None;
    }

    let timestamp = parse_iso8601(iso_str)?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();

    if now > timestamp {
        Some(now - timestamp)
    } else {
        Some(0)
    }
}

fn parse_iso8601(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.len() < 19 {
        return None;
    }

    let date_part = &s[..10];
    let time_part = &s[11..19];

    let mut parts = date_part.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;

    let mut tparts = time_part.split(':');
    let hour: i64 = tparts.next()?.parse().ok()?;
    let min: i64 = tparts.next()?.parse().ok()?;
    let sec: i64 = tparts.next()?.parse().ok()?;

    let days = days_from_civil(year, month, day);
    let timestamp = days * 86400 + hour * 3600 + min * 60 + sec;

    if timestamp >= 0 {
        Some(timestamp as u64)
    } else {
        None
    }
}

/// Days from 1970-01-01 (civil date to days since epoch).
/// Algorithm from Howard Hinnant's date algorithms.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) as u64 + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json() {
        let json = r#"{"model":"sonnet 4.6","contextWindow":200000,"tokensUsed":96000,"workingDirectory":"/tmp","sessionStartTime":""}"#;
        let parsed: StdinInput = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.model, "sonnet 4.6");
        assert_eq!(parsed.context_window, 200000);
        assert_eq!(parsed.tokens_used, 96000);
    }

    #[test]
    fn parse_missing_fields() {
        let json = r#"{"model":"opus"}"#;
        let parsed: StdinInput = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.model, "opus");
        assert_eq!(parsed.context_window, 0);
        assert_eq!(parsed.tokens_used, 0);
        assert!(parsed.working_directory.is_empty());
    }

    #[test]
    fn parse_malformed_json() {
        let result = serde_json::from_str::<StdinInput>("not json");
        assert!(result.is_err());
    }

    #[test]
    fn iso8601_parsing() {
        let ts = parse_iso8601("2024-01-15T00:00:00Z");
        assert!(ts.is_some());
        assert!(ts.unwrap() > 1700000000);
    }

    #[test]
    fn empty_duration() {
        assert_eq!(parse_duration(""), None);
    }
}
