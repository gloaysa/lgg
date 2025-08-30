# lgg Roadmap

This document outlines the development status and future plans for `lgg`. Our immediate goal is to complete the Minimum Viable Product (MVP) as defined in the original spec.

## MVP Status

The MVP is focused on providing the essential features for a command-line journaling tool.

### **Completed Features**

- **Insert Mode**:
  - [x] Create entries from inline text (`lgg <text>`).
  - [x] Create entries using a terminal editor (`lgg`).
  - [x] Parse natural dates (`yesterday`, `tomorrow`) and ISO dates (`YYYY-MM-DD`).
  - [x] Automatically split title and body (first sentence/newline rule).
- **Reading & Filtering Mode**:
  - [x] Implement the core query engine to parse journal files into structured entries.
  - [x] Implement the `--on <date>` flag to view all entries for a specific day.
    - `--from <date>`
    - `--to <date>` / `--until <date>`
    - `--short` displays the entry without the body.
    - `--tag <tag1 tag2>` displays all entries with the searched tags.
  - [x] Implement a resilient query system that returns warnings for malformed files instead of crashing.
  - [x] Better UI on Reader mode, parsing Markdown.
- **Storage**:
  - [x] Store entries in the `YYYY/MM/YYYY-MM-DD.md` file structure.
  - [x] Automatically create daily files and directories.
  - [x] Add human-readable headers to new daily files.
- **Configuration**:
  - [x] Load `config.toml` from standard OS directories.
  - [x] Support for `journal_dir`, `editor`, `default_time`, and `date_format`.
  - [x] Support for date keyword `[synonyms]`.
  - [x] Support for custom dates in journal entries and queries.
- **Utility**:
  - [x] **Localization**: Support for non-English date parsing and formatting.
  - [x] `lgg --path` command to print the journal's root directory.
  - [x] **Improved Tag Support**: Parsing and indexing for `@tags`.

### **In Progress / Next Steps**

These are the highest-priority features required to complete the MVP.

- **Expand Reading & Filtering Mode**:
  - [ ] Implement remaining CLI flags for filtering entries:
    - `--contains <text>`
    - `-n`/`--limit <count>`
    - `--reverse`

## Future Ideas (Post-MVP)

Once the MVP is complete and stable, we will explore features from the original project spec's roadmap.

- [ ] **Multiple Journals**: Support for creating and switching between different journals.
- [ ] **`lgg fix` Command**: A utility to normalize and clean up manually edited files (e.g., add missing timestamps, fix formatting).
- [ ] **Exporters**: Add options to export journal entries to different formats (JSON, consolidated Markdown, etc.).
- [ ] **Raycast extension**: No idea how Raycast extensions work, but will look into it. Would be nice to be able to create entries from it when not at the terminal. There's a work-around for it at the moment.
- [ ] **Templates**: If the app gets some tracking and users want this feature.

## Todo list

Part of lgg-core will be a functionality that lets you manage your TODOs. It will have a separate cli that will follow the same principles than `lgg`.
It will also be plain markdown. Instead of having the files organized by dates, will have files organized by the status (pending and done).

### Statuses

Possible statuses will be `pending` and `done`. We will not introduce statuses like ongoing to encourage users to focus on a task at a time. A task that needs multiple steps (like waiting for someone else's feedback) is simply a task that have not been properly defined and needs to be broken on smaller steps.

### Format

The format on which it will write the TODOs to the .md file must be something simple that users can replicate when writing from outside the application.
The following will be a good example:

```md
- [ ] Task pending
- [ ] Task pending with due date | 11/02/2025 12:00
- [x] Task done no due date | | 12/02/2025 11:22
- [x] Task done that had due date | 11/02/2025 12:15 | 12/02/2025 13:14
```

When a TODO is pending and has a due date, the date will be found after the pipe symbol (|).
When the TODO is done, will reflect the date on which it was finished after two pipe symbols separated by an space (| |). If the original TODO had a due date, instead of the space will contain the due date.

### Body

TODOs, as journal entries, will optionally have a body. The body of a TODO will be text indented below the line of the title, skipping the - []. Example:

```md
- [ ] Task with body
      This is the body of the todo above.
      It can have multiple lines.
      And even contain another list: - Item 1 - Item 2
- [ ] Next task
```

### Cli

Simply calling `todo My new todo` will log a new TODO. As in lgg, everything after a dot (.) will be considered the body of the TODO. Then you will have the same flags that `lgg` uses for editing, reading...
Calling `todo DATE: some todo` will create a new TODO with a deadline. Users will be able to read the TODOs with a deadline by filtering `todo --on DATE`. The TODOs without a date will always appear as current pending TODOs for `today`.
A new flag that is not present in lgg will be added, to allow users to introduce more than one TODO at a time: `--multiple`. When this flags is present, each line after a minus (-) will represent a new TODO. Example: `todo -m My first todo - My second todo`
