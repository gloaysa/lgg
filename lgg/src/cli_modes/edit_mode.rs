use super::CliModeResult;
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::Journal;

pub fn edit_mode(cli: &Cli, renderer: &Renderer, _journal: &Journal) -> Result<CliModeResult> {
    if let Some(_date_str) = &cli.edit {
        // TODO: We need to implement .find_entry in journal. Returns a path to specified date's file.
        renderer.print_info(&format!("Coming soon, editing single files!"));
        return Ok(CliModeResult::Finish);
    }
    Ok(CliModeResult::NothingToDo)
}
