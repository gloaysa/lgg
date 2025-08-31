use crate::{
    Config,
    journal::Journal,
    todo_list::TodoList,
    utils::{
        parse_input::{parse_date_token, parse_raw_user_input},
        parsed_entry::DateFilter,
        parsed_input::ParseInputOptions,
    },
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
    pub todos: TodoList,
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
            journal_date_format: config.journal_date_format.clone(),
            reference_date: config.reference_date,
        };
        let todos = TodoList {};
        Ok(Self {
            config,
            journal,
            todos,
        })
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
            match parsed_input.date {
                Some(_) => self.config.default_time,
                None => Local::now().time(),
            }
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

#[cfg(test)]
mod tests {
    use crate::{Config, Lgg, tests::mk_config};
    use chrono::{Local, NaiveDate, NaiveTime, Timelike};
    use tempfile::tempdir;

    fn mk_lgg_with_default(reference_date: Option<NaiveDate>) -> (Lgg, tempfile::TempDir) {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("lgg");
        let config = mk_config(root, reference_date);

        let lgg = Lgg::with_config(config).expect("lgg with config");
        (lgg, tmp)
    }

    #[test]
    fn natural_language_date_with_time() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15);
        let (lgg, _) = mk_lgg_with_default(anchor);

        let p1 = lgg
            .parse_user_input("yesterday at 6am: Note 1")
            .expect("ok");

        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 14).unwrap());
        assert_eq!(p1.time, NaiveTime::from_hms_opt(6, 0, 0).unwrap());
        assert_eq!(p1.title, "Note 1");
    }

    #[test]
    fn natural_language_time() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15);
        let (lgg, _) = mk_lgg_with_default(anchor);

        let p1 = lgg
            .parse_user_input("saturday at noon: Note 1")
            .expect("ok");

        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 09).unwrap());
        assert_eq!(p1.time, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        assert_eq!(p1.title, "Note 1");
        assert_eq!(p1.body, "");
    }

    #[test]
    fn no_date_no_time_defaults() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15);
        let now = Local::now().time();
        let (lgg, _) = mk_lgg_with_default(anchor);

        let p1 = lgg.parse_user_input("Note 1. With body.").expect("ok");

        assert_eq!(p1.date, anchor.unwrap());
        // checking that the hour is correct, enough for this test
        assert_eq!(p1.time.hour(), now.hour());
        assert_eq!(p1.title, "Note 1.");
        assert_eq!(p1.body, "With body.");
    }

    #[test]
    fn custom_format_dd_mm_yyyy() {
        let tmp = tempdir().unwrap();
        let jour_tmp_dir = tmp.path().join("lgg/journal");
        let todo_tmp_dir = tmp.path().join("lgg/todos");
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let fmts = vec!["%d-%m-%Y".to_string(), "%d/%m/%Y".to_string()];
        let default_time = NaiveTime::from_hms_opt(21, 0, 0).expect("valid time");
        let conf = Config {
            journal_dir: jour_tmp_dir,
            todo_list_dir: todo_tmp_dir,
            editor: None,
            default_time,
            reference_date: anchor,
            journal_date_format: "%A, %d %b %Y".to_string(),
            input_date_formats: fmts,
        };
        let lgg = Lgg::with_config(conf).expect("lgg created");

        let p1 = lgg
            .parse_user_input("01-08-2025: Title 1.")
            .expect("parsed ok");
        let p2 = lgg.parse_user_input("01/09/2025: Title 2.").expect("ok");

        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert_eq!(p1.time, default_time);
        assert_eq!(p1.title, "Title 1.");
        assert!(p1.body.is_empty());

        assert_eq!(p2.date, NaiveDate::from_ymd_opt(2025, 9, 1).unwrap());

        assert_eq!(p2.time, default_time);
        assert_eq!(p2.title, "Title 2.");
        assert!(p2.body.is_empty());
    }
}
