use crate::edit_message::edit_message;
use crate::extract_message_ids::{Message, extract_messages};
use crate::redact::{generate_redacted, generate_shakespeare};
use colored::Colorize;
use rand::random_range;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

mod edit_message;
mod emojis;
mod extract_message_ids;
mod redact;
mod wordlist;

fn count_messages(channels: &HashMap<u64, Vec<Message>>) -> usize {
    let mut count: usize = 0;
    for channel_messages in channels.values() {
        count += channel_messages.len();
    }
    count
}

fn main() -> Result<(), String> {
    dotenv::dotenv().map_err(|e| format!("Could not initialize environment variables: {e}"))?;
    let messages_directory: PathBuf = PathBuf::from(
        std::env::var("DISCORD_MESSAGES_DIR")
            .map_err(|_| "Environment variable DISCORD_MESSAGES_DIR not set")?,
    );
    let token: String =
        std::env::var("DISCORD_TOKEN").map_err(|_| "Environment variable DISCORD_TOKEN not set")?;

    let channels: HashMap<u64, Vec<Message>> = extract_messages(&messages_directory)?;
    println!(
        "{}",
        format!(
            "Got {} messages in {} channels.\n",
            count_messages(&channels),
            channels.len()
        )
        .bright_purple()
    );

    let client: Client = Client::builder()
            .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| format!("Could not create reqwest client: {e}"))?;

    let shakespeare_mode = std::env::args().nth(1) == Some(String::from("shakespeare"));

    const QUOTES: &[u8] = include_bytes!("shakespeare.txt");

    for (channel_id, messages) in channels {
        for message in messages {
            loop {
                sleep(Duration::from_millis(5500));
                println!(
                    "Redacting message {} in channel {}.",
                    message.ID.to_string().yellow(),
                    channel_id.to_string().yellow()
                );
                let redacted_message: String;

                if shakespeare_mode {
                    println!("Running shakespeare_mode");
                    redacted_message = generate_shakespeare(message.Contents.len(), QUOTES);
                } else {
                    redacted_message = generate_redacted();
                }

                match edit_message(&client, &token, channel_id, message.ID, &redacted_message) {
                    Ok(_) => break,
                    Err((e, retry_after)) => {
                        println!("{}", format!("Error while editing message: {e}").red());
                        if retry_after == 0.0 {
                            break;
                        }
                        sleep(Duration::from_secs_f64(retry_after * 1.2));
                    }
                };
            }
        }
    }

    println!("{}", "\nDone!".bright_green());

    Ok(())
}
