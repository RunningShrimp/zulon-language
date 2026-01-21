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
//! - File I/O: `File`, `Path`, `Metadata`
//! - I/O traits: `Read`, `Write`, `Seek`

#![warn(unused_extern_crates)]

mod fs;
mod hashmap;
mod hashset;
mod io;
mod option;
mod prelude;
mod result;
mod string;
mod test;
mod test_runner;
mod traits;
mod vec;
mod vecdeque;

// Re-export core traits
pub use traits::{Clone, Copy, Eq, Hash, Ord, Ordering, PartialEq, PartialOrd};

// Re-export core types
#[allow(unused_imports)]
pub use fs::error::IoResult;
pub use fs::{error::IoError, File, FileType, Metadata, OpenOptions, Path, PathBuf};
pub use hashmap::HashMap;
pub use hashset::HashSet;
pub use io::buffered::{BufReader, BufWriter};
pub use io::traits::{Read, Seek, SeekFrom, Write};
pub use option::Optional;
pub use result::Outcome;
pub use string::String;
pub use vec::{IntoIter as VecIntoIter, Iter as VecIter, IterMut as VecIterMut, Vec};
pub use vecdeque::VecDeque;

// Re-export testing functions
pub use test::{assert as zassert, assert_eq, assert_ne, panic};

// Re-export test runner
pub use test_runner::{run_test_verbose, run_tests, Test, TestFunc, TestResult, TestStats};
