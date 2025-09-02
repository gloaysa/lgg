use std::path::PathBuf;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug)]
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
