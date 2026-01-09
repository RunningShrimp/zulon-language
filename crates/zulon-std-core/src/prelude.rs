// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Standard Prelude
//!
//! This module contains documentation for the ZULON prelude that is
//! automatically injected into every ZULON program by the compiler.

// NOTE: This is a Rust module for documentation purposes only.
// The actual prelude content is defined in compiler/src/compiler.rs
// as a string literal that gets prepended to all user code.

/// The ZULON prelude contains:
/// - extern function declarations (printf, scanf, etc.)
/// - Core type imports when module system is implemented
///
/// Users don't need to manually declare `extern fn printf` - it's
/// automatically included by the compiler.
pub mod prelude_doc {
    //! This module documents what the compiler automatically injects.
    //! The actual injection happens in compiler/src/compiler.rs.
}

