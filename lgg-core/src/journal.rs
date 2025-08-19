//! The core `Journal` struct and its associated types, providing the primary API for interaction.

use crate::config::Config;
use crate::dates::get_dates_in_range;
use crate::entry::Entry;
use crate::parse_entries::parse_day_file;
use crate::parse_input::DateFilter::{Range, Single};
use crate::parse_input::{ParseOptions, parse_date_token, parse_entry};
use crate::paths::day_path;
use crate::render::{format_day_header, format_entry_block};
use anyhow::{Context, Result, anyhow};
use chrono::{Local, NaiveDate, NaiveTime};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// A reference to a newly created entry, containing its metadata.
#[derive(Debug)]
pub struct EntryRef {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub title: String,
    pub path: PathBuf,
}

/// The central struct for all journal operations.
///
/// An instance of `Journal` holds the configuration and provides methods for
/// reading from and writing to the journal files.
#[derive(Debug)]
pub struct Journal {
    pub config: Config,
}

/// Represents a non-critical issue that occurred during a query.
///
/// This is used to report problems (e.g., malformed files, invalid input)
/// without stopping a larger query operation.
#[derive(Debug)]
pub enum QueryError {
    InvalidDate { input: String, error: String },
    FileError { path: PathBuf, error: anyhow::Error },
}

/// The complete result of a query, containing successfully parsed entries and any warnings.
#[derive(Debug)]
pub struct QueryResult {
    pub entries: Vec<Entry>,
    pub errors: Vec<QueryError>,
}

