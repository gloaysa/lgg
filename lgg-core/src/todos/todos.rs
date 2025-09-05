use super::{
    parse_todos::parse_todo_file_content,
    todo_entry::{ReadTodoOptions, TodoEntry, TodoQueryResult, TodoStatus, TodoWriteEntry},
    todos_paths::todos_file,
};
use crate::todos::format_utils::format_todo_entry_block;
use crate::utils::date_utils::DateFilter;
use crate::{QueryError, QueryTagsResult};
use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::{collections::HashSet, io::Write};
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Todos {
    pub todo_list_dir: PathBuf,
    pub todo_datetime_format: String,
    /// The date to use as "today" for relative keywords.
    pub reference_date: NaiveDate,
    pub default_time: NaiveTime,
}
impl Todos {
    pub fn create_entry(&self, input: TodoWriteEntry) -> Result<TodoEntry> {
        let due_date = match input.due_date {
            Some(date) => match input.time {
                Some(time) => Some(NaiveDateTime::new(date, time)),
                None => Some(NaiveDateTime::new(date, self.default_time)),
            },
            None => None,
        };
        let todos_file = todos_file(&self.todo_list_dir);
        if let Some(parent) = todos_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent directory {}", parent.display()))?;
        }

        let is_new = !todos_file.exists();
        let header = "# Todos\n".to_string();
        let todo_subheader = "## Pending\n".to_string();
        let done_subheader = "## Done\n".to_string();
        let block = format_todo_entry_block(
            &input.title,
            &input.body,
            due_date,
            None,
            &self.todo_datetime_format,
        );

