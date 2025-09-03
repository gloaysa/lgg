use chrono::{NaiveDate, NaiveTime};
use std::path::PathBuf;
use crate::QueryError;
use crate::utils::date_utils::DateFilter;

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

/// The complete result of a query.
/// Contains successfully parsed entries and any errors.
#[derive(Debug)]
pub struct JournalQueryResult {
    pub entries: Vec<JournalEntry>,
    pub errors: Vec<QueryError>,
}

#[derive(Clone, Debug, Default)]
pub struct ReadEntriesOptions<'a> {
    pub dates: Option<DateFilter>,
    pub time: Option<&'a str>,
    pub tags: Option<&'a Vec<String>>,
}
