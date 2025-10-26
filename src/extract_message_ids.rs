use crate::Args;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "Contents")]
    pub content: String,
    #[serde(rename = "Attachments")]
    pub attachments: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    channel_type: String,
    pub guild: Guild,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Guild {
    pub id: u64,
    pub name: String,
}

fn read_text_file(dir: &Path, file: &'static str) -> Result<String, String> {
    let mut path = dir.to_path_buf();
    path.push(file);
    std::fs::read_to_string(&path).map_err(|e| format!("Could not read file {path:?}: {e}"))
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
        let channel: Channel = serde_json::from_str(&channel)
            .map_err(|e| format!("Could not get JSON from channel metadata file {path:?}: {e}"))?;

        // Skip channels and guilds in the preserve list (blacklist).
        if args.preserve_list.contains(&channel.id)
            || args.preserve_list.contains(&channel.guild.id)
        {
            println!(
                "Skipping preserved channel {:?} ({}) in guild {:?} ({:?})",
                channel.name, channel.id, channel.guild.name, channel.guild.id
            );
            continue;
        }

        // The delete list (whitelist) is only active when it isn't empty.
        if !args.delete_list.is_empty()
            && !(args.delete_list.contains(&channel.id)
                || args.delete_list.contains(&channel.guild.id))
        {
            continue;
        }

        match channel.channel_type.as_str() {
            "GUILD_TEXT" if !args.delete_guilds => continue,
            "DM" if !args.delete_dms => continue,
            "GROUP_DM" if !args.delete_groups => continue,
            _ => {}
        }

        let messages: String = read_text_file(&path, "messages.json")?;
        let messages: Vec<Message> = serde_json::from_str(&messages)
            .map_err(|e| format!("Could not get JSON from messages file {path:?}: {e}"))?;

        channels.push((channel, messages));
    }

    Ok(channels)
}