impl Journal {
    /// Creates a new `Journal` instance, loading configuration from standard paths.
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Self::with_config(config)
    }

    /// Creates a new `Journal` instance with a specific `Config`.
    ///
    /// This also ensures that the journal's root directory exists.
    pub fn with_config(config: Config) -> Result<Self> {
        fs::create_dir_all(&config.journal_dir)
            .with_context(|| format!("creating {}", config.journal_dir.display()))?;
        Ok(Self { config })
    }

    /// Parses and saves a new entry from a single string.
    ///
    /// - Parses `<date>:` (optional) and title/body from the input string.
    /// - Ensures the target directory (`{root}/YYYY/MM/`) exists.
    /// - Creates or appends to the daily file (`{root}/YYYY/MM/YYYY-MM-DD.md`).
    ///
    /// Returns an [`EntryRef`] with metadata about the saved entry.
    ///
    /// # Arguments
    ///
    /// * `input` - a string with the user's input (eg 'yesterday: I did some coding.').
    /// * `reference_date` - Optional argument that will allow to modify the date of reference for
    /// relatives dates (yesterday, tomorrow...)
    pub fn create_entry(&self, input: &str, reference_date: Option<NaiveDate>) -> Result<EntryRef> {
        let format_strs: Vec<&str> = self
            .config
            .input_date_formats
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let opts = ParseOptions {
            reference_date,
            formats: Some(&format_strs),
        };
        let parsed = parse_entry(input, Some(opts));
        let date = parsed.date;
        let time = if let Some(t) = parsed.time {
            t
        } else if parsed.explicit_date {
            self.config.default_time
        } else {
            Local::now().time()
        };

        let path = day_path(&self.config.journal_dir, date);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent directory {}", parent.display()))?;
        }

        let is_new = !path.exists();
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))?;

        if is_new {
            let header = format_day_header(date, &self.config.journal_date_format);
            writeln!(f, "{header}\n")
                .with_context(|| format!("writing day header to {}", path.display()))?;
        } else {
            // Add a blank line before new entry
            writeln!(f).ok();
        }

        let block = format_entry_block(&parsed.title, &parsed.body, Some(time));
        write!(f, "{block}").with_context(|| format!("appending entry to {}", path.display()))?;

        Ok(EntryRef {
            date,
            time,
            title: parsed.title,
            path,
        })
    }

    /// Reads all entries for a specific day.
    ///
    /// This is the primary query function for retrieving entries. It is designed to be
    /// resilient, returning a [`QueryResult`] that contains both parsed entries and
    /// any warnings that occurred.
    ///
    /// # Arguments
    ///
    /// * `date_str` - A string that can be parsed into a date (e.g., "yesterday", "2025-08-15").
    /// * `reference_date` - Optional argument that will allow to modify the date of reference for
    /// relatives dates (yesterday, tomorrow...)
    pub fn read_entries(
        &self,
        start_date: &str,
        end_date: Option<&str>,
        reference_date: Option<NaiveDate>,
    ) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let format_strs: Vec<&str> = self
            .config
            .input_date_formats
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let opts = ParseOptions {
            reference_date,
            formats: Some(&format_strs),
        };

        if let Some(date) = parse_date_token(start_date, end_date, Some(opts)) {
            match date {
                Single(s_date) => {
                    let result = self.read_single_date_entry(s_date);
                    entries = result.entries;
                    errors = result.errors;
                }
                Range(s_date, e_date) => {
                    let result = self.read_range_date_entry(s_date, e_date);
                    entries = result.entries;
                    errors = result.errors;
                }
            }
        } else {
            errors.push(QueryError::InvalidDate {
                input: start_date.to_string(),
                error: "Not a valid date or keyword.".to_string(),
            });
        }
        QueryResult { entries, errors }
    }

    fn read_single_date_entry(&self, date: NaiveDate) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let path = day_path(&self.config.journal_dir, date);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(file_content) => {
                    let parse_result = parse_day_file(&file_content);
                    entries.extend(parse_result.entries);

                    for error in parse_result.errors {
                        errors.push(QueryError::FileError {
                            path: path.clone(),
                            error: anyhow!(error),
                        });
                    }
                }
                Err(error) => {
                    errors.push(QueryError::FileError {
                        path,
                        error: error.into(),
                    });
                }
            }
        }
        QueryResult { entries, errors }
    }

    fn read_range_date_entry(&self, start_date: NaiveDate, end_date: NaiveDate) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let range = get_dates_in_range(start_date, end_date);

        for date in range {
            let path = day_path(&self.config.journal_dir, date);
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(file_content) => {
                        let parse_result = parse_day_file(&file_content);
                        entries.extend(parse_result.entries);

                        for error in parse_result.errors {
                            errors.push(QueryError::FileError {
                                path: path.clone(),
                                error: anyhow!(error),
                            });
                        }
                    }
                    Err(error) => {
                        errors.push(QueryError::FileError {
                            path,
                            error: error.into(),
                        });
                    }
                }
            }
        }

        QueryResult { entries, errors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::mk_config;
    use crate::paths::day_path;
    use std::fs;
    use tempfile::tempdir;

    fn mk_journal_with_default() -> (Journal, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("lgg");
        let cfg = mk_config(root);
        let j = Journal::with_config(cfg).unwrap();
        (j, tmp)
    }

    #[test]
    fn save_entry_creates_day_file_and_appends() {
        let (j, _tmp) = mk_journal_with_default();
        let res = j.create_entry("Test entry. With body.", None).unwrap();
        let expected = day_path(&j.config.journal_dir, res.date);
        assert_eq!(res.path, expected);
        assert!(res.path.exists());

        let s = fs::read_to_string(&res.path).unwrap();
        assert!(s.starts_with("# "));
        assert!(s.contains("## "));
        assert!(s.contains("Test entry"));
    }

    // --- Tests for read_entries ---

    #[test]
    fn read_entries_single_date_success() {
        let (j, _tmp) = mk_journal_with_default();
        let _ = j.create_entry("today: First entry.", None).unwrap();
        let _ = j.create_entry("today: Second entry.", None).unwrap();

        let result = j.read_entries("today", None, None);
        assert!(result.errors.is_empty());
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "First entry");
        assert_eq!(result.entries[1].title, "Second entry");
    }

    #[test]
    fn read_entries_range_date_success() {
        let (j, _tmp) = mk_journal_with_default();
        println!("{}", j.config.journal_dir.as_os_str().display());
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // Monday

        j.create_entry("27/07/2025: Previous week", Some(anchor))
            .unwrap();
        j.create_entry("saturday: First entry.", Some(anchor))
            .unwrap();
        j.create_entry("yesterday: Second entry.", Some(anchor))
            .unwrap();
        j.create_entry("today: This week.", Some(anchor)).unwrap();
        j.create_entry("11/08/2025: Next week", Some(anchor))
            .unwrap();

        let last_week = j.read_entries("last week", None, Some(anchor));
        assert!(last_week.errors.is_empty());
        assert_eq!(last_week.entries.len(), 2);
        assert_eq!(last_week.entries[0].title, "First entry");
        assert_eq!(last_week.entries[1].title, "Second entry");
        let this_week = j.read_entries("this week", None, Some(anchor));
        assert!(this_week.errors.is_empty());
        assert_eq!(this_week.entries.len(), 1);
        assert_eq!(this_week.entries[0].title, "This week");
    }

    #[test]
    fn read_entries_on_date_with_no_file() {
        let (j, _tmp) = mk_journal_with_default();
        let result = j.read_entries("yesterday", None, None);
        assert!(result.errors.is_empty());
        assert!(result.entries.is_empty());
    }

    #[test]
    fn read_entries_with_invalid_date_string() {
        let (j, _tmp) = mk_journal_with_default();
        let result = j.read_entries("not-a-date", None, None);
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::InvalidDate { .. }));
    }

    #[test]
    fn read_entries_with_malformed_file() {
        let (j, _tmp) = mk_journal_with_default();
        let date = Local::now().date_naive();
        let path = day_path(&j.config.journal_dir, date);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "this file is not valid").unwrap();

        let result = j.read_entries("today", None, None);
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::FileError { .. }));
    }
}
