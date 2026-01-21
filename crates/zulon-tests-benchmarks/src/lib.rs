// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # zulon-tests-benchmarks
//!
//! Provides performance benchmarks for ZULON standard library operations.
//!
//! ## Modules
//!
//! - `benchmark`: Core benchmark framework with timing and statistics
//! - `std_benches`: Benchmarks for standard library types (Vec, HashMap, String)
//! - `lang_benches`: Benchmarks for language features (type inference, pattern matching)
//! - `automated_reporting`: Automated benchmark reporting and regression detection
//! - `validation`: Performance validation against 90-95% target
//! - `learning_curve`: Learning curve measurement framework
//!
//! ## Features
//!
//! - Benchmark harness with warmup and measurement phases
//! - Statistical analysis (avg, median, min, max)
//! - Operations per second calculation
//! - Comparison support with baseline programs
//! - Language feature performance measurement
//! - Automated CSV export and regression detection
//! - Performance validation
//! - Developer productivity tracking

#![warn(missing_docs)]

pub mod benchmark;
pub mod std_benches;
pub mod lang_benches;
pub mod automated_reporting;
pub mod validation;
pub mod learning_curve;
