use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

use crate::keywords::{Keyword, Keywords};

/// Default accepted input date formats (parsing only).
const DEFAULT_FORMATS: &[&str] = &["%Y-%m-%d", "%Y%m%d"];

/// Configuration options for parsing functions.
#[derive(Copy, Clone, Debug)]
pub struct ParseOptions<'a> {
    /// The date to use as "today" for relative keywords.
    pub reference_date: Option<NaiveDate>,
    /// A slice of `chrono` format strings to try for parsing dates.
    pub formats: Option<&'a [&'a str]>,
}

impl Default for ParseOptions<'_> {
    fn default() -> Self {
        Self {
            reference_date: None,
            formats: None,
        }
    }
}

/// Parsed result of inline text (e.g., "yesterday: Title. Body").
pub struct ParsedInline {
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub title: String,
    pub body: String,
    /// Whether a date (of any kind) was explicitly provided in the prefix.
    pub explicit_date: bool,
}

/// The main entry point for parsing an inline journal entry from a single string.
///
/// This function orchestrates the parsing of a complete entry, which may
/// contain a date/time prefix, a title, and a body. It handles the logic for splitting
/// the prefix from the content and then the title from the body.
///
/// # Arguments
///
/// * `input` - The raw string provided by the user (e.g., `"yesterday: Title. Body"`).
/// * `options` - An optional [`ParseOptions`] struct to customize parsing behavior.
///   If `None`, default options are used.
///
/// # Returns
///
/// A [`ParsedInline`] struct containing the resolved date, optional time, title, and body.
/// If no date prefix is found, the date defaults to the reference date.
///
/// # Examples
///
/// ```
/// # use chrono::{NaiveDate, NaiveTime};
/// # use lgg_core::parse_input::{parse_entry, ParseOptions};
/// let opts = ParseOptions {
///     reference_date: Some(NaiveDate::from_ymd_opt(2025, 8, 17).unwrap()),
///     ..Default::default()
/// };
///
/// let parsed = parse_entry("yesterday at 8pm: Project kickoff. It went well.", Some(opts));
///
/// assert_eq!(parsed.date, NaiveDate::from_ymd_opt(2025, 8, 16).unwrap());
/// assert_eq!(parsed.time, Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()));
/// assert_eq!(parsed.title, "Project kickoff");
/// assert_eq!(parsed.body, "It went well.");
/// assert!(parsed.explicit_date);
/// ```
pub fn parse_entry(input: &str, options: Option<ParseOptions>) -> ParsedInline {
    let options = options.unwrap_or_default();
    let reference_date = options
        .reference_date
        .unwrap_or_else(|| Local::now().date_naive());
    let formats = options.formats.unwrap_or(DEFAULT_FORMATS);

    let (date_opt, time_opt, rest) = parse_prefix(input, reference_date, formats);
    let (title_raw, body) = split_title_body(rest.trim());
    let title = normalize_title(&title_raw);

    ParsedInline {
        date: date_opt.unwrap_or(reference_date),
        time: time_opt,
        title,
        body,
        explicit_date: date_opt.is_some(),
    }
}

