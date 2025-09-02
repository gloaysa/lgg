use super::{
    TodoStatus,
    todo_entry::{ParsedTodoEntry, ReadTodosResult},
};
use crate::utils::parse_entries::extract_tags;
use chrono::NaiveDateTime;

pub fn parse_todo_file_content(content: &str) -> ReadTodosResult {
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    let lines = content.lines();

    let content = lines.collect::<Vec<_>>().join("\n");
    // Split content by the entry delimiter "- [ ".
    for block in content.split("- [") {
        // Skip empty blocks that can result from the split (e.g., the line before the body).
        if block.trim().is_empty() {
            continue;
        }
        if let Some(newline_pos) = block.find(" - ") {
            let heading = &block[..newline_pos];
            let body = block[newline_pos..].trim().to_string();
            let tags = extract_tags(&block);

            match heading.find(" | ") {
                Some(separator_pos) => {
                    let title = heading[..separator_pos].trim();
                    let time_str = heading[separator_pos + 3..].trim().to_string();
                    let status = match title.find("[x]") {
                        Some(_) => TodoStatus::Done,
                        None => TodoStatus::Pending
                    };

                    match NaiveDateTime::parse_from_str(&time_str, "%d/%m/%Y %H:%M") {
                        Ok(due_date) => entries.push(ParsedTodoEntry {
                            due_date: Some(due_date),
                            done_date: None,
                            title: title.to_string(),
                            body,
                            tags,
                            status,
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
            if let Some(separator_pos) = block.find(" | ") {
                let title = block[..separator_pos].trim();
                let date = block[separator_pos + 3..].trim().to_string();
                let tags = extract_tags(&title);
                let status = match title.find("[x]") {
                    Some(_) => TodoStatus::Done,
                    None => TodoStatus::Pending,
                };

                if let Ok(due_date) = NaiveDateTime::parse_from_str(&date, "%d/%m%Y %H:%M") {
                    entries.push(ParsedTodoEntry {
                        due_date: Some(due_date),
                        done_date: None,
                        title: title.to_string(),
                        body: String::new(),
                        tags,
                        status,
                    });
                }
            }
        }
    }
    ReadTodosResult { entries, errors }
}
