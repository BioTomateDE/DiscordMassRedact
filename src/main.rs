use crate::discord::{DiscordError, delete_message, edit_message};
use crate::extract::{Channel, Message, extract_messages};
use crate::redact::generate_redacted;
use crate::shakespeare::generate_shakespeare;
use chrono::{DateTime, NaiveDate, Utc};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use reqwest::blocking::Client;
use std::io::Read;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

mod discord;
mod emojis;
mod extract;
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

fn main() {
    let args = Args::parse();
    if let Err(error) = execute(args) {
        eprintln!("{}", error.bright_red());
    }
}

fn execute(mut args: Args) -> Result<(), String> {
    if let Some(before) = args.before
        && let Some(after) = args.after
    {
        if after < before {
            return Err(format!(
                "The `before` timestamp {before} is after the `after` timestamp {after}"
            ));
        }
    }

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
        "Got {} messages in {} channels.",
        message_count,
        channels.len()
    );
    println!("{}", text.bright_purple());

    println!("====== Press any key to start ======");
    std::io::stdin().read_exact(&mut [0]).unwrap();

    let mut failed_messages: Vec<u64> = vec![];

    for (channel, messages) in channels {
        for message in messages {
            loop {
                let resp = handle_message(&args, &channel, &message);
                if !resp.success {
                    failed_messages.push(message.id);
                }
                if !resp.retry {
                    break;
                }
            }
            // // Default sleeping duration between messages
            // sleep(Duration::from_millis(5500));
        }
    }

    println!("{}", "\nDone!".bright_green());
    if !failed_messages.is_empty() {
        println!("Some messages could not be redacted:");
        for id in failed_messages {
            println!("{id}");
        }
    }

    Ok(())
}

struct Response {
    success: bool,
    retry: bool,
}
impl Response {
    fn ok() -> Self {
        Self { success: true, retry: false }
    }
}

/// Returns `[true]` if the message handling was successful; continuing to the next message.
/// If `[false]`, a ratelimit or some other error has occurred; retry for a few more attempts.
fn handle_message(args: &Args, channel: &Channel, message: &Message) -> Response {
    if let Some(before) = args.before {
        if message.timestamp > before {
            return Response::ok();
        }
    }

    if let Some(after) = args.before {
        if message.timestamp < after {
            return Response::ok();
        }
    }

    if message.content.is_empty() && message.attachments.is_empty() {
        // Skipping empty message; most likely system message
        return Response::ok();
    }

    let channel_type = match channel.channel_type.as_str() {
        "GUILD_TEXT" => "Guild",
        "DM" => "DM",
        "GROUP_DM" => "Group DM",
        other => other,
    };

    let id_str = channel.id.to_string().dimmed();
    let channel_info = if let Some(name) = &channel.name {
        format!("{} ({})", format!("{name:?}").yellow(), id_str)
    } else {
        format!("with ID {}", id_str)
    };

    let guild_info = if let Some(guild) = &channel.guild {
        format!(
            " in guild {:?} ({})",
            format!("{:?}", guild.name).yellow(),
            guild.id.to_string().dimmed(),
        )
    } else {
        String::new()
    };


    println!(
        "Redacting message with ID {} in {channel_type} channel {channel_info}{guild_info}.",
        message.id.to_string().dimmed(),
    );

    let result = match args.mode {
        DeletionMode::Delete => delete_message(&args.token, channel.id, message.id),
        DeletionMode::RandomWords => {
            let content = generate_redacted();
            edit_message(
                &args.token,
                channel.id,
                message.id,
                &content,
                args.preserve_attachments,
            )
        }
        DeletionMode::Shakespeare => {
            let content = generate_shakespeare(message.content.len());
            edit_message(
                &args.token,
                channel.id,
                message.id,
                &content,
                args.preserve_attachments,
            )
        }
    };

    let Err(error) = result else {
        println!("{}", format!("Redacted message {:?}", message.content).green());
        return Response::ok()
    };

    match error {
        DiscordError::RateLimited(retry_after) => {
            // println!("{}", format!("Too many requests! Retrying after {retry_after:.2} seconds.").yellow());
            sleep(Duration::from_secs_f64(retry_after));
            Response { success: false, retry: true }
        }
        DiscordError::Other(message) => {
            println!("{}", message.red());
            // Do not bother retrying for these errors.
            Response { success: false, retry: false }
        }
    }
}
