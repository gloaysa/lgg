use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct TodoEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub path: PathBuf,
}
