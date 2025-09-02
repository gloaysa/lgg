use super::{format_utils, todo_entry::ReadTodosResult, ParsedTodosEntry, TodoStatus};
use crate::utils::parse_input::extract_tags;

/// Reads all todo entries from the list and applies optional filters.
/// - Loads entries from the pending todos file.
/// - Sorts by `due_date`.
/// - Applies `due_date` filter (`Single` or `Range`) if provided.
/// - Applies `tags` filter if provided.
/// Returns all matching entries plus any parsing errors.
pub fn parse_todo_file_content(content: &str, date_format: &str) -> ReadTodosResult {
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    let mut lines = content.lines().peekable();

    match lines.next() {
        Some(h) if h.trim_start().starts_with('#') => {}
        Some(other) => errors.push(format!(
            "First line must be a header like `# ...`, got `{other}`."
        )),
        None => {
            errors.push("Empty file: expected a `#` header on the first line.".to_string());
            return ReadTodosResult { entries, errors };
        }
    }

    let is_entry_start = |s: &str| {
        let t = s.trim_start();
        t.starts_with("- [ ] ") || t.starts_with("- [x] ") || t.starts_with("- [X] ")
    };

    while let Some(line) = lines.peek() {
        if line.trim().is_empty() || !is_entry_start(line) {
            lines.next();
            continue;
        }

        let header = lines.next().unwrap();
        let trimmed = header.trim_start();

        let (status, rest) = if trimmed.starts_with("- [ ]") {
            (
                TodoStatus::Pending,
                trimmed.trim_start_matches("- [ ]").trim_start(),
            )
        } else {
            (
                TodoStatus::Done,
                trimmed
                    .trim_start_matches("- [x]")
                    .trim_start_matches("- [X]")
                    .trim_start(),
            )
        };

        let mut parts = rest.split(" | ").map(str::trim);
        let title = parts.next().unwrap_or("").to_string();
        let due_str = parts.next().unwrap_or("");
        let done_str = parts.next().unwrap_or("");

        let due_date = match format_utils::parse_datetime(due_str, date_format) {
            Ok(dt) => dt,
            Err(e) => {
                errors.push(format!("In `{header}`: {e}"));
                None
            }
        };
        let done_date = match format_utils::parse_datetime(done_str, date_format) {
            Ok(dt) => dt,
            Err(e) => {
                errors.push(format!("In `{header}`: {e}"));
                None
            }
        };

        let mut body_lines: Vec<String> = Vec::new();
        while let Some(next) = lines.peek().cloned() {
            if is_entry_start(next) {
                break;
            }

            body_lines.push(next.to_string());
            lines.next();
        }
        let body = body_lines.join("\n").trim().to_string();

        let tag_source = if body.is_empty() {
            title.clone()
        } else {
            format!("{title}\n{body}")
        };
        let tags = extract_tags(&tag_source);

        entries.push(ParsedTodosEntry {
            due_date,
            done_date,
            title,
            body,
            tags,
            status,
        });
    }

    ReadTodosResult { entries, errors }
}
