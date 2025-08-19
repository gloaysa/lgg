mod render;

use anyhow::Result;
use clap::Parser;
use lgg_core::{
    EntryRef, Journal,
    journal::{QueryError, QueryResult},
};
use render::{ColorMode, RenderOptions, Renderer};
use std::io::{self, IsTerminal};
use std::{
    fs,
    process::{Command, ExitCode},
};

/// lgg — Simple Markdown journal
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Prints the journal root directory
    #[arg(long, short, exclusive = true)]
    path: bool,
    /// View entries on a specific date (e.g., `lgg --on yesterday`, `lgg --on 14/08/25`)
    #[arg(long, conflicts_with_all=["from", "to"])]
    on: Option<String>,
    /// View entries from, or on, this date (e.g., `lgg --from yesterday`, `lgg --from 14/08/25`)
    #[arg(long, conflicts_with = "on")]
    from: Option<String>,
    /// View entries on a specific date (e.g., `yesterday`, `2025-08-15`)
    #[arg(long, conflicts_with = "on", requires = "from")]
    to: Option<String>,
    /// Opens your $EDITOR with a found day file. Only works on single day searches.
    /// eg. `lgg --edit yesterday`
    #[arg(long, short)]
    edit: Option<String>,
    /// Only shows the date and titles of searched entries.
    #[arg(long, short)]
    short: bool,
    /// Control ANSI colors in output.
    /// By default, colors are disabled when output is redirected (e.g with `>` or `|`).
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    color: ColorMode,
    /// Free text for insert mode (e.g., `lgg yesterday: Title. Body`).
    #[arg(exclusive = true)]
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

    let use_color = match cli.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            if std::env::var_os("NO_COLOR").is_some() {
                false
            } else {
                io::stdout().is_terminal()
            }
        }
    };
    let renderer = Renderer::new(Some(RenderOptions {
        date_format: journal.config.journal_date_format.to_string(),
        use_color,
        short_mode: cli.short,
    }));

    if cli.path {
        renderer.print_info(&format!("{}", journal.config.journal_dir.display()));
        return Ok(());
    }

    // Read mode
    if let Some(date_str) = cli.on {
        let result = journal.read_entries(&date_str, None, None);
        print_entries(renderer, result, &date_str);
        return Ok(());
    }

    match (cli.from.as_deref(), cli.to.as_deref()) {
        (Some(from), Some(to)) => {
            let result = journal.read_entries(&from, Some(&to), None);
            print_entries(renderer, result, &from);
            return Ok(());
        }
        (Some(from), None) => {
            let result = journal.read_entries(&from, Some(&"today"), None);
            print_entries(renderer, result, &from);
            return Ok(());
        }
        (None, Some(_)) => {} // We can't have a 'to' without 'from'
        (None, None) => {}
    }

    // Edit mode
    if let Some(_date_str) = cli.edit {
        // TODO: We need to implement .find_entry in journal. Returns a path to specified date's file.
        renderer.print_info(&format!("Coming soon, editing single files!"));
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
            renderer.print_info(&format!("No entry to save, because no text was received."));
            return Ok(());
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

    Ok(())
}

fn print_entries(renderer: Renderer, result: QueryResult, date_str: &str) {
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
