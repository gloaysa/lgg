use chrono::{NaiveDate, NaiveTime, Timelike};

/// Check whether `time` satisfies the time filter.
/// - `Single(s)`: matches any time WITHIN the hour.
/// - `Range(start, end)`: covers all times from `start` up to but not including `end`
pub fn time_is_in_range(filter: TimeFilter, time: NaiveTime) -> bool {
    match filter {
        TimeFilter::Single(s) => time.hour() == s.hour(),
        TimeFilter::Range(start, end) => {
            if start <= end {
                start <= time && time < end
            } else {
                time >= start || time < end
            }
        }
    }
}

/// The result of parsing a date string, which can be a single day or a range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DateFilter {
    Single(NaiveDate),
    Range(NaiveDate, NaiveDate),
}

/// The result of parsing a time string, which can be a single time or a range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeFilter {
    Single(NaiveTime),
    Range(NaiveTime, NaiveTime),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime as T;

    fn t(h: u32, m: u32, s: u32) -> T {
        T::from_hms_opt(h, m, s).unwrap()
    }

    #[test]
    fn single_time_matches_by_hour() {
        assert!(time_is_in_range(
            TimeFilter::Single(t(12, 0, 0)),
            t(12, 0, 0)
        ));
        assert!(time_is_in_range(
            TimeFilter::Single(t(12, 0, 0)),
            t(12, 0, 1)
        ));
    }

    #[test]
    fn range_is_half_open_normal() {
        let f = TimeFilter::Range(t(6, 0, 0), t(12, 0, 0)); // [06:00, 12:00)
        assert!(time_is_in_range(f, t(6, 0, 0))); // start included
        assert!(time_is_in_range(f, t(11, 59, 59)));
        assert!(!time_is_in_range(f, t(12, 0, 0))); // end excluded
        assert!(!time_is_in_range(f, t(5, 59, 59)));
    }

    #[test]
    fn range_wraps_midnight() {
        let f = TimeFilter::Range(t(22, 0, 0), t(2, 0, 0)); // [22:00, 02:00)
        assert!(time_is_in_range(f, t(23, 0, 0))); // before midnight
        assert!(time_is_in_range(f, t(1, 59, 59))); // after midnight
        assert!(!time_is_in_range(f, t(2, 0, 0))); // end excluded
        assert!(!time_is_in_range(f, t(21, 59, 59)));
    }

    #[test]
    fn boundaries_across_adjacent_ranges_do_not_double_count() {
        // morning [06:00, 12:00), afternoon [12:00, 18:00)
        let morning = TimeFilter::Range(t(6, 0, 0), t(12, 0, 0));
        let afternoon = TimeFilter::Range(t(12, 0, 0), t(18, 0, 0));
        assert!(time_is_in_range(morning, t(6, 0, 0)));
        assert!(!time_is_in_range(morning, t(12, 0, 0))); // boundary belongs to next range
        assert!(time_is_in_range(afternoon, t(12, 0, 0)));
    }
}
