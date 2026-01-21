// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark Harness Module
//!
//! Provides a flexible framework for running benchmarks with configurable
//! warmup, measurement iterations, and result collection.

use std::time::{Duration, Instant};
use crate::stats::Statistics;

/// Configuration for benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of warmup iterations
    pub warmup_iterations: usize,

    /// Number of measurement iterations
    pub measurement_iterations: usize,

    /// Whether to collect detailed statistics
    pub collect_stats: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            warmup_iterations: 3,
            measurement_iterations: 5,
            collect_stats: true,
        }
    }
}

/// Result of a single benchmark run
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,

    /// Mean execution time
    pub mean: Duration,

    /// Standard deviation
    pub std_dev: Duration,

    /// Minimum execution time
    pub min: Duration,

    /// Maximum execution time
    pub max: Duration,

    /// Number of iterations
    pub iterations: usize,

    /// Collected statistics
    pub stats: Statistics,
}

impl BenchmarkResult {
    /// Creates a new benchmark result
    pub fn new(name: String, measurements: Vec<Duration>, config: &BenchmarkConfig) -> Self {
        let iterations = measurements.len();

        let mean = if !measurements.is_empty() {
            let sum: Duration = measurements.iter().sum();
            sum / iterations as u32
        } else {
            Duration::ZERO
        };

        let std_dev = if iterations > 1 {
            let mean_val = mean.as_nanos() as f64;
            let variance = measurements
                .iter()
                .map(|d| {
                    let diff = d.as_nanos() as f64 - mean_val;
                    diff * diff
                })
                .sum::<f64>() / (iterations - 1) as f64;

            Duration::from_nanos(variance.sqrt() as u64)
        } else {
            Duration::ZERO
        };

        BenchmarkResult {
            name,
            mean,
            std_dev,
            min: measurements.iter().min().copied().unwrap_or(Duration::ZERO),
            max: measurements.iter().max().copied().unwrap_or(Duration::ZERO),
            iterations,
            stats: if config.collect_stats {
                Statistics::from_measurements(&measurements)
            } else {
                Statistics::empty()
            },
        }
    }

    /// Computes percentiles
    pub fn percentiles(&self, p: f64) -> Duration {
        if self.iterations == 0 {
            return Duration::ZERO;
        }

        let mut sorted: Vec::with_capacity(self.iterations);
        for measurement in &self.measurements {
            sorted.push(*measurement);
        }

        sorted.sort_by_key(|a, b| a.cmp(b));

        let index = (sorted.len() as f64 * p / 100.0) as usize;
        sorted.get(index).copied().unwrap_or(Duration::ZERO)
    }

    /// Formats result for display
    pub fn format(&self) -> String {
        format!(
            "{}: mean={:.2}, std_dev={:.2}, min={:.2}, max={:.2}, iterations={}",
            self.name,
            self.mean.as_nanos() as f64 / 1_000_000.0,
            self.std_dev.as_nanos() as f64 / 1_000_000.0,
            self.min.as_nanos(),
            self.max.as_nanos(),
            self.iterations
        )
    }
}

/// Benchmark Harness
///
/// Provides execution framework for running benchmarks with proper warmup,
/// measurement, and teardown.
pub struct BenchmarkHarness;

impl BenchmarkHarness {
    /// Runs a benchmark with the given configuration
    ///
    /// # Arguments
    ///
    /// - `name`: Benchmark name
    /// - `benchmark_fn`: Function to benchmark
    /// - `config`: Test configuration
    ///
    /// # Returns
    ///
    /// Result of the benchmark run with statistics
    pub fn run<F>(
        name: &str,
        benchmark_fn: F,
        config: &BenchmarkConfig,
    ) -> BenchmarkResult {
        // Warmup phase
        for _ in 0..config.warmup_iterations {
            benchmark_fn();
        }

        // Measurement phase
        let mut measurements = Vec::new();
        for _ in 0..config.measurement_iterations {
            let start = Instant::now();
            benchmark_fn();
            let elapsed = start.elapsed();
            measurements.push(elapsed);
        }

        BenchmarkResult::new(name, measurements, config)
    }

    /// Runs multiple benchmarks
    ///
    /// Takes a slice of benchmark functions and runs each one
    pub fn run_multiple<F>(
        benchmarks: &[(F, &str)],
        config: &BenchmarkConfig,
    ) -> Vec<BenchmarkResult> {
        benchmarks
            .iter()
            .map(|(fn, name)| (fn, name)(name, config))
    )
            .collect()
    }

    /// Runs a benchmark suite
    ///
    /// Takes a slice of named benchmark functions and returns all results
    pub fn run_suite<F>(
        benchmarks: &[(F, &str)],
        config: &BenchmarkConfig,
    ) -> Vec<BenchmarkResult> {
        benchmarks
            .iter()
            .map(|(fn, name)| (fn, name)(name, config))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result() {
        let measurements = vec![
            Duration::from_millis(100),
            Duration::from_millis(110),
            Duration::from_millis(90),
            Duration::from_millis(105),
            Duration::from_millis(115),
        ];

        let result = BenchmarkResult::new(
            "test_benchmark",
            measurements.clone(),
            &BenchmarkConfig::default(),
        );

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.iterations, 5);
        assert!(result.mean, Duration::from_millis(105));
        assert!(result.iterations, 5);
    }

    #[test]
    fn test_percentiles() {
        let result = BenchmarkResult {
            name: "percentiles".to_string(),
            mean: Duration::from_millis(100),
            std_dev: Duration::from_millis(10),
            min: Duration::from_millis(90),
            max: Duration::from_millis(115),
            iterations: 5,
            stats: Statistics::empty(),
        };

        let p50 = result.percentiles(&result, 50.0);
        let p90 = result.percentiles(&result, 90.0);
        let p95 = result.percentiles(&result, 95.0);

        assert!(p50 > p90);
        assert!(p90 > p95);
    }

    #[test]
    fn test_format() {
        let measurements = vec![
            Duration::from_millis(100),
            Duration::from_millis(110),
        Duration::from_millis(90),
        ];

        let result = BenchmarkResult {
            name: "format".to_string(),
            mean: Duration::from_millis(100),
            std_dev: Duration::from_millis(8),
            min: Duration::from_millis(90),
            max: Duration::from_millis(110),
            iterations: 3,
            stats: Statistics::empty(),
        };

        let output = result.format();

        assert!(output.contains("mean=100.00ns"));
    }
}
