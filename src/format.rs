// SPDX-License-Identifier: MIT

//! Turns a [`StatusState`] into the strings and colors the UI renders.
//!
//! Keeping this logic out of `app.rs` makes the view function a thin layer
//! over plain data, and keeps the percentage/threshold rules in one place.

use crate::icons::IconKind;
use crate::status::{DailyRoutines, ErrorEntry, StatusFile, StatusState, UsageMetric};
use cosmic::iced::Color;

/// Mirrors the defaults in `docs/templates/config.example.json`.
pub const WARNING_USED_PERCENT: f64 = 75.0;
pub const CRITICAL_USED_PERCENT: f64 = 95.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Normal,
    Warning,
    Critical,
    /// No data available yet (missing file, no primary provider).
    Muted,
    /// Session expired, `ai-usage-auth <provider>` must be run again.
    LoginRequired,
}

impl Severity {
    /// Color override for the topbar label. `None` keeps the theme's
    /// default text color.
    pub fn color(self) -> Option<Color> {
        match self {
            Severity::Normal => None,
            Severity::Warning => Some(Color::from_rgb(0.95, 0.61, 0.07)),
            Severity::Critical => Some(Color::from_rgb(0.90, 0.18, 0.22)),
            Severity::Muted => Some(Color::from_rgb(0.55, 0.57, 0.60)),
            Severity::LoginRequired => Some(Color::from_rgb(0.36, 0.54, 0.93)),
        }
    }
}

/// Everything the topbar button needs to render itself.
pub struct Topbar {
    pub icon: IconKind,
    pub label: String,
    pub severity: Severity,
    pub tooltip: String,
}

fn severity_for_percent(used_percent: Option<f64>) -> Severity {
    match used_percent {
        Some(p) if p >= CRITICAL_USED_PERCENT => Severity::Critical,
        Some(p) if p >= WARNING_USED_PERCENT => Severity::Warning,
        _ => Severity::Normal,
    }
}

fn provider_name(provider: &str) -> &'static str {
    match provider {
        "codex" => "Codex",
        "claude" => "Claude",
        _ => "AI",
    }
}

/// Computes the topbar icon, label and color for the current status.
pub fn topbar(state: &StatusState) -> Topbar {
    match state {
        StatusState::Missing => Topbar {
            icon: IconKind::Ai,
            label: "--".to_string(),
            severity: Severity::Muted,
            tooltip: "AI Usage — ejecuta ai-usage-collect".to_string(),
        },
        StatusState::Invalid => Topbar {
            icon: IconKind::Ai,
            label: "!".to_string(),
            severity: Severity::Critical,
            tooltip: "AI Usage — status.json inválido".to_string(),
        },
        StatusState::Loaded(status) => topbar_loaded(status),
    }
}

fn topbar_loaded(status: &StatusFile) -> Topbar {
    if status.login_required() {
        return Topbar {
            icon: IconKind::for_provider(&status.primary.provider),
            label: "login".to_string(),
            severity: Severity::LoginRequired,
            tooltip: format!(
                "AI Usage — {} requiere iniciar sesión (ai-usage-auth)",
                provider_name(&status.primary.provider)
            ),
        };
    }

    if status.primary.provider == "none" || status.primary.used_percent.is_none() {
        return Topbar {
            icon: IconKind::Ai,
            label: "--".to_string(),
            severity: Severity::Muted,
            tooltip: "AI Usage — sin datos disponibles".to_string(),
        };
    }

    let stale = status.stale || status.is_stale_by_age();
    let mut severity = severity_for_percent(status.primary.used_percent);
    let mut label = status.primary.label.clone();

    if !status.ok || stale {
        label.push('!');
        if severity == Severity::Normal {
            severity = Severity::Warning;
        }
    }

    let tooltip = format!(
        "AI Usage — {} {}{}",
        provider_name(&status.primary.provider),
        status.primary.label,
        if stale { " (datos desactualizados)" } else { "" }
    );

    Topbar {
        icon: IconKind::for_provider(&status.primary.provider),
        label,
        severity,
        tooltip,
    }
}

/// "Actualizado hace 2 min" / "Datos desactualizados" line for the popup header.
pub fn updated_at_line(status: &StatusFile) -> String {
    let relative = crate::status::format_relative(&status.updated_at)
        .unwrap_or_else(|| "fecha desconocida".to_string());

    if status.stale || status.is_stale_by_age() {
        format!("Datos desactualizados · actualizado {relative}")
    } else {
        format!("Actualizado {relative}")
    }
}

/// `26%` / `--` for a raw percentage.
pub fn percent(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{}%", v.round() as i64),
        None => "--".to_string(),
    }
}

/// `reinicia <label>` / `reinicia --`.
pub fn reset_label(metric: &Option<UsageMetric>) -> String {
    metric
        .as_ref()
        .and_then(|m| m.reset_label.as_deref())
        .filter(|s| !s.is_empty())
        .unwrap_or("--")
        .to_string()
}

/// `used_percent` / `remaining_percent` formatted for a metric row,
/// e.g. `100% usado · 0% restante`.
pub fn usage_summary(metric: &Option<UsageMetric>) -> String {
    let used = metric.as_ref().and_then(|m| m.used_percent);
    let remaining = metric.as_ref().and_then(|m| m.remaining_percent);
    format!("{} usado · {} restante", percent(used), percent(remaining))
}

/// Remaining Codex credits, e.g. `128` or `--`.
pub fn credits(value: Option<i64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "--".to_string())
}

/// `0 / 5` for Claude's included daily routine runs.
pub fn routines(routines: &Option<DailyRoutines>) -> String {
    match routines {
        Some(DailyRoutines {
            used: Some(used),
            limit: Some(limit),
        }) => format!("{used} / {limit}"),
        _ => "--".to_string(),
    }
}

/// Human-readable hint shown under an error entry, with the command to fix it.
pub fn error_hint(error: &ErrorEntry) -> String {
    match error.code.as_str() {
        "login_required" | "profile_missing" => {
            format!("Ejecuta: ai-usage-auth {}", error.provider)
        }
        "page_changed" => {
            "El layout pudo haber cambiado. Revisa el collector con debug_text activado."
                .to_string()
        }
        "network_error" => "Sin conexión o la página no respondió a tiempo.".to_string(),
        _ => "Revisa el collector con: ai-usage-collect".to_string(),
    }
}
