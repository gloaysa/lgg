# lgg

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

`lgg` is designed for quick, natural language logging. Text you provide is saved as a new entry. The first sentence ending with a period (`.`) or the first line is treated as the title, and the rest becomes the body.

**Basic Usage**

```sh
# Write a quick entry for today. "A new entry" is the title.
lgg A new entry. The rest is the body.

# Open your default editor ($VISUAL/$EDITOR) to write a longer entry
lgg
```

**Using Dates and Times**

You can specify a date and time for your entry in a flexible, human-readable format. If you only provide a date, the time will be set to the `default_time` in your configuration (21:00 or 9 PM by default).

The prefix before the colon (`:`) is parsed for date and time information.

```sh
# Simple date keywords
lgg yesterday: Finally fixed that annoying bug.
lgg tomorrow: I will prepare for the big presentation.

# Combine dates and times
lgg tomorrow at noon: Team lunch. I hope there is pizza.
lgg today at 6pm: Dinner with family.
lgg friday at 19:30: Movie night.

# Use days of the week (resolves to the most recent past date)
lgg monday: Planned the week\'s tasks.

# Use specific dates
lgg 2025-12-25: Christmas day.
lgg 2025-12-25 at 8am: Opened presents.
```

### Available Keywords

You can use the following keywords (and user-defined synonyms) to specify dates and times. Keywords are case-insensitive.

**Relative Dates**

- `today`
- `yesterday`
- `tomorrow`

**Days of the Week**

- `monday`
- `tuesday`
- `wednesday`
- `thursday`
- `friday`
- `saturday`
- `sunday`

**Times of Day**

- `morning` (08:00)
- `noon` (12:00)
- `evening` (18:00)
- `night` (21:00)
- `midnight` (00:00)

**Time Separator**

- `at` (used to separate date and time parts, e.g., `yesterday at 5pm`)

### Reading Entries

Use the `--on` flag to view all entries for a given day.

```sh
# See all entries from yesterday
lgg --on yesterday

# See all entries from a specific date
lgg --on 2025-08-15

# You can use the days of the week too,
# it will fallback to the closest day in the past
lgg --on monday
```

## Configuration

`lgg` can be configured via a `config.toml` file. It looks for this file in the standard user config directory for your OS:

- **Linux**: `$XDG_CONFIG_HOME/lgg/config.toml` or `~/.config/lgg/config.toml`
- **macOS**: `~/Library/Application Support/lgg/config.toml` or `~/.config/lgg/config.toml`
- **Windows**: `%APPDATA%\lgg\config.toml`

> [!NOTE] You can use the configuration to extend the behaviour of lgg, and even translate it to your language. For that, use the `synonyms` configuration and look at the [keywords](#available-keywords) that can be extended.

Here are all the available options with their defaults:

```toml
# The absolute path to the directory where your journal files are stored.
# If not set, a default directory is chosen based on your OS.
# You can see the active path by running `lgg --path`.
journal_dir = "/path/to/your/journal"

# The command to use for the editor when running `lgg` with no text.
# If not set, it falls back to $VISUAL, then $EDITOR, then "vim".
editor = "hx"

# The time to use for an entry when only a date is provided (eg "yesterday").
# Format is "HH:MM".
default_time = "21:00"

# The format string used for the date in the header of daily files.
# Uses chrono's strftime format specifiers.
# See: https://docs.rs/chrono/latest/chrono/format/strftime/
date_format = "%A, %d %b %Y" # e.g., "Friday, 15 Aug 2025"

# A table of custom synonyms for date keywords.
# The key is your alias, and the value must be a built-in keyword
# (today, yesterday, tomorrow, noon, midnight...).
[synonyms]
ytd = "yesterday"
tmrw = "tomorrow"

# You can even use it to add translations
# Spanish
ayer = "yesterday"
"a las" = "at"

# German
gestern = "yesterday"

# Italian
ieri = "yesterday"

# Japanese
kino = "yesterday"

# now you can do: kino a las 3: I had coffe yesterday at 3.
```

## Roadmap

The project is currently in the Minimum Viable Product (MVP) phase. To see what's done, what's in progress, and what's planned for the future, please see our official [**ROADMAP.md**](./ROADMAP.md).

## Contributing

_(Contribution guidelines to be added.)_

## License

This project is licensed under the MIT License.
