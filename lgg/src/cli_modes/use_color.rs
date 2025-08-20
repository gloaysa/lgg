use std::io::{self, IsTerminal};

use crate::{Cli, render::ColorMode};

pub fn use_color(cli: &Cli) -> bool {
    match cli.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            if std::env::var_os("NO_COLOR").is_some() {
                false
            } else {
                io::stdout().is_terminal()
            }
        }
    }
}
