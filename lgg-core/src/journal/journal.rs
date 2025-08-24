//! The core `Journal` struct and its associated types, providing the primary API for interaction.
use super::date_utils::time_is_in_range;
use super::format_utils::{format_day_header, format_entry_block};
use super::journal_entry::JournalEntry;
use super::parse_entries::parse_file_content;
use super::parse_input::DateFilter::{Range, Single};
use super::parse_input::{ParseOptions, parse_date_token, parse_raw_user_input, parse_time_token};
use super::path_utils::{day_file, month_dir, scan_dir_for_md_files, year_dir};
use crate::config::Config;
use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Datelike, Days, Local, NaiveDate};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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

#[derive(Debug)]
pub struct QueryTagsResult {
    pub tags: Vec<String>,
    pub errors: Vec<QueryError>,
}

#[derive(Clone, Debug, Default)]
pub struct ReadEntriesOptions<'a> {
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub time: Option<&'a str>,
    pub tags: Option<&'a Vec<String>>,
}

/// The central struct for all journal operations.
///
/// An instance of `Journal` holds the configuration and provides methods for
/// reading from and writing to the journal files.
#[derive(Debug)]
pub struct Journal {
    pub config: Config,
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
    pub fn create_entry(&self, input: &str) -> Result<JournalEntry> {
        let format_strs: Vec<&str> = self
            .config
            .input_date_formats
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let opts = ParseOptions {
            reference_date: Some(self.config.reference_date),
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

        let day_file = day_file(&self.config.journal_dir, date);
        if let Some(parent) = day_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent directory {}", parent.display()))?;
        }

        let is_new = !day_file.exists();
        let header = format_day_header(&self.config.journal_date_format, date);
        let block = format_entry_block(&parsed_input.title, &parsed_input.body, Some(time));

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&day_file)
            .with_context(|| format!("opening {}", day_file.display()))?;

        if is_new {
            writeln!(file, "{header}\n")
                .with_context(|| format!("writing day header to {}", day_file.display()))?;
            write!(file, "{block}")
                .with_context(|| format!("appending entry to {}", day_file.display()))?;
        } else {
            // Read the file and find, based on time, where to put the new entry.
            let new_entry = JournalEntry {
                date,
                time,
                title: parsed_input.title.to_string(),
                body: parsed_input.body.to_string(),
                tags: parsed_input.tags.clone(),
                path: day_file.clone(),
            };
            let mut result = self.parse_file(&day_file);

            if !result.errors.is_empty() {
                // TODO: This function should be able to gracefully return errors.
                // We need to let the user know that there's a problem with their file.
                // We still append the entry because is better than simply erroring out.
                writeln!(file, "{header}\n")
                    .with_context(|| format!("writing day header to {}", day_file.display()))?;
                write!(file, "{block}")
                    .with_context(|| format!("appending entry to {}", day_file.display()))?;

                return Ok(new_entry);
            }

            result.entries.push(new_entry);
            result.entries.sort_by_key(|e| e.time);
            let mut new_content = header;
            for entry in result.entries {
                let block = format_entry_block(&entry.title, &entry.body, Some(entry.time));

                new_content.push_str(&block);
            }

            fs::write(&day_file, new_content)?;
        }

