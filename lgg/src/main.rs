mod cli;
mod cli_modes;
mod render;

use anyhow::Result;
use cli::{Cli, Style};
use cli_modes::{CliModeResult, edit_mode, read_mode, write_mode};
use lgg_core::Journal;
use render::{ColorMode, RenderOptions, Renderer};
use std::io::{self, IsTerminal};
use std::process::ExitCode;

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
    let cli = Cli::new();
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
    let short_mode = match cli.style {
        Style::Short => true,
        Style::Long => false,
    };
    let renderer = Renderer::new(Some(RenderOptions {
        date_format: journal.config.journal_date_format.to_string(),
        use_color,
        short_mode,
    }));

    if cli.path {
        renderer.print_info(&format!("{}", journal.config.journal_dir.display()));
        return Ok(());
    }

    if let CliModeResult::Finish = read_mode(&cli, &renderer, &journal)? {
        return Ok(());
    };

    if let CliModeResult::Finish = edit_mode(&cli, &renderer, &journal)? {
        return Ok(());
    };

    if let CliModeResult::Finish = write_mode(&cli, &renderer, &journal)? {
        return Ok(());
    };

    Ok(())
}
