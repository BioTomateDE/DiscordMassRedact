use serde_json::json;
use std::str::FromStr;
use url::Url;
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;

pub fn edit_message(client: &Client, token: &str, channel_id: u64, message_id: u64, content: &str) -> Result<(), String> {
    let url: Url = Url::from_str(&format!("https://discord.com/api/v9/channels/{channel_id}/messages/{message_id}"))
        .map_err(|e| format!("Could not generate URL for channel id \"{channel_id}\" and message id \"{message_id}\": {e}"))?;

    let response: Response = client
        .patch(url)
        .header("Authorization", token)
        .json(&json!({
            "content": content,
            "attachments": [],
        }))
        .send()
        .map_err(|e| format!("Failed to send request: {e}"))?;
    
    let status: StatusCode = response.status();
    if !status.is_success() {
        let text: String = response.text().map_err(|e| format!("Failed to get text response: {e}"))?;
        return Err(format!("Discord responded with status code {} - {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("<unknown status>"), text))
    }
    
    Ok(())
}

