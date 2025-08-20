use chrono::NaiveDate;
use std::path::{Path, PathBuf};

pub fn day_file_name(date: NaiveDate) -> String {
    format!("{}.md", date.format("%Y-%m-%d"))
}

pub fn day_dir(root: &Path, date: NaiveDate) -> PathBuf {
    root.join(date.format("%Y").to_string())
        .join(date.format("%m").to_string())
}

pub fn day_path(root: &Path, date: NaiveDate) -> PathBuf {
    day_dir(root, date).join(day_file_name(date))
}
