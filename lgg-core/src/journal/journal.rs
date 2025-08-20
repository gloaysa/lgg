//! The core `Journal` struct and its associated types, providing the primary API for interaction.
use super::date_utils::get_dates_in_range;
use super::format_utils::{format_day_header, format_entry_block};
use super::journal_entry::JournalEntry;
use super::parse_entries::parse_file_content;
use super::parse_input::DateFilter::{Range, Single};
use super::parse_input::{ParseOptions, parse_date_token, parse_raw_user_input};
use super::path_utils::day_path;
use crate::config::Config;
use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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
    pub entries: Vec<JournalEntry>,
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
    pub fn create_entry(
        &self,
        input: &str,
        reference_date: Option<NaiveDate>,
    ) -> Result<JournalEntry> {
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
        let parsed_input = parse_raw_user_input(input, Some(opts));
        let date = parsed_input.date;
        let time = if let Some(t) = parsed_input.time {
            t
        } else if parsed_input.explicit_date {
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
        let header = format_day_header(&self.config.journal_date_format, date);
        let block = format_entry_block(&parsed_input.title, &parsed_input.body, Some(time));

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))?;

        if is_new {
            writeln!(file, "{header}\n")
                .with_context(|| format!("writing day header to {}", path.display()))?;
            write!(file, "{block}")
                .with_context(|| format!("appending entry to {}", path.display()))?;
        } else {
            // Read the file and find, based on time, where to put the new entry.
            let new_entry = JournalEntry {
                date,
                time,
                title: parsed_input.title.to_string(),
                body: parsed_input.body.to_string(),
                tags: parsed_input.tags.clone(),
                path: path.clone(),
            };
            let mut result = self.parse_file(&path);

            if !result.errors.is_empty() {
                // TODO: This function should be able to gracefully return errors.
                // We need to let the user know that there's a problem with their file.
                // We still append the entry because is better than simply erroring out.
                writeln!(file, "{header}\n")
                    .with_context(|| format!("writing day header to {}", path.display()))?;
                write!(file, "{block}")
                    .with_context(|| format!("appending entry to {}", path.display()))?;

                return Ok(new_entry);
            }

            result.entries.push(new_entry);
            result.entries.sort_by_key(|e| e.time);
            let mut new_content = header;
            for entry in result.entries {
                let block = format_entry_block(&entry.title, &entry.body, Some(entry.time));

                new_content.push_str(&block);
            }

            fs::write(&path, new_content)?;
        }

        Ok(JournalEntry {
            date,
            time,
            title: parsed_input.title,
            body: parsed_input.body,
            tags: parsed_input.tags,
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

    /// Parses the entire content of a daily journal file.
    ///
    /// This function acts as the main entry point for file parsing.
    /// It expects the file to have a specific format:
    /// - A mandatory header on the first line (e.g., `# Friday, 15 Aug 2025`).
    /// - Zero or more entry blocks, each starting with `## HH:MM - Title`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read.
    ///
    /// # Returns
    ///
    /// A `QueryResult` containing a `Vec<JournalEntry>` and `errors` in case some where found,
    /// if the path isn't valid, the file is empty or the header is malformed or a specific entry is invalid.
    pub fn parse_file(&self, path: &PathBuf) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        if !path.exists() {
            errors.push(QueryError::FileError {
                path: path.clone(),
                error: anyhow!(format!("File does not exist in path: {}", path.display())),
            });
            return QueryResult { entries, errors };
        }
        match fs::read_to_string(&path) {
            Ok(file_content) => {
                let parse_result = parse_file_content(&file_content);
                for entry in parse_result.entries {
                    entries.push(JournalEntry {
                        date: entry.date,
                        time: entry.time,
                        title: entry.title,
                        body: entry.body,
                        tags: entry.tags,
                        path: path.clone(),
                    });
                }

                for error in parse_result.errors {
                    errors.push(QueryError::FileError {
                        path: path.clone(),
                        error: anyhow!(error),
                    });
                }
            }
            Err(error) => {
                errors.push(QueryError::FileError {
                    path: path.clone(),
                    error: error.into(),
                });
            }
        }
        QueryResult { entries, errors }
    }

    fn read_single_date_entry(&self, date: NaiveDate) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let path = day_path(&self.config.journal_dir, date);
        if path.exists() {
            let parse_result = self.parse_file(&path);
            entries.extend(parse_result.entries);
            errors.extend(parse_result.errors);
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
                let parse_result = self.parse_file(&path);
                entries.extend(parse_result.entries);
                errors.extend(parse_result.errors);
            }
        }

        QueryResult { entries, errors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mk_config;
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
        assert_eq!(result.entries[0].title, "First entry.");
        assert_eq!(result.entries[1].title, "Second entry.");
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
        j.create_entry("yesterday: Second entry!", Some(anchor))
            .unwrap();
        j.create_entry("today: This week?", Some(anchor)).unwrap();
        j.create_entry("11/08/2025: Next week", Some(anchor))
            .unwrap();

        let last_week = j.read_entries("last week", None, Some(anchor));
        assert!(last_week.errors.is_empty());
        assert_eq!(last_week.entries.len(), 2);
        assert_eq!(last_week.entries[0].title, "First entry.");
        assert_eq!(last_week.entries[1].title, "Second entry!");
        let this_week = j.read_entries("this week", None, Some(anchor));
        assert!(this_week.errors.is_empty());
        assert_eq!(this_week.entries.len(), 1);
        assert_eq!(this_week.entries[0].title, "This week?");
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

    #[test]
    fn header_formats_readably() {}
}
