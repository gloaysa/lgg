# lgg

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)

A tiny, **human-first** command-line journaling tool inspired by [jrnl.sh](https://jrnl.sh).

## Mission

`lgg` makes it effortless to log your day in **plain Markdown** from the CLI while keeping your files **yours**: readable, portable, and editable without our app.

There is **no hidden metadata** and **no lock-in**. If you never run `lgg` again, your notes remain simple `.md` files that any editor can open. `lgg` is a convenience layer that timestamps, files, and filters entries; **the Markdown is the product.**

## Core Principles

- **Simplicity over features.** Prefer conventions to configuration.
- **Markdown only.** No custom markup, no binary formats.
- **Zero hidden state.** Everything important is visible in the file.
- **Manual edits are first-class.** Users can create or edit entries on any device and `lgg` should still understand them.

## Installation

_(Instructions to be added once packaging is set up.)_

## Usage

### Creating Entries

`lgg` is designed to be intuitive. Text you provide is saved as a new entry for today.

```sh
# Write a quick entry for today
lgg Wrote the first draft of the README.

# Open your default editor ($VISUAL/$EDITOR) to write a longer entry
lgg

# Create an entry for a specific day
lgg yesterday: Finally fixed that annoying bug.
lgg 2025-08-15: Started a new side project.
```

### Reading Entries

Use the `--on` flag to view all entries for a given day.

```sh
# See all entries from yesterday
lgg --on yesterday

# See all entries from a specific date
lgg --on 2025-08-15
```

## Configuration

`lgg` can be configured via a `config.toml` file. It looks for this file in the standard user config directory for your OS:

- **Linux**: `$XDG_CONFIG_HOME/lgg/config.toml` or `~/.config/lgg/config.toml`
- **macOS**: `~/Library/Application Support/lgg/config.toml` or `~/.config/lgg/config.toml`
- **Windows**: `%APPDATA%\lgg\config.toml`

Here are the available options with their defaults:

```toml
# The absolute path to the directory where your journal files are stored.
# If not set, a default directory is chosen based on your OS.
# You can see the active path by running `lgg --path`.
journal_dir = "/path/to/your/journal"

# The command to use for the editor when running `lgg` with no text.
# If not set, it falls back to $VISUAL, then $EDITOR, then "vim".
editor = "hx"

# The time to use for an entry when only a date is provided (e.g., "yesterday: ...").
# Format is "HH:MM".
default_time = "21:00"

# The format string used for the date in the header of daily files.
# Uses chrono's strftime format specifiers.
# See: https://docs.rs/chrono/latest/chrono/format/strftime/
date_format = "%A, %d %b %Y" # e.g., "Friday, 15 Aug 2025"

# A table of custom synonyms for date keywords.
# The key is your alias, and the value must be a built-in keyword
# (today, yesterday, tomorrow, noon, midnight, "last week").
[synonyms]
ytd = "yesterday"
tmrw = "tomorrow"

# You can even use it to add translations
# Spanish
ayer = "yesterday"

# German
gestern = "yesterday"

# Italian
ieri = "yesterday"

# Japanese
kino = "yesterday"
```

## Roadmap

The project is currently in the Minimum Viable Product (MVP) phase. To see what's done, what's in progress, and what's planned for the future, please see our official [**ROADMAP.md**](./ROADMAP.md).

## Contributing

_(Contribution guidelines to be added.)_

## License

This project is licensed under the MIT License.
