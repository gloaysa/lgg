use std::fs::{self};
use std::path::PathBuf;

use crate::utils::parsed_input::ParseInputOptions;
use crate::{Config, utils::parse_input::parse_raw_user_input};
use anyhow::{Context, Result};
use chrono::Local;

use super::todo_entry::TodoEntry;

#[derive(Debug)]
pub struct TodoList {
    pub config: Config,
}
impl TodoList {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Self::with_config(config)
    }

    /// Creates a new `TodoList` instance with a specific `Config`.
    /// This also ensures that the todo list's root directory exists.
    pub fn with_config(config: Config) -> Result<Self> {
        fs::create_dir_all(&config.todo_list_dir)
            .with_context(|| format!("creating {}", config.todo_list_dir.display()))?;
        Ok(Self { config })
    }

    pub fn create_entry(&self, input: &str) -> Result<TodoEntry> {
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
        let date = if let Some(d) = parsed_input.date {
            d
        } else {
            self.config.reference_date
        };
        let time = if let Some(t) = parsed_input.time {
            t
        } else {
            Local::now().time()
        };

        Ok(TodoEntry {
            date,
            time,
            title: parsed_input.title,
            body: parsed_input.body,
            path: PathBuf::new(),
        })
    }
}