        Ok(JournalEntry {
            date,
            time,
            title: parsed_input.title,
            body: parsed_input.body,
            tags: parsed_input.tags,
            path: day_file,
        })
    }

    /// Reads and returns all entries, the results can be filtered by `options`.
    ///
    /// This is the primary query function for retrieving entries. It is designed to be
    /// resilient, returning a [`QueryResult`] that contains both parsed entries and
    /// any errors that occurred.
    ///
    /// # Arguments
    ///
    /// * `options` - Those are filtering options, of type time, date and tags. If none are passed, the function returns all entries.
    pub fn read_entries(&self, options: &ReadEntriesOptions) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        if let Some(start_date) = options.start_date {
            let results = self.search_files_by_date(start_date, options.end_date);
            entries.extend(results.entries);
            errors.extend(results.errors);
        } else {
            let results = self.search_all_files();
            entries.extend(results.entries);
            errors.extend(results.errors);
        }

        entries.sort_by_key(|k| k.date);

        if let Some(time) = &options.time {
            if let Some(parsed_time) = parse_time_token(time) {
                entries = entries
                    .into_iter()
                    .filter(|entry| time_is_in_range(parsed_time, entry.time))
                    .collect();
            }
        }

        if let Some(tags) = &options.tags {
            let found_tags: Vec<String> = tags
                .into_iter()
                .map(|t| t.trim().to_ascii_lowercase())
                .collect();

            entries = entries
                .into_iter()
                .filter(|e| found_tags.iter().any(|t| e.tags.contains(t)))
                .collect();
        }

        QueryResult { entries, errors }
    }

    pub fn search_all_tags(&self) -> QueryTagsResult {
        let mut tags: Vec<String> = Vec::new();
        let mut errors = Vec::new();

        if let Ok(files) = scan_dir_for_md_files(&self.config.journal_dir) {
            for file in files {
                let parse_result = self.parse_file(&file);
                for entry in parse_result.entries {
                    tags.extend(entry.tags);
                }
                errors.extend(parse_result.errors);
            }
        }

        tags = tags
            .iter()
            .map(|mat| mat.as_str().to_string().trim().to_ascii_lowercase())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();

        QueryTagsResult { tags, errors }
    }

    fn search_all_files(&self) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();

        if let Ok(files) = scan_dir_for_md_files(&self.config.journal_dir) {
            for file in files {
                let parse_result = self.parse_file(&file);
                entries.extend(parse_result.entries);
                errors.extend(parse_result.errors);
            }
        }

        QueryResult { entries, errors }
    }

    fn search_files_by_date(&self, start_date: &str, end_date: Option<&str>) -> QueryResult {
        let format_strs: Vec<&str> = self
            .config
            .input_date_formats
            .iter()
            .map(AsRef::as_ref)
            .collect();
        let opts = ParseOptions {
            reference_date: Some(self.config.reference_date),
            formats: Some(&format_strs),
        };
        let mut entries = Vec::new();
        let mut errors = Vec::new();

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
        let day_file = day_file(&self.config.journal_dir, date);
        if day_file.exists() {
            let parse_result = self.parse_file(&day_file);
            entries.extend(parse_result.entries);
            errors.extend(parse_result.errors);
        }

        QueryResult { entries, errors }
    }

    fn read_range_date_entry(&self, range_start: NaiveDate, range_end: NaiveDate) -> QueryResult {
        let mut entries = Vec::new();
        let mut errors = Vec::new();

        if range_start > range_end {
            errors.push(QueryError::InvalidDate {
                input: format!("start date: {} end date: {}", range_start, range_start),
                error: "End date can't be before a date before the starting date.".to_string(),
            });
            return QueryResult { entries, errors };
        }

        let mut start_date = range_start;

        while start_date <= range_end {
            let year_dir = year_dir(&self.config.journal_dir, start_date);
            if !year_dir.exists() {
                let next_year = start_date.year() + 1;
                start_date = NaiveDate::from_ymd_opt(next_year, 1, 1).unwrap();
                continue;
            }
            let month_dir = month_dir(&self.config.journal_dir, start_date);
            if !month_dir.exists() {
                let year = start_date.year();
                let next_month = start_date.month() + 1;
                start_date = NaiveDate::from_ymd_opt(year, next_month, 1).unwrap();
                continue;
            }
            let day_file = day_file(&self.config.journal_dir, start_date);
            if day_file.exists() {
                let parse_result = self.parse_file(&day_file);
                entries.extend(parse_result.entries);
                errors.extend(parse_result.errors);
            }

            start_date = start_date.checked_add_days(Days::new(1)).unwrap();
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

    fn mk_journal_with_default(reference_date: Option<NaiveDate>) -> (Journal, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("lgg");
        let config = mk_config(root, reference_date);

        let j = Journal::with_config(config).unwrap();
        (j, tmp)
    }

    #[test]
    fn save_entry_creates_day_file_and_appends() {
        let (j, _tmp) = mk_journal_with_default(None);
        let res = j.create_entry("Test entry. With body.").unwrap();
        let expected = day_file(&j.config.journal_dir, res.date);
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
        let (j, _tmp) = mk_journal_with_default(None);
        let _ = j.create_entry("today: First entry.").unwrap();
        let _ = j.create_entry("today: Second entry.").unwrap();
        let options = ReadEntriesOptions {
            start_date: Some("today"),
            ..Default::default()
        };

        let result = j.read_entries(&options);
        assert!(result.errors.is_empty());
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "First entry.");
        assert_eq!(result.entries[1].title, "Second entry.");
    }

    #[test]
    fn read_entries_date_range_success() {
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // Monday
        let (j, _tmp) = mk_journal_with_default(Some(anchor));

        j.create_entry("27/07/2025: Previous week").unwrap();
        j.create_entry("saturday: First entry.").unwrap();
        j.create_entry("yesterday: Second entry!").unwrap();
        j.create_entry("today: This week?").unwrap();
        j.create_entry("11/08/2025: Next week").unwrap();

        let options = ReadEntriesOptions {
            start_date: Some("last week"),
            ..Default::default()
        };
        let last_week = j.read_entries(&options);
        assert!(last_week.errors.is_empty());
        assert_eq!(last_week.entries.len(), 2);
        assert_eq!(last_week.entries[0].title, "First entry.");
        assert_eq!(last_week.entries[1].title, "Second entry!");
        let options = ReadEntriesOptions {
            start_date: Some("this week"),
            ..Default::default()
        };
        let this_week = j.read_entries(&options);
        assert!(this_week.errors.is_empty());
        assert_eq!(this_week.entries.len(), 1);
        assert_eq!(this_week.entries[0].title, "This week?");
    }

    #[test]
    fn read_entries_time_range_success() {
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // Monday
        let (j, _tmp) = mk_journal_with_default(Some(anchor));

        j.create_entry("today at morning: Morning entry").unwrap();
        j.create_entry("today at night: Night entry.").unwrap();
        j.create_entry("today at night: Second night entry!")
            .unwrap();
        j.create_entry("today at noon: Noon entry").unwrap();
        j.create_entry("today at morning: Another morning entry.")
            .unwrap();

        let options = ReadEntriesOptions {
            start_date: Some("today"),
            time: Some("night"),
            ..Default::default()
        };
        let night = j.read_entries(&options);
        assert!(night.errors.is_empty());
        assert_eq!(night.entries.len(), 2);
        assert_eq!(night.entries[0].title, "Night entry.");
        assert_eq!(night.entries[1].title, "Second night entry!");

        let options = ReadEntriesOptions {
            start_date: Some("today"),
            time: Some("noon"),
            ..Default::default()
        };
        let noon = j.read_entries(&options);
        assert!(noon.errors.is_empty());
        assert_eq!(noon.entries.len(), 1);
        assert_eq!(noon.entries[0].title, "Noon entry");
    }

    #[test]
    fn read_entries_with_tag_and_date_filter() {
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // Monday
        let (j, _tmp) = mk_journal_with_default(Some(anchor));
        let expected_tags: Vec<String> = vec!["@test".to_string()];

        j.create_entry("27/07/2025: Previous week with @test tag")
            .unwrap();
        j.create_entry("today: This week with @test tag").unwrap();
        j.create_entry("tomorrow: This week with @test tag too.")
            .unwrap();
        j.create_entry("11/08/2025: Next week with @test tag.")
            .unwrap();

        let options = ReadEntriesOptions {
            start_date: Some("this week"),
            tags: Some(&expected_tags),
            ..Default::default()
        };
        let results = j.read_entries(&options);
        assert!(results.errors.is_empty());
        assert_eq!(results.entries.len(), 2);

        assert_eq!(results.entries[0].tags.len(), 1);
        assert!(results.entries[0].tags.contains(&"@test".to_string()));
    }

    #[test]
    fn read_all_files_to_find_tags() {
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // A day in 2025
        let (j, _tmp) = mk_journal_with_default(Some(anchor));
        let expected_tags: Vec<String> = vec![
            "@future".to_string(),
            "@past".to_string(),
            "@double_tag".to_string(),
        ];

        j.create_entry("27/07/2020: Day in the past with @past tag.")
            .unwrap();
        j.create_entry("27/07/2048: Day way in the future with @future. Has @double_tag in body.")
            .unwrap();
        j.create_entry("yesterday: Has a tag in body. This is another @double_tag")
            .unwrap();
        j.create_entry("today: No tag.").unwrap();

        let options = ReadEntriesOptions {
            tags: Some(&expected_tags),
            ..Default::default()
        };
        let results = j.read_entries(&options);
        assert!(results.errors.is_empty());
        assert_eq!(results.entries.len(), 3);

        assert_eq!(results.entries[0].tags.len(), 1);
        assert!(results.entries[0].tags.contains(&"@past".to_string()));

        assert_eq!(results.entries[1].tags.len(), 1);
        assert!(results.entries[1].tags.contains(&"@double_tag".to_string()));

        assert_eq!(results.entries[2].tags.len(), 2);
        assert!(results.entries[2].tags.contains(&"@future".to_string()));
        assert!(results.entries[2].tags.contains(&"@double_tag".to_string()));
    }

    #[test]
    fn find_all_tags() {
        let anchor = NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(); // A day in 2025
        let (j, _tmp) = mk_journal_with_default(Some(anchor));

        j.create_entry("27/07/2020: Day in the past with @past tag.")
            .unwrap();
        j.create_entry("27/07/2048: Day way in the future with @future. Has @double_tag in body.")
            .unwrap();
        j.create_entry("yesterday: Has a tag in body. This is another @double_tag")
            .unwrap();

        let results = j.search_all_tags();
        assert!(results.errors.is_empty());
        assert_eq!(results.tags.len(), 3);

        assert!(results.tags.contains(&"@past".to_string()));
        assert!(results.tags.contains(&"@double_tag".to_string()));
        assert!(results.tags.contains(&"@future".to_string()));
    }

    #[test]
    fn read_entries_on_date_with_no_file() {
        let (j, _tmp) = mk_journal_with_default(None);
        let options = ReadEntriesOptions {
            start_date: Some("yesterday"),
            ..Default::default()
        };
        let result = j.read_entries(&options);
        assert!(result.errors.is_empty());
        assert!(result.entries.is_empty());
    }

    #[test]
    fn read_entries_with_invalid_date_string() {
        let (j, _tmp) = mk_journal_with_default(None);
        let options = ReadEntriesOptions {
            start_date: Some("not-a-date"),
            ..Default::default()
        };
        let result = j.read_entries(&options);
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::InvalidDate { .. }));
    }

    #[test]
    fn read_entries_with_malformed_file() {
        let (j, _tmp) = mk_journal_with_default(None);
        let date = Local::now().date_naive();
        let path = day_file(&j.config.journal_dir, date);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "this file is not valid").unwrap();

        let options = ReadEntriesOptions {
            start_date: Some("today"),
            ..Default::default()
        };
        let result = j.read_entries(&options);
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::FileError { .. }));
    }
}
