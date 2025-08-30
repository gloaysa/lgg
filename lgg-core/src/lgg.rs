use crate::{
    journal::Journal,
    utils::{
        parse_input::{parse_date_token, parse_raw_user_input},
        parsed_entry::DateFilter,
        parsed_input::ParseInputOptions,
    },
    Config,
};
use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, NaiveTime};
use std::fs;

pub struct ParsedInput {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub body: String,
}

pub struct Lgg {
    pub config: Config,
    pub journal: Journal,
}
impl Lgg {
    /// Creates a new `Lgg` instance, loading configuration from standard paths.
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Self::with_config(config)
    }

    /// Creates a new `Lgg` instance with a specific `Config`.
    ///
    /// This also ensures that the root writing directory exists.
    pub fn with_config(config: Config) -> Result<Self> {
        fs::create_dir_all(&config.journal_dir)
            .with_context(|| format!("creating journal dir {}", config.journal_dir.display()))?;
        fs::create_dir_all(&config.todo_list_dir)
            .with_context(|| format!("creating todos dir {}", config.journal_dir.display()))?;

        let journal = Journal {
            journal_dir: config.journal_dir.clone(),
            default_time: config.default_time,
            journal_date_format: config.journal_date_format.clone(),
            input_date_formats: config.input_date_formats.clone(),
            reference_date: config.reference_date,
        };
        Ok(Self { config, journal })
    }

    /// This function orchestrates the parsing of a complete user input, which may
    /// contain a date/time prefix, a title, and a body. It handles the logic for splitting
    /// the prefix from the content and then the title from the body.
    /// The input would look something like this: `(optional DATE-TIME): some title. some body.`
    pub fn parse_user_input(&self, input: &str) -> Result<ParsedInput> {
        let format_strs: Vec<&str> = self
            .config
            .input_date_formats
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let opts = ParseInputOptions {
            reference_date: Some(self.config.reference_date),
            formats: Some(&format_strs),
        };
        let parsed_input = parse_raw_user_input(input, Some(opts));
        let date = parsed_input.date.unwrap_or(self.config.reference_date);
        let time = if let Some(t) = parsed_input.time {
            t
        } else {
            Local::now().time()
        };

        Ok(ParsedInput {
            date,
            time,
            title: parsed_input.title,
            body: parsed_input.body,
        })
    }

    pub fn parse_dates(&self, start_date: &str, end_date: Option<&str>) -> Option<DateFilter> {
        let format_strs: Vec<&str> = self
            .config
            .input_date_formats
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let opts = ParseInputOptions {
            reference_date: Some(self.config.reference_date),
            formats: Some(&format_strs),
        };
        parse_date_token(start_date, end_date, Some(opts))
    }
}
