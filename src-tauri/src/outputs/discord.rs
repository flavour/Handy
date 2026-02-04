//! Discord HTTP output
//!
//! Sends transcribed text to a Discord channel via the REST API.

use anyhow::{anyhow, ensure, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct CreateMessage {
    content: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    id: String,
}

/// Send text to a Discord channel via REST API.
pub async fn submit(bot_token: &str, channel_id: &str, text: &str) -> Result<String> {
    let bot_token = bot_token.trim();
    let channel_id = channel_id.trim();

    ensure!(!bot_token.is_empty(), "Discord bot token is empty");
    ensure!(!channel_id.is_empty(), "Discord channel ID is empty");
    ensure!(!text.is_empty(), "Message text is empty");

    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        channel_id
    );

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .build()
        .context("build Discord HTTP client")?;

    let payload = CreateMessage {
        content: text.to_string(),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bot {}", bot_token))
        .header("Content-Type", "application/json")
        .header("User-Agent", "Handy/1.0 (https://github.com/flavour/Handy)")
        .json(&payload)
        .send()
        .await
        .context("POST to Discord API")?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        let mut meta = Vec::new();
        for (label, header) in [
            ("retry-after", "retry-after"),
            ("x-ratelimit-limit", "x-ratelimit-limit"),
            ("x-ratelimit-remaining", "x-ratelimit-remaining"),
            ("x-ratelimit-reset-after", "x-ratelimit-reset-after"),
            ("x-ratelimit-scope", "x-ratelimit-scope"),
        ] {
            if let Some(v) = headers.get(header).and_then(|v| v.to_str().ok()) {
                meta.push(format!("{}={}", label, v));
            }
        }
        let meta = if meta.is_empty() {
            String::new()
        } else {
            format!(" ({})", meta.join(", "))
        };

        let body = if body.len() > 4000 {
            format!("{}… (truncated, {} bytes)", &body[..4000], body.len())
        } else {
            body
        };

        return Err(anyhow!(
            "Discord API non-2xx (status={}){}: {}",
            status,
            meta,
            body
        ));
    }

    let response: MessageResponse =
        serde_json::from_str(&body).context("Parse Discord message response")?;

    Ok(response.id)
}
