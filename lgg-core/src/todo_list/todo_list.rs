use crate::utils::format_utils::format_todo_entry_block;

use super::{
    todo_entry::{TodoEntry, TodoStatus, TodoWriteEntry},
    todo_list_paths::pending_todos_file,
};
use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};
use std::io::Write;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

#[derive(Debug)]
pub struct TodoList {
    pub todo_list_dir: PathBuf,
    pub todo_datetime_format: String,
    /// The date to use as "today" for relative keywords.
    pub reference_date: NaiveDate,
    pub default_time: NaiveTime,
}
impl TodoList {
    pub fn create_entry(&self, input: TodoWriteEntry) -> Result<TodoEntry> {
        let due_date = match input.due_date {
            Some(date) => match input.time {
                Some(time) => Some(NaiveDateTime::new(date, time)),
                None => Some(NaiveDateTime::new(date, self.default_time)),
            },
            None => None,
        };
        let pending_file = pending_todos_file(&self.todo_list_dir);
        if let Some(parent) = pending_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent directory {}", parent.display()))?;
        }

        let is_new = !pending_file.exists();
        let header = format!("# All my pending todos\n");
        let block = format_todo_entry_block(
            &input.title,
            &input.body,
            due_date,
            None,
            &self.todo_datetime_format,
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&pending_file)
            .with_context(|| format!("opening {}", pending_file.display()))?;

        if is_new {
            writeln!(file, "{header}\n")
                .with_context(|| format!("writing day header to {}", pending_file.display()))?;
            write!(file, "{block}")
                .with_context(|| format!("appending entry to {}", pending_file.display()))?;
        } else {
            write!(file, "{block}")
                .with_context(|| format!("appending entry to {}", pending_file.display()))?;
        }

        Ok(TodoEntry {
            date: Local::now().date_naive(),
            time: Local::now().time(),
            title: input.title,
            body: input.body,
            path: pending_file.clone(),
            status: TodoStatus::Pending,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use std::fs;
    use tempfile::tempdir;

    use crate::{
        tests::mk_config,
        todo_list::{
            todo_entry::{TodoStatus, TodoWriteEntry},
            todo_list_paths::pending_todos_file,
        },
    };

    use super::TodoList;

    fn mk_todo_list_with_default(
        reference_date: Option<NaiveDate>,
    ) -> (TodoList, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let config = mk_config(root, reference_date);

        let todos = TodoList {
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
        let expected = pending_todos_file(&t.todo_list_dir);
        assert_eq!(res.path, expected);
        assert!(res.path.exists());

        let s = fs::read_to_string(&res.path).unwrap();
        assert!(s.starts_with("# All my pending todos\n"));
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
        assert!(s.starts_with("# All my pending todos\n"));
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
}
