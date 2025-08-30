//! The core `Journal` struct and its associated types, providing the primary API for interaction.
use super::journal_entry::{JournalEntry, JournalWriteEntry};
use super::journal_paths::{day_file, month_dir, year_dir};
use crate::utils::date_utils::time_is_in_range;
use crate::utils::format_utils::{format_day_header, format_entry_block};
use crate::utils::parse_entries::parse_file_content;
use crate::utils::parse_input::parse_time_token;
use crate::utils::parsed_entry::DateFilter;
use crate::utils::path_utils::scan_dir_for_md_files;
use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::{Datelike, Days, NaiveDate};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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

/// The central struct for all journal operations.
///
/// An instance of `Journal` holds the configuration and provides methods for
/// reading from and writing to the journal files.
#[derive(Debug)]
pub struct Journal {
    pub journal_dir: PathBuf,
    pub journal_date_format: String,
    /// The date to use as "today" for relative keywords.
    pub reference_date: NaiveDate,
}
impl Journal {
    /// Parses and saves a new entry from a single string.
    /// Creates or appends to the daily file (`{root}/YYYY/MM/YYYY-MM-DD.md`).
    /// Returns an [`JournalEntry`] with metadata about the saved entry.
    pub fn create_entry(&self, input: JournalWriteEntry) -> Result<JournalEntry> {
        let date = input.date;
        let time = input.time;
        let day_file = day_file(&self.journal_dir, date);
        if let Some(parent) = day_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent directory {}", parent.display()))?;
        }

        let is_new = !day_file.exists();
        let header = format_day_header(&self.journal_date_format, date);
        let block = format_entry_block(&input.title, &input.body, &time);

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
                title: input.title.to_string(),
                body: input.body.to_string(),
                tags: input.tags.clone(),
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
                let block = format_entry_block(&entry.title, &entry.body, &entry.time);

