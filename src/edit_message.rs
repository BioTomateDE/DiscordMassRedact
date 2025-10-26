use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde_json::{Value, json};
use std::str::FromStr;
use url::Url;

pub fn edit_message(
    client: &Client,
    token: &str,
    channel_id: u64,
    message_id: u64,
    content: &str,
) -> Result<(), (String, f64)> {
    let url: Url = Url::from_str(&format!("https://discord.com/api/v9/channels/{channel_id}/messages/{message_id}"))
        .map_err(|e| (format!("Could not generate URL for channel id \"{channel_id}\" and message id \"{message_id}\": {e}"), 0.0))?;

    let response: Response = client
        .patch(url)
        .header("Authorization", token)
        .json(&json!({
            "content": content,
            "attachments": [],
        }))
        .send()
        .map_err(|e| (format!("Failed to send request: {e}"), 0.0))?;

    let status: StatusCode = response.status();
    if !status.is_success() {
        let json: Value = response
            .json()
            .map_err(|e| (format!("Failed to get JSON response: {e}"), 0.0))?;
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after: f64 = json
                .as_object()
                .and_then(|i| i.get("retry_after"))
                .and_then(|i| i.as_f64())
                .unwrap_or(12.0);
            return Err((
                format!("Too many requests! Retrying after {retry_after} seconds."),
                retry_after,
            ));
        }
        return Err((
            format!(
                "Discord responded with status code {} - {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("<unknown status>"),
                json
            ),
            0.0,
        ));
    }

    Ok(())
}
