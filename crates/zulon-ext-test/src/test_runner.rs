// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test runner for executing discovered tests
//!
//! This module handles the execution of tests and reporting of results

use crate::test_discovery::TestMetadata;

/// Result of running a single test
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed
    Passed,
    /// Test failed with error message
    Failed(String),
    /// Test was ignored
    Ignored,
}

/// Statistics about test run
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestStats {
    /// Total number of tests
    pub total: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of ignored tests
    pub ignored: usize,
}

impl TestStats {
    /// Create empty test stats
    pub fn new() -> Self {
        TestStats {
            total: 0,
            passed: 0,
            failed: 0,
            ignored: 0,
        }
    }

    /// Check if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.total > 0
    }

    /// Calculate pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    /// Add a test result to stats
    pub fn add_result(&mut self, result: &TestResult) {
        self.total += 1;
        match result {
            TestResult::Passed => self.passed += 1,
            TestResult::Failed(_) => self.failed += 1,
            TestResult::Ignored => self.ignored += 1,
        }
    }

    /// Format stats as string
    pub fn format(&self) -> String {
        format!(
            "Test result: {}. {} passed; {} failed; {} ignored",
            if self.is_success() { "ok" } else { "FAILED" },
            self.passed,
            self.failed,
            self.ignored
        )
    }
}

impl Default for TestStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Test runner
///
/// Executes discovered tests and tracks results
pub struct TestRunner {
    /// Statistics for this test run
    stats: TestStats,
    /// Verbose output
    verbose: bool,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        TestRunner {
            stats: TestStats::new(),
            verbose: false,
        }
    }

    /// Create a verbose test runner
    pub fn verbose() -> Self {
        TestRunner {
            stats: TestStats::new(),
            verbose: true,
        }
    }

    /// Run a single test
    ///
    /// This is a placeholder. The real implementation would:
    /// 1. Load the test binary
    /// 2. Call the test function
    /// 3. Catch panics/errors
    /// 4. Return result
    pub fn run_test(&mut self, _test: &TestMetadata) -> TestResult {
        // TODO: Implement actual test execution
        // Strategy:
        // 1. Dynamic loading of test function
        // 2. Set up panic handler
        // 3. Execute test
        // 4. Capture panic/error
        // 5. Return result

        // For now, just return passed as placeholder
        TestResult::Passed
    }

    /// Run all discovered tests
    ///
    /// Returns statistics about the test run
    pub fn run_all(&mut self, tests: &[TestMetadata]) -> TestStats {
        println!("Running tests...");
        println!();

        for test in tests {
            if test.ignored {
                if self.verbose {
                    println!("{} ... ignored (#[ignore])", test.name);
                }
                self.stats.add_result(&TestResult::Ignored);
                continue;
            }

            if self.verbose {
                print!("{} ... ", test.name);
            }

            let result = self.run_test(test);

            match result {
                TestResult::Passed => {
                    if self.verbose {
                        println!("ok");
                    }
                }
                TestResult::Failed(ref msg) => {
                    if self.verbose {
                        println!("FAILED");
                        println!("    {}", msg);
                    }
                }
                TestResult::Ignored => {
                    // Should not happen as we filter above
                }
            }

            self.stats.add_result(&result);
        }

        println!();
        println!("{}", self.stats.format());

        self.stats.clone()
    }

    /// Get current statistics
    pub fn stats(&self) -> &TestStats {
        &self.stats
    }

    /// Check if all tests passed so far
    pub fn is_success(&self) -> bool {
        self.stats.is_success()
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_stats_new() {
        let stats = TestStats::new();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.passed, 0);
        assert_eq!(stats.failed, 0);
        assert!(stats.is_success()); // No tests means success
    }

    #[test]
    fn test_test_stats_success() {
        let mut stats = TestStats::new();
        stats.add_result(&TestResult::Passed);
        stats.add_result(&TestResult::Passed);

        assert_eq!(stats.total, 2);
        assert_eq!(stats.passed, 2);
        assert!(stats.is_success());
    }

    #[test]
    fn test_test_stats_failure() {
        let mut stats = TestStats::new();
        stats.add_result(&TestResult::Passed);
        stats.add_result(&TestResult::Failed("error".to_string()));

        assert_eq!(stats.total, 2);
        assert_eq!(stats.passed, 1);
        assert_eq!(stats.failed, 1);
        assert!(!stats.is_success());
    }

    #[test]
    fn test_test_stats_pass_rate() {
        let mut stats = TestStats::new();
        stats.add_result(&TestResult::Passed);
        stats.add_result(&TestResult::Passed);
        stats.add_result(&TestResult::Failed("error".to_string()));

        assert_eq!(stats.pass_rate(), 66.66666666666666);
    }

    #[test]
    fn test_test_runner_new() {
        let runner = TestRunner::new();
        assert!(!runner.verbose);
        assert!(runner.is_success());
    }

    #[test]
    fn test_test_runner_run_placeholder() {
        let mut runner = TestRunner::new();
        let test = TestMetadata::new(
            "test".to_string(),
            "mod".to_string(),
            "file.zl".to_string(),
            1,
            false,
        );

        let result = runner.run_test(&test);
        assert_eq!(result, TestResult::Passed);
    }
}
