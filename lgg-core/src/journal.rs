//! The core `Journal` struct and its associated types, providing the primary API for interaction.

use crate::config::Config;
use crate::entry::Entry;
use crate::parse_entries::parse_day_file;
use crate::parse_input::{parse_date_token, parse_entry};
use crate::paths::day_path;
use crate::render::{format_day_header, format_entry_block};
use anyhow::{Context, Result};
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
    pub fn save_entry(&self, input: &str) -> Result<EntryRef> {
        let parsed = parse_entry(input, None);
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
            let header = format_day_header(date, &self.config);
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
    pub fn read_entries_on_date(&self, date_str: &str) -> QueryResult {
        let mut entries = Vec::new();
        let mut warnings = Vec::new();

        if let Some(date) = parse_date_token(date_str, None) {
            let path = day_path(&self.config.journal_dir, date);
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => match parse_day_file(&content) {
                        Ok(parsed_entries) => {
                            entries.extend(parsed_entries);
                        }
                        Err(error) => {
                            warnings.push(QueryError::FileError { path, error });
                        }
                    },
                    Err(error) => {
                        warnings.push(QueryError::FileError {
                            path,
                            error: error.into(),
                        });
                    }
                }
            }
        } else {
            warnings.push(QueryError::InvalidDate {
                input: date_str.to_string(),
                error: "Not a valid date or keyword.".to_string(),
            });
        }
        QueryResult {
            entries,
            errors: warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::mk_config;
    use crate::paths::day_path;
    use std::fs;
    use tempfile::tempdir;

    fn mk_journal_with_default(default_time: Option<NaiveTime>) -> (Journal, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("lgg");
        let mut cfg = mk_config(root);
        if let Some(time) = default_time {
            cfg.default_time = time;
        }
        let j = Journal::with_config(cfg).unwrap();
        (j, tmp)
    }

    #[test]
    fn save_entry_creates_day_file_and_appends() {
        let (j, _tmp) = mk_journal_with_default(None);
        let res = j.save_entry("Test entry. With body.").unwrap();
        let expected = day_path(&j.config.journal_dir, res.date);
        assert_eq!(res.path, expected);
        assert!(res.path.exists());

        let s = fs::read_to_string(&res.path).unwrap();
        assert!(s.starts_with("# "));
        assert!(s.contains("## "));
        assert!(s.contains("Test entry"));
    }

    // --- Tests for read_entries_on_date ---

    #[test]
    fn read_entries_success() {
        let (j, _tmp) = mk_journal_with_default(None);
        let _ = j.save_entry("today: First entry.").unwrap();
        let _ = j.save_entry("today: Second entry.").unwrap();

        let result = j.read_entries_on_date("today");
        assert!(result.errors.is_empty());
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "First entry");
        assert_eq!(result.entries[1].title, "Second entry");
    }

    #[test]
    fn read_entries_on_date_with_no_file() {
        let (j, _tmp) = mk_journal_with_default(None);
        let result = j.read_entries_on_date("yesterday");
        assert!(result.errors.is_empty());
        assert!(result.entries.is_empty());
    }

    #[test]
    fn read_entries_with_invalid_date_string() {
        let (j, _tmp) = mk_journal_with_default(None);
        let result = j.read_entries_on_date("not-a-date");
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::InvalidDate { .. }));
    }

    #[test]
    fn read_entries_with_malformed_file() {
        let (j, _tmp) = mk_journal_with_default(None);
        let date = Local::now().date_naive();
        let path = day_path(&j.config.journal_dir, date);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "this file is not valid").unwrap();

        let result = j.read_entries_on_date("today");
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::FileError { .. }));
    }
}
