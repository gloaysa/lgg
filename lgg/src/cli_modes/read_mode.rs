use super::CliModeResult;
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::{Journal, QueryError, QueryResult, QueryTagsResult, ReadEntriesOptions};

pub fn read_mode(cli: &Cli, renderer: &Renderer, journal: &Journal) -> Result<CliModeResult> {
    let mut start_date: Option<&str> = None;
    let mut end_date: Option<&str> = None;
    let mut tags: Option<Vec<String>> = None;

    if cli.all_tags {
        let tags = journal.search_all_tags();
        print_tags(renderer, tags);
        return Ok(CliModeResult::Finish);
    }

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
    if let Some(has_tags) = &cli.tags {
        tags = Some(has_tags.to_vec());
    }

    if start_date.is_none() && tags.is_none() {
        return Ok(CliModeResult::NothingToDo);
    }

    let options = ReadEntriesOptions {
        start_date,
        end_date,
        tags: cli.tags.as_ref(),
        ..Default::default()
    };
    let result = journal.read_entries(&options);
    print_entries(renderer, result);
    Ok(CliModeResult::Finish)
}

fn print_entries(renderer: &Renderer, result: QueryResult) {
    if result.entries.is_empty() {
        renderer.print_info(&format!("No entries found"));
    } else {
        renderer.print_info(&format!("{} entries found.", result.entries.len()));
        renderer.print_entries(&result);
    }
    if !result.errors.is_empty() {
        print_errors(renderer, result.errors);
    }
}

fn print_tags(renderer: &Renderer, result: QueryTagsResult) {
    if result.tags.is_empty() {
        renderer.print_info(&format!("No tags found"));
    } else {
        renderer.print_info(&format!("{} unique tags found.", result.tags.len()));
        renderer.print_tags(&result.tags);
    }
    if !result.errors.is_empty() {
        print_errors(renderer, result.errors);
    }
}

fn print_errors(renderer: &Renderer, errors: Vec<QueryError>) {
    renderer.print_md("\n# Errors:");
    for error in errors {
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
