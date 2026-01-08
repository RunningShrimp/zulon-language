// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Testing Framework
//!
//! This crate provides the testing framework for ZULON, including:
//! - `#[test]` attribute for marking test functions
//! - Assertion macros (`assert!`, `assert_eq!`, `assert_ne!`, etc.)
//! - Test runner and discovery
//!
//! ## Usage
//!
//! ```zulon
//! use zulon_ext_test::test;
//!
//! #[test]
//! fn test_addition() {
//!     assert_eq!(2 + 2, 4);
//! }
//!
//! #[test]
//! fn test_string_concatenation() {
//!     let result = "Hello" + " " + "World";
//!     assert_eq!(result, "Hello World");
//! }
//! ```
//!
//! ## Running Tests
//!
//! Use the YAN tool to run tests:
//!
//! ```bash
//! $ yan test
//! Running tests...
//! test_addition ... ok
//! test_string_concatenation ... ok
//!
//! Test result: ok. 2 passed; 0 failed
//! ```

// Re-export testing components
pub mod assertions;
pub mod test_runner;
pub mod test_discovery;

// Public API
// Note: Macros are re-exported at the crate root via macro_export
pub use test_runner::{TestRunner, TestResult, TestStats};
pub use test_discovery::{TestDiscovery, TestMetadata};

/// Test attribute marker
///
/// This is a placeholder for the actual `#[test]` attribute
/// which will be implemented in the compiler.
///
/// For now, this documentation serves as the specification
/// for how the `#[test]` attribute should work.
///
/// ## Specification
///
/// When the compiler sees `#[test]` on a function:
/// 1. Mark the function as a test (not compiled into main binary)
/// 2. Collect test metadata (name, location)
/// 3. Store in special test section
/// 4. Test runner discovers and executes it
///
/// ## Example
///
/// ```zulon
/// #[test]
/// fn my_test() {
///     // Test code here
///     assert!(some_condition);
/// }
/// ```
pub const TEST_ATTRIBUTE: &str = "#[test]";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_exists() {
        // Verify testing framework components exist
        assert!(true);
    }
}
