use crate::CLIENT;
use colored::Colorize;
use reqwest::StatusCode;
use reqwest::blocking::Response;
use serde::Deserialize;
use serde_json::{Value, json};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

pub enum DiscordError {
    RateLimited(f64),
    Other(String),
}

impl From<String> for DiscordError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

const API_PREFIX: &str = "https://discord.com/api/v9";

fn get_url(channel_id: u64, message_id: u64) -> Result<Url, String> {
    let url: String = format!("{API_PREFIX}/channels/{channel_id}/messages/{message_id}");
    Url::from_str(&url).map_err(|e| format!("Could not deserialize URL {url:?}: {e}"))
}

fn handle_response(response: Response) -> Result<(), DiscordError> {
    let status: StatusCode = response.status();
    if status.is_success() {
        return Ok(());
    }

    let text: String = response
        .text()
        .map_err(|e| format!("Failed to read response text: {e}"))?;

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse JSON: {e}\nResponse body: {text}"))?;

    if status == StatusCode::TOO_MANY_REQUESTS {
        let retry_after: f64 = extract_retry_after(json);
        return Err(DiscordError::RateLimited(retry_after + 0.1));
    }

    let message = match json.get("message").and_then(|v| v.as_str()) {
        Some(msg) => msg.to_string(),
        None => json.to_string(),
    };

    Err(format!(
        "Discord responded with status code {} {}: {:?}",
        status.as_u16(),
        status.canonical_reason().unwrap_or("<unknown status>"),
        message,
    )
    .into())
}

fn extract_retry_after(json: Value) -> f64 {
    json.get("retry_after")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| {
            println!("{} Json: {json}", "Discord did not provide a `retry_after` field. Defaulting to 1 second.".cyan());
            1.0
        })
}

pub fn edit_message(
    token: &str,
    channel_id: u64,
    message_id: u64,
    content: &str,
    preserve_attachments: bool,
) -> Result<(), DiscordError> {
    let url = get_url(channel_id, message_id)?;

    let json = if preserve_attachments {
        json!({ "content": content })
    } else {
        json!({ "content": content, "attachments": [] })
    };

    let response: Response = CLIENT
        .patch(url)
        .header("Authorization", token)
        .json(&json)
        .send()
        .map_err(|e| format!("Failed to send request: {e}"))?;

    handle_response(response)
}

pub fn delete_message(token: &str, channel_id: u64, message_id: u64) -> Result<(), DiscordError> {
    let url = get_url(channel_id, message_id)?;

    let response: Response = CLIENT
        .delete(url)
        .header("Authorization", token)
        .send()
        .map_err(|e| format!("Failed to send request: {e}"))?;

    handle_response(response)
}

pub fn user_get_displayname(token: &str, user_id: u64) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Resp {
        user: User,
    }
    #[derive(Deserialize)]
    struct User {
        global_name: String,
    }

    let url = format!("{API_PREFIX}/users/{user_id}/profile");
    let url = Url::from_str(&url).map_err(|e| format!("Could not deserialize URL {url:?}: {e}"))?;

    loop {
        let response: Response = CLIENT
            .get(url.clone())
            .header("Authorization", token)
            .send()
            .map_err(|e| format!("Failed to send request: {e}"))?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let json: Value = response
                .json()
                .map_err(|e| format!("Could not get JSON from response: {e}"))?;
            let retry_after: f64 = extract_retry_after(json);
            sleep(Duration::from_secs_f64(retry_after));
            continue;
        }

        if !status.is_success() {
            let json: Value = response
                .json()
                .map_err(|e| format!("Could not get JSON from response: {e}"))?;

            let message = match json.get("message").and_then(|v| v.as_str()) {
                Some(msg) => msg.to_string(),
                None => json.to_string(),
            };

            return Err(format!(
                "Discord responded with status code {} {}: {:?}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("<unknown status>"),
                message,
            ));
        }

        let json: Resp = response
            .json()
            .map_err(|e| format!("Could not get JSON from response: {e}"))?;
        return Ok(json.user.global_name);
    }
}
