use anyhow::Result;
use lgg_cli::{BaseCli, LggCli};
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
    let lgg_cli = LggCli::new(cli, lgg);
    lgg_cli.run()
}
