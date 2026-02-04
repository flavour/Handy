//! OpenCode HTTP output
//!
//! Sends transcribed text to the OpenCode TUI via HTTP API.

use anyhow::{ensure, Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct AppendPrompt<'a> {
    text: &'a str,
}

/// Submit text to OpenCode via HTTP API.
///
/// # Arguments
/// * `base_url` - OpenCode base URL (e.g., "http://127.0.0.1:4096")
/// * `text` - The transcribed text to submit
pub async fn submit(base_url: &str, text: &str) -> Result<()> {
    let base_url = base_url.trim();
    ensure!(!base_url.is_empty(), "OpenCode base URL is empty");

    let base_url = base_url.trim_end_matches('/');
    let append_url = format!("{}/tui/append-prompt", base_url);
    let submit_url = format!("{}/tui/submit-prompt", base_url);

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(1))
        .timeout(Duration::from_secs(2))
        .build()
        .context("build OpenCode HTTP client")?;

    let append_ok: bool = client
        .post(&append_url)
        .json(&AppendPrompt { text })
        .send()
        .await
        .context("POST /tui/append-prompt")?
        .error_for_status()
        .context("/tui/append-prompt non-2xx")?
        .json()
        .await
        .context("/tui/append-prompt parse bool")?;

    let submit_ok: bool = client
        .post(&submit_url)
        .send()
        .await
        .context("POST /tui/submit-prompt")?
        .error_for_status()
        .context("/tui/submit-prompt non-2xx")?
        .json()
        .await
        .context("/tui/submit-prompt parse bool")?;

    ensure!(append_ok && submit_ok, "OpenCode returned false");
    Ok(())
}
