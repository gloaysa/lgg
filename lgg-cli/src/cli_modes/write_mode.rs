use super::{
    CliModeResult,
    editor_utils::{create_editor_buffer, resolve_editor},
};
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::{JournalEntry, JournalWriteEntry, Lgg};

pub fn write_mode(cli: &Cli, renderer: &Renderer, lgg: &Lgg) -> Result<CliModeResult> {
    let new_entry: JournalEntry;
    if !cli.text.is_empty() {
        let inline = cli.text.join(" ");
        let parsed_entry = lgg.parse_user_input(&inline)?;
        let entry_to_create = JournalWriteEntry {
            date: parsed_entry.date,
            time: parsed_entry.time,
            title: parsed_entry.title,
            body: parsed_entry.body,
            tags: Vec::new(),
        };

        new_entry = lgg.journal.create_entry(entry_to_create)?;
        renderer.print_info(&format!("Added new entry to {}", new_entry.path.display()));
        renderer.print_entry_line(&new_entry);
        Ok(CliModeResult::Finish)
    } else {
        return Ok(CliModeResult::NothingToDo);
    }
}

pub fn editor_mode(cli: &Cli, renderer: &Renderer, lgg: &Lgg) -> Result<CliModeResult> {
    if !cli.text.is_empty() {
        return write_mode(cli, renderer, lgg);
    }

    let new_entry: JournalEntry;

    let editor = resolve_editor(&lgg.config.editor)?;
    let input = create_editor_buffer(&editor)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        renderer.print_info(&format!("No entry to save, because no text was received."));
        return Ok(CliModeResult::Finish);
    }
    let inline = cli.text.join(" ");
    let parsed_entry = lgg.parse_user_input(&inline)?;
    let entry_to_create = JournalWriteEntry {
        date: parsed_entry.date,
        time: parsed_entry.time,
        title: parsed_entry.title,
        body: parsed_entry.body,
        tags: Vec::new(),
    };

    new_entry = lgg.journal.create_entry(entry_to_create)?;
    renderer.print_info(&format!("Added new entry to {}", new_entry.path.display()));
    renderer.print_entry_line(&new_entry);
    Ok(CliModeResult::Finish)
}
