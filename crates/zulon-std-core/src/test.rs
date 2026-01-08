// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Testing support
//!
//! This module provides assertion functions used in test functions.
//!
//! # Example
//!
//! ```zulon
//! #[test]
//! fn test_addition() {
//!     assert_eq(2 + 2, 4);
//! }
//!
//! #[test]
//! fn test_not_equal() {
//!     assert_ne(5, 10, "values should not be equal");
//! }
//! ```

/// Assert that a condition is true.
///
/// # Panics
///
/// Panics with the provided message if the condition is false.
///
/// # Examples
///
/// ```zulon
/// assert!(x > 0, "x must be positive");
/// assert!(not_nil(ptr));
/// ```
pub fn assert(condition: bool, message: Option<&str>) {
    if !condition {
        match message {
            Some(msg) => panic!("assertion failed: {}", msg),
            None => panic!("assertion failed"),
        }
    }
}

/// Assert that two values are equal.
///
/// # Panics
///
/// Panics with a detailed message if the values are not equal.
///
/// # Examples
///
/// ```zulon
/// assert_eq(2 + 2, 4);
/// assert_eq(result, expected, "calculation failed");
/// ```
pub fn assert_eq<T: PartialEq + std::fmt::Display>(left: T, right: T, message: Option<&str>) {
    if left != right {
        match message {
            Some(msg) => panic!("assertion failed: {} == {}: {}", left, right, msg),
            None => panic!("assertion failed: {} == {}", left, right),
        }
    }
}

/// Assert that two values are not equal.
///
/// # Panics
///
/// Panics with a detailed message if the values are equal.
///
/// # Examples
///
/// ```zulon
/// assert_ne(vec.len(), 0, "vector should not be empty");
/// assert_ne(x, y);
/// ```
pub fn assert_ne<T: PartialEq + std::fmt::Display>(left: T, right: T, message: Option<&str>) {
    if left == right {
        match message {
            Some(msg) => panic!("assertion failed: {} != {}: {}", left, right, msg),
            None => panic!("assertion failed: {} != {}", left, right),
        }
    }
}

/// Panic with a formatted message.
///
/// This is used internally by assertion functions and can be used
/// directly in test code.
///
/// # Examples
///
/// ```zulon
/// if invalid_condition {
///     panic!("invalid state: {}", state);
/// }
/// ```
pub fn panic(message: &str) -> ! {
    // TODO: Implement proper panic mechanism
    // For now, this will be handled by the runtime
    // In the future, this will:
    // 1. Unwind the stack
    // 2. Print the panic message
    // 3. Exit with non-zero status
    eprintln!("PANIC: {}", message);
    std::process::exit(1)
}
