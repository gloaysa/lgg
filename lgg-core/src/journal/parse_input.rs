use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

use crate::keywords::{Keyword, Keywords};

/// Default accepted input date formats (parsing only).
const DEFAULT_FORMATS: &[&str] = &["%d/%m/%Y"];

/// The result of parsing a date string, which can be a single day or a range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DateFilter {
    Single(NaiveDate),
    Range(NaiveDate, NaiveDate),
}

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
    pub tags: Vec<String>,
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
pub fn parse_raw_user_input(input: &str, options: Option<ParseOptions>) -> ParsedInline {
    let options = options.unwrap_or_default();
    let reference_date = options
        .reference_date
        .unwrap_or_else(|| Local::now().date_naive());
    let formats = options.formats.unwrap_or(DEFAULT_FORMATS);

    let (date_opt, time_opt, rest) = parse_prefix(input, reference_date, formats);
    let (title_raw, body) = split_title_body(rest.trim());
    let title = normalize_title(&title_raw);

    let (date, explicit_date) = match date_opt {
        Some(DateFilter::Single(d)) => (d, true),
        Some(DateFilter::Range(start, _)) => (start, true), // For now, just use the start of the range
        None => (reference_date, false),
    };

    ParsedInline {
        date,
        time: time_opt,
        title,
        body,
        tags: Vec::new(),
        explicit_date,
    }
}

/// Parses one or two string tokens into a concrete calendar filter (`DateFilter`).
///
/// This function resolves the **start** token (`start_date`) and, optionally, an **end** token
/// (`end_date`) into either a single day or a date range. Resolution happens in this order:
///
/// 1. **Relative keywords** (case-insensitive), resolved against `reference_date`:
///    - Singles: `today`, `yesterday`, `tomorrow`, weekdays (`monday` … `sunday`)
///    - Ranges:  `last week`, `last month`
///    - User-defined synonyms are supported via the global `Keywords` registry.
/// 2. **Formatted dates** using any format string provided by `formats` (e.g. `"%Y-%m-%d"`).
///
/// # Behavior
///
/// - If the **start** token resolves to a **range**, that range is returned and `end_date` is ignored.
/// - If the **start** token resolves to a **single day** and `end_date` is:
///   - **Absent** → returns `Single(start)`.
///   - **A single day** → returns `Range(start, end)`, **preserving the user’s order**
///     (no reordering even if `start > end`; consumers may yield no results).
///   - **A range** → returns that **range** (the `end_date` range wins in this case).
///
/// Defaults:
/// - If `options.reference_date` is `None`, `Local::now().date_naive()` is used.
/// - If `options.formats` is `None`, `DEFAULT_FORMATS` is used.
///
/// # Arguments
///
/// * `start_date` – Required start token (keyword, weekday, or formatted date).
/// * `end_date`   – Optional end token (keyword, weekday, or formatted date).
/// * `options`    – Optional [`ParseOptions`] to control `reference_date` and `formats`.
///
/// # Returns
///
/// `Some(DateFilter)` if parsing succeeds, `None` otherwise.
///
pub fn parse_date_token(
    start_date: &str,
    end_date: Option<&str>,
    options: Option<ParseOptions>,
) -> Option<DateFilter> {
    let options = options.unwrap_or_default();
    let reference_date = options
        .reference_date
        .unwrap_or_else(|| Local::now().date_naive());
    let formats = options.formats.unwrap_or(DEFAULT_FORMATS);

    let a = resolve_date_token(start_date, reference_date, &formats)?;
    let b = end_date.and_then(|date| resolve_date_token(date, reference_date, &formats));

    match (a, b) {
        // If either side is an range, always return the range.
        (DateFilter::Range(s_date, e_date), _) => Some(DateFilter::Range(s_date, e_date)),

        (DateFilter::Single(_), Some(DateFilter::Range(s_date, e_date))) => {
            Some(DateFilter::Range(s_date, e_date))
        }

        // Two singles
        (DateFilter::Single(a_single_date), Some(DateFilter::Single(b_single_date))) => {
            Some(DateFilter::Range(a_single_date, b_single_date))
        }

        // Only one single
        (DateFilter::Single(a_single_date), None) => Some(DateFilter::Single(a_single_date)),
    }
}

