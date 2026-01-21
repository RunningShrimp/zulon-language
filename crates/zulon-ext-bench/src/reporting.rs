// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark Reporting Module
//!
//! Provides result collection and report generation in JSON and HTML formats.

use std::collections::HashMap;

/// Benchmark report containing multiple benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    /// Benchmark results
    pub results: Vec<crate::harness::BenchmarkResult>,

    /// Total execution time across all benchmarks
    pub total_time: crate::time::Duration,

    /// Metadata about the benchmark run
    pub metadata: ReportMetadata,
}

/// Metadata about a benchmark report
#[derive(Debug, Clone)]
pub struct ReportMetadata {
    /// Hostname
    pub hostname: String,

    /// CPU information
    pub cpu_info: String,

    /// Number of CPU cores
    pub cpu_count: usize,

    /// Memory information
    pub memory_info: String,

    /// Build information
    pub build_info: String,

    /// ZULON version
    pub zulon_version: String,
}

impl BenchmarkReport {
    /// Creates a new benchmark report
    pub fn new() -> Self {
        BenchmarkReport {
            results: Vec::new(),
            total_time: crate::time::Duration::ZERO,
            metadata: ReportMetadata::default(),
        }
    }

    /// Adds a benchmark result to the report
    pub fn add_result(&mut self, result: crate::harness::BenchmarkResult) {
        self.results.push(result);
        self.total_time = self.total_time + result.mean;
    }

    /// Generates JSON report
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self)
    }

    /// Generates HTML report
    pub fn to_html(&self) -> String {
        format!(
            "<html>
            <head>
                <title>ZULON Benchmark Report</title>
                <style>
                    body {{ font-family: Arial, sans-serif; }}
                    table {{ border-collapse: collapse; width: 100%; }}
                        th {{ background: #4CAF50; color: white; }}
                        td {{ padding: 8px; }}
                            Name
                        </td>
                        <td>Iterations</td>
                        <td>Mean (ns)</td>
                        <td>Std Dev (ns)</td>
                        <td>Min (ns)</td>
                        <td>Max (ns)</td>
                        </tr>
                    {{ for result in &self.results {{
                        <tr>
                            <td>{{}}</td>
                            <td>{{ result.iterations }}}</td>
                            <td>{{ result.mean.as_nanos() }}</td>
                            <td>{{ result.std_dev.as_nanos() }}</td>
                            <td>{{ result.min.as_nanos() }}</td>
                            <td>{{ result.max.as_nanos() }}</td>
                        </tr>
                    }}}
                </table>
            </body>
            </html>"
        )
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_report() {
        let mut report = BenchmarkReport::new();

        let result1 = crate::harness::BenchmarkResult {
            name: "test1".to_string(),
            mean: crate::time::Duration::from_millis(100),
            std_dev: crate::time::Duration::from_millis(10),
            min: crate::time::time::Duration::from_millis(90),
            max: crate::time::time::Duration::from_millis(110),
            iterations: 5,
            stats: crate::stats::Statistics::empty(),
        };

        let result2 = crate::harness::BenchmarkResult {
            name: "test2".to_string(),
            mean: crate::time::Duration::from_millis(105),
            std_dev: crate::time::Duration::from_millis(5),
            min: crate::time::time::Duration::Duration::from_millis(100),
            max: crate::time::time::Duration::from_millis(115),
            iterations: 5,
            stats: crate::stats::Statistics::empty(),
        };

        report.add_result(result1);
        report.add_result(result2);

        assert_eq!(report.results.len(), 2);

        let json = report.to_json();
        assert!(json.contains("test1"));
        assert!(json.contains("test2"));
    }
}
