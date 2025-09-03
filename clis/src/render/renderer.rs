use super::theme::OneDark;
use lgg_core::{JournalEntry, JournalQueryResult, TodoEntry, TodoQueryResult, TodoStatus};
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
        let md = format!("|-|\n| {message} |\n|-|\n");
        if self.opts.use_color {
            self.print_md(&md);
        } else {
            println!("{}", message);
        }
    }

    pub fn print_journal_entry_line(&self, entry: &JournalEntry) {
        let mut date = entry.date.to_string();
        let mut time = entry.time.format("%H:%M").to_string();
        let mut title = entry.title.to_string();

        let tags = if entry.tags.is_empty() {
            String::new()
        } else if self.opts.use_color {
            let colored_tags = print_colored_list(&entry.tags);
            format!("[{}]", colored_tags.join(" - "))
        } else {
            format!("[{}]", entry.tags.join(" - "))
        };
        if self.opts.use_color {
            date = date.with(Color::Cyan).to_string();
            time = time.with(Color::Blue).to_string();
            title = title.with(Color::Yellow).to_string();
        }
        println!("{} {} - {} {}", date, time, title, tags);
    }

    pub fn print_journal_entries<'a>(&self, result: &JournalQueryResult) {
        for (i, entry) in result.entries.iter().enumerate() {
            if self.opts.short_mode {
                self.print_journal_entry_line(&entry);
                continue;
            }
            let date = entry.date.format(&self.opts.date_format).to_string();
            let time = entry.time.format("%H:%M").to_string();
            let title = entry.title.trim();
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

    pub fn print_todo_entry_line(&self, entry: &TodoEntry) {
        let mut date = match entry.due_date {
            Some(dt) => {
                let d = dt.date().format(&self.opts.date_format).to_string();
                format!("- {d}")
            }
            None => "".to_string(),
        };
        let mut time = match entry.due_date {
            Some(dt) => dt.time().format("%H:%M").to_string(),
            None => "".to_string(),
        };
        let mut title = if self.opts.use_color {
            let icons = todo_icons(&entry.status);
            let i = icons.color.with(Color::Red);
            let t = entry.title.clone().with(Color::Yellow);
            format!("{i} {t}")
        } else {
            let icons = todo_icons(&entry.status);
            let i = icons.no_color;
            let t = entry.title.clone();
            format!("{i} {t}")
        };

        let tags = if entry.tags.is_empty() {
            String::new()
        } else if self.opts.use_color {
            let colored_tags = print_colored_list(&entry.tags);
            format!("[{}]", colored_tags.join(" - "))
        } else {
            format!("[{}]", entry.tags.join(" - "))
        };
        if self.opts.use_color {
            date = date.with(Color::Cyan).to_string();
            time = time.with(Color::Blue).to_string();
            title = title.with(Color::Yellow).to_string();
        }
        println!("{} {} {} {}", title, date, time, tags);
    }

    pub fn print_todos_entries<'a>(&self, result: &TodoQueryResult) {
        for (i, entry) in result.todos.iter().enumerate() {
            if self.opts.short_mode {
                self.print_todo_entry_line(&entry);
                continue;
            }

            if entry.body.trim().is_empty() {
                continue;
            }

            let mut parsed_body = entry.body.trim_end().to_string();
            parsed_body = highlight_tags(&parsed_body);
            let spaces = if self.opts.use_color {
                " ".repeat(2)
            } else {
                " ".repeat(4)
            };

            self.print_todo_entry_line(&entry);
            println!("{spaces}{parsed_body}");

            if i + 1 < result.todos.len() {
                println!();
            }

            if self.opts.use_color {
                self.print_md("---");
            } else {
                println!("---");
            }
        }
    }
    pub fn print_tags(&self, tags: &Vec<String>) {
        let tags = if tags.is_empty() {
            String::new()
        } else if self.opts.use_color {
            let colored_tags = print_colored_list(&tags);
            format!("{}", colored_tags.join(" - "))
        } else {
            format!("{}", tags.join(" - "))
        };
        println!("{}", tags);
    }
}

fn highlight_tags(body: &str) -> String {
    let re = regex::Regex::new(r"(?m)(^|\s)@([A-Za-z0-9_][\w-]*)").unwrap();
    re.replace_all(body, "$1`@$2`").to_string()
}

pub fn print_colored_list(values: &Vec<String>) -> Vec<String> {
    values.iter().map(|v| colorize_value(v)).collect()
}

struct Icons {
    color: &'static str,
    no_color: &'static str,
}

fn todo_icons(status: &TodoStatus) -> Icons {
    match status {
        TodoStatus::Pending => Icons {
            color: "☐",
            no_color: "[ ]",
        },
        TodoStatus::Done => Icons {
            color: "☑",
            no_color: "[ ]",
        },
    }
}

fn colorize_value(val: &str) -> String {
    let palette = [
        Color::Red,
        Color::DarkRed,
        Color::Green,
        Color::DarkGreen,
        Color::Yellow,
        Color::DarkYellow,
        Color::Blue,
        Color::DarkBlue,
        Color::Magenta,
        Color::DarkMagenta,
        Color::Cyan,
        Color::DarkCyan,
        Color::Grey,
    ];

    fn stable_index(s: &str, modulo: usize) -> usize {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        (h as usize) % modulo
    }

    let idx = stable_index(val, palette.len());
    format!("{}", val.with(palette[idx]))
}
