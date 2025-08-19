//! Parses the content of a daily journal file into structured `Entry` objects.

use crate::entry::Entry;
use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct ParseResult {
    pub entries: Vec<Entry>,
    pub errors: Vec<String>,
}

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
/// A `ParseResult` containing a `Vec<Entry>` and `errors` in case some where found,
/// if the file is empty or the header is malformed or a specific entry is invalid.
pub fn parse_day_file(content: &str) -> ParseResult {
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    let mut lines = content.lines();
    let header_line = match lines.next() {
        Some(h) => h,
        None => {
            errors.push(
                "Empty file: expected a date header like `# DATE` on the first line.".to_string(),
            );
            return ParseResult { entries, errors };
        }
    };

    let date = match parse_date_from_header_line(header_line) {
        Some(d) => d,
        None => {
            errors.push(
                format!("Invalid or missing H1 date header: expected first line like `# DATE`, found {header_line}.").to_string(),
            );
            return ParseResult { entries, errors };
        }
    };

    let content = lines.collect::<Vec<_>>().join("\n");
    // Split content by the entry delimiter "## ".
    for block in content.split("\n## ") {
        // Skip empty blocks that can result from the split (e.g., the content before the first `##`).
        if block.trim().is_empty() {
            continue;
        }
        if let Some(newline_pos) = block.find('\n') {
            let heading = &block[..newline_pos];
            let body = block[newline_pos..].trim().to_string();

            match heading.find(" - ") {
                Some(separator_pos) => {
                    let time_str = heading[..separator_pos].trim();
                    let title = heading[separator_pos + 3..].trim().to_string();

                    match NaiveTime::parse_from_str(time_str, "%H:%M") {
                        Ok(time) => entries.push(Entry {
                            date,
                            time,
                            title,
                            body,
                            tags: Vec::new(),
                        }),
                        Err(_) => errors.push(
                            format!("Invalid time in entry header `{heading}`. Expected a 24-hour time `HH:MM`.").to_string(),
                        ),
                    }
                }
                None => errors
                    .push(format!("Invalid H2 entry header: `{heading}`. Expected `HH:MM - Title.` (e.g., `08:03 - Morning coffe`)." ).to_string()),
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
    ParseResult { entries, errors }
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

### Header 3 is valid
"#;
        let result = parse_day_file(content.trim());
        assert_eq!(result.entries.len(), 2);

        let expected_date = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        assert_eq!(result.entries[0].date, expected_date);
        assert_eq!(result.entries[0].title, "Quiet morning");
        assert_eq!(result.entries[0].body, "Body... with @work and @fav");

        assert_eq!(result.entries[1].date, expected_date);
        assert_eq!(result.entries[1].title, "Walk by the river");
        assert_eq!(
            result.entries[1].body,
            "Another paragraph... @health\n\n### Header 3 is valid"
        );
    }

    #[test]
    fn parse_file_with_no_entries() {
        let content = "# Friday, 15 Aug 2025";
        let result = parse_day_file(content);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn parse_file_with_malformed_header_fails() {
        let content = "# Not a date";
        let result = parse_day_file(content);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("Invalid or missing H1 date header"));
    }

    #[test]
    fn parse_empty_file_fails() {
        let content = "";
        let result = parse_day_file(content);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("Empty file"));
    }

    #[test]
    fn parse_file_with_malformed_entry_returns_good_entry_and_errors() {
        let content = r#"# Friday, 15 Aug 2025

## NOT A TIME - Bad entry

Body...

## 18:05 - Good entry

Body...
"#;
        let result = parse_day_file(content.trim());
        // It should gracefully skip the bad entry and parse the good one.
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].title, "Good entry");
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("Invalid time"));
    }

    #[test]
    fn parse_entry_with_no_body() {
        let content = r#"# Friday, 15 Aug 2025

## 12:34 - Title only
## 18:05 - Another entry

With a body.
"#;
        let result = parse_day_file(content.trim());
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "Title only");
        assert!(result.entries[0].body.is_empty());
        assert_eq!(result.entries[1].title, "Another entry");
        assert!(!result.entries[1].body.is_empty());
    }
}
