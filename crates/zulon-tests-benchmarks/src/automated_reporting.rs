// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Automated benchmark reporting and regression detection
//!
//! Provides:
//! - CSV result storage and export
//! - Performance trend analysis
//! - Regression detection algorithms
//! - Historical baseline management

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path as StdPath;
use std::sync::Mutex;

/// Stored benchmark result with metadata
#[derive(Debug, Clone)]
pub struct StoredBenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Timestamp of the run
    pub timestamp: String,
    /// Ops per second
    pub ops_per_sec: f64,
    /// Platform information
    pub platform: String,
}

/// Regression analysis result
#[derive(Debug, Clone)]
pub struct RegressionAnalysis {
    /// Has regression detected
    pub has_regression: bool,
    /// Performance change percentage
    pub change_pct: f64,
    /// Number of data points analyzed
    pub sample_size: usize,
    /// Statistical significance
    pub significance: f64,
}

/// Performance trend
#[derive(Debug, Clone)]
pub enum Trend {
    /// Performance improving
    Improving,
    /// Performance stable
    Stable,
    /// Performance degrading
    Degrading,
}

/// Benchmark history storage
pub struct BenchmarkHistory {
    storage_path: String,
    data: Mutex<Vec<StoredBenchmarkResult>>,
}

impl BenchmarkHistory {
    pub fn new(storage_path: &str) -> Self {
        BenchmarkHistory {
            storage_path: storage_path.to_string(),
            data: Mutex::new(Vec::new()),
        }
    }

    /// Add a benchmark result to history
    pub fn add_result(&self, result: &StoredBenchmarkResult) {
        let mut data = self.data.lock().unwrap();
        data.push(result.clone());
        self.save_to_disk(&data);
    }

    /// Load all historical results for a benchmark
    pub fn load_results(&self, benchmark_name: &str) -> Vec<StoredBenchmarkResult> {
        let data = self.data.lock().unwrap();
        data.iter()
            .filter(|r| r.name == benchmark_name)
            .cloned()
            .collect()
    }

    /// Analyze regression for a specific benchmark
    pub fn analyze_regression(
        &self,
        benchmark_name: &str,
        threshold_pct: f64,
    ) -> RegressionAnalysis {
        let results = self.load_results(benchmark_name);

        if results.len() < 2 {
            return RegressionAnalysis {
                has_regression: false,
                change_pct: 0.0,
                sample_size: results.len(),
                significance: 1.0,
            };
        }

        // Split into current (last result) and historical (previous results)
        let current = results.last().unwrap();
        let historical: Vec<_> = results.iter().take(results.len() - 1).cloned().collect();

        // Calculate median of historical results
        let mut ops_per_sec: Vec<_> = historical.iter().map(|r| r.ops_per_sec).collect();
        ops_per_sec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let historical_median = ops_per_sec[ops_per_sec.len() / 2];

        // Calculate change percentage
        let change_pct = if historical_median > 0.0 {
            ((current.ops_per_sec - historical_median) / historical_median) * 100.0
        } else {
            0.0
        };

        // Detect regression
        let has_regression = change_pct < -threshold_pct;

        // Significance calculation
        let significance = (results.len() as f64).ln() + 1.0;

        RegressionAnalysis {
            has_regression,
            change_pct,
            sample_size: results.len(),
            significance,
        }
    }

    /// Detect performance trend
    pub fn detect_trend(&self, benchmark_name: &str, window_size: usize) -> Trend {
        let results = self.load_results(benchmark_name);

        if results.len() < window_size {
            return Trend::Stable;
        }

        // Calculate average ops/sec for recent window
        let recent: Vec<_> = results
            .iter()
            .rev()
            .take(window_size)
            .map(|r| r.ops_per_sec)
            .collect();

        let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;

        // Calculate average ops/sec for previous window
        let historical: Vec<_> = results
            .iter()
            .take(results.len() - window_size)
            .map(|r| r.ops_per_sec)
            .collect();

        let historical_avg: f64 = if historical.is_empty() {
            recent_avg
        } else {
            historical.iter().sum::<f64>() / historical.len() as f64
        };

        // Determine trend
        let improvement = (recent_avg - historical_avg) / historical_avg.abs();

        if improvement > 0.05 {
            Trend::Improving
        } else if improvement < -0.05 {
            Trend::Degrading
        } else {
            Trend::Stable
        }
    }

    /// Export all results to CSV
    pub fn export_to_csv(&self, output_path: &str) -> std::io::Result<()> {
        let data = self.data.lock().unwrap();

        let mut file = File::create(output_path)?;

        // Write CSV header
        writeln!(file, "name,timestamp,ops_per_sec,platform")?;

        // Write all results
        for result in data.iter() {
            writeln!(
                file,
                "{},{},{},{}",
                result.name, result.timestamp, result.ops_per_sec, result.platform
            )?;
        }

        Ok(())
    }

    /// Save results to disk
    fn save_to_disk(&self, data: &Vec<StoredBenchmarkResult>) {
        if let Some(parent) = StdPath::new(&self.storage_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let _ = File::create(&self.storage_path);
        let file = File::create(&self.storage_path);
        let _ = file.write_all(serde_json::to_string(data).as_bytes());
    }
}

/// Generate automated benchmark report
pub fn generate_report(
    benchmark_name: &str,
    ops_per_sec: f64,
    output_path: &str,
) -> std::io::Result<()> {
    let history = BenchmarkHistory::new("benchmark_history.json");

    let timestamp = chrono::Utc::now().to_rfc3339();
    let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);

    let result = StoredBenchmarkResult {
        name: benchmark_name.to_string(),
        timestamp: timestamp.clone(),
        ops_per_sec,
        platform: platform.clone(),
    };

    history.add_result(&result);

    // Export to CSV
    history.export_to_csv(output_path)?;

    // Generate summary report
    let summary = format!(
        "=== Benchmark Report ===\n\
         Benchmark: {}\n\
         Timestamp: {}\n\
         Platform: {}\n\
         Ops/sec: {:.2}\n\
         CSV exported to: {}",
        benchmark_name, timestamp, platform, ops_per_sec, output_path
    );

    println!("{}", summary);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_detection_no_change() {
        let history = BenchmarkHistory::new("/tmp/test_history.json");
        let analysis = history.analyze_regression("test_benchmark", 5.0);
        assert!(!analysis.has_regression);
    }
}
