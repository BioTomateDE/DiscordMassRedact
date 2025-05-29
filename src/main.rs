use std::collections::HashMap;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use colored::Colorize;
use reqwest::blocking::Client;
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
    println!("{}", format!("Got {} messages in {} channels.\n", count_messages(&channels), channels.len()).bright_purple());

    let client: Client = Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Could not create reqwest client: {e}"))?;

    for (channel_id, message_ids) in &channels {
        for message_id in message_ids {
            loop {
                sleep(Duration::from_millis(5500));
                println!("Redacting message {} in channel {}.", message_id.to_string().yellow(), channel_id.to_string().yellow());
                let redacted_message: String = generate_redacted();
                match edit_message(&client, &token, *channel_id, *message_id, &redacted_message) {
                    Ok(_) => break,
                    Err((e, retry_after)) => {
                        println!("{}", format!("Error while editing message: {e}").red());
                        if retry_after == 0.0 { break }
                        sleep(Duration::from_secs_f64(retry_after * 1.2));
                    }
                };
            }
        }
    }
    
    println!("{}", "\nDone!".bright_green());

    Ok(())
}


