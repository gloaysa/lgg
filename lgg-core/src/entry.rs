use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct Entry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}
