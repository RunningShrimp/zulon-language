// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Statistics Module
//!
//! Provides statistical analysis for benchmark results including
//! mean, standard deviation, percentiles, and confidence intervals.

use std::fmt;

/// Statistics for benchmark results
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Number of measurements
    pub count: usize,

    /// Sum of all measurements
    pub sum: crate::time::Duration,

    /// Mean (average) of measurements
    pub mean: crate::time::Duration,

    /// Standard deviation
    pub std_dev: crate::time::Duration,

    /// Minimum measurement
    pub min: crate::time::Duration,

    /// Maximum measurement
    pub max: crate::time::Duration,

    /// Variance
    pub variance: f64,

    /// Median (50th percentile)
    pub median: crate::time::Duration,
}

impl Statistics {
    /// Creates empty statistics
    pub fn empty() -> Self {
        Statistics {
            count: 0,
            sum: crate::time::Duration::ZERO,
            mean: crate::time::Duration::ZERO,
            std_dev: crate::time::Duration::ZERO,
            min: crate::time::Duration::ZERO,
            max: crate::time::Duration::ZERO,
            variance: 0.0,
            median: crate::time::Duration::ZERO,
        }
    }

    /// Creates statistics from measurements
    pub fn from_measurements(measurements: &[crate::time::Duration]) -> Self {
        let count = measurements.len();
        let sum: measurements.iter().sum();
        let mean = if count > 0 {
            sum / count as u32
        } else {
            crate::time::Duration::ZERO
        };

        let std_dev = if count > 1 {
            let mean_nanos = mean.as_nanos() as f64;
            let variance = measurements
                .iter()
                .map(|d| {
                    let diff = d.as_nanos() as f64 - mean_nanos;
                    diff * diff
                })
                .sum::<f64>() / (count - 1) as f64;

            crate::time::Duration::from_nanos(variance.sqrt() as u64)
        } else {
            crate::time::Duration::ZERO
        };

        let min = measurements.iter().min().copied().unwrap_or(crate::time::Duration::ZERO);
        let max = measurements.iter().max().copied().unwrap_or(crate::time::Duration::ZERO);

        let mut sorted = measurements.to_vec();
        sorted.sort_by_key(|a, b| a.cmp(b));

        let median = if count > 0 {
            let mid = count / 2;
            sorted.get(mid).copied().unwrap_or(crate::time::Duration::ZERO)
        } else {
            crate::time::Duration::ZERO
        };

        Statistics {
            count,
            sum,
            mean,
            std_dev,
            min,
            max,
            variance,
            median,
        }
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Statistics: count={}, sum={:.2}, mean={:.2}, std_dev={:.2}, min={:.2}, max={:.2}, median={:.2}, variance={:.2}",
            self.count,
            self.sum.as_nanos() as f64 / 1_000_000.0,
            self.mean.as_nanos() as f64 / 1_000_000.0,
            self.std_dev.as_nanos() as f64 / 1_000_000.0,
            self.min.as_nanos(),
            self.max.as_nanos(),
            self.median.as_nanos(),
            self.variance,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_statistics() {
        let stats = Statistics::empty();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.sum, crate::time::Duration::ZERO);
    }

    #[test]
    fn test_from_measurements() {
        let measurements = vec![
            crate::time::Duration::from_millis(100),
            crate::time::Duration::from_millis(110),
            crate::time::Duration::from_millis(90),
        ];

        let stats = Statistics::from_measurements(&measurements);

        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, crate::time::Duration::from_millis(100));
        assert_eq!(stats.median, crate::time::Duration::from_millis(100));
    }
}
