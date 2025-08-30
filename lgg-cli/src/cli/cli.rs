use clap::{ArgGroup, Parser, arg, command};

use crate::render::ColorMode;

use super::style::Style;

/// lgg — Simple Markdown journal
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    group(ArgGroup::new("read_mode").args(["on", "from", "to", "at", "tags"]).multiple(true)),
    group(ArgGroup::new("edit_mode").args(["edit"])),
    group(ArgGroup::new("write_mode").args(["text"])),
    group(ArgGroup::new("solo").args(["path", "all_tags"]).conflicts_with_all(["read_mode", "edit_mode", "write_mode"])),
)]
pub struct Cli {
    /// Prints the journal root directory
    #[arg(long, short)]
    pub path: bool,
    /// Prints all the tags within all entries.
    #[arg(long)]
    pub all_tags: bool,
    /// Control ANSI colors in output.
    /// By default, colors are disabled when output is redirected (e.g with `>` or `|`).
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    pub color: ColorMode,

    /// View entries on a specific date (e.g., `lgg --on yesterday`, `lgg --on 14/08/25`)
    #[arg(long)]
    pub on: Option<String>,
    /// View entries from, or on, this date (e.g., `lgg --from yesterday`, `lgg --from 14/08/25`)
    #[arg(long, conflicts_with = "on")]
    pub from: Option<String>,
    /// View entries on a specific date (e.g., `yesterday`, `2025-08-15`)
    #[arg(long, conflicts_with = "on", requires = "from")]
    pub to: Option<String>,
    /// View entries for (or from) an specific time. E.g.
    /// `lgg --at morning` will return all entries written from 06:00 til 11:59.
    /// `lgg --on today --at 12:23` will return all entries written today from 12:00 til 12:59.
    #[arg(long)]
    pub at: Option<String>,
    /// Prints the count of found entries/tags.
    #[arg(long)]
    pub count: bool,

    /// Output style: "long" or "short". Short style only shows the date, titles and tags of searched entries.
    #[arg(long, short, value_enum, env = "LGG_STYLE", default_value_t= Style::Long)]
    pub style: Style,
    /// Search for entries with the given tags (e.g., `lgg --tags dogs cats`)
    #[arg(long, short, num_args(1..))]
    pub tags: Option<Vec<String>>,
    /// Opens your $EDITOR with a found day file. Only works on single day searches.
    /// eg. `lgg --edit yesterday`
    #[arg(long, short)]
    pub edit: Option<String>,

    /// Free text for insert mode (e.g., `lgg yesterday: Title. Body`).
    #[arg()]
    pub text: Vec<String>,
}
impl Cli {
    pub fn new() -> Self {
        let cli = Cli::parse();
        cli
    }
}