/// Parses a string token into a concrete calendar date (`NaiveDate`).
///
/// This function understands several formats, processed in the following order:
/// 1.  **Relative Keywords**: `today`, `yesterday`, `tomorrow`, and any user-defined
///     synonyms (case-insensitive). These are resolved relative to `reference_date`.
/// 2.  **Formatted Dates**: Any format string provided in the `formats` slice,
///     such as `"%Y-%m-%d"`.
///
/// # Arguments
///
/// * `s` - The string slice to parse.
/// * `options` - An optional [`ParseOptions`] struct to customize parsing behavior.
///
/// # Returns
///
/// `Some(NaiveDate)` if parsing is successful, `None` otherwise.
///
/// # Examples
///
/// ```
/// # use chrono::NaiveDate;
/// # use lgg_core::parse_input::{parse_date_token, ParseOptions};
/// let opts = ParseOptions {
///     reference_date: Some(NaiveDate::from_ymd_opt(2025, 8, 17).unwrap()),
///     formats: Some(&["%Y-%m-%d"]),
/// };
///
/// // Using a keyword
/// let yesterday = parse_date_token("yesterday", Some(opts)).unwrap();
/// assert_eq!(yesterday, NaiveDate::from_ymd_opt(2025, 8, 16).unwrap());
///
/// // Using a formatted string
/// let specific_date = parse_date_token("2025-01-20", Some(opts)).unwrap();
/// assert_eq!(specific_date, NaiveDate::from_ymd_opt(2025, 1, 20).unwrap());
/// ```
pub fn parse_date_token(s: &str, options: Option<ParseOptions>) -> Option<NaiveDate> {
    let options = options.unwrap_or_default();
    let reference_date = options
        .reference_date
        .unwrap_or_else(|| Local::now().date_naive());
    let formats = options.formats.unwrap_or(DEFAULT_FORMATS);

    if Keywords::matches(Keyword::Today, s) {
        return Some(reference_date);
    }
    if Keywords::matches(Keyword::Yesterday, s) {
        return Some(reference_date - Duration::days(1));
    }
    if Keywords::matches(Keyword::Tomorrow, s) {
        return Some(reference_date + Duration::days(1));
    }

    let day_keyword = [
        (Keyword::Monday, Weekday::Mon),
        (Keyword::Tuesday, Weekday::Tue),
        (Keyword::Wednesday, Weekday::Wed),
        (Keyword::Thursday, Weekday::Thu),
        (Keyword::Friday, Weekday::Fri),
        (Keyword::Saturday, Weekday::Sat),
        (Keyword::Sunday, Weekday::Sun),
    ]
    .iter()
    .find(|(keyword, _)| Keywords::matches(*keyword, s));

    if let Some((_, weekday)) = day_keyword {
        let today_wd = reference_date.weekday();
        let days_ago = (today_wd.num_days_from_monday() + 7 - weekday.num_days_from_monday()) % 7;
        return Some(reference_date - Duration::days(days_ago as i64));
    }

    // Fallback to formatted dates
    formats
        .iter()
        .filter_map(|fmt| NaiveDate::parse_from_str(s, fmt).ok())
        .next()
}

/// Parses a string token into a specific time of day (`NaiveTime`).
///
/// This function is case-insensitive and understands several formats, processed in order:
/// 1.  **Keywords**: `noon` (12:00), `midnight` (00:00).
/// 2.  **12-hour Format**: A time ending in `am` or `pm`, with optional minutes.
///     Examples: "6am", "6 pm", "12:30pm".
/// 3.  **24-hour Format (HH:MM)**: e.g., "14:30", "08:00".
/// 4.  **24-hour Format (Hour only)**: A single integer from 0-23. e.g., "8", "17".
///
/// # Arguments
///
/// * `s` - The string slice to parse.
///
/// # Returns
///
/// `Some(NaiveTime)` if parsing is successful, `None` otherwise.
///
/// # Examples
///
/// ```
/// # use chrono::NaiveTime;
/// # use lgg_core::parse_input::parse_time_token;
/// // Using a keyword
/// let noon = parse_time_token("noon").unwrap();
/// assert_eq!(noon, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
///
/// // Using 12-hour format
/// let six_thirty_pm = parse_time_token("6:30 pm").unwrap();
/// assert_eq!(six_thirty_pm, NaiveTime::from_hms_opt(18, 30, 0).unwrap());
///
/// // Using 24-hour format
/// let three_oclock = parse_time_token("15").unwrap();
/// assert_eq!(three_oclock, NaiveTime::from_hms_opt(15, 0, 0).unwrap());
/// ```
pub fn parse_time_token(s: &str) -> Option<NaiveTime> {
    if Keywords::matches(Keyword::Now, s) {
        return Some(Local::now().time());
    }
    if Keywords::matches(Keyword::Noon, s) {
        return NaiveTime::from_hms_opt(12, 0, 0);
    }
    if Keywords::matches(Keyword::Evening, s) {
        return NaiveTime::from_hms_opt(18, 0, 0);
    }
    if Keywords::matches(Keyword::Night, s) {
        return NaiveTime::from_hms_opt(21, 0, 0);
    }
    if Keywords::matches(Keyword::Midnight, s) {
        return NaiveTime::from_hms_opt(0, 0, 0);
    }
    // 12h with am/pm, optional minutes: "6am", "6 pm", "12:30PM"
    if let Some(stripped) = s.strip_suffix("am").or_else(|| s.strip_suffix("pm")) {
        let is_pm = s.ends_with("pm");
        let core = stripped.trim();
        let (h, m, s) = if let Some(colon) = core.find(':') {
            let (h_str, rest) = core.split_at(colon);
            let rest = &rest[1..]; // drop ':'
            let (m_str, s_opt) = if let Some(colon2) = rest.find(':') {
                let (m_str, s_str) = rest.split_at(colon2);
                (m_str, Some(&s_str[1..]))
            } else {
                (rest, None)
            };
            (
                h_str.parse::<u32>().ok()?,
                m_str.parse::<u32>().ok()?,
                s_opt.and_then(|ss| ss.parse::<u32>().ok()).unwrap_or(0),
            )
        } else {
            (core.parse::<u32>().ok()?, 0, 0)
        };
        if h == 0 || h > 12 || m > 59 || s > 59 {
            return None;
        }
        let h24 = match (h, is_pm) {
            (12, false) => 0,
            (12, true) => 12,
            (_, true) => h + 12,
            (_, false) => h,
        };
        return NaiveTime::from_hms_opt(h24, m, s);
    }
    // 24h: "HH:MM"
    if let Ok(nt) = NaiveTime::parse_from_str(&s, "%H:%M") {
        return Some(nt);
    }
    // Single hour (24h format implied): "H" or "HH"
    if let Ok(h) = s.parse::<u32>() {
        if h <= 23 {
            return NaiveTime::from_hms_opt(h, 0, 0);
        }
    }
    None
}

