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
pub fn get_dates_in_range(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut current = start;
    while current <= end {
        dates.push(current);
        current += Duration::days(1);
    }
    dates
}
