use anyhow::Result;
use clap::Parser;
use lgg_core::{EntryRef, Journal, journal::QueryError, render::format_date};
use std::{
    fs,
    process::{Command, ExitCode},
};

/// lgg — Simple Markdown journal
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Prints the journal root directory
    #[arg(long, short)]
    path: bool,
    /// View entries on a specific date (e.g., `lgg --on yesterday`, `lgg --on 14/08/25`)
    #[arg(long)]
    on: Option<String>,
    /// View entries from, or on, this date (e.g., `lgg --from yesterday`, `lgg --from 14/08/25`)
    #[arg(long)]
    from: Option<String>,
    /// View entries on a specific date (e.g., `yesterday`, `2025-08-15`)
    #[arg(long)]
    to: Option<String>,
    /// Opens your $EDITOR with a found day file. Only works on single day searches.
    /// eg. `lgg --edit yesterday`
    #[arg(long, short)]
    edit: Option<String>,
    /// Only shows the date and titles of searched entries.
    #[arg(long, short)]
    short: bool,
    /// Free text for insert mode (e.g., `lgg yesterday: Title. Body`).
    text: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lgg: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let journal = Journal::new()?;

    if cli.path {
        println!("{}", journal.config.journal_dir.display());
        return Ok(());
    }

    // Read mode
    if let Some(date_str) = cli.on {
        println!("Filtering on: {}", date_str);
        let result = journal.read_entries(&date_str, None);
        if result.entries.is_empty() {
            println!("No entries found for {}.", date_str);
        } else {
            for entry in result.entries {
                println!(
                    "{} {}: {}",
                    format_date(entry.date, &journal.config.journal_date_format),
                    entry.time.format("%H:%M"),
                    entry.title
                );
                if !entry.body.is_empty() && !cli.short {
                    println!("  {}", entry.body.replace('\n', "\n"));
                }
            }
        }
        if !result.errors.is_empty() {
            eprintln!("\nWarnings:");
            for error in result.errors {
                match error {
                    QueryError::FileError { path, error } => {
                        eprintln!("- Could not process '{}': {}", path.display(), error);
                    }
                    QueryError::InvalidDate { input, error } => {
                        eprintln!("- Could not process '{}': {}", input, error);
                    }
                }
            }
        }
        // TODO: Implement reading logic
        return Ok(());
    }

    if let Some(date_str) = cli.edit {
        println!("Filtering on: {}", date_str);
        // TODO: We need to implement .find_entry in journal. Returns a path to specified date's file.
        println!("Coming soon, editing single files!");
        return Ok(());
    }

    // Insert mode (default)
    let new_entry: EntryRef;
    if !cli.text.is_empty() {
        let inline = cli.text.join(" ");
        new_entry = journal.create_entry(&inline, None)?;
    } else {
        let editor = resolve_editor(&journal)?;
        let input = create_editor_buffer(&editor)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("No entry to save, because no text was received.");
            return Ok(());
        }
        new_entry = journal.create_entry(&trimmed, None)?;
    }
    println!(
        "Saved: {} {} - {} -> {}",
        format_date(new_entry.date, &journal.config.journal_date_format),
        new_entry.time.format("%H:%M"),
        new_entry.title,
        new_entry.path.display()
    );

    Ok(())
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
