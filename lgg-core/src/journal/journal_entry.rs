use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct ParsedEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct JournalEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub path: PathBuf,
}
