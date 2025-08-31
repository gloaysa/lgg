use chrono::{NaiveDate, NaiveTime};
use std::path::PathBuf;

use crate::utils::parsed_entry::DateFilter;

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

/// Represents a non-critical issue that occurred during a query.
/// This is used to report problems (e.g., malformed files, invalid input)
/// without stopping a larger query operation.
#[derive(Debug)]
pub enum QueryError {
    InvalidDate { input: String, error: String },
    FileError { path: PathBuf, error: anyhow::Error },
}

/// The complete result of a query.
/// Contains successfully parsed entries and any errors.
#[derive(Debug)]
pub struct QueryResult {
    pub entries: Vec<JournalEntry>,
    pub errors: Vec<QueryError>,
}

/// The complete result of a query.
/// Contains successfully parsed tags and any errors.
#[derive(Debug)]
pub struct QueryTagsResult {
    pub tags: Vec<String>,
    pub errors: Vec<QueryError>,
}

#[derive(Clone, Debug, Default)]
pub struct ReadEntriesOptions<'a> {
    pub dates: Option<DateFilter>,
    pub time: Option<&'a str>,
    pub tags: Option<&'a Vec<String>>,
}
