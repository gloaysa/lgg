//! Parses the content of a daily journal file into structured `Entry` objects.

use crate::entry::Entry;
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime};

/// Parses the entire content of a daily journal file.
///
/// This function acts as the main entry point for file parsing. It expects the file
/// to have a specific format:
/// - A mandatory header on the first line (e.g., `# Friday, 15 Aug 2025`).
/// - Zero or more entry blocks, each starting with `## HH:MM - Title`.
///
/// # Arguments
///
/// * `content` - A string slice containing the full content of the journal file.
///
/// # Returns
///
/// A `Result` containing either a `Vec<Entry>` on success or an `anyhow::Error`
/// if the file is empty or the header is malformed. An empty `Vec` signifies a
/// valid file with no entries.
pub fn parse_day_file(content: &str) -> Result<Vec<Entry>> {
    let mut lines = content.lines();
    let header_line = lines.next().context("Cannot parse an empty file.")?;

    let date = parse_date_from_header_line(header_line)
        .context("Could not parse date from file header. Is the file malformed?")?;

    let content = lines.collect::<Vec<_>>().join("\n");
    let mut entries = Vec::new();
    // Split content by the entry delimiter "## ".
    for block in content.split("## ") {
        // Skip empty blocks that can result from the split (e.g., the content before the first `##`).
        if block.trim().is_empty() {
            continue;
        }
        if let Some(newline_pos) = block.find('\n') {
            let heading = &block[..newline_pos];
            let body = block[newline_pos..].trim().to_string();

            if let Some(separator_pos) = heading.find(" - ") {
                let time_str = heading[..separator_pos].trim();
                let title = heading[separator_pos + 3..].trim().to_string();

                // If time parsing fails, we skip this block, assuming it's malformed.
                if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                    entries.push(Entry {
                        date,
                        time,
                        title,
                        body,
                        tags: Vec::new(),
                    });
                }
            }
        } else {
            // Handle case where an entry is just a single line (e.g. "## 12:34 - Title only")
            if let Some(separator_pos) = block.find(" - ") {
                let time_str = block[..separator_pos].trim();
                let title = block[separator_pos + 3..].trim().to_string();
                if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                    entries.push(Entry {
                        date,
                        time,
                        title,
                        body: String::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    Ok(entries)
}

/// Parses a `NaiveDate` from a markdown header line.
///
/// # Arguments
///
/// * `line` - A string slice of the header line (e.g., "# Friday, 15 Aug 2025").
fn parse_date_from_header_line(line: &str) -> Option<NaiveDate> {
    // TODO: This format should be configurable, as when we are writing to the file
    line.trim()
        .strip_prefix("# ")
        .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%A, %d %b %Y").ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn parse_valid_day_file() {
        let content = r#"# Friday, 15 Aug 2025

## 12:34 - Quiet morning

Body... with @work and @fav

## 18:05 - Walk by the river

Another paragraph... @health
"#;
        let entries = parse_day_file(content.trim()).unwrap();
        assert_eq!(entries.len(), 2);

        let expected_date = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        assert_eq!(entries[0].date, expected_date);
        assert_eq!(entries[0].title, "Quiet morning");
        assert_eq!(entries[0].body, "Body... with @work and @fav");

        assert_eq!(entries[1].date, expected_date);
        assert_eq!(entries[1].title, "Walk by the river");
        assert_eq!(entries[1].body, "Another paragraph... @health");
    }

    #[test]
    fn parse_file_with_no_entries() {
        let content = "# Friday, 15 Aug 2025";
        let entries = parse_day_file(content).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_file_with_malformed_header_fails() {
        let content = "# Not a date";
        let result = parse_day_file(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not parse date"));
    }

    #[test]
    fn parse_empty_file_fails() {
        let content = "";
        let result = parse_day_file(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot parse an empty file"));
    }

    #[test]
    fn parse_file_with_malformed_entry_is_skipped() {
        let content = r#"# Friday, 15 Aug 2025

## NOT A TIME - Bad entry

Body...

## 18:05 - Good entry

Body...
"#;
        let entries = parse_day_file(content.trim()).unwrap();
        // It should gracefully skip the bad entry and parse the good one.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Good entry");
    }
    
    #[test]
    fn parse_entry_with_no_body() {
        let content = r#"# Friday, 15 Aug 2025

## 12:34 - Title only
## 18:05 - Another entry

With a body.
"#;
        let entries = parse_day_file(content.trim()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Title only");
        assert!(entries[0].body.is_empty());
        assert_eq!(entries[1].title, "Another entry");
        assert!(!entries[1].body.is_empty());
    }
}