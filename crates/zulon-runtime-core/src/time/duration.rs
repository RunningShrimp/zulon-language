// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Duration type for time spans

use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::time::Duration as StdDuration;

/// Error type for Duration operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationError {
    /// Duration is too large
    Overflow,

    /// Duration is negative
    Negative,

    /// Invalid conversion
    InvalidConversion,
}

impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationError::Overflow => write!(f, "Duration overflow"),
            DurationError::Negative => write!(f, "Duration is negative"),
            DurationError::InvalidConversion => write!(f, "Invalid duration conversion"),
        }
    }
}

/// Time duration
///
/// Represents a span of time with nanosecond precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Duration {
    nanos: u64,
}

impl Duration {
    /// Creates a new Duration from nanoseconds
    pub const fn from_nanos(nanos: u64) -> Self {
        Duration { nanos }
    }

    /// Creates a new Duration from microseconds
    pub const fn from_micros(micros: u64) -> Self {
        Duration {
            nanos: micros.saturating_mul(1_000),
        }
    }

    /// Creates a new Duration from milliseconds
    pub const fn from_millis(millis: u64) -> Self {
        Duration {
            nanos: millis.saturating_mul(1_000_000),
        }
    }

    /// Creates a new Duration from seconds
    pub const fn from_secs(secs: u64) -> Self {
        Duration {
            nanos: secs.saturating_mul(1_000_000_000),
        }
    }

    /// Creates a new Duration from minutes
    pub const fn from_mins(mins: u64) -> Self {
        Duration {
            nanos: mins.saturating_mul(60_000_000_000),
        }
    }

    /// Creates a new Duration from hours
    pub const fn from_hours(hours: u64) -> Self {
        Duration {
            nanos: hours.saturating_mul(3_600_000_000_000),
        }
    }

    /// Creates a new Duration from days
    pub const fn from_days(days: u64) -> Self {
        Duration {
            nanos: days.saturating_mul(86_400_000_000_000),
        }
    }

    /// Creates a new Duration from weeks
    pub const fn from_weeks(weeks: u64) -> Self {
        Duration {
            nanos: weeks.saturating_mul(604_800_000_000_000),
        }
    }

    /// Zero duration
    pub const ZERO: Duration = Duration { nanos: 0 };

    /// One nanosecond
    pub const NANOS: Duration = Duration { nanos: 1 };

    /// One microsecond
    pub const MICROS: Duration = Duration { nanos: 1_000 };

    /// One millisecond
    pub const MILLIS: Duration = Duration { nanos: 1_000_000 };

    /// One second
    pub const SECS: Duration = Duration {
        nanos: 1_000_000_000,
    };

    /// One minute
    pub const MINUTES: Duration = Duration {
        nanos: 60_000_000_000,
    };

    /// One hour
    pub const HOURS: Duration = Duration {
        nanos: 3_600_000_000_000,
    };

    /// One day
    pub const DAYS: Duration = Duration {
        nanos: 86_400_000_000_000,
    };

    /// One week
    pub const WEEKS: Duration = Duration {
        nanos: 604_800_000_000_000,
    };

    /// Creates a Duration from std::time::Duration
    pub fn from_std(std_duration: StdDuration) -> Result<Self, DurationError> {
        let nanos = std_duration.as_nanos();
        #[allow(unused_comparisons)]
        if nanos >= 0 {
            Ok(Duration {
                nanos: nanos as u64,
            })
        } else {
            Err(DurationError::Negative)
        }
    }

    /// Converts to std::time::Duration
    pub fn to_std(&self) -> StdDuration {
        StdDuration::from_nanos(self.nanos)
    }

    /// Gets duration as nanoseconds
    pub const fn as_nanos(&self) -> u64 {
        self.nanos
    }

    /// Checks if duration is zero
    pub const fn is_zero(&self) -> bool {
        self.nanos == 0
    }

