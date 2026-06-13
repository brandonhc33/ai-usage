// SPDX-License-Identifier: MIT

//! Types and loader for `~/.cache/ai-usage-status/status.json`.
//!
//! This module only reads the file written by the external `ai-usage-collect`
//! collector (see docs/docs/03-data-contract-status-json.md). The applet never
//! talks to the network, never logs in, and never runs Playwright itself.

use serde::Deserialize;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// How old `updated_at` can be before the applet treats the data as stale,
/// independent of the `stale` flag the collector may have written.
pub const STALE_AFTER: Duration = Duration::from_secs(15 * 60);

/// Usage limit window such as `five_hour`, `weekly` or `session`.
#[derive(Debug, Clone, Deserialize)]
pub struct UsageMetric {
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    #[serde(default)]
    pub reset_label: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailyRoutines {
    pub used: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodexStatus {
    pub ok: bool,
    #[serde(default)]
    pub five_hour: Option<UsageMetric>,
    #[serde(default)]
    pub weekly: Option<UsageMetric>,
    #[serde(default)]
    pub credits_remaining: Option<i64>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeStatus {
    pub ok: bool,
    #[serde(default)]
    pub session: Option<UsageMetric>,
    #[serde(default)]
    pub weekly: Option<UsageMetric>,
    #[serde(default)]
    pub daily_routines: Option<DailyRoutines>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorEntry {
    pub provider: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusFile {
    #[allow(dead_code)]
    pub schema_version: u32,
    pub updated_at: String,
    #[serde(default)]
    pub stale: bool,
    #[serde(default)]
    pub codex: Option<CodexStatus>,
    #[serde(default)]
    pub claude: Option<ClaudeStatus>,
    #[serde(default)]
    pub errors: Vec<ErrorEntry>,
}

impl StatusFile {
    /// True when `updated_at` is older than [`STALE_AFTER`], regardless of the
    /// `stale` flag written by the collector. This catches a collector that has
    /// stopped running entirely (see docs/docs/11-error-states.md).
    pub fn is_stale_by_age(&self) -> bool {
        match chrono::DateTime::parse_from_rfc3339(&self.updated_at) {
            Ok(updated_at) => {
                let age = chrono::Utc::now().signed_duration_since(updated_at);
                age > chrono::Duration::from_std(STALE_AFTER).unwrap_or_default()
            }
            Err(_) => false,
        }
    }

}

/// Outcome of trying to load and parse `status.json`.
#[derive(Debug, Clone)]
pub enum StatusState {
    /// The file does not exist yet (collector never ran).
    Missing,
    /// The file exists but is not valid JSON / does not match the schema.
    Invalid,
    /// Successfully parsed status.
    Loaded(StatusFile),
}

impl Default for StatusState {
    fn default() -> Self {
        StatusState::Missing
    }
}

/// `~/.cache/ai-usage-status/status.json`.
pub fn status_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    PathBuf::from(home)
        .join(".cache")
        .join("ai-usage-status")
        .join("status.json")
}

/// Reads and parses `status.json`. Never panics: missing files and invalid
/// JSON are reported as [`StatusState`] variants instead of errors.
pub fn load_status() -> StatusState {
    match std::fs::read_to_string(status_path()) {
        Ok(contents) => match serde_json::from_str::<StatusFile>(&contents) {
            Ok(status) => StatusState::Loaded(status),
            Err(_) => StatusState::Invalid,
        },
        Err(_) => StatusState::Missing,
    }
}

/// Best-effort "last updated" timestamp for display, e.g. `hace 2 min`.
pub fn format_relative(updated_at: &str) -> Option<String> {
    let updated_at = chrono::DateTime::parse_from_rfc3339(updated_at).ok()?;
    let now = SystemTime::now();
    let now: chrono::DateTime<chrono::Utc> = now.into();
    let age = now.signed_duration_since(updated_at);

    let minutes = age.num_minutes();
    Some(if minutes <= 0 {
        "hace instantes".to_string()
    } else if minutes == 1 {
        "hace 1 min".to_string()
    } else if minutes < 60 {
        format!("hace {minutes} min")
    } else {
        let hours = minutes / 60;
        if hours == 1 {
            "hace 1 h".to_string()
        } else {
            format!("hace {hours} h")
        }
    })
}
