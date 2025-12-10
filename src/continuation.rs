use std::io::Write;
use std::{collections::HashSet, fs::OpenOptions, path::Path};

pub fn parse_continuation_file(path: &Path) -> Result<HashSet<u64>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Could not read continuation file: {e}"))?;

    let mut ids = HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let id: u64 = line
            .parse()
            .map_err(|e| format!("Invalid Message ID {line:?} in continuation file: {e}"))?;
        ids.insert(id);
    }

    Ok(ids)
}

/// Appends a message ID to the continuation file.
/// Errors are logged rather than returned since continuation file failures
/// should not halt processing.
pub fn write_continuation_file(path: &Path, message_id: u64) {
    let result = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .and_then(|mut f| writeln!(f, "{}", message_id));

    if let Err(e) = result {
        eprintln!("Failed to write to continuation file: {e}");
    }
}
