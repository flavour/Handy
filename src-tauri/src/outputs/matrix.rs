//! Matrix output
//!
//! Sends transcribed text to a Matrix room via the Client-Server API.

use anyhow::{bail, ensure, Context, Result};
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

#[derive(Deserialize)]
struct RoomAliasResponse {
    room_id: String,
}

#[derive(Deserialize, Debug)]
struct MatrixErrorResponse {
    errcode: Option<String>,
    error: Option<String>,
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
/// Uses: `PUT /_matrix/client/{v3|r0}/rooms/{roomId}/send/m.room.message/{txnId}`
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

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()
        .context("build Matrix HTTP client")?;

    let room_id_or_alias = room_id.trim();
    let join_target_enc =
        utf8_percent_encode(room_id_or_alias, PATH_SEGMENT_ENCODE_SET).to_string();

    let via_server = room_id_or_alias
        .rsplit_once(':')
        .map(|(_, server)| server)
        .filter(|s| !s.is_empty());

    async fn resolve_room_id(
        client: &Client,
        homeserver_url: &str,
        access_token: &str,
        room_id_or_alias: &str,
    ) -> Result<String> {
        if !room_id_or_alias.starts_with('#') {
            return Ok(room_id_or_alias.to_string());
        }

        let homeserver_url = homeserver_url.trim().trim_end_matches('/');
        let room_alias_enc =
            utf8_percent_encode(room_id_or_alias, PATH_SEGMENT_ENCODE_SET).to_string();

        let url = format!(
            "{}/_matrix/client/v3/directory/room/{}",
            homeserver_url, room_alias_enc
        );

        let resp = client
            .get(&url)
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .send()
            .await
            .context("GET Matrix room directory")?;

        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .context("Read Matrix directory response body")?
            .to_vec();

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            bail!(
                "Matrix room alias lookup failed (status={} body={})",
                status,
                body
            );
        }

