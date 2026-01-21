// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Language feature benchmarks
//!
//! Provides performance benchmarks for ZULON language features.
//!
//! ## Features
//!
//! - Type inference benchmarks
//! - Pattern matching benchmarks
//! - Generic specialization benchmarks

use crate::benchmark::{BenchmarkResult, BenchmarkRunner};
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_type_inference_simple() -> Duration {
        let iterations = 10_000;
        let start = Instant::now();

        for i in 0..iterations {
            let _ = simple_inference(i);
        }

        start.elapsed()
    }

    #[test]
    fn benchmark_type_inference_complex() -> Duration {
        let iterations = 10_000;
        let start = Instant::now();

        for i in 0..iterations {
            let _ = complex_inference(i);
        }

        start.elapsed()
    }

    #[test]
    fn benchmark_pattern_matching_simple() -> Duration {
        let iterations = 10_000;
        let start = Instant::now();

        for i in 0..iterations {
            let _ = simple_pattern_match(i);
        }

        start.elapsed()
    }

    #[test]
    fn benchmark_pattern_matching_complex() -> Duration {
        let iterations = 10_000;
        let start = Instant::now();

        for i in 0..iterations {
            let _ = complex_pattern_match(i);
        }

        start.elapsed()
    }
}

/// Simple type inference benchmark
fn simple_inference(value: i32) {
    if value > 0 {
        InferredType::Positive
    } else if value < 0 {
        InferredType::Negative
    } else {
        InferredType::Zero
    }
}

/// Complex type inference benchmark
fn complex_inference(value: i32) -> ComplexInferredType {
    if value % 2 == 0 {
        ComplexInferredType::Even
    } else if value % 3 == 0 {
        ComplexInferredType::DivisibleBy3
    } else if value > 100 {
        ComplexInferredType::Large
    } else {
        ComplexInferredType::Unknown
    }
}

/// Simple pattern matching benchmark
fn simple_pattern_match(value: i32) -> PatternResult {
    match value {
        0 => PatternResult::Zero,
        1..=10 => PatternResult::Small,
        _ => PatternResult::Large,
    }
}

/// Complex pattern matching benchmark
fn complex_pattern_match(value: i32) -> ComplexPatternResult {
    match value {
        n if n % 2 == 0 => ComplexPatternResult::Even,
        n if n % 3 == 0 => ComplexPatternResult::DivisibleBy3,
        n if n.is_power_of_two() => ComplexPatternResult::PowerOfTwo,
        n if is_prime(n) => ComplexPatternResult::Prime,
        _ => ComplexPatternResult::Other,
    }
}

/// Inferred type for simple inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferredType {
    Positive,
    Negative,
    Zero,
}

/// Inferred type for complex inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexInferredType {
    Even,
    DivisibleBy3,
    Large,
    Unknown,
}

/// Pattern matching result for simple patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternResult {
    Zero,
    Small,
    Large,
}

/// Pattern matching result for complex patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexPatternResult {
    Even,
    DivisibleBy3,
    PowerOfTwo,
    Prime,
    Other,
}

/// Helper trait for power of two check
trait PowerOfTwo {
    fn is_power_of_two(&self) -> bool;
}

impl PowerOfTwo for i32 {
    fn is_power_of_two(&self) -> bool {
        if self <= 0 {
            return false;
        }
        (self & (self - 1)) == 0
    }
}

/// Check if number is prime
fn is_prime(n: i32) -> bool {
    if n <= 1 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let sqrt_n = (n as f64).sqrt() as i32;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}
