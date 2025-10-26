use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{DirEntry, ReadDir};
use std::path::PathBuf;
use std::str::Chars;

#[derive(Deserialize)]
#[allow(non_snake_case)] // the field names have to match the one's in discord's json
pub struct Message {
    pub ID: u64,
    pub Timestamp: String,
    pub Contents: String,
}

pub fn extract_messages(directory: &PathBuf) -> Result<HashMap<u64, Vec<Message>>, String> {
    const FOLDER_NAME_ERR_MSG: &str =
        "Names of subfolders should be formatted like `c136132671724532136`!";

    let entries: ReadDir = directory
        .read_dir()
        .map_err(|e| format!("Could not get children of directory {directory:?}: {e}"))?;

    let mut channels: HashMap<u64, Vec<Message>> = HashMap::new();

    for entry in entries {
        let entry: DirEntry =
            entry.map_err(|e| format!("Could not get child of directory: {e}"))?;
        let mut path: PathBuf = entry.path();
        if !path.is_dir() {
            continue;
        }

        path.push("messages.json");
        let file_content: String = std::fs::read_to_string(&path)
            .map_err(|e| format!("Could not read file {path:?}: {e}"))?;
        let messages: Vec<Message> = serde_json::from_str(&file_content)
            .map_err(|e| format!("Could not get JSON from messages file {path:?}: {e}"))?;

        let folder_name = entry.file_name();
        let folder_name: &str = folder_name.to_str().ok_or_else(|| {
            format!(
                "Could not convert file name into UTF-8 string: {:?}",
                folder_name
            )
        })?;
        let mut folder_name_chars: Chars = folder_name.chars();
        if folder_name_chars.next().unwrap() != 'c' {
            return Err(format!(
                "{FOLDER_NAME_ERR_MSG} Folder name does not start with 'c': {folder_name}"
            ));
        }
        let folder_name: &str = folder_name_chars.as_str(); // remove the leading c
        let channel_id: u64 = folder_name.parse()
            .map_err(|_| format!("{FOLDER_NAME_ERR_MSG} Folder name does start with 'c' but is not followed by a number: {folder_name}"))?;

        channels.insert(channel_id, messages);
    }

    Ok(channels)
}