        let resolved = serde_json::from_slice::<RoomAliasResponse>(&bytes)
            .context("Parse Matrix directory response")?;
        Ok(resolved.room_id)
    }

    let resolved_room_id =
        resolve_room_id(&client, homeserver_url, access_token, room_id_or_alias).await?;

    let room_id_enc = utf8_percent_encode(&resolved_room_id, PATH_SEGMENT_ENCODE_SET).to_string();
    // Use a stable transaction id across retries to avoid duplicate sends.
    let txn_id = txn_id();
    let txn_id_enc = utf8_percent_encode(&txn_id, PATH_SEGMENT_ENCODE_SET).to_string();

    let payload = RoomMessage {
        msgtype: "m.text",
        body: text.to_string(),
    };

    #[derive(Debug, Clone)]
    struct AttemptFailure {
        attempt: String,
        status: reqwest::StatusCode,
        bytes: Vec<u8>,
    }

    async fn attempt_send(
        client: &Client,
        version: &str,
        homeserver_url: &str,
        access_token: &str,
        room_id_enc: &str,
        txn_id_enc: &str,
        payload: &RoomMessage,
        use_query_access_token: bool,
    ) -> Result<(reqwest::StatusCode, Vec<u8>)> {
        let url = format!(
            "{}/_matrix/client/{}/rooms/{}/send/m.room.message/{}",
            homeserver_url, version, room_id_enc, txn_id_enc
        );

        let mut req = client
            .put(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(payload);

        if use_query_access_token {
            // Some homeserver/proxy stacks have issues with Authorization headers.
            // This fallback keeps tokens out of our logs (we never print the URL), but note
            // that servers may log query strings.
            req = req.query(&[("access_token", access_token)]);
        } else {
            req = req.bearer_auth(access_token);
        }

        let resp = req.send().await.with_context(|| {
            format!(
                "PUT Matrix send message ({}, auth={})",
                version,
                if use_query_access_token {
                    "query"
                } else {
                    "bearer"
                }
            )
        })?;

        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .context("Read Matrix response body")?
            .to_vec();

        Ok((status, bytes))
    }

    async fn attempt_join(
        client: &Client,
        version: &str,
        homeserver_url: &str,
        access_token: &str,
        join_target_enc: &str,
        use_query_access_token: bool,
        via_server: Option<&str>,
    ) -> Result<reqwest::StatusCode> {
        let url = format!(
            "{}/_matrix/client/{}/join/{}",
            homeserver_url, version, join_target_enc
        );

        let mut req = client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}));

        let mut query: Vec<(String, String)> = Vec::new();
        if use_query_access_token {
            query.push(("access_token".to_string(), access_token.to_string()));
        } else {
            req = req.bearer_auth(access_token);
        }
        if let Some(via) = via_server {
            query.push(("server_name".to_string(), via.to_string()));
        }
        if !query.is_empty() {
            req = req.query(&query);
        }

        let resp = req
            .send()
            .await
            .with_context(|| format!("POST Matrix join room ({})", version))?;

        Ok(resp.status())
    }

    async fn send_once(
        client: &Client,
        homeserver_url: &str,
        access_token: &str,
        room_id_enc: &str,
        txn_id_enc: &str,
        payload: &RoomMessage,
    ) -> Result<Result<String, AttemptFailure>> {
        let attempts = [("v3", false), ("v3", true), ("r0", false), ("r0", true)];

        let mut last_failure: Option<AttemptFailure> = None;

        for (version, use_query_access_token) in attempts {
            let (status, bytes) = attempt_send(
                client,
                version,
                homeserver_url,
                access_token,
                room_id_enc,
                txn_id_enc,
                payload,
                use_query_access_token,
            )
            .await?;

            if status.is_success() {
                let response = serde_json::from_slice::<SendEventResponse>(&bytes)
                    .context("Parse Matrix send response")?;
                return Ok(Ok(response.event_id));
            }

            last_failure = Some(AttemptFailure {
                attempt: format!(
                    "{}/{}",
                    version,
                    if use_query_access_token {
                        "query"
                    } else {
                        "bearer"
                    }
                ),
                status,
                bytes,
            });
        }

        Ok(Err(
            last_failure.expect("attempt loop always sets last_failure")
        ))
    }

    let first = send_once(
        &client,
        homeserver_url,
        access_token,
        &room_id_enc,
        &txn_id_enc,
        &payload,
    )
    .await?;

    match first {
        Ok(event_id) => return Ok(event_id),
        Err(failure) => {
            let body = String::from_utf8_lossy(&failure.bytes);
            let parsed = serde_json::from_slice::<MatrixErrorResponse>(&failure.bytes).ok();

            // Some homeserver stacks return a 500 with "room of unknown version" when the user is
            // invited but hasn't joined yet. Try accepting the invite, then retry send once.
            let should_try_join = body.contains("unknown version")
                || body.contains("membership `invite` is not `join`")
                || parsed
                    .as_ref()
                    .and_then(|e| e.error.as_deref())
                    .is_some_and(|e| {
                        e.contains("unknown version")
                            || e.contains("membership `invite` is not `join`")
                    });

            if should_try_join {
                let join_attempts = [("v3", false), ("v3", true), ("r0", false), ("r0", true)];
                for (version, use_query_access_token) in join_attempts {
                    let status = attempt_join(
                        &client,
                        version,
                        homeserver_url,
                        access_token,
                        &join_target_enc,
                        use_query_access_token,
                        via_server,
                    )
                    .await?;
                    if status.is_success() {
                        let retry = send_once(
                            &client,
                            homeserver_url,
                            access_token,
                            &room_id_enc,
                            &txn_id_enc,
                            &payload,
                        )
                        .await?;
                        match retry {
                            Ok(event_id) => return Ok(event_id),
                            Err(failure) => {
                                let body = String::from_utf8_lossy(&failure.bytes);
                                if let Ok(err) =
                                    serde_json::from_slice::<MatrixErrorResponse>(&failure.bytes)
                                {
                                    bail!(
                                        "Matrix send message failed (attempt={} status={} errcode={:?} error={:?} body={})",
                                        failure.attempt,
                                        failure.status,
                                        err.errcode,
                                        err.error,
                                        body
                                    );
                                }
                                bail!(
                                    "Matrix send message failed (attempt={} status={} body={})",
                                    failure.attempt,
                                    failure.status,
                                    body
                                );
                            }
                        }
                    }
                }
            }

            if let Some(err) = parsed {
                bail!(
                    "Matrix send message failed (attempt={} status={} errcode={:?} error={:?} body={})",
                    failure.attempt,
                    failure.status,
                    err.errcode,
                    err.error,
                    body
                );
            }

            bail!(
                "Matrix send message failed (attempt={} status={} body={})",
                failure.attempt,
                failure.status,
                body
            );
        }
    }
}
