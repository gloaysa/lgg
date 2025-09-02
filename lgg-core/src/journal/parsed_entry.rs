use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct ReadJournalResult {
    pub entries: Vec<ParsedJournalEntry>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedJournalEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

