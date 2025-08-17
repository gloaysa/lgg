//! Pure Markdown rendering helpers.
//!
//! Day header:  `# Friday, 15 Aug 2025`
//! Entry block:
//!   ## HH:MM - Title
//!
//!   Body…

use chrono::{Local, NaiveDate, NaiveTime};

use crate::Config;

/// `# Friday, 15 Aug 2025`
pub fn format_day_header(date: NaiveDate, config: &Config) -> String {
    format!("# {}", format_date(date, config))
}

/// Render an entry block.
pub fn format_entry_block(title: &str, body: &str, time: Option<NaiveTime>) -> String {
    let t = time.unwrap_or_else(|| Local::now().time());
    let time = t.format("%H:%M");
    if body.trim().is_empty() {
        format!("## {time} - {title}\n\n")
    } else {
        let body = body.trim_end_matches('\n');
        format!("## {time} - {title}\n\n{body}\n\n")
    }
}

/// Formats a date according to the user's configuration.
pub fn format_date(date: NaiveDate, config: &Config) -> String {
    date.format(&config.date_format).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::mk_config;
    use chrono::{NaiveDate, NaiveTime};
    use tempfile::tempdir;

    #[test]
    fn header_formats_readably() {
        let tmp = tempdir().unwrap();
        let cfg = mk_config(tmp.path().to_path_buf());
        let d = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap(); // Friday
        let s = format_day_header(d, &cfg);
        assert!(s.starts_with("# Fri") || s.starts_with("# Friday"));
        assert!(s.contains("15 Aug 2025"));
    }

    #[test]
    fn entry_block_with_body() {
        let t = NaiveTime::from_hms_opt(12, 34, 0).unwrap();
        let s = format_entry_block("Quiet morning", "Body...", Some(t));
        assert!(s.starts_with("## 12:34 - Quiet morning\n\nBody...\n\n"));
        assert!(s.ends_with("Body...\n\n"));
    }

    #[test]
    fn entry_block_without_body() {
        let t = NaiveTime::from_hms_opt(7, 5, 0).unwrap();
        let s = format_entry_block("Title only", "", Some(t));
        assert_eq!(s, "## 07:05 - Title only\n\n");
    }
}

