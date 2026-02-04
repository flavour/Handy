//! OpenClaw HTTP output
//!
//! Sends transcribed text to OpenClaw via the Chat Completions API.

use anyhow::{ensure, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

/// Submit text to OpenClaw via Chat Completions API.
///
/// # Arguments
/// * `base_url` - OpenClaw gateway URL (e.g., "http://127.0.0.1:18789")
/// * `token` - Gateway auth token
/// * `session_key` - Optional session key to route to a specific session
/// * `text` - The transcribed text to submit
///
/// # Returns
/// * `Ok(Some(reply))` if OpenClaw responded with content
/// * `Ok(None)` if OpenClaw returned no content
pub async fn submit(
    base_url: &str,
    token: &str,
    session_key: Option<&str>,
    text: &str,
) -> Result<Option<String>> {
    let base_url = base_url.trim().trim_end_matches('/');
    ensure!(!base_url.is_empty(), "OpenClaw base URL is empty");
    ensure!(!token.is_empty(), "OpenClaw token is empty");

    let url = format!("{}/v1/chat/completions", base_url);

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(120))
        .build()
        .context("build OpenClaw HTTP client")?;

    let request = ChatCompletionRequest {
        model: "openclaw:main".to_string(),
        messages: vec![Message {
            role: "user",
            content: text.to_string(),
        }],
        // Only use 'user' field if no explicit session key provided
        user: if session_key.is_some() {
            None
        } else {
            Some("handy-voice".to_string())
        },
    };

    let mut req_builder = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json");

    // Add session key header if provided
    if let Some(key) = session_key {
        if !key.is_empty() {
            req_builder = req_builder.header("x-openclaw-session-key", key);
        }
    }

    let response: ChatCompletionResponse = req_builder
        .json(&request)
        .send()
        .await
        .context("POST /v1/chat/completions")?
        .error_for_status()
        .context("Chat completions non-2xx")?
        .json()
        .await
        .context("Parse chat completion response")?;

    let reply = response.choices.first().map(|c| c.message.content.clone());
    Ok(reply)
}
