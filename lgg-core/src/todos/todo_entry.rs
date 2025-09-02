use crate::utils::date_utils::DateFilter;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum TodoStatus {
    Pending,
    Done,
}

#[derive(Debug)]
pub struct TodoEntry {
    pub due_date: Option<NaiveDateTime>,
    pub done_date: Option<NaiveDateTime>,
    pub title: String,
    pub body: String,
    pub path: PathBuf,
    pub status: TodoStatus,
    pub tags: Vec<String>,
}

/// Properties to create a new todo entry
#[derive(Debug)]
pub struct TodoWriteEntry {
    pub due_date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

/// Represents a non-critical issue that occurred during a query.
/// This is used to report problems (e.g., malformed files, invalid input)
/// without stopping a larger query operation.
#[derive(Debug)]
pub enum TodoQueryError {
    InvalidDate { input: String, error: String },
    FileError { path: PathBuf, error: anyhow::Error },
}

/// The complete result of a query.
/// Contains successfully parsed entries and any errors.
#[derive(Debug)]
pub struct TodoQueryResult {
    pub entries: Vec<TodoEntry>,
    pub errors: Vec<TodoQueryError>,
}

#[derive(Clone, Debug, Default)]
pub struct ReadTodoOptions<'a> {
    pub due_date: Option<DateFilter>,
    pub done_date: Option<DateFilter>,
    pub time: Option<&'a str>,
    pub tags: Option<&'a Vec<String>>,
    pub status: Option<TodoStatus>,
}

#[derive(Debug)]
pub struct ParsedTodosEntry {
    pub due_date: Option<NaiveDateTime>,
    pub done_date: Option<NaiveDateTime>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: TodoStatus,
}

#[derive(Debug)]
pub struct ReadTodosResult {
    pub entries: Vec<ParsedTodosEntry>,
    pub errors: Vec<String>,
}
