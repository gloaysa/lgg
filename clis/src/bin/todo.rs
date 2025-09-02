use anyhow::Result;
use lgg_cli::{BaseCli, TodoCli};
use lgg_core::Lgg;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("todo: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = BaseCli::new();
    let lgg = Lgg::new()?;
    let todo_cli = TodoCli::new(cli, lgg);
    todo_cli.run()
}
