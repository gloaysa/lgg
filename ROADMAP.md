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
- [ ] **Improved Tag Support**: Enhanced parsing and indexing for `@tags`.
- [ ] **Raycast extension**: No idea how Raycast extensions work, but will look into it. Would be nice to be able to create entries from it when not at the terminal. There's a work-around for it at the moment.
- [ ] **Templates**: If the app gets some tracking and users want this feature.