fn resolve_date_token(
    date_string: &str,
    reference_date: NaiveDate,
    formats: &[&str],
) -> Option<DateFilter> {
    if Keywords::matches(Keyword::Today, date_string) {
        return Some(DateFilter::Single(reference_date));
    }
    if Keywords::matches(Keyword::Yesterday, date_string) {
        return Some(DateFilter::Single(reference_date - Duration::days(1)));
    }
    if Keywords::matches(Keyword::Tomorrow, date_string) {
        return Some(DateFilter::Single(reference_date + Duration::days(1)));
    }
    if Keywords::matches(Keyword::LastWeek, date_string) {
        let today_wd = reference_date.weekday();
        let days_to_last_sunday = today_wd.num_days_from_sunday();
        let last_sunday = reference_date - Duration::days(days_to_last_sunday as i64);
        let start_of_last_week = last_sunday - Duration::days(6);
        return Some(DateFilter::Range(start_of_last_week, last_sunday));
    }
    if Keywords::matches(Keyword::ThisWeek, date_string) {
        let days_from_monday = reference_date.weekday().num_days_from_monday();
        let start_of_week = reference_date - Duration::days(days_from_monday as i64);
        let end_of_week = start_of_week + Duration::days(6);
        return Some(DateFilter::Range(start_of_week, end_of_week));
    }
    if Keywords::matches(Keyword::LastMonth, date_string) {
        let first_of_this_month = reference_date.with_day(1)?;
        let end_of_last_month = first_of_this_month - Duration::days(1);
        let start_of_last_month = end_of_last_month.with_day(1)?;
        return Some(DateFilter::Range(start_of_last_month, end_of_last_month));
    }
    if Keywords::matches(Keyword::ThisMonth, date_string) {
        let start_of_month = reference_date.with_day(1)?;
        let (y, m) = if start_of_month.month() == 12 {
            (start_of_month.year() + 1, 1)
        } else {
            (start_of_month.year(), start_of_month.month() + 1)
        };
        let end_of_month = NaiveDate::from_ymd_opt(y, m, 1)? - Duration::days(1);
        return Some(DateFilter::Range(start_of_month, end_of_month));
    }

    if Keywords::matches(Keyword::LastYear, date_string) {
        let y = reference_date.year() - 1;
        let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
        let end = NaiveDate::from_ymd_opt(y, 12, 31)?;
        return Some(DateFilter::Range(start, end));
    }
    if Keywords::matches(Keyword::ThisYear, date_string) {
        let y = reference_date.year();
        let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
        let end = NaiveDate::from_ymd_opt(y, 12, 31)?;
        return Some(DateFilter::Range(start, end));
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
    .find(|(keyword, _)| Keywords::matches(*keyword, date_string));

    if let Some((_, weekday)) = day_keyword {
        let today_wd = reference_date.weekday();
        let days_ago = (today_wd.num_days_from_monday() + 7 - weekday.num_days_from_monday()) % 7;
        return Some(DateFilter::Single(
            reference_date - Duration::days(days_ago as i64),
        ));
    }

    // Fallback to formatted dates
    formats
        .iter()
        .filter_map(|fmt| NaiveDate::parse_from_str(date_string, fmt).ok())
        .map(|d| DateFilter::Single(d))
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
pub fn parse_time_token(s: &str) -> Option<NaiveTime> {
    if Keywords::matches(Keyword::Morning, s) {
        return NaiveTime::from_hms_opt(8, 0, 0);
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

    let lower_s = s.to_ascii_lowercase();
    if lower_s.ends_with("am") || lower_s.ends_with("pm") {
        let (core_str, suffix) = s.split_at(s.len() - 2);
        let is_pm = suffix.to_ascii_lowercase() == "pm";
        let core = core_str.trim();

        let parts = if let Some(colon) = core.find(':') {
            let (h_str, rest) = core.split_at(colon);
            let rest = &rest[1..];
            let (m_str, s_str_opt) = if let Some(colon2) = rest.find(':') {
                let (m, s_part) = rest.split_at(colon2);
                (m, Some(&s_part[1..]))
            } else {
                (rest, None)
            };

            if let (Ok(h), Ok(m)) = (h_str.parse::<u32>(), m_str.parse::<u32>()) {
                let s = s_str_opt.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                Some((h, m, s))
            } else {
                None
            }
        } else {
            if let Ok(h) = core.parse::<u32>() {
                Some((h, 0, 0))
            } else {
                None
            }
        };

        if let Some((h, m, s)) = parts {
            if h == 0 || h > 12 || m > 59 || s > 59 {
                return None;
            }
            let h24 = match (h, is_pm) {
                (12, false) => 0, // 12am is midnight
                (12, true) => 12, // 12pm is noon
                (_, true) => h + 12,
                (_, false) => h,
            };
            return NaiveTime::from_hms_opt(h24, m, s);
        } else {
            return None; // Parsing of h,m,s failed
        }
    }

    // 24h: "HH:MM"
    if let Ok(nt) = NaiveTime::parse_from_str(s, "%H:%M") {
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
) -> (Option<DateFilter>, Option<NaiveTime>, &'a str) {
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
                if let Some(date) = parse_date_token(date_part, None, Some(opts)) {
                    let time = parse_time_token(time_part);
                    return (Some(date), time, rest);
                } else {
                    let time = parse_time_token(time_part);
                    return (None, time, rest);
                }
            }
        }
        // Only a date word or formatted date (no time)
        let opts = ParseOptions {
            reference_date: Some(reference_date),
            formats: Some(formats),
        };
        if let Some(date) = parse_date_token(prefix_trim, None, Some(opts)) {
            return (Some(date), None, rest);
        }
    }
    // Not recognized: fall through and treat entire input as text.
    (None, None, input)
}

fn parse_iso_datetime(s: &str) -> Option<(DateFilter, NaiveTime)> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M") {
        return Some((DateFilter::Single(dt.date()), dt.time()));
    }
    None
}

