use std::collections::HashMap;
use std::path::PathBuf;
use crate::edit_message::edit_message;
use crate::extract_message_ids::extract_message_ids;
use crate::redact::generate_redacted;

mod wordlist;
mod channels;
mod emojis;
mod redact;
mod extract_message_ids;
mod edit_message;

fn count_messages(channels: &HashMap<u64, Vec<u64>>) -> usize {
    let mut count: usize = 0;
    for channel_messages in channels.values() {
        count += channel_messages.len();
    }
    count
}

fn main() -> Result<(), String> {
    dotenv::dotenv().map_err(|e| format!("Could not initialize environment variables: {e}"))?;
    let messages_directory: PathBuf = PathBuf::from(std::env::var("DISCORD_MESSAGES_DIR").map_err(|_| "Environment variable DISCORD_MESSAGES_DIR not set")?);
    let token: String = std::env::var("DISCORD_TOKEN").map_err(|_| "Environment variable DISCORD_TOKEN not set")?;

    let channels: HashMap<u64, Vec<u64>> = extract_message_ids(&messages_directory)?;
    println!("Got {} messages in {} channels.", count_messages(&channels), channels.len());
    
    
    for (channel_id, message_ids) in &channels {
        for message_id in message_ids {
            let redacted_message: String = generate_redacted();
            edit_message(&token, *channel_id, *message_id, &redacted_message)?;
        }
    }

    
    Ok(())
}

