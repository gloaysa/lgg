use super::{
    CliModeResult,
    editor_utils::{create_editor_buffer, resolve_editor},
};
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::{Journal, JournalEntry};

pub fn write_mode(cli: &Cli, renderer: &Renderer, journal: &Journal) -> Result<CliModeResult> {
    let new_entry: JournalEntry;
    if !cli.text.is_empty() {
        let inline = cli.text.join(" ");
        new_entry = journal.create_entry(&inline)?;
    } else {
        let editor = resolve_editor(&journal)?;
        let input = create_editor_buffer(&editor)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            renderer.print_info(&format!("No entry to save, because no text was received."));
            return Ok(CliModeResult::Finish);
        }
        new_entry = journal.create_entry(&trimmed)?;
    }
    renderer.print_info(&format!("Added new entry to {}", new_entry.path.display()));
    renderer.print_entry_line(&new_entry);
    Ok(CliModeResult::Finish)
}
