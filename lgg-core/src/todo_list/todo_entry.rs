use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::path::PathBuf;

use crate::utils::parsed_entry::DateFilter;

/// Represents a non-critical issue that occurred during a query.
/// This is used to report problems (e.g., malformed files, invalid input)
/// without stopping a larger query operation.
#[derive(Debug)]
pub enum TodoQueryError {
    InvalidDate { input: String, error: String },
    FileError { path: PathBuf, error: anyhow::Error },
}

#[derive(Debug)]
pub struct ReadTodosResult {
    pub entries: Vec<ParsedTodoEntry>,
    pub errors: Vec<String>,
}
#[derive(Debug)]
pub struct ParsedTodoEntry {
    pub due_date: Option<NaiveDateTime>,
    pub done_date: Option<NaiveDateTime>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: TodoStatus,
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
    status: Option<TodoStatus>,
}

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

pub struct TodoWriteEntry {
    pub due_date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}
