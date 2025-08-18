use chrono::{Duration, NaiveDate};

/// Generates a vector of `NaiveDate`s, inclusive of the start and end dates.
/// If `start` is after `end`, the resultin vector will be empty.
///
/// # Arguments
///
/// * `start` - The `NaiveDate` to start the range from (inclusive).
/// * `end` - The `NaiveDate` to end the range at (inclusive).
///
/// # Returns
///
/// A `Vec<NaiveDate>` containing all the dates from `start` to `end`.
///
/// # Examples
///
/// ```
/// # use chrono::NaiveDate;
/// # use lgg_core::dates::get_dates_in_range;
/// let start_date = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
/// let end_date = NaiveDate::from_ymd_opt(2025, 8, 17).unwrap();
///
/// let dates = get_dates_in_range(start_date, end_date);
///
/// assert_eq!(dates.len(), 3);
/// assert_eq!(dates[0], NaiveDate::from_ymd_opt(2025, 8, 15).unwrap());
/// assert_eq!(dates[1], NaiveDate::from_ymd_opt(2025, 8, 16).unwrap());
/// assert_eq!(dates[2], NaiveDate::from_ymd_opt(2025, 8, 17).unwrap());
/// ```
pub fn get_dates_in_range(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut current = start;
    while current <= end {
        dates.push(current);
        current += Duration::days(1);
    }
    dates
}