/// Try to parse `<prefix>:` where prefix may contain date and/or time.
/// Returns (date, time, remainder_after_colon).
fn parse_prefix<'a>(
    input: &'a str,
    reference_date: NaiveDate,
    formats: &[&str],
) -> (Option<NaiveDate>, Option<NaiveTime>, &'a str) {
    if let Some(idx) = input.find(": ") {
        let (prefix, rest_with_colon) = input.split_at(idx);
        let rest = &rest_with_colon[1..]; // skip ':'
        let prefix_trim = prefix.trim();
        // Try full ISO-like datetime (no timezone): YYYY-MM-DDTHH:MM[:SS]
        if let Some((d, t)) = parse_iso_datetime(prefix_trim) {
            return (Some(d), Some(t), rest);
        }
        // Split on " at "
        if let Some(word) = Keywords::find_word(Keyword::At, prefix_trim) {
            if let Some(pos) = Keywords::find_position(Keyword::At, prefix_trim) {
                let (date_part, time_part) = prefix_trim.split_at(pos);
                let date_part = date_part.trim();
                let time_part = time_part[word.len()..].trim(); // skip keyword
                let opts = ParseOptions {
                    reference_date: Some(reference_date),
                    formats: Some(formats),
                };
                if let Some(date) = parse_date_token(date_part, Some(opts)) {
                    let time = parse_time_token(time_part);
                    return (Some(date), time, rest);
                }
            }
        }
        // Only a date word or formatted date (no time)
        let opts = ParseOptions {
            reference_date: Some(reference_date),
            formats: Some(formats),
        };
        if let Some(date) = parse_date_token(prefix_trim, Some(opts)) {
            return (Some(date), None, rest);
        }
    }
    // Not recognized: fall through and treat entire input as text.
    (None, None, input)
}

fn parse_iso_datetime(s: &str) -> Option<(NaiveDate, NaiveTime)> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M") {
        return Some((dt.date(), dt.time()));
    }
    None
}

fn split_title_body(text: &str) -> (String, String) {
    if let Some((i, ch)) = text
        .char_indices()
        .find(|&(_, ch)| ch == '\n' || ch == '\r')
    {
        let title = text[..i].trim().to_string();
        let body = text[i + ch.len_utf8()..].trim().to_string();
        return (title, body);
    }
    for (i, ch) in text.char_indices() {
        if ch == '.' || ch == '?' || ch == '!' {
            let title = text[..i].trim().to_string();
            let body = text[i + ch.len_utf8()..].trim().to_string();
            return (title, body);
        }
    }
    (text.trim().to_string(), String::new())
}

