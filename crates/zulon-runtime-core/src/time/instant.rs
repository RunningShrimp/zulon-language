// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Instant type for time measurement

use std::fmt;
use std::time::{Duration as StdDuration, SystemTime};

/// Error type for Instant operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstantError {
    /// Instant is in the future
    InFuture,

    /// Duration would overflow
    Overflow,
}

impl fmt::Display for InstantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstantError::InFuture => write!(f, "Instant is in the future"),
            InstantError::Overflow => write!(f, "Duration overflow"),
        }
    }
}

/// Represents a point in time
///
/// Used for measuring elapsed time and creating deadlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    /// Unix timestamp in nanoseconds
    timestamp: i128,
}

impl Instant {
    /// Creates a new Instant from a Unix timestamp
    pub const fn from_nanos(timestamp: i128) -> Self {
        Instant { timestamp }
    }

    /// Creates a new Instant from a Unix timestamp in seconds
    pub const fn from_secs(timestamp: i64) -> Self {
        Instant {
            timestamp: (timestamp as i128) * 1_000_000_000,
        }
    }

    /// Creates a new Instant from a Unix timestamp in milliseconds
    pub const fn from_millis(timestamp: i64) -> Self {
        Instant {
            timestamp: (timestamp as i128) * 1_000_000,
        }
    }

    /// Gets the current time
    pub fn now() -> Self {
        let sys_time = SystemTime::now();
        let duration = sys_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(StdDuration::ZERO);

        Instant {
            timestamp: duration.as_nanos() as i128,
        }
    }

    /// Gets the Unix timestamp in seconds
    pub const fn as_secs(&self) -> i64 {
        (self.timestamp / 1_000_000_000) as i64
    }

    /// Gets the Unix timestamp in milliseconds
    pub const fn as_millis(&self) -> i64 {
        (self.timestamp / 1_000_000) as i64
    }

    /// Gets the Unix timestamp in nanoseconds
    pub const fn as_nanos(&self) -> i128 {
        self.timestamp
    }

    /// Calculates the elapsed time since this instant
    pub fn elapsed(&self) -> super::Duration {
        let now = Self::now();
        let elapsed_nanos = now.timestamp - self.timestamp;

        if elapsed_nanos > 0 {
            super::Duration::from_nanos(elapsed_nanos as u64)
        } else {
            super::Duration::ZERO
        }
    }

    /// Checks if this instant is in the past
    pub fn is_past(&self) -> bool {
        self.timestamp < Self::now().timestamp
    }

    /// Checks if this instant is in the future
    pub fn is_future(&self) -> bool {
        self.timestamp > Self::now().timestamp
    }

    /// Adds a duration to this instant
    pub fn checked_add(&self, duration: super::Duration) -> Result<Self, InstantError> {
        match self.timestamp.checked_add(duration.as_nanos() as i128) {
            Some(timestamp) => Ok(Instant { timestamp }),
            None => Err(InstantError::Overflow),
        }
    }

    /// Subtracts a duration from this instant
    pub fn checked_sub(&self, duration: super::Duration) -> Result<Self, InstantError> {
        match self.timestamp.checked_sub(duration.as_nanos() as i128) {
            Some(timestamp) => Ok(Instant { timestamp }),
            None => Err(InstantError::Overflow),
        }
    }

    /// Calculates the duration since another instant
    pub fn duration_since(&self, earlier: Self) -> super::Duration {
        let elapsed_nanos = self.timestamp - earlier.timestamp;
        super::Duration::from_nanos(elapsed_nanos as u64)
    }

    /// Checks if this instant is before another
    pub const fn is_before(&self, other: Self) -> bool {
        self.timestamp < other.timestamp
    }

    /// Checks if this instant is after another
    pub const fn is_after(&self, other: Self) -> bool {
        self.timestamp > other.timestamp
    }

    /// Gets the earliest of two instants
    pub const fn min(self, other: Self) -> Self {
        if self.timestamp < other.timestamp {
            self
        } else {
            other
        }
    }

    /// Gets the latest of two instants
    pub const fn max(self, other: Self) -> Self {
        if self.timestamp > other.timestamp {
            self
        } else {
            other
        }
    }

    /// Unix epoch (1970-01-01 00:00:00 UTC)
    pub const UNIX_EPOCH: Self = Instant { timestamp: 0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instant_from_units() {
        let now = Instant::now();
        assert!(now.as_nanos() >= 0);

        let from_secs = Instant::from_secs(now.as_secs());
        assert_eq!(from_secs.as_secs(), now.as_secs());

        let from_millis = Instant::from_millis(now.as_millis());
        assert_eq!(from_millis.as_millis(), now.as_millis());
    }

    #[test]
    fn test_instant_elapsed() {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() >= 100);
    }

    #[test]
    fn test_instant_past_future() {
        let now = Instant::now();
        let past = Instant::from_nanos(now.as_nanos() - 1_000_000);
        let future = Instant::from_nanos(now.as_nanos() + 1_000_000);

        assert!(past.is_past());
        assert!(!past.is_future());
        assert!(!now.is_past());
        assert!(!now.is_future());

        assert!(future.is_future());
        assert!(!future.is_past());
    }

    #[test]
    fn test_instant_comparison() {
        let now = Instant::now();
        let earlier = Instant::from_nanos(now.as_nanos() - 1_000_000);
        let later = Instant::from_nanos(now.as_nanos() + 1_000_000);

        assert!(earlier.is_before(now));
        assert!(later.is_after(now));
        assert!(now.is_before(later));
        assert!(now.is_after(earlier));

        assert_eq!(Instant::min(now, later), now);
        assert_eq!(Instant::max(now, earlier), now);
    }

    #[test]
    fn test_instant_arithmetic() {
        let now = Instant::now();
        let later = now.checked_add(super::Duration::from_secs(10)).unwrap();
        assert!(later.is_after(now));

        let earlier = now.checked_sub(super::Duration::from_secs(5)).unwrap();
        assert!(earlier.is_before(now));
    }

    #[test]
    fn test_instant_duration_since() {
        let now = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let later = Instant::now();

        let elapsed = later.duration_since(now);
        assert!(elapsed.as_millis() >= 100);
    }

    #[test]
    fn test_unix_epoch() {
        assert_eq!(Instant::UNIX_EPOCH.as_nanos(), 0);
    }
}
