// SPDX-License-Identifier: MIT

//! Embedded topbar icons. Each provider keeps its own color so the topbar
//! icon already communicates "which provider" without extra UI.

use cosmic::widget;

const AI_ICON: &[u8] = include_bytes!("../resources/icons/ai-orb.svg");
const CODEX_ICON: &[u8] = include_bytes!("../resources/icons/codex-orb.svg");
const CLAUDE_ICON: &[u8] = include_bytes!("../resources/icons/claude-orb.svg");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    /// Generic AI icon: used when there is no primary provider, no data, or
    /// the status file is missing/invalid.
    Ai,
    Codex,
    Claude,
}

impl IconKind {
    pub fn handle(self) -> widget::icon::Handle {
        let bytes = match self {
            IconKind::Ai => AI_ICON,
            IconKind::Codex => CODEX_ICON,
            IconKind::Claude => CLAUDE_ICON,
        };
        widget::icon::from_svg_bytes(bytes)
    }
}
