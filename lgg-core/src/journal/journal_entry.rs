use chrono::{NaiveDate, NaiveTime};
use std::path::PathBuf;

#[derive(Debug)]
pub struct JournalEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub path: PathBuf,
}

/// Properties to create a new JournalEntry
pub struct JournalWriteEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}
