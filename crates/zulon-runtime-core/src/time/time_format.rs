// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Time formatting and parsing utilities

use crate::time::{Date, Instant};
use std::fmt;

/// Formats a timestamp into a string
///
/// # Format Specifiers
///
/// - `%Y`: 4-digit year
/// - `%y`: 2-digit year
/// - `%m`: 2-digit month
/// - `%d`: 2-digit day
/// - `%H`: 2-digit hour (00-23)
/// - `%M`: 2-digit minute (00-59)
/// - `%S`: 2-digit second (00-59)
/// - `%s`: Unix timestamp (seconds since epoch)
pub fn format_timestamp(timestamp: Instant, format: &str) -> String {
    let mut result = String::new();
    let mut chars = format.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(fmt_ch) = chars.next() {
                match fmt_ch {
                    'Y' => {
                        let year = timestamp.as_secs() / 31536000 + 1970;
                        result.push_str(&format!("{:04}", year));
                    }
                    'y' => {
                        let year = (timestamp.as_secs() / 31536000 + 1970) % 100;
                        result.push_str(&format!("{:02}", year));
                    }
                    'm' => {
                        let date = Date::from_timestamp(timestamp.as_secs());
                        result.push_str(&format!("{:02}", date.month()));
                    }
                    'd' => {
                        let date = Date::from_timestamp(timestamp.as_secs());
                        result.push_str(&format!("{:02}", date.day()));
                    }
                    'H' => {
                        let secs = timestamp.as_secs() % 86400;
                        let hours = (secs / 3600) % 24;
                        result.push_str(&format!("{:02}", hours));
                    }
                    'M' => {
                        let secs = timestamp.as_secs() % 3600;
                        let mins = (secs / 60) % 60;
                        result.push_str(&format!("{:02}", mins));
                    }
                    'S' => {
                        let secs = timestamp.as_secs() % 60;
                        result.push_str(&format!("{:02}", secs));
                    }
                    's' => {
                        result.push_str(&format!("{}", timestamp.as_secs()));
                    }
                    _ => {
                        result.push('%');
                        result.push(fmt_ch);
                    }
                }
            } else {
                result.push('%');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Parses a time string into a Unix timestamp
///
/// Supports formats:
/// - Unix timestamp (seconds): "1234567890"
pub fn parse_time(input: &str) -> Result<i64, ParseError> {
    let trimmed = input.trim();

    if let Ok(timestamp) = trimmed.parse::<i64>() {
        if timestamp >= 0 && timestamp <= 4102444800 {
            return Ok(timestamp);
        }
    }

    Err(ParseError::InvalidFormat)
}

/// Error type for parsing operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid time format
    InvalidFormat,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid time format"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        let timestamp = Instant::from_secs(1234567890);

        assert_eq!(
            format_timestamp(timestamp, "%Y-%m-%d %H:%M:%S"),
            "2009-02-13 23:31:30"
        );

        assert_eq!(format_timestamp(timestamp, "%Y/%m/%d"), "2009/02/13");

        assert_eq!(format_timestamp(timestamp, "%s"), "1234567890");
    }

    #[test]
    fn test_parse_time() {
        let timestamp = "1234567890";
        assert_eq!(parse_time(timestamp).unwrap(), 1234567890);

        let invalid = "not-a-timestamp";
        assert!(parse_time(invalid).is_err());
    }

    #[test]
    fn test_format_edge_cases() {
        let timestamp = Instant::UNIX_EPOCH;
        assert_eq!(
            format_timestamp(timestamp, "%Y-%m-%d %H:%M:%S"),
            "1970-01-01 00:00:00"
        );
    }
}
