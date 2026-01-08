// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Test runner
//!
//! This module provides the test runner for executing test functions.
//!
//! # Example
//!
//! ```zulon
//! #[test]
//! fn test_addition() {
//!     assert_eq(2 + 2, 4);
//! }
//!
//! // Test main (can be auto-generated in the future)
//! fn main() {
//!     run_tests(&[test_addition]);
//! }
//! ```

/// Test function pointer type
pub type TestFunc = unsafe extern "C" fn() -> i32;

/// Test metadata
#[derive(Debug, Clone)]
pub struct Test {
    /// Test name
    pub name: &'static str,
    /// Test function pointer
    pub func: TestFunc,
    /// Whether the test should be ignored
    pub ignored: bool,
    /// Whether the test is expected to panic
    pub should_panic: bool,
}

/// Test result
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// Test passed
    Passed,
    /// Test failed with error message
    Failed(String),
    /// Test was ignored
    Ignored,
    /// Test panicked with message
    Panicked(String),
}

/// Test statistics
#[derive(Debug, Clone, Default)]
pub struct TestStats {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub panicking: usize,
}

/// Run all tests and return statistics
///
/// # Example
///
/// ```zulon
/// fn main() {
///     let tests = &[
///         Test { name: "test_addition", func: test_addition, ignored: false, should_panic: false },
///         Test { name: "test_slow", func: test_slow, ignored: true, should_panic: false },
///     ];
///
///     let stats = run_tests(tests);
///
///     if stats.failed > 0 {
///         std::process::exit(1);
///     }
/// }
/// ```
pub fn run_tests(tests: &[Test]) -> TestStats {
    let mut stats = TestStats::default();

    println!("Running {} test(s)", tests.len());
    println!();

    for test in tests {
        let result = run_test(test);

        match result {
            TestResult::Passed => {
                println!("  test {} ... ok", test.name);
                stats.passed += 1;
            }
            TestResult::Failed(msg) => {
                println!("  test {} ... FAILED", test.name);
                println!("    {}", msg);
                stats.failed += 1;
            }
            TestResult::Ignored => {
                println!("  test {} ... ignored", test.name);
                stats.ignored += 1;
            }
            TestResult::Panicked(msg) => {
                if test.should_panic {
                    println!("  test {} ... ok (panicked as expected)", test.name);
                    stats.passed += 1;
                } else {
                    println!("  test {} ... FAILED (panicked)", test.name);
                    println!("    {}", msg);
                    stats.panicking += 1;
                }
            }
        }
    }

    println!();
    println!("Test result:");
    println!("  {} passed, {} failed, {} ignored, {} panicking",
        stats.passed, stats.failed, stats.ignored, stats.panicking);

    if stats.failed > 0 || stats.panicking > 0 {
        println!();
        println!("Some tests failed. See output above for details.");
    }

    stats
}

/// Run a single test
fn run_test(test: &Test) -> TestResult {
    if test.ignored {
        return TestResult::Ignored;
    }

    // TODO: Implement proper panic catching
    // For now, just call the test function
    // In the future, we'll need to:
    // 1. Set up a panic handler
    // 2. Catch panics
    // 3. Return Panicked result if panic occurred

    unsafe {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (test.func)()
        })) {
            Ok(return_code) => {
                if return_code == 0 {
                    TestResult::Passed
                } else {
                    TestResult::Failed("test returned non-zero exit code".to_string())
                }
            }
            Err(_) => {
                // Extract panic message if possible
                TestResult::Panicked("test panicked".to_string())
            }
        }
    }
}

/// Run a single test with detailed output
pub fn run_test_verbose(test: &Test) {
    println!("Running test: {}", test.name);

    if test.ignored {
        println!("  Status: ignored");
        return;
    }

    let result = run_test(test);

    match result {
        TestResult::Passed => {
            println!("  Status: passed");
        }
        TestResult::Failed(msg) => {
            println!("  Status: failed");
            println!("  Error: {}", msg);
        }
        TestResult::Ignored => {
            println!("  Status: ignored");
        }
        TestResult::Panicked(msg) => {
            if test.should_panic {
                println!("  Status: passed (panicked as expected)");
            } else {
                println!("  Status: panicked");
                println!("  Panic: {}", msg);
            }
        }
    }
}