        let new_entry = TodoEntry {
            due_date,
            done_date: None,
            title: input.title,
            body: input.body,
            path: todos_file.clone(),
            status: TodoStatus::Pending,
            tags: input.tags,
        };
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&todos_file)
            .with_context(|| format!("opening {}", todos_file.display()))?;

        if is_new {
            writeln!(file, "{header}{todo_subheader}{block}")
                .with_context(|| format!("writing day header to {}", todos_file.display()))?;
        } else {
            let all_entries = self.parse_file(&todos_file);
            let mut all_todos = Vec::new();
            all_todos.extend(all_entries.todos);
            all_todos.push(new_entry.clone());
            all_todos.sort_by_key(|e| e.due_date);
            let pending_todos: Vec<&TodoEntry> = all_todos.iter().filter(|td| matches!(td.status, TodoStatus::Pending)).collect();
            let done_todos: Vec<&TodoEntry> = all_todos.iter().filter(|td| matches!(td.status, TodoStatus::Done)).collect();

            let mut new_content = header;
            new_content.push_str(&todo_subheader);
            for td in pending_todos {
                let block = format_todo_entry_block(
                    &td.title,
                    &td.body,
                    td.due_date,
                    td.done_date,
                    &self.todo_datetime_format,
                );
                new_content.push_str(&block);
            }

            if !done_todos.is_empty() {
                new_content.push_str(&done_subheader);
                for td in done_todos {
                    let block = format_todo_entry_block(
                        &td.title,
                        &td.body,
                        td.due_date,
                        td.done_date,
                        &self.todo_datetime_format,
                    );
                    new_content.push_str(&block);
                }
            }
            fs::write(&todos_file, new_content)?;
        }

        Ok(new_entry)
    }

    /// Reads and returns all entries, the results can be filtered by `options`.
    /// This is the primary query function for retrieving todos. It is designed to be
    /// resilient, returning a [`TodoQueryResult `] that contains both parsed entries and
    /// any errors that occurred.
    pub fn read_entries(&self, options: &ReadTodoOptions) -> TodoQueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let pending_file = todos_file(&self.todo_list_dir);
        if pending_file.exists() {}

        let results = self.parse_file(&pending_file);

        entries.extend(results.todos);
        errors.extend(results.errors);

        entries.sort_by_key(|k| k.due_date);

        if let Some(dates) = options.due_date {
            match dates {
                DateFilter::Single(s_date) => {
                    entries = entries
                        .into_iter()
                        .filter(|e| e.due_date.map(|d| d.date() == s_date).unwrap_or(false))
                        .collect();
                }
                DateFilter::Range(s_date, e_date) => {
                    entries = entries
                        .into_iter()
                        .filter(|e| {
                            e.due_date
                                .map(|d| d.date() >= s_date && d.date() <= e_date)
                                .unwrap_or(false)
                        })
                        .collect();
                }
            }
        }

        if let Some(tags) = &options.tags {
            let found_tags: Vec<String> = tags
                .into_iter()
                .map(|t| t.trim().to_ascii_lowercase())
                .collect();

            entries = entries
                .into_iter()
                .filter(|e| found_tags.iter().any(|t| e.tags.contains(t)))
                .collect();
        }

        TodoQueryResult {
            todos: entries,
            errors,
        }
    }

    pub fn search_all_tags(&self) -> QueryTagsResult {
        let mut tags: Vec<String> = Vec::new();
        let mut errors = Vec::new();
        let pending_file = todos_file(&self.todo_list_dir);
        let done_file = todos_file(&self.todo_list_dir);
        let pending_todos_result = self.parse_file(&pending_file);
        let done_todos_result = self.parse_file(&done_file);

        errors.extend(pending_todos_result.errors);
        errors.extend(done_todos_result.errors);

        for todo in pending_todos_result.todos {
            tags.extend(todo.tags);
        }
        for todo in done_todos_result.todos {
            tags.extend(todo.tags);
        }

        tags = tags
            .iter()
            .map(|mat| mat.as_str().to_string().trim().to_ascii_lowercase())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();

        QueryTagsResult { tags, errors }
    }

    pub fn parse_file(&self, path: &PathBuf) -> TodoQueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        if !path.exists() {
            errors.push(QueryError::FileError {
                path: path.clone(),
                error: anyhow!(format!("File does not exist in path: {}", path.display())),
            });
            return TodoQueryResult {
                todos: entries,
                errors,
            };
        }
        match fs::read_to_string(&path) {
            Ok(file_content) => {
                let parse_result =
                    parse_todo_file_content(&file_content, &self.todo_datetime_format);
                for entry in parse_result.entries {
                    entries.push(TodoEntry {
                        due_date: entry.due_date,
                        done_date: entry.done_date,
                        title: entry.title,
                        body: entry.body,
                        tags: entry.tags,
                        status: entry.status,
                        path: path.clone(),
                    });
                }

                for error in parse_result.errors {
                    errors.push(QueryError::FileError {
                        path: path.clone(),
                        error: anyhow!(error),
                    });
                }
            }
            Err(error) => {
                errors.push(QueryError::FileError {
                    path: path.clone(),
                    error: error.into(),
                });
            }
        }
        TodoQueryResult {
            todos: entries,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};
    use std::fs;
    use tempfile::tempdir;

    use super::Todos;
    use crate::utils::date_utils::DateFilter;
    use crate::{
        tests::mk_config,
        todos::{
            todo_entry::{ReadTodoOptions, TodoStatus, TodoWriteEntry},
            todos_paths::todos_file,
        },
    };

    fn mk_todo_list_with_default(reference_date: Option<NaiveDate>) -> (Todos, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let config = mk_config(root, reference_date);

        let todos = Todos {
            todo_list_dir: config.todo_list_dir,
            todo_datetime_format: config.todo_datetime_format,
            reference_date: config.reference_date,
            default_time: config.default_time,
        };
        (todos, tmp)
    }

    #[test]
    fn write_first_todo_creates_file_and_appends() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry = TodoWriteEntry {
            due_date: None,
            time: None,
            title: "Test entry.".to_string(),
            body: "With body.".to_string(),
            tags: Vec::new(),
        };
        let res = t.create_entry(entry).unwrap();
        let expected = todos_file(&t.todo_list_dir);
        assert_eq!(res.path, expected);
        assert!(res.path.exists());

        let s = fs::read_to_string(&res.path).unwrap();
        assert!(s.starts_with("# Todos\n"));
        assert!(s.contains("## Pending\n"));
        assert!(s.contains("Test entry."));
    }

    #[test]
    fn write_second_todo_appends() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry1 = TodoWriteEntry {
            due_date: None,
            time: None,
            title: "First entry.".to_string(),
            body: "With body.".to_string(),
            tags: Vec::new(),
        };
        let entry2 = TodoWriteEntry {
            due_date: None,
            time: None,
            title: "Second entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        t.create_entry(entry1).unwrap();
        let res2 = t.create_entry(entry2).unwrap();

        let s = fs::read_to_string(&res2.path).unwrap();
        assert!(s.starts_with("# Todos\n"));
        assert!(s.contains("## Pending\n"));
        assert!(s.contains("First entry."));
        assert!(s.contains("Second entry."));
    }

    #[test]
    fn write_todo_returns_valid_entry() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry = TodoWriteEntry {
            due_date: None,
            time: None,
            title: "Test entry.".to_string(),
            body: "With body.".to_string(),
            tags: Vec::new(),
        };
        let res = t.create_entry(entry).unwrap();
        assert_eq!(res.title, "Test entry.");
        assert_eq!(res.body, "With body.");
        assert!(matches!(res.status, TodoStatus::Pending));
    }

    #[test]
    fn read_entries_success() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry1 = TodoWriteEntry {
            due_date: None,
            time: None,
            title: "First entry.".to_string(),
            body: "With body and @tag.".to_string(),
            tags: Vec::new(),
        };
        let entry2 = TodoWriteEntry {
            due_date: None,
            time: None,
            title: "Second entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        t.create_entry(entry1).unwrap();
        t.create_entry(entry2).unwrap();

        let options = ReadTodoOptions {
            ..Default::default()
        };

        let result = t.read_entries(&options);
        assert!(result.errors.is_empty());
        assert_eq!(result.todos.len(), 2);
        assert_eq!(result.todos[0].title, "First entry.");
        assert_eq!(result.todos[0].tags.len(), 1);
        assert_eq!(result.todos[0].body, "With body and @tag.");
        assert_eq!(result.todos[1].title, "Second entry.");
    }
    #[test]
    fn read_entries_due_date_success() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry1 = TodoWriteEntry {
            due_date: Some(NaiveDate::from_ymd_opt(2025, 08, 15).unwrap()),
            time: Some(NaiveTime::from_hms_opt(12, 00, 00).unwrap()),
            title: "First entry.".to_string(),
            body: "With body and @tag.".to_string(),
            tags: Vec::new(),
        };
        t.create_entry(entry1).unwrap();

        let options = ReadTodoOptions {
            ..Default::default()
        };

        let result = t.read_entries(&options);
        assert!(result.errors.is_empty());
        assert_eq!(result.todos.len(), 1);
        assert_eq!(result.todos[0].title, "First entry.");
        assert_eq!(
            result.todos[0].due_date.unwrap().date(),
            NaiveDate::from_ymd_opt(2025, 08, 15).unwrap()
        );
        assert_eq!(
            result.todos[0].due_date.unwrap().time(),
            NaiveTime::from_hms_opt(12, 00, 00).unwrap()
        );
    }
    #[test]
    fn read_entries_filter_single_date() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry1 = TodoWriteEntry {
            due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 15).unwrap()),
            time: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            title: "First entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        let entry2 = TodoWriteEntry {
            due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 16).unwrap()),
            time: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            title: "Second entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        t.create_entry(entry1).unwrap();
        t.create_entry(entry2).unwrap();

        let options = ReadTodoOptions {
            due_date: Some(DateFilter::Single(
                NaiveDate::from_ymd_opt(2025, 8, 15).unwrap(),
            )),
            ..Default::default()
        };

        let result = t.read_entries(&options);
        assert!(result.errors.is_empty());
        assert_eq!(result.todos.len(), 1);
        assert_eq!(result.todos[0].title, "First entry.");
    }

    #[test]
    fn read_entries_filter_date_range() {
        let (t, _tmp) = mk_todo_list_with_default(None);
        let entry1 = TodoWriteEntry {
            due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 14).unwrap()),
            time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            title: "Entry before range.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        let entry2 = TodoWriteEntry {
            due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 15).unwrap()),
            time: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            title: "Entry in range.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        let entry3 = TodoWriteEntry {
            due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 16).unwrap()),
            time: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            title: "Entry after range.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        t.create_entry(entry1).unwrap();
        t.create_entry(entry2).unwrap();
        t.create_entry(entry3).unwrap();

        let options = ReadTodoOptions {
            due_date: Some(DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 15).unwrap(),
                NaiveDate::from_ymd_opt(2025, 8, 15).unwrap(),
            )),
            ..Default::default()
        };

        let result = t.read_entries(&options);
        assert!(result.errors.is_empty());
        assert_eq!(result.todos.len(), 1);
        assert_eq!(result.todos[0].title, "Entry in range.");
    }

    #[test]
    fn find_all_tags() {
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // A day in 2025
        let (todos, _tmp) = mk_todo_list_with_default(Some(anchor));

        todos
            .create_entry(TodoWriteEntry {
                due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 14).unwrap()),
                time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                title: "Day in the past with @past tag.".to_string(),
                body: "".to_string(),
                tags: Vec::new(),
            })
            .unwrap();
        todos
            .create_entry(TodoWriteEntry {
                due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 14).unwrap()),
                time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                title: "Day way in the future with @future. Has @double_tag in body.".to_string(),
                body: "".to_string(),
                tags: Vec::new(),
            })
            .unwrap();

        todos
            .create_entry(TodoWriteEntry {
                due_date: Some(NaiveDate::from_ymd_opt(2025, 8, 14).unwrap()),
                time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                title: "Has a tag in body".to_string(),
                body: "This is another @double_tag".to_string(),
                tags: Vec::new(),
            })
            .unwrap();

        let results = todos.search_all_tags();
        assert_eq!(results.errors.len(), 0);
        assert_eq!(results.tags.len(), 3);

        assert!(results.tags.contains(&"@past".to_string()));
        assert!(results.tags.contains(&"@double_tag".to_string()));
        assert!(results.tags.contains(&"@future".to_string()));
    }
}
