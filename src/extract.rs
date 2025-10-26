use crate::Args;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer};
use serde_with::{DisplayFromStr, serde_as};
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(
        rename = "Timestamp",
        deserialize_with = "deserialize_discord_timestamp"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "Contents")]
    pub content: String,
    #[serde(rename = "Attachments")]
    pub attachments: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub guild: Option<Guild>,
    pub recipients: Option<Vec<String>>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Guild {
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    pub name: String,
}

fn deserialize_discord_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .map_err(serde::de::Error::custom)
}

fn read_text_file(dir: &Path, file: &'static str) -> Result<String, String> {
    let mut path = dir.to_path_buf();
    path.push(file);
    std::fs::read_to_string(&path).map_err(|e| format!("Could not read file {path:?}: {e}"))
}

fn is_blacklisted(preserve_list: &Vec<u64>, channel: &Channel) -> bool {
    if preserve_list.contains(&channel.id) {
        return true;
    }
    if let Some(guild) = &channel.guild {
        if preserve_list.contains(&guild.id) {
            return true;
        }
    }
    false
}

fn is_whitelisted(delete_list: &Vec<u64>, channel: &Channel) -> bool {
    // If the delete list is empty, messages in all channels are deleted.
    if delete_list.is_empty() {
        return true;
    }
    if delete_list.contains(&channel.id) {
        return true;
    }
    if let Some(guild) = &channel.guild {
        if delete_list.contains(&guild.id) {
            return true;
        }
    }
    false
}

pub fn extract_messages(args: &Args) -> Result<Vec<(Channel, Vec<Message>)>, String> {
    let mut channels: Vec<(Channel, Vec<Message>)> = vec![];

    // Get `messages` subfolder
    let mut directory = args.export_dir.to_path_buf();
    directory.push("messages");

    let entries: ReadDir = directory
        .read_dir()
        .map_err(|e| format!("Could not get children of directory {directory:?}: {e}"))?;

    for entry in entries {
        let entry: DirEntry =
            entry.map_err(|e| format!("Could not get child of directory: {e}"))?;
        let path: PathBuf = entry.path();
        if !path.is_dir() {
            continue;
        }

        let channel: String = read_text_file(&path, "channel.json")?;
        let channel: Channel = serde_json::from_str(&channel).map_err(|e| {
            format!("Could not get JSON from channel metadata file in {path:?}: {e}")
        })?;

        match channel.channel_type.as_str() {
            "GUILD_TEXT" if !args.delete_guilds => continue,
            "DM" if !args.delete_dms => continue,
            "GROUP_DM" if !args.delete_groups => continue,
            _ => {}
        }

        if !is_whitelisted(&args.delete_list, &channel) {
            continue;
        }

        // Skip channels and guilds in the preserve list (blacklist).
        if is_blacklisted(&args.preserve_list, &channel) {
            let guild_info = channel
                .guild
                .map(|g| format!(" in guild {:?} ({})", g.name, g.id))
                .unwrap_or_default();
            println!(
                "Skipping preserved channel {:?} ({}){guild_info}",
                channel.name, channel.id,
            );
            continue;
        }

        let messages: String = read_text_file(&path, "messages.json")?;
        let messages: Vec<Message> = serde_json::from_str(&messages)
            .map_err(|e| format!("Could not get JSON from messages file in {path:?}: {e}"))?;

        channels.push((channel, messages));
    }

    Ok(channels)
}
