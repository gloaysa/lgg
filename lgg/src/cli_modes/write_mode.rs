use super::CliModeResult;
use crate::{Cli, render::Renderer};
use anyhow::Result;
use lgg_core::{Journal, JournalEntry};
use std::{fs, process::Command};

pub fn write_mode(cli: &Cli, renderer: &Renderer, journal: &Journal) -> Result<CliModeResult> {
    let new_entry: JournalEntry;
    if !cli.text.is_empty() {
        let inline = cli.text.join(" ");
        new_entry = journal.create_entry(&inline, None)?;
    } else {
        let editor = resolve_editor(&journal)?;
        let input = create_editor_buffer(&editor)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            renderer.print_info(&format!("No entry to save, because no text was received."));
            return Ok(CliModeResult::Finish);
        }
        new_entry = journal.create_entry(&trimmed, None)?;
    }
    let date = new_entry
        .date
        .format(&journal.config.journal_date_format)
        .to_string();
    let time = new_entry.time.format("%H:%M").to_string();
    let title = new_entry.title.trim();
    renderer.print_info(&format!("Added new entry to {}", new_entry.path.display()));
    renderer.print_entry_line(&date, &time, title);
    Ok(CliModeResult::NothingToDo)
}

fn resolve_editor(j: &Journal) -> Result<String> {
    let editor = j
        .config
        .editor
        .as_deref()
        .map(str::to_string)
        .or_else(|| std::env::var("VISUAL").ok())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| "vim".into());
    Ok(editor)
}

fn create_editor_buffer(editor_cmd: &str) -> Result<String> {
    let file = tempfile::Builder::new()
        .prefix("lgg")
        .suffix(".md")
        .tempfile()?;

    let path = file.path().to_path_buf();
    let status = Command::new(editor_cmd).arg(&path).status()?;
    if !status.success() {
        anyhow::bail!("Editor exited with status {}", status);
    }
    Ok(fs::read_to_string(&path)?)
}