    /// Saturating addition
    pub fn saturating_add(self, rhs: Duration) -> Duration {
        Duration {
            nanos: self.nanos.saturating_add(rhs.nanos),
        }
    }

    /// Saturating subtraction
    pub fn saturating_sub(self, rhs: Duration) -> Duration {
        Duration {
            nanos: self.nanos.saturating_sub(rhs.nanos),
        }
    }

    /// Saturating multiplication
    pub fn saturating_mul(self, rhs: u32) -> Duration {
        Duration {
            nanos: self.nanos.saturating_mul(rhs as u64),
        }
    }

    /// Saturating division
    pub fn saturating_div(self, rhs: u32) -> Duration {
        Duration {
            nanos: self.nanos.saturating_div(rhs as u64),
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        Duration {
            nanos: self.nanos + rhs.nanos,
        }
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.nanos += rhs.nanos;
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        Duration {
            nanos: self.nanos.saturating_sub(rhs.nanos),
        }
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.nanos = self.nanos.saturating_sub(rhs.nanos);
    }
}

impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        Duration {
            nanos: self.nanos.saturating_mul(rhs as u64),
        }
    }
}

impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        Duration {
            nanos: self.nanos.saturating_div(rhs as u64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_from_units() {
        assert_eq!(Duration::from_nanos(1_000), Duration::NANOS);
        assert_eq!(Duration::from_micros(1_000), Duration::MICROS);
        assert_eq!(Duration::from_millis(1_000), Duration::MILLIS);
        assert_eq!(Duration::from_secs(60), Duration::MINUTES);
        assert_eq!(Duration::from_mins(60), Duration::HOURS);
        assert_eq!(Duration::from_hours(24), Duration::DAYS);
    }

    #[test]
    fn test_duration_consts() {
        assert_eq!(Duration::ZERO.as_nanos(), 0);
        assert_eq!(Duration::NANOS.as_nanos(), 1);
        assert_eq!(Duration::MICROS.as_nanos(), 1_000);
        assert_eq!(Duration::MILLIS.as_nanos(), 1_000_000);
        assert_eq!(Duration::SECS.as_nanos(), 1_000_000_000);
        assert_eq!(Duration::MINUTES.as_nanos(), 60_000_000_000);
        assert_eq!(Duration::HOURS.as_nanos(), 3_600_000_000_000);
        assert_eq!(Duration::DAYS.as_nanos(), 86_400_000_000_000);
        assert_eq!(Duration::WEEKS.as_nanos(), 604_800_000_000_000);
    }

    #[test]
    fn test_duration_conversions() {
        let d = Duration::from_secs(3661);
        assert_eq!(d.as_nanos(), 3661000000000);
    }

    #[test]
    fn test_duration_addition() {
        let d1 = Duration::from_secs(1);
        let d2 = Duration::from_secs(2);
        let d3 = d1 + d2;
        assert_eq!(d3.as_nanos(), 3_000_000_000);
    }

    #[test]
    fn test_duration_subtraction() {
        let d1 = Duration::from_secs(5);
        let d2 = Duration::from_secs(2);
        let d3 = d1 - d2;
        assert_eq!(d3.as_nanos(), 3_000_000_000);
    }

    #[test]
    fn test_duration_saturating() {
        let d = Duration::from_millis(u64::MAX);
        assert_eq!(
            d.saturating_add(Duration::from_secs(1)).as_nanos(),
            u64::MAX
        );

        let d2 = Duration::from_millis(100);
        let d3 = d2.saturating_sub(Duration::from_secs(1));
        assert_eq!(d3.as_nanos(), 0);
    }

    #[test]
    fn test_duration_zero() {
        assert!(Duration::ZERO.is_zero());
        assert!(!Duration::SECS.is_zero());
    }

    #[test]
    fn test_duration_comparison() {
        let d1 = Duration::from_secs(1);
        let d2 = Duration::from_secs(2);
        let d3 = Duration::from_secs(2);

        assert!(d1 < d2);
        assert!(d2 >= d1);
        assert!(d2 == d3);
    }
}
