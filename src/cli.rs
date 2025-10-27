use chrono::{DateTime, NaiveDate, Utc};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum DeletionMode {
    Delete,
    Shakespeare,
}

/// Discord Selfbot mass message redaction tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Your personal Discord Token.
    pub token: String,

    /// The directory of your Discord data export
    pub export_dir: PathBuf,

    /// How messages should be deleted/edited
    pub mode: DeletionMode,

    /// Whether to preserve message attachments like images or videos. Default: false
    #[arg(short, long, default_value_t = false)]
    pub preserve_attachments: bool,

    /// Whether to delete messages from Guilds (Servers). Default: true
    #[arg(long, default_value_t = true)]
    pub delete_guilds: bool,

    /// Whether to delete messages from DMs (Direct Messages). Default: true
    #[arg(long, default_value_t = true)]
    pub delete_dms: bool,

    /// Whether to delete messages from Groups DMs. Default: true
    #[arg(long, default_value_t = true)]
    pub delete_groups: bool,

    /// Only delete messages after this date (YYYY-MM-DD format)
    #[arg(short, long, value_parser = parse_date)]
    pub after: Option<DateTime<Utc>>,

    /// Only delete messages before this date (YYYY-MM-DD format)
    #[arg(short, long, value_parser = parse_date)]
    pub before: Option<DateTime<Utc>>,

    /// A comma separated list of Channel or Guild IDs where messages shouldn't be deleted.
    #[arg(long, value_delimiter = ',')]
    pub preserve_list: Vec<u64>,

    /// A path to a file containing a newline separated blacklist (see --preserve-list).
    #[arg(long)]
    pub preserve_list_file: Option<PathBuf>,

    /// A comma separated list of Channel or Guild IDs where messages should be deleted.
    /// If unset, ALL messages are deleted.
    #[arg(long, value_delimiter = ',')]
    pub delete_list: Vec<u64>,

    /// A path to a file containing a newline separated whitelist (see --delete-list).
    #[arg(long)]
    pub delete_list_file: Option<PathBuf>,
}

fn parse_date(s: &str) -> Result<DateTime<Utc>, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc())
        .map_err(|e| format!("Invalid date: '{}'. Use YYYY-MM-DD format: {}", s, e))
}
