use chrono::{NaiveDate, NaiveTime};

/// Configuration options for parsing functions.
#[derive(Copy, Clone, Debug, Default)]
pub struct ParseInputOptions<'a> {
    /// The date to use as "today" for relative keywords.
    pub reference_date: Option<NaiveDate>,
    /// A slice of `chrono` format strings to try for parsing dates.
    pub formats: Option<&'a [&'a str]>,
}

/// Parsed result of inline text (e.g., "yesterday: Title. Body").
pub struct ParsedInput {
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub title: String,
    pub body: String,
}
