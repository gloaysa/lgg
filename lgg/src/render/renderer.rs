use super::theme::OneDark;
use lgg_core::{JournalEntry, QueryResult};
use termimad::{
    MadSkin,
    crossterm::style::{Color, Stylize},
};

#[derive(Clone)]
pub struct RenderOptions {
    pub date_format: String,
    pub use_color: bool,
    pub short_mode: bool,
}

pub struct Renderer {
    skin: MadSkin,
    opts: RenderOptions,
}

impl Renderer {
    pub fn new(config: Option<RenderOptions>) -> Self {
        Self {
            skin: OneDark::default_onedark_skin(),
            opts: match config {
                Some(config) => config,
                None => RenderOptions {
                    date_format: "%a, %d %b %Y".to_string(),
                    use_color: true,
                    short_mode: false,
                },
            },
        }
    }

    pub fn print_md(&self, md: &str) {
        self.skin.print_text(md);
    }

    pub fn print_info(&self, message: &str) {
        if self.opts.use_color {
            let md = format!("|-|\n| {message} |\n|-|\n");
            self.skin.print_text(&md);
        }
    }

    pub fn print_entry_line(&self, entry: &JournalEntry) {
        let mut date = entry.date.to_string();
        let mut time = entry.time.format("%H:%M").to_string();
        let mut title = entry.title.to_string();
        let mut tags = String::new();
        if !entry.tags.is_empty() {
            tags = format!("[{}], ", entry.tags.join(", "));
        }
        if self.opts.use_color {
            date = date.with(Color::Cyan).to_string();
            time = time.with(Color::Blue).to_string();
            title = title.with(Color::Yellow).to_string();
            tags = tags.with(Color::Green).to_string();
        }
        println!("{} {} - {} {}", date, time, title, tags);
    }

    pub fn print_entries<'a>(&self, result: &QueryResult) {
        if result.entries.is_empty() {
            self.print_info("No entries found.");
            return;
        }

        for (i, entry) in result.entries.iter().enumerate() {
            let date = entry.date.format(&self.opts.date_format).to_string();
            let time = entry.time.format("%H:%M").to_string();
            let title = entry.title.trim();
            if self.opts.short_mode {
                self.print_entry_line(&entry);
                continue;
            }
            let heading = format!("## {} {}: {}", &date, &time, &title);

            let body = if entry.body.trim().is_empty() {
                String::new()
            } else {
                let mut parsed_body = entry.body.trim_end().to_string();
                parsed_body = highlight_tags(&parsed_body);
                parsed_body
            };

            let md = if body.is_empty() {
                format!("{heading}\n")
            } else {
                format!("{heading}\n{body}\n")
            };

            if self.opts.use_color {
                self.print_md(&md);
            } else {
                print!("{md}");
            }

            if i + 1 < result.entries.len() {
                println!();
            }

            if self.opts.use_color {
                self.print_md("---");
            } else {
                println!("---");
            }
        }
    }
}

fn highlight_tags(body: &str) -> String {
    let re = regex::Regex::new(r"(?m)(^|\s)@([A-Za-z0-9_][\w-]*)").unwrap();
    re.replace_all(body, "$1`@$2`").to_string()
}
