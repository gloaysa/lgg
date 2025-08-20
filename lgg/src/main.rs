mod cli_modes;
mod render;

use anyhow::Result;
use clap::Parser;
use cli_modes::{CliModeResult, edit_mode, read_mode, write_mode};
use lgg_core::Journal;
use render::{ColorMode, RenderOptions, Renderer};
use std::io::{self, IsTerminal};
use std::process::ExitCode;

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
    /// Search for entries with the given tags (e.g., `lgg --tags dogs cats`)
    #[arg(long, short, num_args(1..))]
    tags: Option<Vec<String>>,
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

    if let CliModeResult::Finish = read_mode(&cli, &renderer, &journal)? {
        return Ok(());
    };

    // Edit mode
    if let CliModeResult::Finish = edit_mode(&cli, &renderer, &journal)? {
        return Ok(());
    };

    // Insert mode (default)
    if let CliModeResult::Finish = write_mode(&cli, &renderer, &journal)? {
        return Ok(());
    };

    Ok(())
}
