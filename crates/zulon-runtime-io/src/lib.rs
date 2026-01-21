// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Runtime IO (Minimal)
//!
//! Simplified IO crate using standard library types only.
//! This eliminates type system conflicts between crates.
//!
//! ## Purpose
//!
//! - Provides minimal IO functionality for ZULON
//! - Uses std::fs, std::io, std::path directly
//! - Removes dependency on zulon-std-core (circular dependency)
//!
//! ## Exports
//!
//! - `print` - Simple output
//! - `println` - Simple output with newline
//!
//! ## Example
//!
//! ```rust
//! use zulon_runtime_io::{print, println};
//!
//! print("Hello, ");
//! println("world!");
//! ```

mod stdout;

pub use std::io::{self, Write};
