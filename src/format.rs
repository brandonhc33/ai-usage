// SPDX-License-Identifier: MIT

//! Turns a [`StatusState`] into the strings and colors the UI renders.
//!
//! Keeping this logic out of `app.rs` makes the view function a thin layer
//! over plain data, and keeps the percentage/threshold rules in one place.

use crate::icons::IconKind;
use crate::status::{DailyRoutines, ErrorEntry, StatusFile, StatusState, UsageMetric};
use cosmic::iced::Color;

/// Mirrors the defaults in `docs/templates/config.example.json`.
pub const WARNING_USED_PERCENT: f64 = 51.0;
pub const CRITICAL_USED_PERCENT: f64 = 76.0;

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
            Severity::Normal => Some(Color::from_rgb(0.18, 0.75, 0.35)),
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

/// Computes one topbar icon/label/severity per enabled provider, so the
/// topbar can show Codex and Claude side by side instead of a single
/// "primary" provider.
pub fn topbar_entries(state: &StatusState) -> Vec<Topbar> {
    match state {
        StatusState::Missing => vec![Topbar {
            icon: IconKind::Ai,
            label: "--".to_string(),
            severity: Severity::Muted,
            tooltip: "AI Usage — ejecuta ai-usage-collect".to_string(),
        }],
        StatusState::Invalid => vec![Topbar {
            icon: IconKind::Ai,
            label: "!".to_string(),
            severity: Severity::Critical,
            tooltip: "AI Usage — status.json inválido".to_string(),
        }],
        StatusState::Loaded(status) => {
            let stale = status.stale || status.is_stale_by_age();
            let mut entries = Vec::new();

            if let Some(codex) = &status.codex {
                entries.push(provider_topbar(
                    IconKind::Codex,
                    "Codex",
                    "codex",
                    codex.ok,
                    codex.error_code.as_deref(),
                    codex.five_hour.as_ref(),
                    stale,
                ));
            }

            if let Some(claude) = &status.claude {
                entries.push(provider_topbar(
                    IconKind::Claude,
                    "Claude",
                    "claude",
                    claude.ok,
                    claude.error_code.as_deref(),
                    claude.session.as_ref(),
                    stale,
                ));
            }

            if entries.is_empty() {
                entries.push(Topbar {
                    icon: IconKind::Ai,
                    label: "--".to_string(),
                    severity: Severity::Muted,
                    tooltip: "AI Usage — sin datos disponibles".to_string(),
                });
            }

            entries
        }
    }
}

/// Topbar icon/label/severity for a single provider.
fn provider_topbar(
    icon: IconKind,
    name: &str,
    provider: &str,
    ok: bool,
    error_code: Option<&str>,
    metric: Option<&UsageMetric>,
    stale: bool,
) -> Topbar {
    if matches!(error_code, Some("login_required" | "profile_missing")) {
        return Topbar {
            icon,
            label: "login".to_string(),
            severity: Severity::LoginRequired,
            tooltip: format!("AI Usage — {name} requiere iniciar sesión (ai-usage-auth {provider})"),
        };
    }

    let used_percent = metric.and_then(|m| m.used_percent);

    if !ok || used_percent.is_none() {
        return Topbar {
            icon,
            label: "--".to_string(),
            severity: Severity::Muted,
            tooltip: format!("AI Usage — {name} sin datos disponibles"),
        };
    }

    let mut severity = severity_for_percent(used_percent);
    let mut label = percent(used_percent);

    if stale {
        label.push('!');
        if severity == Severity::Normal {
            severity = Severity::Warning;
        }
    }

    let tooltip = format!(
        "AI Usage — {name} {label}{}",
        if stale { " (datos desactualizados)" } else { " usado" }
    );

    Topbar {
        icon,
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
        "profile_busy" => {
            "Ya hay una sesión de Chrome en uso para este proveedor. Esperá a que termine."
                .to_string()
        }
        _ => "Revisa el collector con: ai-usage-collect".to_string(),
    }
}
