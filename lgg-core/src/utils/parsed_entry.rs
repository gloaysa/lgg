use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct ReadResult {
    pub entries: Vec<ParsedEntry>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}
/// The result of parsing a date string, which can be a single day or a range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DateFilter {
    Single(NaiveDate),
    Range(NaiveDate, NaiveDate),
}

/// The result of parsing a time string, which can be a single time or a range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeFilter {
    Single(NaiveTime),
    Range(NaiveTime, NaiveTime),
}