/// Remove leading/trailing Markdown `#` and surrounding spaces from the title.
fn normalize_title(s: &str) -> String {
    let mut t = s.trim();
    t = t.trim_start_matches(|c: char| c == '#' || c.is_whitespace());
    t = t.trim_end_matches(|c: char| c == '#' || c.is_whitespace());
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn opts(anchor: NaiveDate) -> Option<ParseOptions<'static>> {
        Some(ParseOptions {
            reference_date: Some(anchor),
            ..Default::default()
        })
    }

    #[test]
    fn iso_date_prefix() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_entry("2025-08-01: Title. Body", opts(anchor));
        assert_eq!(p.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert!(p.time.is_none());
        assert_eq!(p.title, "Title");
        assert_eq!(p.body, "Body");
        assert!(p.explicit_date);
    }

    #[test]
    fn iso_datetime_prefix() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_entry("2025-08-01T13:30: # Title\nBody", opts(anchor));
        assert_eq!(p.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(13, 30, 0).unwrap()));
        assert_eq!(p.title, "Title");
        assert_eq!(p.body, "Body");
        assert!(p.explicit_date);
    }

    #[test]
    fn natural_yesterday_with_time() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p1 = parse_entry("yesterday at 6am: Note 1", opts(anchor));
        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 14).unwrap());
        assert_eq!(p1.time, Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap()));
        assert_eq!(p1.title, "Note 1");
    }

    #[test]
    fn natural_single_hour_with_time() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p1 = parse_entry("today at 9: Note 1", opts(anchor));
        let p2 = parse_entry("today at 17: Note 2", opts(anchor));
        let p3 = parse_entry("today at 9am: Note 3", opts(anchor));
        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p1.time, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(p1.title, "Note 1");
        assert_eq!(p2.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p2.time, Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));
        assert_eq!(p2.title, "Note 2");
        assert_eq!(p3.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p3.time, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(p3.title, "Note 3");
    }

    #[test]
    fn title_newline_body() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_entry("My title\nAnd the body.", opts(anchor));
        assert_eq!(p.title, "My title");
        assert_eq!(p.body, "And the body.");
        assert!(!p.explicit_date);
        assert!(p.time.is_none());
    }

    #[test]
    fn custom_format_dd_mm_yyyy() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let fmts = &["%d-%m-%Y", "%Y-%m-%d", "%d/%m/%Y"];
        let custom_opts = Some(ParseOptions {
            reference_date: Some(anchor),
            formats: Some(fmts),
        });
        let p1 = parse_entry("01-08-2025: Title 1.", custom_opts);
        let p2 = parse_entry("01/09/2025: Title 2.", custom_opts);
        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert!(p1.time.is_none());
        assert_eq!(p1.title, "Title 1");
        assert!(p1.body.is_empty());
        assert!(p1.explicit_date);
        assert_eq!(p2.date, NaiveDate::from_ymd_opt(2025, 9, 1).unwrap());
        assert!(p2.time.is_none());
        assert_eq!(p2.title, "Title 2");
        assert!(p2.body.is_empty());
        assert!(p2.explicit_date);
    }

    #[test]
    fn hashes_stripped_from_title() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_entry("today: # My Title ##\n### Body", opts(anchor));
        assert_eq!(p.title, "My Title");
        assert_eq!(p.body, "### Body");
    }

    #[test]
    fn natural_days_of_week() {
        // Anchor date is a Wednesday
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        // Test parsing of each day of the week relative to the anchor
        let monday = parse_date_token("monday", p_opts).unwrap();
        assert_eq!(monday, NaiveDate::from_ymd_opt(2025, 8, 18).unwrap());

        let tuesday = parse_date_token("tuesday", p_opts).unwrap();
        assert_eq!(tuesday, NaiveDate::from_ymd_opt(2025, 8, 19).unwrap());

        // A day keyword matching the anchor date should return the anchor date
        let wednesday = parse_date_token("wednesday", p_opts).unwrap();
        assert_eq!(wednesday, anchor);

        // Days from the "previous week" should resolve correctly
        let thursday = parse_date_token("thursday", p_opts).unwrap();
        assert_eq!(thursday, NaiveDate::from_ymd_opt(2025, 8, 14).unwrap());

        let friday = parse_date_token("friday", p_opts).unwrap();
        assert_eq!(friday, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());

        let saturday = parse_date_token("saturday", p_opts).unwrap();
        assert_eq!(saturday, NaiveDate::from_ymd_opt(2025, 8, 16).unwrap());

        let sunday = parse_date_token("sunday", p_opts).unwrap();
        assert_eq!(sunday, NaiveDate::from_ymd_opt(2025, 8, 17).unwrap());
    }
}