fn split_title_body(text: &str) -> (String, String) {
    if let Some((i, ch)) = text
        .char_indices()
        .find(|&(_, ch)| ch == '\n' || ch == '\r')
    {
        let title = text[..(i + ch.len_utf8())].trim().to_string();
        let body = text[i + ch.len_utf8()..].trim().to_string();
        return (title, body);
    }
    for (i, ch) in text.char_indices() {
        if ch == '.' || ch == '?' || ch == '!' {
            let title = text[..(i + ch.len_utf8())].trim().to_string();
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
        let p = parse_raw_user_input("01/08/2025: Title.\n Body", opts(anchor));
        assert_eq!(p.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert!(p.time.is_none());
        assert_eq!(p.title, "Title.");
        assert_eq!(p.body, "Body");
        assert!(p.explicit_date);
    }

    #[test]
    fn iso_datetime_prefix() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_raw_user_input("2025-08-01T13:30: # Title\nBody", opts(anchor));
        assert_eq!(p.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(13, 30, 0).unwrap()));
        assert_eq!(p.title, "Title");
        assert_eq!(p.body, "Body");
        assert!(p.explicit_date);
    }

    #[test]
    fn natural_yesterday_with_time() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p1 = parse_raw_user_input("yesterday at 6am: Note 1", opts(anchor));
        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 14).unwrap());
        assert_eq!(p1.time, Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap()));
        assert_eq!(p1.title, "Note 1");
    }

    #[test]
    fn natural_single_hour_with_time() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p1 = parse_raw_user_input("today at 9: Note 1", opts(anchor));
        let p2 = parse_raw_user_input("today at 17: Note 2", opts(anchor));
        let p3 = parse_raw_user_input("today at 9am: Note 3", opts(anchor));
        let p4 = parse_raw_user_input("at morning: Note 4", opts(anchor));
        let p5 = parse_raw_user_input("today at morning: Note 5", opts(anchor));
        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p1.time, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(p1.title, "Note 1");
        assert_eq!(p2.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p2.time, Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));
        assert_eq!(p2.title, "Note 2");
        assert_eq!(p3.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p3.time, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(p3.title, "Note 3");
        assert_eq!(p4.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p4.time, Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));
        assert_eq!(p4.title, "Note 4");
        assert_eq!(p5.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
        assert_eq!(p5.time, Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));
        assert_eq!(p5.title, "Note 5");
    }

    #[test]
    fn title_newline_body() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_raw_user_input("My title\nAnd the body.", opts(anchor));
        assert_eq!(p.title, "My title");
        assert_eq!(p.body, "And the body.");
        assert!(!p.explicit_date);
        assert!(p.time.is_none());
    }

    #[test]
    fn body_with_sub_headers() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_raw_user_input("My title\nAnd the body.\n### Header 3", opts(anchor));
        assert_eq!(p.title, "My title");
        assert_eq!(p.body, "And the body.\n### Header 3");
        assert!(!p.explicit_date);
        assert!(p.time.is_none());
    }

    #[test]
    fn custom_format_dd_mm_yyyy() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let fmts = &["%d-%m-%Y", "%d/%m/%Y"];
        let custom_opts = Some(ParseOptions {
            reference_date: Some(anchor),
            formats: Some(fmts),
        });
        let p1 = parse_raw_user_input("01-08-2025: Title 1.", custom_opts);
        let p2 = parse_raw_user_input("01/09/2025: Title 2.", custom_opts);
        assert_eq!(p1.date, NaiveDate::from_ymd_opt(2025, 8, 1).unwrap());
        assert!(p1.time.is_none());
        assert_eq!(p1.title, "Title 1.");
        assert!(p1.body.is_empty());
        assert!(p1.explicit_date);
        assert_eq!(p2.date, NaiveDate::from_ymd_opt(2025, 9, 1).unwrap());
        assert!(p2.time.is_none());
        assert_eq!(p2.title, "Title 2.");
        assert!(p2.body.is_empty());
        assert!(p2.explicit_date);
    }

    #[test]
    fn hashes_stripped_from_title() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        let p = parse_raw_user_input("today: # My Title ##\n### Body", opts(anchor));
        assert_eq!(p.title, "My Title");
        assert_eq!(p.body, "### Body");
    }

    #[test]
    fn natural_days_of_week() {
        // Anchor date is a Wednesday
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        // Test parsing of each day of the week relative to the anchor
        let monday = parse_raw_user_input("monday: Task A", p_opts);
        assert_eq!(monday.date, NaiveDate::from_ymd_opt(2025, 8, 18).unwrap());

        let tuesday = parse_raw_user_input("tuesday: Task B", p_opts);
        assert_eq!(tuesday.date, NaiveDate::from_ymd_opt(2025, 8, 19).unwrap());

        // A day keyword matching the anchor date should return the anchor date
        let wednesday = parse_raw_user_input("wednesday: Task C", p_opts);
        assert_eq!(wednesday.date, anchor);

        // Days from the "previous week" should resolve correctly
        let thursday = parse_raw_user_input("thursday: Task D", p_opts);
        assert_eq!(thursday.date, NaiveDate::from_ymd_opt(2025, 8, 14).unwrap());

        let friday = parse_raw_user_input("friday: Task E", p_opts);
        assert_eq!(friday.date, NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());

        let saturday = parse_raw_user_input("saturday: Task F", p_opts);
        assert_eq!(saturday.date, NaiveDate::from_ymd_opt(2025, 8, 16).unwrap());

        let sunday = parse_raw_user_input("sunday: Task G", p_opts);
        assert_eq!(sunday.date, NaiveDate::from_ymd_opt(2025, 8, 17).unwrap());
    }

    #[test]
    fn time_token_parsing() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        let p = parse_raw_user_input("at morning: Title", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));

        let p = parse_raw_user_input("today at morning: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));

        let p = parse_raw_user_input("tuesday at noon: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()));

        let p = parse_raw_user_input("wednesday at evening: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()));

        let p = parse_raw_user_input("thursday at night: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(21, 0, 0).unwrap()));

        let p = parse_raw_user_input("friday at midnight: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));

        // 12-hour format
        let p = parse_raw_user_input("11/04/2025 at 5am: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(5, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 5pm: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 5:30am: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(5, 30, 0).unwrap()));

        let p = parse_raw_user_input("at 5:30 pm: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(17, 30, 0).unwrap()));

        let p = parse_raw_user_input("at 12am: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 12pm: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 5PM: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 12:45AM: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(0, 45, 0).unwrap()));

        // 24-hour format
        let p = parse_raw_user_input("at 08:00: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 23:59: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(23, 59, 0).unwrap()));

        let p = parse_raw_user_input("at 8: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));

        let p = parse_raw_user_input("at 17: Title A", p_opts);
        assert_eq!(p.time, Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));

        // Invalid
        let p = parse_raw_user_input("at 25:00: Title A", p_opts);
        assert!(p.time.is_none());
        let p = parse_raw_user_input("at 13:00pm: Title A", p_opts);
        assert!(p.time.is_none());
        let p = parse_raw_user_input("at not-a-time: Title A", p_opts);
        assert!(p.time.is_none());
    }

    #[test]
    fn natural_date_ranges() {
        // Anchor date is a Wednesday
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        // Last Week
        let last_week = parse_date_token("last week", None, p_opts).unwrap();
        assert_eq!(
            last_week,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 11).unwrap(), // Monday
                NaiveDate::from_ymd_opt(2025, 8, 17).unwrap()  // Sunday
            )
        );

        let last_month = parse_date_token("last month", None, p_opts).unwrap();
        assert_eq!(
            last_month,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 7, 31).unwrap()
            )
        );
    }

    #[test]
    fn natural_this_date_ranges() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap(); // Wed
        let p_opts = opts(anchor);

        let this_week = parse_date_token("this week", None, p_opts).unwrap();
        assert_eq!(
            this_week,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 18).unwrap(), // Monday
                NaiveDate::from_ymd_opt(2025, 8, 24).unwrap()  // Sunday
            )
        );

        let this_month = parse_date_token("this month", None, p_opts).unwrap();
        assert_eq!(
            this_month,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 8, 31).unwrap()
            )
        );

        let this_year = parse_date_token("this year", None, p_opts).unwrap();
        assert_eq!(
            this_year,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
            )
        );
    }

    #[test]
    fn start_range_ignores_end_single() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        let res = parse_date_token("last week", Some("2025-08-01"), p_opts).unwrap();
        assert_eq!(
            res,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 11).unwrap(), // Mon
                NaiveDate::from_ymd_opt(2025, 8, 17).unwrap(), // Sun
            )
        );
    }

    #[test]
    fn start_range_ignores_end_range() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        let res = parse_date_token("last month", Some("last week"), p_opts).unwrap();
        assert_eq!(
            res,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 7, 31).unwrap(),
            )
        );
    }

    #[test]
    fn two_singles_preserve_user_order_even_when_fucked_up() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        let res = parse_date_token("20/08/2025", Some("10/08/2025"), p_opts).unwrap();
        assert_eq!(
            res,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 20).unwrap(),
                NaiveDate::from_ymd_opt(2025, 8, 10).unwrap(),
            )
        );
    }

    #[test]
    fn weekday_plus_single_end_becomes_range_preserving_order() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap(); // Wed
        let end_date = Some("19/08/2025");
        let p_opts = opts(anchor);

        let res = parse_date_token("monday", end_date, p_opts).unwrap();
        assert_eq!(
            res,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 18).unwrap(),
                NaiveDate::from_ymd_opt(2025, 8, 19).unwrap(),
            )
        );
    }

    #[test]
    fn start_single_end_is_range_returns_that_range() {
        let anchor = NaiveDate::from_ymd_opt(2025, 8, 20).unwrap();
        let p_opts = opts(anchor);

        let res = parse_date_token("10/08/2025", Some("last week"), p_opts).unwrap();
        assert_eq!(
            res,
            DateFilter::Range(
                NaiveDate::from_ymd_opt(2025, 8, 11).unwrap(),
                NaiveDate::from_ymd_opt(2025, 8, 17).unwrap(),
            )
        );
    }
}
