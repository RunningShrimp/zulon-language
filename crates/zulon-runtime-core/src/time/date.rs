// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Date type for calendar operations

use std::fmt;

/// Error type for Date operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    /// Invalid date components
    InvalidComponents,

    /// Invalid year
    InvalidYear,

    /// Invalid month
    InvalidMonth,

    /// Invalid day
    InvalidDay,

    /// Date is out of range
    OutOfRange,
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateError::InvalidComponents => write!(f, "Invalid date components"),
            DateError::InvalidYear => write!(f, "Invalid year"),
            DateError::InvalidMonth => write!(f, "Invalid month"),
            DateError::InvalidDay => write!(f, "Invalid day"),
            DateError::OutOfRange => write!(f, "Date out of range"),
        }
    }
}

/// Calendar date
///
/// Represents a date in the Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// Creates a new Date from year, month, and day
    ///
    /// # Panics
    ///
    /// Panics if month is not in range 1..=12 or day is not valid for the given month.
    pub const fn from_ymd(year: u16, month: u8, day: u8) -> Self {
        assert!(month >= 1 && month <= 12, "month must be in range 1..=12");
        assert!(
            day >= 1 && day <= Self::days_in_month(year, month),
            "day must be valid for the given month and year"
        );

        Date { year, month, day }
    }

    /// Creates a new Date from components, returning an error if invalid
    pub fn try_from_ymd(year: u16, month: u8, day: u8) -> Result<Self, DateError> {
        if month < 1 || month > 12 {
            return Err(DateError::InvalidMonth);
        }
        if year < 1 || year > 9999 {
            return Err(DateError::InvalidYear);
        }

        let max_day = Self::days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(DateError::InvalidDay);
        }

        Ok(Date { year, month, day })
    }

    /// Creates a new Date from a Unix timestamp (seconds since epoch)
    pub fn from_timestamp(secs: i64) -> Self {
        let days_since_epoch = secs / 86400;
        let mut remaining_days = days_since_epoch;

        let mut year = 1970u16;
        let month = 1u8;

        let days_in_year = |y: u16| -> u16 {
            if Date::is_leap_year(y) {
                366
            } else {
                365
            }
        };

        while remaining_days > 0 {
            let days_this_year = days_in_year(year);
            if remaining_days < days_this_year as i64 {
                return Self::from_ymd_approximate(year, month, remaining_days as u8);
            } else {
                remaining_days -= days_this_year as i64;
                year += 1;
            }
        }

        Self::from_ymd(year, month, 1)
    }

    /// Creates a new Date from components without validation
    ///
    /// # Safety
    ///
    /// This function assumes valid components. Use `try_from_ymd` for safe construction.
    const fn from_ymd_approximate(year: u16, month: u8, day: u8) -> Self {
        Date { year, month, day }
    }

    /// Gets the current date (local time)
    pub fn today() -> Self {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        Self::from_timestamp(duration.as_secs() as i64)
    }

    /// Gets the current UTC date
    pub fn today_utc() -> Self {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        let timestamp = duration.as_secs() as i64;
        // Adjust for timezone (UTC = no adjustment)
        Self::from_timestamp(timestamp)
    }

    /// Gets the year
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// Gets the month (1-12)
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// Gets the day (1-31)
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Checks if year is a leap year
    pub const fn is_leap_year(year: u16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Gets the number of days in a given month
    #[allow(unconditional_panic)]
    pub const fn days_in_month(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    /// Converts to Unix timestamp (seconds since epoch)
    pub fn to_timestamp(&self) -> i64 {
        let mut total_days = 0i64;

        for y in 1970..self.year {
            total_days += if Self::is_leap_year(y) { 366 } else { 365 } as i64;
        }

        for m in 1..self.month {
            total_days += Self::days_in_month(self.year, m) as i64;
        }

        total_days += (self.day - 1) as i64;

        total_days * 86400
    }

    /// Gets the day of year (1-366)
    pub fn day_of_year(&self) -> u16 {
        let mut day_count = 0u16;

        for m in 1..self.month {
            day_count += Self::days_in_month(self.year, m) as u16;
        }

        day_count + self.day as u16
    }

    /// Gets the day of week (0=Sunday, 6=Saturday)
    ///
    /// Uses Zeller's congruence algorithm for Gregorian calendar.
    pub fn day_of_week(&self) -> u8 {
        let (q, m) = if self.month < 3 {
            (self.year - 1, self.month + 12)
        } else {
            (self.year, self.month)
        };

        let _h = (q + q / 4 - q / 100 + q / 400) % 7;
        let k = (self.day + 13 * (m + 1) / 5) % 7;
        let d = (k + k - k % 4 - 2 * k) % 7;

        (d + 7) % 7
    }

    /// Checks if this date is before another date
    pub const fn is_before(&self, other: Date) -> bool {
        self.year < other.year
            || (self.year == other.year && self.month < other.month)
            || (self.year == other.year && self.month == other.month && self.day < other.day)
    }

    /// Checks if this date is after another date
    pub const fn is_after(&self, other: Date) -> bool {
        self.year > other.year
            || (self.year == other.year && self.month > other.month)
            || (self.year == other.year && self.month == other.month && self.day > other.day)
    }

    /// Adds days to this date
    ///
    /// Returns `Err` if date becomes invalid.
    pub fn add_days(&self, days: i32) -> Result<Date, DateError> {
        let mut new_date = *self;
        let mut remaining = days;

        while remaining != 0 {
            let days_this_year = if Date::is_leap_year(new_date.year) {
                366
            } else {
                365
            };

            if (remaining > 0) == (remaining < days_this_year as i32) {
                if new_date.month < 12 {
                    new_date.month += 1;
                    new_date.day = 1;
                } else {
                    new_date.year += 1;
                    new_date.month = 1;
                    new_date.day = 1;
                }

                let days_in_month = Date::days_in_month(new_date.year, new_date.month) as i32;
                remaining -= days_in_month;
            } else {
                new_date.day = (new_date.day as i32 + remaining) as u8;
                remaining = 0;
            }
        }

        if Date::is_leap_year(new_date.year) {
            Ok(new_date)
        } else {
            Err(DateError::OutOfRange)
        }
    }

    /// Adds months to this date
    ///
    /// Returns `Err` if date becomes invalid.
    pub fn add_months(&self, months: i32) -> Result<Date, DateError> {
        let mut new_date = *self;
        let mut remaining = months;

        while remaining != 0 {
            let months_this_year = 12 - new_date.month + 1;

            if (remaining > 0) == (remaining < months_this_year as i32) {
                new_date.month += remaining as u8;
                remaining = 0;
            } else {
                new_date.month += months_this_year as u8;
                remaining -= months_this_year as i32;
            }

            if new_date.month > 12 {
                new_date.year += 1;
                new_date.month = 1;
            }
        }

        // Adjust day if the new month doesn't have enough days
        let max_day = Date::days_in_month(new_date.year, new_date.month);
        if new_date.day > max_day {
            new_date.day = max_day;
        }

        Ok(new_date)
    }

    /// Adds years to this date
    ///
    /// Returns `Err` if date becomes invalid.
    pub fn add_years(&self, years: i32) -> Result<Date, DateError> {
        let new_year = self.year as i32 + years;
        if new_year < 1 || new_year > 9999 {
            return Err(DateError::InvalidYear);
        }

        let mut new_date = *self;
        new_date.year = new_year as u16;

        // Adjust day if the new date is Feb 29 on a non-leap year
        if new_date.month == 2 && new_date.day == 29 && !Date::is_leap_year(new_date.year) {
            new_date.day = 28;
        }

        Ok(new_date)
    }

    /// Calculates the difference in days between two dates
    pub fn days_since(&self, other: Date) -> i64 {
        let self_ts = self.to_timestamp();
        let other_ts = other.to_timestamp();
        (self_ts - other_ts).abs() / 86400
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_from_ymd() {
        let date = Date::from_ymd(2024, 2, 29);
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 29);
    }

    #[test]
    fn test_date_try_from_ymd_invalid() {
        assert!(Date::try_from_ymd(2024, 2, 30).is_ok());
        assert!(Date::try_from_ymd(2024, 2, 30).is_err());

        assert!(Date::try_from_ymd(2024, 13, 1).is_err()); // Invalid month
        assert!(Date::try_from_ymd(0, 2, 1).is_err()); // Invalid year
    }

    #[test]
    fn test_leap_year() {
        assert!(Date::is_leap_year(2000));
        assert!(!Date::is_leap_year(2001));
        assert!(Date::is_leap_year(2004));
        assert!(!Date::is_leap_year(2100));
        assert!(Date::is_leap_year(2400));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(Date::days_in_month(2024, 1), 31);
        assert_eq!(Date::days_in_month(2024, 2), 29); // Leap year Feb
        assert_eq!(Date::days_in_month(2023, 2), 28); // Non-leap year Feb
    }

    #[test]
    fn test_date_comparison() {
        let date1 = Date::from_ymd(2024, 1, 1);
        let date2 = Date::from_ymd(2024, 1, 2);

        assert!(date1.is_before(date2));
        assert!(date2.is_after(date1));
        assert!(!date1.is_after(date2));
        assert!(!date2.is_before(date1));
    }

    #[test]
    fn test_day_of_year() {
        let date = Date::from_ymd(2024, 3, 1);
        assert_eq!(date.day_of_year(), 61); // Jan (31) + Feb (29) + Mar (1) = 61
    }

    #[test]
    fn test_day_of_week() {
        // 2024-01-01 was a Monday
        let date = Date::from_ymd(2024, 1, 1);
        assert!(matches!(date.day_of_week(), 1 | 2 | 3 | 4 | 5 | 6));
    }

    #[test]
    fn test_date_arithmetic() {
        let date = Date::from_ymd(2024, 1, 1);

        let result = date.add_days(365).unwrap();
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 1);

        let result2 = date.add_months(12).unwrap();
        assert_eq!(result2.year(), 2025);
        assert_eq!(result2.month(), 1);
    }

    #[test]
    fn test_to_timestamp() {
        let date = Date::from_ymd(1970, 1, 1);
        assert_eq!(date.to_timestamp(), 0); // Epoch

        let date2 = Date::from_ymd(1970, 1, 2);
        assert_eq!(date2.to_timestamp(), 86400); // One day later
    }

    #[test]
    fn test_from_timestamp() {
        let date = Date::from_timestamp(86400); // One day after epoch
        assert_eq!(date.year(), 1970);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 2);

        let date2 = Date::from_timestamp(86400 * 366); // One year later (1971 is leap)
        assert_eq!(date2.year(), 1971);
        assert_eq!(date2.month(), 1);
        assert_eq!(date2.day(), 2);
    }

    #[test]
    fn test_date_display() {
        let date = Date::from_ymd(2024, 2, 29);
        assert_eq!(format!("{}", date), "2024-02-29");
    }

    #[test]
    fn test_leap_year_edge_case() {
        let date = Date::from_ymd(2020, 2, 29);
        assert!(date.add_years(4).is_ok()); // 2024 is leap year
        assert!(Date::try_from_ymd(2020, 2, 30).is_ok()); // 2020 is leap year, no Feb 30
        assert!(Date::try_from_ymd(2020, 2, 29).is_err()); // 2020 is leap year, no Feb 30
    }

    #[test]
    fn test_century_edge_cases() {
        assert!(Date::is_leap_year(1900)); // Century is not divisible by 400
        assert!(!Date::is_leap_year(2000)); // 2000 is divisible by 400
        assert!(!Date::is_leap_year(2400)); // 2400 is not divisible by 400

        assert!(Date::try_from_ymd(9999, 12, 31).is_ok()); // Year within range
        assert!(Date::try_from_ymd(10000, 12, 31).is_err()); // Year too high
        assert!(Date::try_from_ymd(0, 1, 1).is_err()); // Year too low
    }

    #[test]
    fn test_year_boundaries() {
        assert!(Date::is_leap_year(1900)); // Century is not divisible by 400
        assert!(!Date::is_leap_year(2000)); // 2000 is divisible by 400

        // Test year boundaries
        assert!(Date::try_from_ymd(9999, 12, 31).is_ok());
        assert!(Date::try_from_ymd(10000, 12, 31).is_err()); // Out of range

        // Test leap year centuries
        assert!(Date::is_leap_year(1600)); // Not divisible by 400
        assert!(!Date::is_leap_year(2000)); // Divisible by 400
        assert!(Date::is_leap_year(2400)); // Divisible by 400

        // Test year 0
        assert!(Date::try_from_ymd(0, 1, 1).is_ok()); // Valid date (year 0, month 1, day 1)
        assert!(Date::try_from_ymd(0, 1, 1).is_ok()); // Valid date
    }

    #[test]
    fn test_year_boundaries() {
        assert!(Date::try_from_ymd(9999, 12, 31).is_err()); // Year too high
        assert!(Date::try_from_ymd(0, 1, 1).is_ok()); // Year too low
    }

    #[test]
    fn test_month_boundaries() {
        // Month 0 is invalid
        assert!(Date::try_from_ymd(2024, 0, 1).is_err()); // Invalid month 0

        // Month 13 is invalid
        assert!(Date::try_from_ymd(2024, 13, 1).is_err()); // Invalid month 13

        // Month is bounded by 12
        assert!(Date::try_from_ymd(2024, 1, 31).is_ok()); // January has 31 days
        assert!(Date::try_from_ymd(2024, 4, 30).is_ok()); // April has 30 days
        assert!(Date::try_from_ymd(2024, 6, 31).is_ok()); // June has 30 days
    }

    #[test]
    fn test_day_boundaries() {
        // Day 0 is invalid
        assert!(Date::try_from_ymd(2024, 1, 0).is_err()); // Invalid day 0

        // Day bounds based on month/year
        assert!(Date::try_from_ymd(2024, 2, 29).is_ok()); // Feb 2024 has 29 days (leap)
        assert!(Date::try_from_ymd(2024, 2, 28).is_err()); // Feb 2025 has 28 days
        assert!(Date::try_from_ymd(2024, 4, 30).is_ok()); // April has 30 days
        assert!(Date::try_from_ymd(2024, 4, 31).is_err()); // April has 31 days
    }

    #[test]
    fn test_date_comparison_consistency() {
        // Test transitive property: if a < b and b < c, then a < c
        let date1 = Date::from_ymd(2024, 1, 1);
        let date2 = Date::from_ymd(2024, 1, 2);
        let date3 = Date::from_ymd(2024, 1, 3);
        let date4 = Date::from_ymd(2024, 1, 4);
        let date5 = Date::from_ymd(2024, 1, 5);

        assert!(date1.is_before(date2));
        assert!(date2.is_before(date3));
        assert!(date3.is_before(date4));
        assert!(date4.is_before(date5));
        assert!(!date2.is_before(date1)); // Not before itself
    }

    #[test]
    fn test_date_arithmetic_consistency() {
        // Test that date + n days = expected result
        let date = Date::from_ymd(2024, 1, 1);

        // Add 10 days
        let result = date.add_days(10).unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 11);

        // Add 1 month
        let result = date.add_months(1).unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month(), 2);
        assert_eq!(result.day(), 1); // Feb has 28 days
    }

    #[test]
    fn test_day_of_week_consistency() {
        // Day of week should be consistent with actual calendar
        let date = Date::from_ymd(2024, 1, 3); // March 1, 2024 is a Friday (day 5)
        assert_eq!(date.day_of_week(), 5);

        let date2 = Date::from_ymd(2024, 1, 1); // Jan 1, 1900 was a Monday (day 1)
        assert_eq!(date2.day_of_week(), 1);
    }

    #[test]
    fn test_days_since() {
        let date1 = Date::from_ymd(2024, 1, 1);
        let date2 = Date::from_ymd(2024, 1, 10);
        assert_eq!(date1.days_since(date2), -9); // 9 days difference
        assert_eq!(date2.days_since(date1), 9);
    }
}
