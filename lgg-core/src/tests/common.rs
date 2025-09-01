use crate::Config;
use chrono::{Local, NaiveDate, NaiveTime};
use std::path::PathBuf;

/// Test helper to create a default `Config` for testing purposes.
///
/// This is the single source of truth for test configuration.
/// If you add a field to `Config`, you only need to update it here.
pub fn mk_config(tmp_dir: PathBuf, reference_date: Option<NaiveDate>) -> Config {
    Config {
        journal_dir: tmp_dir.clone(),
        todo_list_dir: tmp_dir.clone(),
        editor: None,
        default_time: NaiveTime::from_hms_opt(21, 0, 0).expect("valid time"),
        reference_date: reference_date.unwrap_or(Local::now().date_naive()),
        journal_date_format: "%A, %d %b %Y".to_string(),
        todo_datetime_format: "%d/%b/%Y %H:%M".to_string(),
        input_date_formats: ["%d/%m/%Y".to_string()].to_vec(),
    }
}
