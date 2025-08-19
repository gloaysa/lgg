use lgg_core::journal::QueryResult;
use termimad::{
    Alignment, MadSkin,
    crossterm::style::{Attribute, Color, Stylize},
};

pub struct OneDark;

impl OneDark {
    pub const BG: Color = Color::Rgb {
        r: 0x28,
        g: 0x2C,
        b: 0x34,
    }; // #282C34
    pub const FG: Color = Color::Rgb {
        r: 0xAB,
        g: 0xB2,
        b: 0xBF,
    }; // #ABB2BF

    pub const RED: Color = Color::Rgb {
        r: 0xE0,
        g: 0x6C,
        b: 0x75,
    }; // #E06C75
    pub const ORANGE: Color = Color::Rgb {
        r: 0xD1,
        g: 0x9A,
        b: 0x66,
    }; // #D19A66
    pub const YELLOW: Color = Color::Rgb {
        r: 0xE5,
        g: 0xC0,
        b: 0x7B,
    }; // #E5C07B
    pub const GREEN: Color = Color::Rgb {
        r: 0x98,
        g: 0xC3,
        b: 0x79,
    }; // #98C379
    pub const BLUE: Color = Color::Rgb {
        r: 0x61,
        g: 0xAF,
        b: 0xEF,
    }; // #61AFEF
    pub const PURPLE: Color = Color::Rgb {
        r: 0xC6,
        g: 0x78,
        b: 0xDD,
    }; // #C678DD
    pub const CYAN: Color = Color::Rgb {
        r: 0x56,
        g: 0xB6,
        b: 0xC2,
    }; // #56B6C2

    // useful neutrals
    pub const COMMENT: Color = Color::Rgb {
        r: 0x5C,
        g: 0x63,
        b: 0x70,
    }; // #5C6370
    pub const GUTTER: Color = Color::Rgb {
        r: 0x4B,
        g: 0x52,
        b: 0x63,
    }; // #4B5263
    pub const SEL_BG: Color = Color::Rgb {
        r: 0x3E,
        g: 0x44,
        b: 0x51,
    }; // #3E4451
    pub const BG2: Color = Color::Rgb {
        r: 0x2C,
        g: 0x31,
        b: 0x3C,
    }; // #2C313C

    // optional accent
    pub const ACCENT_BLUE: Color = Color::Rgb {
        r: 0x52,
        g: 0x8B,
        b: 0xFF,
    }; // #528BFF
}

#[derive(Clone)]
pub struct RenderOptions {
    pub date_format: String,
}

pub struct Renderer {
    skin: MadSkin,
    opts: RenderOptions,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            skin: default_onedark_skin(),
            opts: RenderOptions {
                date_format: "%a, %d %b %Y".to_string(),
            },
        }
    }

    pub fn with_options(mut self, opts: RenderOptions) -> Self {
        self.opts = opts;
        self
    }

    pub fn print_md(&self, md: &str) {
        self.skin.print_text(md);
    }

    pub fn print_info(&self, message: &str) {
        let md = format!("|-|\n| {message} |\n|-|\n");
        self.skin.print_text(&md);
    }

    pub fn print_entries<'a>(&self, result: &QueryResult, short_mode: bool) {
        if result.entries.is_empty() {
            self.print_info("No entries found.");
            return;
        }

        for (i, e) in result.entries.iter().enumerate() {
            let date = e.date.format(&self.opts.date_format).to_string();
            let time = e.time.format("%H:%M").to_string();
            let title = e.title.trim();
            if short_mode {
                print_entry_line(&date, &time, &title);
                continue;
            }
            let heading = format!("## {} {}: {}", &date, &time, &title);

            let body = if e.body.trim().is_empty() {
                String::new()
            } else {
                let mut parsed_body = e.body.trim_end().to_string();
                parsed_body = highlight_tags(&parsed_body);
                parsed_body
            };

            let md = if body.is_empty() {
                format!("{heading}\n")
            } else {
                format!("{heading}\n{body}\n")
            };
            self.skin.print_text(&md);

            if i + 1 < result.entries.len() {
                println!();
            }
            self.print_md("---");
        }
    }
}

fn highlight_tags(body: &str) -> String {
    let re = regex::Regex::new(r"(?m)(^|\s)@([A-Za-z0-9_][\w-]*)").unwrap();
    re.replace_all(body, "$1`@$2`").to_string()
}

fn print_entry_line(date: &str, time: &str, title: &str) {
    let date = date.with(Color::Cyan);
    let time = time.with(Color::Blue);
    let title = title.with(Color::Yellow);
    println!("{} {} - {}", date, time, title);
}
fn default_onedark_skin() -> MadSkin {
    let mut skin = MadSkin::default();

    skin.paragraph.set_fg(OneDark::FG);
    skin.bold.set_fg(OneDark::FG);
    skin.italic.set_fg(OneDark::FG);

    skin.headers[0].set_fg(OneDark::RED);
    skin.headers[0].add_attr(Attribute::Bold);
    skin.headers[0].align = Alignment::Left;

    skin.headers[1].set_fg(OneDark::YELLOW);
    skin.headers[1].add_attr(Attribute::Bold);

    skin.headers[2].set_fg(OneDark::BLUE);
    skin.headers[2].add_attr(Attribute::Bold);

    skin.table.set_fg(OneDark::PURPLE);
    skin.bullet.set_fg(OneDark::RED);
    skin.quote_mark.set_char('┃');
    skin.quote_mark.set_fg(OneDark::COMMENT);

    skin.quote_mark.set_fg(OneDark::COMMENT);
    skin.inline_code.set_fg(OneDark::GREEN);
    skin.inline_code.set_bg(OneDark::BG);
    skin.code_block.set_fg(OneDark::ORANGE);
    skin.code_block.set_bg(OneDark::BG);

    skin
}
