//! Matrix output
//!
//! Sends transcribed text to a Matrix room via the Client-Server API.

use anyhow::{ensure, Context, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Encode everything except unreserved characters.
// https://www.rfc-editor.org/rfc/rfc3986#section-2.3
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ') // space
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[derive(Serialize)]
struct RoomMessage {
    msgtype: &'static str,
    body: String,
}

#[derive(Deserialize)]
struct SendEventResponse {
    event_id: String,
}

fn txn_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("handy-{}", ms)
}

/// Send text to a Matrix room.
///
/// Uses: `POST /_matrix/client/v3/rooms/{roomId}/send/m.room.message/{txnId}`
pub async fn submit(
    homeserver_url: &str,
    access_token: &str,
    room_id: &str,
    text: &str,
) -> Result<String> {
    let homeserver_url = homeserver_url.trim().trim_end_matches('/');
    let access_token = access_token.trim();
    let room_id = room_id.trim();

    ensure!(!homeserver_url.is_empty(), "Matrix homeserver URL is empty");
    ensure!(!access_token.is_empty(), "Matrix access token is empty");
    ensure!(!room_id.is_empty(), "Matrix room id is empty");
    ensure!(!text.is_empty(), "Message text is empty");

    let room_id_enc = utf8_percent_encode(room_id, PATH_SEGMENT_ENCODE_SET).to_string();
    let txn_id_enc = utf8_percent_encode(&txn_id(), PATH_SEGMENT_ENCODE_SET).to_string();

    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
        homeserver_url, room_id_enc, txn_id_enc
    );

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()
        .context("build Matrix HTTP client")?;

    let payload = RoomMessage {
        msgtype: "m.text",
        body: text.to_string(),
    };

    let response = client
        .post(&url)
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("POST Matrix send message")?
        .error_for_status()
        .context("Matrix send message non-2xx")?
        .json::<SendEventResponse>()
        .await
        .context("Parse Matrix send response")?;

    Ok(response.event_id)
}
