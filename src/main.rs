use crate::discord::{delete_message, edit_message, DiscordError};
use crate::extract_message_ids::{Channel, Message, extract_messages};
use crate::redact::generate_redacted;
use crate::shakespeare::generate_shakespeare;
use chrono::{DateTime, NaiveDate, Utc};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use reqwest::blocking::Client;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

mod discord;
mod emojis;
mod extract_message_ids;
mod redact;
mod shakespeare;
mod user_agents;
mod wordlist;

#[derive(Debug, Clone, ValueEnum)]
enum DeletionMode {
    Delete,
    RandomWords,
    Shakespeare,
}

/// Discord Selfbot mass message redaction tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Your personal Discord Token.
    token: String,

    /// The directory of your Discord data export
    export_dir: PathBuf,

    /// How messages should be deleted/edited
    mode: DeletionMode,

    /// Whether to preserve message attachments like images or videos. Default: false
    #[arg(short, long, default_value_t = false)]
    preserve_attachments: bool,

    /// Whether to delete messages from Guilds (Servers). Default: true
    #[arg(long, default_value_t = true)]
    delete_guilds: bool,

    /// Whether to delete messages from DMs (Direct Messages). Default: true
    #[arg(long, default_value_t = true)]
    delete_dms: bool,

    /// Whether to delete messages from Groups DMs. Default: true
    #[arg(long, default_value_t = true)]
    delete_groups: bool,

    /// Only delete messages after this date (YYYY-MM-DD format)
    #[arg(short, long, value_parser = parse_date)]
    after: Option<DateTime<Utc>>,

    /// Only delete messages before this date (YYYY-MM-DD format)
    #[arg(short, long, value_parser = parse_date)]
    before: Option<DateTime<Utc>>,

    /// A comma separated list of Channel or Guild IDs where messages shouldn't be deleted.
    #[arg(long, value_delimiter = ',')]
    preserve_list: Vec<u64>,

    /// A path to a file containing a newline separated blacklist (see --preserve-list).
    #[arg(long)]
    preserve_list_file: Option<PathBuf>,

    /// A comma separated list of Channel or Guild IDs where messages should be deleted.
    /// If unset, ALL messages are deleted.
    #[arg(long, value_delimiter = ',')]
    delete_list: Vec<u64>,

    /// A path to a file containing a newline separated whitelist (see --delete-list).
    #[arg(long)]
    delete_list_file: Option<PathBuf>,
}

fn parse_date(s: &str) -> Result<DateTime<Utc>, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc())
        .map_err(|e| format!("Invalid date: '{}'. Use YYYY-MM-DD format: {}", s, e))
}

fn append_id_list(list: &mut Vec<u64>, file_content: String) -> Result<(), String> {
    for line in file_content.trim().lines() {
        let id: u64 = line
            .trim()
            .parse()
            .map_err(|_| format!("Invalid ID {line:?}"))?;
        list.push(id);
    }
    Ok(())
}

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(user_agents::get_random_user_agent())
        .build()
        .expect("Failed to create reqwest client")
});

fn main() -> Result<(), String> {
    let mut args = Args::parse();
    println!("{args:?}");
    // std::process::exit(0);

    if let Some(file) = &args.preserve_list_file {
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Could not read preserve list file: {e}"))?;
        append_id_list(&mut args.preserve_list, content)?;
    }

    if let Some(file) = &args.delete_list_file {
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Could not read delete list file: {e}"))?;
        append_id_list(&mut args.delete_list, content)?;
    }

    let channels = extract_messages(&args)?;
    let message_count: usize = channels.iter().map(|(_, x)| x.len()).sum();

    let text = format!(
        "Got {} messages in {} channels.\n",
        message_count,
        channels.len()
    );
    println!("{}", text.bright_purple());

    for (channel, messages) in channels {
        for message in messages {
            loop {
                sleep(Duration::from_millis(5500));
                handle_message(&args, &channel, &message);
            }
        }
    }

    println!("{}", "\nDone!".bright_green());

    Ok(())
}

/// Returns `[true]` if the message handling was successful; continuing to the next message.
/// If `[false]`, a ratelimit or some other error has occurred; retry for ~5 more attempts.
fn handle_message(args: &Args, channel: &Channel, message: &Message) -> bool {
    println!(
        "Redacting message with ID {} in channel {:?} ({}) in guild {:?} ({}).",
        message.id.to_string().dimmed(),
        channel.name.yellow(),
        channel.id.to_string().dimmed(),
        channel.guild.name.yellow(),
        channel.guild.id.to_string().dimmed(),
    );

    let result = match args.mode {
        DeletionMode::Delete => {
            delete_message(&args.token, channel.id, message.id)
        }
        DeletionMode::RandomWords => {
            let content = generate_redacted();
            edit_message(&args.token, channel.id, message.id, &content, args.preserve_attachments)
        }
        DeletionMode::Shakespeare => {
            let content = generate_shakespeare(message.content.len());
            edit_message(&args.token, channel.id, message.id, &content, args.preserve_attachments)
        }
    };

    let Err(error) = result else {
        return true
    };

    match error {
        DiscordError::RateLimited(retry_after) => {
            println!("Too many requests! Retrying after {retry_after:.2} seconds.");
            sleep(Duration::from_secs_f64(retry_after));
        }
        DiscordError::Other(message) => {
            println!("{}", message.red());
        }
    }

    false
}
