// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type display helpers
//!
//! This module provides utilities for converting types to human-readable strings.

/// Display a type in a user-friendly format
pub trait TypeDisplay {
    /// Convert type to display string
    fn display_type(&self) -> String;

    /// Convert type to short display string (for error messages)
    fn display_short(&self) -> String {
        self.display_type()
    }
}

/// Helper function for displaying types with optional formatting
pub fn format_type_list<T: TypeDisplay>(types: &[T], separator: &str) -> String {
    types.iter()
        .map(|t| t.display_type())
        .collect::<Vec<_>>()
        .join(separator)
}

/// Format a type mismatch error message
pub fn format_type_mismatch(expected: &dyn TypeDisplay, found: &dyn TypeDisplay) -> String {
    format!(
        "expected type `{}`, found type `{}`",
        expected.display_short(),
        found.display_short()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestType(String);

    impl TypeDisplay for TestType {
        fn display_type(&self) -> String {
            self.0.clone()
        }
    }

    #[test]
    fn test_format_type_list() {
        let types = vec![
            TestType("i32".to_string()),
            TestType("f64".to_string()),
            TestType("bool".to_string()),
        ];

        assert_eq!(format_type_list(&types, ", "), "i32, f64, bool");
    }

    #[test]
    fn test_format_type_mismatch() {
        let expected = TestType("i32".to_string());
        let found = TestType("f64".to_string());

        let msg = format_type_mismatch(&expected, &found);
        assert_eq!(msg, "expected type `i32`, found type `f64`");
    }
}