                new_content.push_str(&block);
            }

            fs::write(&day_file, new_content)?;
        }

        Ok(JournalEntry {
            date,
            time,
            title: input.title,
            body: input.body,
            tags: input.tags,
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
        if let Some(dates) = options.dates {
            match dates {
                DateFilter::Single(s_date) => {
                    let result = self.read_single_date_entry(s_date);
                    entries.extend(result.entries);
                    errors.extend(result.errors);
                }
                DateFilter::Range(s_date, e_date) => {
                    let result = self.read_range_date_entry(s_date, e_date);
                    entries.extend(result.entries);
                    errors.extend(result.errors);
                }
            }
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

        if let Ok(files) = scan_dir_for_md_files(&self.journal_dir) {
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

        if let Ok(files) = scan_dir_for_md_files(&self.journal_dir) {
            for file in files {
                let parse_result = self.parse_file(&file);
                entries.extend(parse_result.entries);
                errors.extend(parse_result.errors);
            }
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
        let day_file = day_file(&self.journal_dir, date);
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
            let year_dir = year_dir(&self.journal_dir, start_date);
            if !year_dir.exists() {
                let next_year = start_date.year() + 1;
                start_date = NaiveDate::from_ymd_opt(next_year, 1, 1).unwrap();
                continue;
            }
            let month_dir = month_dir(&self.journal_dir, start_date);
            if !month_dir.exists() {
                let year = start_date.year();
                let next_month = start_date.month() + 1;
                start_date = NaiveDate::from_ymd_opt(year, next_month, 1).unwrap();
                continue;
            }
            let day_file = day_file(&self.journal_dir, start_date);
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
    use crate::config::mk_journal_config;
    use chrono::{Local, NaiveTime};
    use std::fs;
    use tempfile::tempdir;

    fn mk_journal_with_default(reference_date: Option<NaiveDate>) -> (Journal, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("lgg");
        let config = mk_journal_config(root, reference_date);

        let j = Journal {
            journal_dir: config.journal_dir,
            journal_date_format: config.journal_date_format,
            reference_date: config.reference_date,
        };
        (j, tmp)
    }

    #[test]
    fn save_entry_creates_day_file_and_appends() {
        let (j, _tmp) = mk_journal_with_default(None);
        let entry = JournalWriteEntry {
            date: Local::now().date_naive(),
            time: Local::now().time(),
            title: "Test entry.".to_string(),
            body: "With body.".to_string(),
            tags: Vec::new(),
        };
        let res = j.create_entry(entry).unwrap();
        let expected = day_file(&j.journal_dir, res.date);
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
        let entry = JournalWriteEntry {
            date: Local::now().date_naive(),
            time: Local::now().time(),
            title: "First entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        let entry2 = JournalWriteEntry {
            date: Local::now().date_naive(),
            time: Local::now().time(),
            title: "Second entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        let _ = j.create_entry(entry).unwrap();
        let _ = j.create_entry(entry2).unwrap();
        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Single(Local::now().date_naive())),
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
        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 07, 27).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Previous week".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 02).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "First entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 03).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Second entry!".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: anchor,
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "This week?".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 11).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Next week".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 07, 28).expect("valid date"),
                NaiveDate::from_ymd_opt(2025, 08, 03).expect("valid date"),
            )),
            ..Default::default()
        };
        let last_week = j.read_entries(&options);
        assert!(last_week.errors.is_empty());
        assert_eq!(last_week.entries.len(), 2);
        assert_eq!(last_week.entries[0].title, "First entry.");
        assert_eq!(last_week.entries[1].title, "Second entry!");
        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Range(
                anchor,
                NaiveDate::from_ymd_opt(2025, 08, 10).expect("valid date"),
            )),
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

        let entry = JournalWriteEntry {
            date: anchor,
            time: NaiveTime::from_hms_opt(06, 00, 00).unwrap(),
            title: "Morning entry".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: anchor,
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Night entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: anchor,
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Second night entry!".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: anchor,
            time: NaiveTime::from_hms_opt(12, 00, 00).unwrap(),
            title: "Noon entry".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: anchor,
            time: NaiveTime::from_hms_opt(07, 00, 00).unwrap(),
            title: "Another morning entry.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Single(anchor)),
            time: Some("night"),
            ..Default::default()
        };
        let night = j.read_entries(&options);
        assert!(night.errors.is_empty());
        assert_eq!(night.entries.len(), 2);
        assert_eq!(night.entries[0].title, "Night entry.");
        assert_eq!(night.entries[1].title, "Second night entry!");

        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Single(anchor)),
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

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 07, 27).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "27/07/2025: Previous week with @test tag".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 04).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "This week with @test tag".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 05).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "This week with @test tag too.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 11).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Next week with @test tag.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Range(
                anchor,
                NaiveDate::from_ymd_opt(2025, 08, 10).expect("valid date"),
            )),
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

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2020, 07, 27).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Day in the past with @past tag.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2048, 07, 27).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Day way in the future with @future. Has @double_tag in body.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 03).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Has a tag in body. This is another @double_tag".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2048, 08, 04).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "No tag.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

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

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2020, 07, 27).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Day in the past with @past tag.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2048, 07, 27).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Day way in the future with @future. Has @double_tag in body.".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

        let entry = JournalWriteEntry {
            date: NaiveDate::from_ymd_opt(2025, 08, 03).unwrap(),
            time: NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
            title: "Has a tag in body. This is another @double_tag".to_string(),
            body: "".to_string(),
            tags: Vec::new(),
        };
        j.create_entry(entry).unwrap();

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
            dates: Some(DateFilter::Single(
                NaiveDate::from_ymd_opt(2025, 08, 03).expect("valid date"),
            )),
            ..Default::default()
        };
        let result = j.read_entries(&options);
        assert!(result.errors.is_empty());
        assert!(result.entries.is_empty());
    }

    #[test]
    fn read_entries_with_malformed_file() {
        let (j, _tmp) = mk_journal_with_default(None);
        let date = Local::now().date_naive();
        let path = day_file(&j.journal_dir, date);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "this file is not valid").unwrap();

        let options = ReadEntriesOptions {
            dates: Some(DateFilter::Single(Local::now().date_naive())),
            ..Default::default()
        };
        let result = j.read_entries(&options);
        assert!(result.entries.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(&result.errors[0], QueryError::FileError { .. }));
    }
}
