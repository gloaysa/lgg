use crate::utils::date_utils::DateFilter;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::path::PathBuf;
use crate::QueryError;

#[derive(Clone, Debug)]
pub enum TodoStatus {
    Pending,
    Done,
}

#[derive(Debug, Clone)]
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

/// The complete result of a query.
/// Contains successfully parsed entries and any errors.
#[derive(Debug)]
pub struct TodoQueryResult {
    pub todos: Vec<TodoEntry>,
    pub errors: Vec<QueryError>,
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
