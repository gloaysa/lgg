use super::todo_entry::{TodoEntry, TodoWriteEntry};
use anyhow::Result;
use chrono::Local;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TodoList {}
impl TodoList {
    pub fn create_entry(&self, input: TodoWriteEntry) -> Result<TodoEntry> {
        Ok(TodoEntry {
            date: Local::now().date_naive(),
            time: Local::now().time(),
            title: "".to_string(),
            body: "".to_string(),
            path: PathBuf::new(),
        })
    }
}
