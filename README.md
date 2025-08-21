# lgg

A tiny, **human-first** command-line journal tool inspired by [jrnl.sh](https://jrnl.sh).

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
- `last week`
- `last month`
- `last year`

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

- Use the `--on` flag to view all entries for a given date.
- Use the `--from` flag to view all entries from the give day til today.
- Use the `--to` flag in combination with `--from` to view all entries in a range.

> [!NOTE] The current implementation assumes the user won't try and crashed their own computer.
> If you provide a `--from` flag with a date very away in the past (01/01/01), you'll be waiting a while.

```sh
# See all entries from yesterday
lgg --on yesterday

# See all entries from last week (notice the '')
lgg --on 'last week'

# See all entries from a specific date
lgg --on 2025-08-15

# You can use the days of the week too,
# it will fallback to the closest day in the past
lgg --on monday
```

### Editing Entries

- Use the `--edit` flag to edit entries for a given date. Only works on single date searches (`today`, `monday`...).

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
journal_date_format = "%A, %d %b %Y" # e.g., "Friday, 15 Aug 2025"

# The different formats you want to use for the date
# when creating new entries or searching for them.
# Uses chrono's strftime format specifiers.
# See: https://docs.rs/chrono/latest/chrono/format/strftime/
input_date_formats = ["%d/%m/%Y", "%d%m%Y"]

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

## Tips and tricks

You can find some [here](./tips_and_tricks.md).

## Roadmap

The project is currently in the Minimum Viable Product (MVP) phase. To see what's done, what's in progress, and what's planned for the future, please see our official [**ROADMAP.md**](./ROADMAP.md).

## FAQ

- **Why the name lgg?**

* While most people like journaling, I like to do `logging`. While at work, when I finish a task I take a timestamp of it. The name `lgg` is the closest consonant abbreviation that also plays nice when writing it in the keyboard for quick access.

- **Why use this if `jrnl` exits?**

* That's a very good question. [jrnl](https:://jrnl.sh) has been a fantastic tool and allowed me to ditch my physical notebook. I recommend you to use it instead of `lgg`. I created this tool because I want full markdown support. I want to write entries from my phone, knowing that I will be able to access them with `lgg` later on (and hopefully in the future, with the --fix command, auto-format them). If that's not important to you and you want a battle tested tool, go for `jrnl`.

* **How do you use it in your day to day?**

- I like to make entries directly from my [Helix Editor](http://helix-editor.com "Helix website"), this way I never lose context of what I'm doing. I simply type `:! lgg Whatever it's I'm doing or I've finished.` and the entry is created. I also like doing quick searches for today's entries from the editor, in case I left a note to myself earlier in the day or the day before (remember, lgg has the keyword `tomorrow`, so you can do `tomorrow at noon: something to do.`). Once I finish implementing searching for tags, I will probably use them like I used them with jrnl: in meetings. It's very handy to quickly summon your notes about a topic (if you were kind to yourself in the past and properly tagged the topics).

- **How do you use it with your phone?**

* I have this [shortcut](https://www.icloud.com/shortcuts/7fd8cdbbb7bb44038577c953388d593f "iOS Shorcut") to create formatted entries in my shared folder. To read and edit, I use a fantastic app called [Fountain](https://apps.apple.com/es/app/fountain-easy-screenwriting/id6504728966?l "Fountain app in App Store").

## Contributing

_(Contribution guidelines to be added.)_

## License

This project is licensed under the MIT License. Be nice.
