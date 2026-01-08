// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Assertion macros for the ZULON testing framework
//!
//! This module provides assertion macros used in tests:
//! - `assert!(condition)` - Asserts a condition is true
//! - `assert_eq!(left, right)` - Asserts two values are equal
//! - `assert_ne!(left, right)` - Asserts two values are not equal
//! - `assert_matches!(value, pattern)` - Asserts a value matches a pattern
//! - `assert_true!(value)` - Asserts a boolean is true
//! - `assert_false!(value)` - Asserts a boolean is false

/// Asserts that a condition is true
///
/// # Panics
///
/// Panics with the provided message if the condition is false
///
/// # Examples
///
/// ```zulon
/// assert!(x > 0, "x must be positive");
/// assert!(y.len() > 0);
/// ```
///
/// # Equivalent To
///
/// This is a macro that expands to:
/// ```zulon
/// if !(condition) {
///     panic!("assertion failed: {}", message);
/// }
/// ```
#[macro_export]
macro_rules! assert {
    ($cond:expr $(, $msg:expr)?) => {
        if !$cond {
            panic!("assertion failed");
        }
    };
}

/// Asserts that two values are equal
///
/// # Panics
///
/// Panics if the left and right values are not equal
///
/// # Examples
///
/// ```zulon
/// assert_eq!(2 + 2, 4);
/// assert_eq!(vec![1, 2], vec![1, 2]);
/// assert_eq!("hello", "hello");
/// ```
///
/// # Message Format
///
/// On failure, prints:
/// ```text
/// assertion failed: `(left == right)`
///   left: `<left_value>`
///   right: `<right_value>`
/// ```
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr $(, $msg:expr)?) => {
        if $left != $right {
            panic!("assertion failed: `(left == right)`");
        }
    };
}

/// Asserts that two values are not equal
///
/// # Panics
///
/// Panics if the left and right values are equal
///
/// # Examples
///
/// ```zulon
/// assert_ne!(1, 2);
/// assert_ne!(vec![1], vec![2]);
/// assert_ne!("hello", "world");
/// ```
///
/// # Message Format
///
/// On failure, prints:
/// ```text
/// assertion failed: `(left != right)`
///   value: `<value>`
/// ```
#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr $(, $msg:expr)?) => {
        if $left == $right {
            panic!("assertion failed: `(left != right)`");
        }
    };
}

/// Asserts that a value matches a pattern
///
/// # Panics
///
/// Panics if the value doesn't match the given pattern
///
/// # Examples
///
/// ```zulon
/// enum Option<T> {
///     Some(T),
///     None,
/// }
///
/// let result = Some(42);
/// assert_matches!(result, Some(_));
/// ```
#[macro_export]
macro_rules! assert_matches {
    ($value:expr, $pat:pat $(, $msg:expr)?) => {
        match $value {
            $pat => {},
            _ => panic!("assertion failed: value doesn't match pattern"),
        }
    };
}

/// Asserts that a boolean value is true
///
/// # Panics
///
/// Panics if the value is false
///
/// # Examples
///
/// ```zulon
/// assert_true!(x.is_some());
/// assert_true!(y > 0);
/// ```
#[macro_export]
macro_rules! assert_true {
    ($value:expr $(, $msg:expr)?) => {
        assert!($value);
    };
}

/// Asserts that a boolean value is false
///
/// # Panics
///
/// Panics if the value is true
///
/// # Examples
///
/// ```zulon
/// assert_false!(x.is_none());
/// assert_false!(y < 0);
/// ```
#[macro_export]
macro_rules! assert_false {
    ($value:expr $(, $msg:expr)?) => {
        assert!(!$value);
    };
}

#[cfg(test)]
mod tests {
    // Note: These are meta-tests that verify the macro syntax
    // They don't actually run until the macros are integrated
    // into the compiler
}
