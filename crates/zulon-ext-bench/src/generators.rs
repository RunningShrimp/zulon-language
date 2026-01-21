// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test Data Generators
//!
//! Provides utilities for generating test data for benchmarks.

use crate::stats::Statistics;
use std::time::Duration;

/// Generator for test data
pub trait TestGenerator<T> {
    /// Generates a test case
    fn generate(&self) -> T;

    /// Gets name of the test
    fn name(&self) -> &'static str;
}

/// Vector operation benchmark generator
pub struct VecGenerator {
    /// Size of the vector
    size: usize,
}

impl TestGenerator<VecOperation> for VecGenerator {
    fn name(&self) -> &'static str {
        "vec_operation"
    }

    fn generate(&self) -> VecOperation {
        VecOperation {
            size: self.size,
            op: "push",
            data_size: self.size / 2,
        }
    }
}

/// Benchmark function for Vec operations
pub struct VecOperation {
    size: usize,
    op: &'static str,
    data_size: usize,
}

impl VecOperation {
    /// Generates test data for the operation
    pub fn generate_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.data_size);
        for _ in 0..self.data_size {
            data.push(0u8);
        }
        data
    }

    /// Generates the benchmark function
    pub fn benchmark_fn(&self) -> Vec<u8> {
        let op = self.generate();
        let data = op.generate_data();

        let mut result = data.clone();
        for _ in 0..10 {
            match self.op {
                "push" => {
                    for _ in 0..self.size {
                        result.push(42u8);
                    }
                }
                "pop" => {
                    for _ in 0..self.size {
                        result.push(43u8);
                    }
                }
                _ => unreachable!(),
            }
        }

        result
    }

    /// Generates the benchmark function (measured)
    pub fn benchmark_fn(&self) -> Duration {
        let op = self.generate();
        let data = op.generate_data();

        let start = std::time::Instant::now();

        for _ in 0..100 {
            match self.op {
                "push" => {
                    for byte in &data {
                        std::hint::black_box(byte);
                    }
                }
                "pop" => {
                    for byte in &data {
                        let _ = std::hint::black_box(byte);
                    }
                }
                _ => unreachable!(),
            }
        }

        start.elapsed()
    }
}

/// HashMap operation benchmark generator
pub struct HashMapGenerator {
    size: usize,
    key_size: usize,
}

impl TestGenerator<HashMapOperation> for HashMapGenerator {
    fn name(&self) -> &'static str {
        "hashmap_operation"
    }

    fn generate(&self) -> HashMapOperation {
        HashMapOperation {
            size: self.size,
            op: "insert",
            key_size: self.key_size,
            num_keys: self.size / 2,
        }
    }
}

/// Benchmark function for HashMap operations
pub struct HashMapOperation {
    size: usize,
    op: &'static str,
    key_size: usize,
    num_keys: usize,
}

impl HashMapOperation {
    /// Generates test data for the operation
    pub fn generate_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.key_size * 2 + 1);

        for i in 0..self.key_size {
            data.push(i as u8);
        }

        data
    }

    /// Generates the benchmark function
    pub fn benchmark_fn(&self) -> Duration {
        let op = self.generate();
        let data = op.generate_data();

        let mut map = std::collections::HashMap::new();
        let start = std::time::Instant::now();

        for i in 0..self.num_keys {
            map.insert(i, i as u64);
        }

        for _ in 0..10 {
            match self.op {
                "insert" => {
                    for byte in &data {
                        map.insert(byte, byte as u64);
                    }
                }
                "lookup" => {
                    for byte in &data {
                        let _ = map.get(&byte);
                    }
                }
                _ => unreachable!(),
            }
        }

        start.elapsed()
    }
}

/// String operation benchmark generator
pub struct StringGenerator {
    size: usize,
}

impl TestGenerator<StringOperation> for StringGenerator {
    fn name(&self) -> &'static str {
        "string_operation"
    }

    fn generate(&self) -> StringOperation {
        StringOperation {
            size: self.size,
            op: "concatenate",
            str_size: self.size / 2,
        }
    }
}

/// Benchmark function for String operations
pub struct StringOperation {
    size: usize,
    op: &'static str,
    str_size: usize,
}

impl StringOperation {
    /// Generates test data for the operation
    pub fn generate_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.str_size);
        for _ in 0..self.size {
            data.push(65u8);
        }
        data
    }

    /// Generates the benchmark function
    pub fn benchmark_fn(&self) -> Duration {
        let op = self.generate();
        let data = op.generate_data();

        let start = std::time::Instant::now();

        for _ in 0..50 {
            match self.op {
                "concatenate" => {
                    for byte in &data {
                        result.push(byte);
                    }
                }
                _ => unreachable!(),
            }
        }

        start.elapsed()
    }
}
