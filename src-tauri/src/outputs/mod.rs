//! Output module dispatcher
//!
//! Routes transcribed text to the configured output destination.

pub mod discord;
pub mod matrix;
pub mod openclaw;
pub mod opencode;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Output mode for transcribed text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Type)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Paste to active application (default)
    #[default]
    Paste,
    /// Send to OpenCode HTTP API
    OpenCode,
    /// Send to OpenClaw Chat Completions API
    OpenClaw,
    /// Send to Discord channel via REST API
    Discord,
    /// Send to Matrix room via Client-Server API
    Matrix,
}

impl std::fmt::Display for OutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputMode::Paste => write!(f, "paste"),
            OutputMode::OpenCode => write!(f, "opencode"),
            OutputMode::OpenClaw => write!(f, "openclaw"),
            OutputMode::Discord => write!(f, "discord"),
            OutputMode::Matrix => write!(f, "matrix"),
        }
    }
}
