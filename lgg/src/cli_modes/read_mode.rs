use super::CliModeResult;
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::{Journal, QueryError, QueryResult};

pub fn read_mode(cli: &Cli, renderer: &Renderer, journal: &Journal) -> Result<CliModeResult> {
    let mut start_date: Option<&str> = None;
    let mut end_date: Option<&str> = None;

    if let Some(on) = &cli.on {
        start_date = Some(on);
    }
    if let Some(to) = &cli.to {
        start_date = Some(to);
    }
    if let Some(from) = &cli.from {
        match &cli.to {
            Some(to) => {
                start_date = Some(from);
                end_date = Some(to);
            }
            None => {
                start_date = Some(from);
                end_date = Some(&"today");
            }
        }
    }

    if let Some(found_tags) = &cli.tags {
        println!("{}", found_tags.join(" "));
    }

    if let Some(start_date) = start_date {
        let result = journal.read_entries(&start_date, end_date, None);
        print_entries(renderer, result, &start_date);
        return Ok(CliModeResult::Finish);
    }
    Ok(CliModeResult::NothingToDo)
}

fn print_entries(renderer: &Renderer, result: QueryResult, date_str: &str) {
    if result.entries.is_empty() {
        renderer.print_info(&format!("No entries found for {}.", date_str));
    } else {
        renderer.print_info(&format!("{} entries found.", result.entries.len()));
        renderer.print_entries(&result);
    }
    if !result.errors.is_empty() {
        renderer.print_md("\n# Errors:");
        for error in result.errors {
            match error {
                QueryError::FileError { path, error } => {
                    let message = format!("* Could not process '{}': {}", path.display(), error);
                    renderer.print_md(&message);
                }
                QueryError::InvalidDate { input, error } => {
                    let message = format!("* Could not process '{}': {}", input, error);
                    renderer.print_md(&message);
                }
            }
        }
    }
}
