use crate::{
    common::{create_editor_buffer, open_file_in_editor, resolve_editor, CliModeResult}, render::Renderer,
    BaseCli,
    RenderOptions,
};
use anyhow::Result;
use lgg_core::{
    JournalEntry, JournalQueryResult, JournalWriteEntry, Lgg, QueryError,
    ReadEntriesOptions,
};
use lgg_core::entries::QueryTagsResult;

enum PrintResult {
    Entries(JournalQueryResult),
    Tags(QueryTagsResult),
}

pub struct LggCli {
    cli: BaseCli,
    renderer: Renderer,
    lgg: Lgg,
}
impl LggCli {
    pub fn new(cli: BaseCli, lgg: Lgg) -> Self {
        let options = cli.load();

        let renderer = Renderer::new(Some(RenderOptions {
            date_format: lgg.config.journal_date_format.to_string(),
            use_color: options.use_color,
            short_mode: options.short_mode,
        }));
        LggCli { cli, renderer, lgg }
    }

    pub fn run(&self) -> Result<()> {
        if self.cli.path {
            self.renderer
                .print_info(&format!("{}", self.lgg.config.journal_dir.display()));
            return Ok(());
        }

        if let CliModeResult::Finish = self.write_mode()? {
            return Ok(());
        };

        if let CliModeResult::Finish = self.read_mode()? {
            return Ok(());
        };

        if let CliModeResult::Finish = self.edit_mode()? {
            return Ok(());
        };

        if let CliModeResult::Finish = self.editor_mode()? {
            return Ok(());
        };

        Ok(())
    }

    pub fn write_mode(&self) -> Result<CliModeResult> {
        let new_entry: JournalEntry;
        if !self.cli.text.is_empty() {
            let inline = self.cli.text.join(" ");
            let parsed_entry = self.lgg.parse_user_input(&inline)?;
            let entry_to_create = JournalWriteEntry {
                date: parsed_entry.date,
                time: parsed_entry.time,
                title: parsed_entry.title,
                body: parsed_entry.body,
                tags: Vec::new(),
            };

            new_entry = self.lgg.journal.create_entry(entry_to_create)?;
            self.renderer
                .print_info(&format!("Added new entry to {}", new_entry.path.display()));
            self.renderer.print_journal_entry_line(&new_entry);
            Ok(CliModeResult::Finish)
        } else {
            Ok(CliModeResult::NothingToDo)
        }
    }

    pub fn editor_mode(&self) -> Result<CliModeResult> {
        if !self.cli.text.is_empty() {
            return self.write_mode();
        }

        let new_entry: JournalEntry;

        let editor = resolve_editor(&self.lgg.config.editor)?;
        let input = create_editor_buffer(&editor)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            self.renderer
                .print_info(&"No entry to save, because no text was received.".to_string());
            return Ok(CliModeResult::Finish);
        }
        let inline = self.cli.text.join(" ");
        let parsed_entry = self.lgg.parse_user_input(&inline)?;
        let entry_to_create = JournalWriteEntry {
            date: parsed_entry.date,
            time: parsed_entry.time,
            title: parsed_entry.title,
            body: parsed_entry.body,
            tags: Vec::new(),
        };

        new_entry = self.lgg.journal.create_entry(entry_to_create)?;
        self.renderer
            .print_info(&format!("Added new entry to {}", new_entry.path.display()));
        self.renderer.print_journal_entry_line(&new_entry);
        Ok(CliModeResult::Finish)
    }

    pub fn read_mode(&self) -> Result<CliModeResult> {
        let mut start_date: Option<&str> = None;
        let mut end_date: Option<&str> = None;
        let mut time: Option<&str> = None;
        let mut tags: Option<Vec<String>> = None;

        if self.cli.all_tags {
            let tags = self.lgg.journal.search_all_tags();
            self.print_results(&PrintResult::Tags(tags), self.cli.count);
            return Ok(CliModeResult::Finish);
        }

        if let Some(on) = &self.cli.on {
            start_date = Some(on);
        }
        if let Some(to) = &self.cli.to {
            start_date = Some(to);
        }
        if let Some(from) = &self.cli.from {
            match &self.cli.to {
                Some(to) => {
                    start_date = Some(from);
                    end_date = Some(to);
                }
                None => {
                    start_date = Some(from);
                    end_date = Some(&"today");
                }
            }
        }
        if let Some(has_time) = &self.cli.at {
            time = Some(has_time);
        }
        if let Some(has_tags) = &self.cli.tags {
            tags = Some(has_tags.to_vec());
        }

        if start_date.is_none() && time.is_none() && tags.is_none() {
            return Ok(CliModeResult::NothingToDo);
        }

        let dates = match start_date {
            Some(d) => self.lgg.parse_dates(d, end_date),
            None => None,
        };
        let options = ReadEntriesOptions {
            dates,
            time,
            tags: self.cli.tags.as_ref(),
            ..Default::default()
        };
        let result = self.lgg.journal.read_entries(&options);
        self.print_results(&PrintResult::Entries(result), self.cli.count);
        Ok(CliModeResult::Finish)
    }

    pub fn edit_mode(&self) -> Result<CliModeResult> {
        if let Some(start_date) = &self.cli.edit {
            let dates = self.lgg.parse_dates(start_date, None);
            let options = ReadEntriesOptions {
                dates,
                ..Default::default()
            };
            let results = self.lgg.journal.read_entries(&options);

            return match results.entries.first() {
                Some(entry) => {
                    let editor = resolve_editor(&self.lgg.config.editor)?;
                    open_file_in_editor(&editor, &entry.path)?;
                    self.renderer
                        .print_info(&format!("Edited file {}", entry.path.display()));
                    Ok(CliModeResult::Finish)
                }
                None => {
                    self.renderer.print_info("No entries found to edit.");
                    Ok(CliModeResult::Finish)
                }
            }
        }
        Ok(CliModeResult::NothingToDo)
    }

    fn print_results(&self, result: &PrintResult, print_count: bool) {
        let mut errors = Vec::new();
        if print_count {
            match result {
                PrintResult::Entries(res) => {
                    self.renderer
                        .print_info(&format!("{} entries found.", res.entries.len()));
                }
                PrintResult::Tags(res) => {
                    self.renderer
                        .print_info(&format!("{} tags found.", res.tags.len()));
                }
            }

            return;
        }

        if let PrintResult::Entries(res) = result {
            errors.extend(&res.errors);
            if res.entries.is_empty() {
                self.renderer.print_info(&"No entries found.".to_string());
            } else {
                self.renderer.print_journal_entries(&res);
            }
        }
        if let PrintResult::Tags(res) = result {
            errors.extend(&res.errors);
            if res.tags.is_empty() {
                self.renderer.print_info(&"No tags found.".to_string());
            } else {
                self.renderer.print_tags(&res.tags);
            }
        }
        if !errors.is_empty() {
            self.print_errors(&errors);
        }
    }

    fn print_errors(&self, errors: &Vec<&QueryError>) {
        self.renderer.print_md("\n# Errors:");
        for error in errors {
            match error {
                QueryError::FileError { path, error } => {
                    let message = format!("* Could not process '{}': {}", path.display(), error);
                    self.renderer.print_md(&message);
                }
                QueryError::InvalidDate { input, error } => {
                    let message = format!("* Could not process '{}': {}", input, error);
                    self.renderer.print_md(&message);
                }
            }
        }
    }
}
