// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Performance validation for 90-95% C++ performance claim

use std::fmt;

/// Performance validation result
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidationResult {
    /// Performance meets 90-95% target
    pub meets_target: bool,
    /// Actual performance percentage
    pub performance_pct: f64,
    /// Number of benchmarks validated
    pub benchmark_count: usize,
    /// Number of benchmarks passing target
    pub passed_count: usize,
}

/// Performance target
pub const PERFORMANCE_TARGET_MIN_PCT: f64 = 90.0;
pub const PERFORMANCE_TARGET_MAX_PCT: f64 = 95.0;

/// Validate performance against C++ baseline
pub fn validate_performance(zulon_ops_per_sec: f64, cpp_ops_per_sec: f64) -> ValidationResult {
    if cpp_ops_per_sec <= 0.0 {
        return ValidationResult {
            meets_target: false,
            performance_pct: 0.0,
            benchmark_count: 1,
            passed_count: 0,
        };
    }

    let performance_pct = (zulon_ops_per_sec / cpp_ops_per_sec) * 100.0;
    let meets_target = performance_pct >= PERFORMANCE_TARGET_MIN_PCT
        && performance_pct <= PERFORMANCE_TARGET_MAX_PCT;

    ValidationResult {
        meets_target,
        performance_pct,
        benchmark_count: 1,
        passed_count: if meets_target { 1 } else { 0 },
    }
}

/// Validate multiple benchmarks against their baselines
pub fn validate_suite(
    results: &[(String, f64, f64)], // (benchmark_name, zulon_ops, cpp_ops)
) -> ValidationResult {
    let mut benchmark_count = 0;
    let mut passed_count = 0;
    let mut total_performance_pct = 0.0;

    for (name, zulon_ops, cpp_ops) in results.iter() {
        let validation = validate_performance(*zulon_ops, *cpp_ops);
        benchmark_count += 1;
        if validation.meets_target {
            passed_count += 1;
        }
        total_performance_pct += validation.performance_pct;
    }

    let avg_performance_pct = total_performance_pct / benchmark_count as f64;
    let meets_target = avg_performance_pct >= PERFORMANCE_TARGET_MIN_PCT
        && avg_performance_pct <= PERFORMANCE_TARGET_MAX_PCT;

    ValidationResult {
        meets_target,
        performance_pct: avg_performance_pct,
        benchmark_count,
        passed_count,
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValidationResult {{ meets_target: {}, performance_pct: {:.2}%, benchmark_count: {}, passed_count: {} }}",
            self.meets_target,
            self.performance_pct,
            self.benchmark_count,
            self.passed_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_performance_pass() {
        let result = validate_performance(95.0, 100.0);
        assert!(result.meets_target);
        assert_eq!(result.performance_pct, 95.0);
    }

    #[test]
    fn test_validate_performance_below_target() {
        let result = validate_performance(80.0, 100.0);
        assert!(!result.meets_target);
        assert_eq!(result.performance_pct, 80.0);
    }

    #[test]
    fn test_validate_performance_above_target() {
        let result = validate_performance(98.0, 100.0);
        assert!(result.meets_target);
        assert_eq!(result.performance_pct, 98.0);
    }
}
