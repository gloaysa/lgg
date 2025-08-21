use super::{
    CliModeResult,
    editor_utils::{open_file_in_editor, resolve_editor},
};
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::Journal;

pub fn edit_mode(cli: &Cli, renderer: &Renderer, journal: &Journal) -> Result<CliModeResult> {
    if let Some(start_date) = &cli.edit {
        let results = journal.read_entries(&start_date, None, None);

        match results.entries.first() {
            Some(entry) => {
                let editor = resolve_editor(&journal)?;
                open_file_in_editor(&editor, &entry.path)?;
                renderer.print_info(&format!("Edited file {}", entry.path.display()));
                return Ok(CliModeResult::Finish);
            }
            None => {
                renderer.print_info("No entries found to edit.");
                return Ok(CliModeResult::Finish);
            }
        }
    }
    Ok(CliModeResult::NothingToDo)
}
