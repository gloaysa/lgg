use anyhow::Result;
use clap::Parser;
use lgg_core::{Journal, journal::QueryError, render::format_date};
use std::{
    fs,
    process::{Command, ExitCode},
};

/// lgg — Simple Markdown journal
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Prints the journal root directory
    #[arg(long, short = 'p')]
    path: bool,

    /// View entries on a specific date (e.g., `yesterday`, `2025-08-15`)
    #[arg(long)]
    on: Option<String>,

    /// Free text for insert mode (e.g., `yesterday: Title. Body`)
    ///
    /// If provided and no subcommand is used, we treat this as an inline entry.
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

    // Reading/Filtering mode
    if let Some(date_str) = cli.on {
        println!("Filtering on: {}", date_str);
        let result = journal.read_entries_on_date(&date_str);
        if result.entries.is_empty() {
            println!("No entries found for {}.", date_str);
        } else {
            for entry in result.entries {
                println!(
                    "{} {}: {}",
                    format_date(entry.date, &journal.config),
                    entry.time.format("%H:%M"),
                    entry.title
                );
                if !entry.body.is_empty() {
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

    // Insert mode (default)
    if !cli.text.is_empty() {
        let inline = cli.text.join(" ");
        let saved = journal.save_entry(&inline)?;
        println!(
            "Saved: {} {} - {} -> {}",
            saved.date,
            saved.time.format("%H:%M"),
            saved.title,
            saved.path.display()
        );
    } else {
        let editor = resolve_editor(&journal)?;
        let input = edit_and_read(&editor)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("No entry to save, because no text was received.");
            return Ok(());
        }
        let saved = journal.save_entry(&trimmed)?;
        println!(
            "Saved: {} {} - {} -> {}",
            saved.date,
            saved.time.format("%H:%M"),
            saved.title,
            saved.path.display()
        );
    }
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

fn edit_and_read(editor_cmd: &str) -> Result<String> {
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
