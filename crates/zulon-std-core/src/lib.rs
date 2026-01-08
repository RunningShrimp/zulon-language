// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # zulon-std-core
//!
//! Core library for ZULON programming language.
//!
//! This library provides foundational types and traits used throughout
//! the ZULON ecosystem, including:
//!
//! - Core traits: `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`
//! - Optional values: `Optional<T>`
//! - Error handling: `Outcome<T, E>`

#![warn(unused_extern_crates)]

mod traits;
mod option;
mod result;
mod vec;
mod hashmap;
mod hashset;
mod vecdeque;
mod test;
mod test_runner;

// Re-export core traits
pub use traits::{
    Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Ordering,
};

// Re-export core types
pub use option::Optional;
pub use result::Outcome;
pub use vec::Vec;
pub use hashmap::{HashMap, Hash};
pub use hashset::HashSet;
pub use vecdeque::VecDeque;

// Re-export testing functions
pub use test::{assert, assert_eq, assert_ne, panic};
// Re-export test runner
pub use test_runner::{run_tests, run_test_verbose, Test, TestFunc, TestResult, TestStats};
