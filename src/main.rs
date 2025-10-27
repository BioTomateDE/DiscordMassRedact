use crate::cli::{Args, DeletionMode};
use crate::discord::{DiscordError, delete_message, edit_message, user_get_displayname};
use crate::extract::{Channel, Message, extract_messages};
use crate::shakespeare::generate_shakespeare;
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::io::Read;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

mod cli;
mod discord;
mod extract;
mod shakespeare;
mod user_agents;

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
    if let Err(error) = run(args) {
        eprintln!("{}", error.bright_red());
    }
}

fn run(mut args: Args) -> Result<(), String> {
    if let Some(before) = args.before
        && let Some(after) = args.after
    {
        if after > before {
            return Err(format!(
                "The `after` timestamp {after} is after the `before` timestamp {before}"
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

    if !args.preserve_list.is_empty() && !args.delete_list.is_empty() {
        return Err("Cannot use both preserve list and delete list!".to_string());
    }

    let channels = extract_messages(&args)?;
    let message_count: usize = channels.iter().map(|(_, x)| x.len()).sum();

    let text = format!(
        "Got {} messages in {} channels.",
        message_count,
        channels.len()
    );
    println!("{}", text.bright_purple());

    println!("====== Press Enter to start ======");
    std::io::stdin().read(&mut []).unwrap();

    let mut displayname_cache = HashMap::new();
    let mut failed_messages: Vec<Message> = vec![];
    let bar = ProgressBar::new(message_count as u64);
    bar.set_style(ProgressStyle::with_template("[{eta}] {wide_bar} {pos}/{len} ({percent_precise}%)").unwrap());

    for (channel, messages) in channels {
        for message in messages {
            loop {
                let resp = handle_message(&args, &bar, &mut displayname_cache, &channel, &message);
                if !resp.retry {
                    if !resp.success {
                        failed_messages.push(message);
                    }
                    break;
                }
            }
            bar.inc(1);
        }
    }

    bar.finish();
    println!("{}", "\nDone!".bright_green());
    if !failed_messages.is_empty() {
        println!("Some messages could not be redacted:");
        for message in failed_messages {
            println!("{} - {:?}", message.id, message.content);
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
        Self {
            success: true,
            retry: false,
        }
    }
}

/// Returns `[true]` if the message handling was successful; continuing to the next message.
/// If `[false]`, a ratelimit or some other error has occurred; retry for a few more attempts.
fn handle_message(
    args: &Args,
    bar: &ProgressBar,
    displayname_cache: &mut HashMap<u64, String>,
    channel: &Channel,
    message: &Message,
) -> Response {
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

    let recipients = if let Some(recipients) = &channel.recipients {
        format!(
            " with recipients {:?}",
            recipients
                .iter()
                .map(|id| id.parse::<u64>().unwrap_or_else(|_| {
                    bar.println(format!("Invalid user id {id:?}").red().to_string());
                    0
                }))
                .map(|id| get_displayname(&args.token, displayname_cache, id))
                .collect::<Vec<_>>()
        )
    } else {
        String::new()
    };

    let guild_info = if let Some(guild) = &channel.guild {
        format!(
            " in guild {} ({})",
            format!("{:?}", guild.name).yellow(),
            guild.id.to_string().dimmed(),
        )
    } else {
        String::new()
    };

    bar.println(format!(
        "Redacting message with ID {} in {channel_type} channel {channel_info}{recipients}{guild_info}.",
        message.id.to_string().dimmed(),
    ));

    let result = match args.mode {
        DeletionMode::Delete => delete_message(&args.token, channel.id, message.id),
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
        bar.println(
            format!("Redacted message {:?}", message.content)
                .green()
                .to_string(),
        );
        return Response::ok();
    };

    match error {
        DiscordError::RateLimited(retry_after) => {
            bar.println(format!("Retrying after {retry_after:.2}s").yellow().to_string());
            sleep(Duration::from_secs_f64(retry_after));
            Response {
                success: false,
                retry: true,
            }
        }
        DiscordError::Other(message) => {
            bar.println(message.red().to_string());
            // Do not bother retrying for these errors.
            Response {
                success: false,
                retry: false,
            }
        }
    }
}

fn get_displayname(token: &str, cache: &mut HashMap<u64, String>, user_id: u64) -> String {
    if let Some(name) = cache.get(&user_id) {
        return name.clone();
    }

    match user_get_displayname(token, user_id) {
        Ok(display_name) => {
            cache.insert(user_id, display_name.clone());
            display_name
        }
        Err(_) => "<unknown user>".to_string(),
    }
}
