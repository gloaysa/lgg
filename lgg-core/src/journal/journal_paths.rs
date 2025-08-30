use std::path::{Path, PathBuf};

use chrono::NaiveDate;

pub fn year_folder_name(date: NaiveDate) -> String {
    format!("{}", date.format("%Y"))
}

pub fn month_folder_name(date: NaiveDate) -> String {
    format!("{}", date.format("%m"))
}

pub fn day_file_name(date: NaiveDate) -> String {
    format!("{}.md", date.format("%Y-%m-%d"))
}

pub fn year_dir(root: &Path, date: NaiveDate) -> PathBuf {
    root.join(year_folder_name(date))
}

pub fn month_dir(root: &Path, date: NaiveDate) -> PathBuf {
    root.join(year_folder_name(date))
        .join(month_folder_name(date))
}

pub fn day_file(root: &Path, date: NaiveDate) -> PathBuf {
    root.join(year_folder_name(date))
        .join(month_folder_name(date))
        .join(day_file_name(date))
}
