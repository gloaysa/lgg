use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub enum TodoStatus {
    Pending,
    Done,
}

#[derive(Debug)]
pub struct TodoEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub path: PathBuf,
    pub status: TodoStatus,
}

pub struct TodoWriteEntry {
    pub due_date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}
