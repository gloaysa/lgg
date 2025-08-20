use chrono::{Local, NaiveDate, NaiveTime};

/// Returns an output like this: `# Friday, 15 Aug 2025`
pub fn format_day_header(date_format: &str, date: NaiveDate) -> String {
    format!("# {}\n\n", date.format(date_format).to_string())
}

/// Render an entry block. `# 12:30 - Title\nBody`
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

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
