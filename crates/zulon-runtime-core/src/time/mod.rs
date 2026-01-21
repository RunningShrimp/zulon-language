// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Time and Date Module
//!
//! This module provides time and date primitives for ZULON programs.
//!
//! ## Features
//!
//! - **Instant**: Represents a point in time
//! - **Duration**: Represents a span of time
//! - **Date**: Calendar date operations
//! - **DateTime**: Date and time with timezone support
//! - **Time Utilities**: Formatting, parsing, and conversions
//!
//! ## Example
//!
//! ```rust
//! use zulon_runtime_core::{Instant, Duration, DateTime};
//!
//! let now = DateTime::now();
//! println!("Current time: {:?}", now);
//! ```

mod date;
mod duration;
mod instant;
mod time_format;

pub use date::{Date, DateError};
pub use duration::{Duration, DurationError};
pub use instant::{Instant, InstantError};
pub use time_format::{format_timestamp, parse_time};

/// Represents a timezone offset from UTC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeZone {
    /// UTC timezone
    Utc,
    /// Local timezone
    Local,
    /// Fixed offset in seconds from UTC
    FixedOffset(i32),
}

/// Date and time with timezone support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u16,
    second: u8,
    nanos: u32,
    tz: TimeZone,
}

struct MonthDays {
    month: u8,
    days_in_month: u8,
}

impl DateTime {
    pub const UNIX_EPOCH: Self = DateTime {
        year: 1970,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
        nanos: 0,
        tz: TimeZone::Utc,
    };

    pub fn now() -> Self {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        let secs = duration.as_secs();
        Self::from_timestamp(secs)
    }

    pub fn utc_now() -> Self {
        Self {
            tz: TimeZone::Utc,
            ..Self::now()
        }
    }

    pub const fn from_timestamp(secs: i64) -> Self {
        let days = secs / 86400;
        let remaining_secs = secs % 86400;

        let (year, remaining_days) = Self::days_to_year(1970u16, days as u64);

        let month = Self::days_to_month(year, remaining_days);
        let day = (remaining_days % month.days_in_month) as u8 + 1;
        let hours = (remaining_secs % 86400) as u8;
        remaining_secs = remaining_secs % 3600;
        let minute = (remaining_secs / 60) as u16;
        let second = remaining_secs % 60;
        let nanos = 0;

        DateTime {
            year,
            month: month.month,
            day,
            hour,
            minute,
            second,
            nanos,
            tz: TimeZone::Utc,
        }
    }

    const fn days_to_year(base_year: u16, days: u64) -> (u16, u64) {
        let days_in_year = if Date::is_leap_year(base_year) {
            366u64
        } else {
            365u64
        };

        let mut year = base_year;
        let mut remaining_days = days;

        while remaining_days >= days_in_year {
            remaining_days -= days_in_year;
            year += 1;
        }

        (year, remaining_days)
    }

    const fn days_to_month(year: u16, mut remaining_days: u16) -> MonthDays {
        let month = 1u8;
        let mut days_in_current_month = Date::days_in_month(year, month);

        while remaining_days >= days_in_current_month {
            remaining_days -= days_in_current_month;
            month += 1;
            if month <= 12 {
                days_in_current_month = Date::days_in_month(year, month);
            }
        }

        MonthDays {
            month,
            days_in_month: days_in_current_month,
        }
    }

    pub fn timestamp(&self) -> i64 {
        let mut total_days = 0i64;

        for y in 1970..self.year {
            total_days += if Date::is_leap_year(y) { 366i64 } else { 365i64 };
        }

        for m in 1..self.month {
            total_days += Date::days_in_month(self.year, m) as i64;
        }

        total_days += (self.day - 1) as i64;

        (total_days * 86400i64) + self.hour as i64 * 3600i64 + self.minute as i64 * 60 + self.second as i64
    }

        for m in 1..self.month {
            total_days += Date::days_in_month(self.year, m) as i64;
        }

        total_days += (self.day - 1) as i64;

        (total_days * 86400i64)
            + self.hours as i64 * 3600i64
            + self.minute as i64 * 60
            + self.second as i64
    }

    pub fn format(&self, fmt: &str) -> String {
        let month_str = format!("{:02}", self.month);
        let day_str = format!("{:02}", self.day);
        let hour_str = format!("{:02}", self.hour);
        let minute_str = format!("{:02}", self.minute);
        let second_str = format!("{:02}", self.second);

        let tz_str = match self.tz {
            TimeZone::Utc => "+00:00".to_string(),
            TimeZone::Local => "Local".to_string(),
            TimeZone::FixedOffset(offset) => {
                let sign = if offset < 0 { "-" } else { "+" };
                format!("{}{:02}:00", sign, offset.abs())
            }
        };

        fmt.replace("%Y", &format!("{:04}", self.year))
            .replace("%m", &month_str)
            .replace("%d", &day_str)
            .replace("%H", &hour_str)
            .replace("%M", &minute_str)
            .replace("%S", &second_str)
            .replace("%z", &tz_str)
    }

    pub const fn with_timezone(mut self, tz: TimeZone) -> Self {
        self.tz = tz;
        self
    }

    pub fn to_utc(&self) -> Self {
            Self {
                tz: TimeZone::Utc,
                ..*self
            }
        }
    }

    /// Gets current time
    pub fn now() -> DateTime {
        DateTime::now()
}

/// Gets current UTC time
pub fn utc_now() -> DateTime {
    DateTime::utc_now()
}

/// Sleeps current thread for a specified duration
pub fn sleep(duration: Duration) {
    std::thread::sleep(duration.to_std());
}

/// Delays current thread for a specified duration (alias for sleep)
pub fn delay(duration: Duration) {
    sleep(duration);
}

/// Gets current time (Instant)
pub fn instant_now() -> Instant {
    Instant::now()
}

/// Gets current UTC time (Instant)
pub fn instant_utc_now() -> Instant {
    Instant::now()
}
