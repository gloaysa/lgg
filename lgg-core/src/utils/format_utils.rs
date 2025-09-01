use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Returns an output like this: `# Friday, 15 Aug 2025`
pub fn format_day_header(date_format: &str, date: NaiveDate) -> String {
    format!("# {}\n\n", date.format(date_format).to_string())
}

/// Render an entry block. `# 12:30 - Title\nBody`
pub fn format_journal_entry_block(title: &str, body: &str, time: &NaiveTime) -> String {
    let time = time.format("%H:%M");
    if body.trim().is_empty() {
        format!("## {time} - {title}\n\n")
    } else {
        let body = body.trim_end_matches('\n');
        format!("## {time} - {title}\n\n{body}\n\n")
    }
}

pub fn format_todo_entry_block(
    title: &str,
    body: &str,
    due_date: Option<NaiveDateTime>,
    done_date: Option<NaiveDateTime>,
    date_format: &str,
) -> String {
    let mut entry = format!("- [ ] {title}");

    match due_date {
        Some(d) => {
            let formatted_date = d.format(date_format);
            entry = format!("{entry} | {formatted_date}");
        }
        None => (),
    };
    match done_date {
        Some(d) => {
            let formatted_date = d.format(date_format);
            if due_date.is_none() {
                entry = format!("{entry} | | {formatted_date}");
            } else {
                entry = format!("{entry} | {formatted_date}");
            };
        }
        None => (),
    };
    if body.trim().is_empty() {
        entry = format!("{entry}\n\n");
        entry
    } else {
        let body = body.trim_end_matches('\n');
        let spaces = " ".repeat(6);
        entry = format!("{entry}\n{spaces}{body}\n\n");
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn entry_block_with_body() {
        let t = NaiveTime::from_hms_opt(12, 34, 0).unwrap();
        let s = format_journal_entry_block("Quiet morning", "Body...", &t);
        assert!(s.starts_with("## 12:34 - Quiet morning\n\nBody...\n\n"));
        assert!(s.ends_with("Body...\n\n"));
    }

    #[test]
    fn entry_block_without_body() {
        let t = NaiveTime::from_hms_opt(7, 5, 0).unwrap();
        let s = format_journal_entry_block("Title only", "", &t);
        assert_eq!(s, "## 07:05 - Title only\n\n");
    }

    #[test]
    fn todo_entry_block_only_title() {
        let format = "%d/%m/%Y %H:%M";
        let e = format_todo_entry_block("Item 1", "", None, None, format);

        assert_eq!(e, "- [ ] Item 1\n\n");
    }

    #[test]
    fn todo_entry_block_with_body() {
        let format = "%d/%m/%Y %H:%M";
        let e = format_todo_entry_block("Item 1", "With body", None, None, format);

        assert_eq!(e, "- [ ] Item 1\n      With body\n\n");
    }

    #[test]
    fn todo_entry_block_with_date() {
        let d = NaiveDate::from_ymd_opt(2025, 08, 20).unwrap();
        let t = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let due_date = NaiveDateTime::new(d, t);
        let format = "%d/%m/%Y %H:%M";
        let e = format_todo_entry_block("Item 1", "", Some(due_date), None, format);

        assert_eq!(e, "- [ ] Item 1 | 20/08/2025 07:00\n\n");
    }

    #[test]
    fn todo_entry_block_with_date_and_end_date() {
        let d = NaiveDate::from_ymd_opt(2025, 08, 20).unwrap();
        let t = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let dd = NaiveDate::from_ymd_opt(2025, 08, 22).unwrap();
        let td = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
        let due_date = NaiveDateTime::new(d, t);
        let done_date = NaiveDateTime::new(dd, td);
        let format = "%d/%m/%Y %H:%M";
        let e = format_todo_entry_block("Item 1", "", Some(due_date), Some(done_date), format);

        assert_eq!(e, "- [ ] Item 1 | 20/08/2025 07:00 | 22/08/2025 18:00\n\n");
    }
    #[test]
    fn todo_entry_block_only_end_date() {
        let dd = NaiveDate::from_ymd_opt(2025, 08, 22).unwrap();
        let td = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
        let done_date = NaiveDateTime::new(dd, td);
        let format = "%d/%m/%Y %H:%M";
        let e = format_todo_entry_block("Item 1", "", None, Some(done_date), format);

        assert_eq!(e, "- [ ] Item 1 | | 22/08/2025 18:00\n\n");
    }
}
