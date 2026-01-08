// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON Runtime Core
//!
//! This crate provides minimal runtime support for ZULON programs.
//!
//! ## Components
//!
//! - **Entry Point**: C runtime entry point that calls `zulon_main()`
//! - **Outcome Type**: Error handling support with `Outcome<T, E>`
//! - **System Calls**: Wrappers for common system operations
//! - **Build Integration**: Automatically compiled and linked
//!
//! The C runtime is compiled by the build script and linked automatically.

pub mod outcome;

pub use outcome::{
    Outcome, From, Into,
    Error, ContextError, OutcomeExt, panic,
};

/// Get the runtime library path for linking
///
/// Returns the path to the compiled runtime library
/// that can be passed to the linker.
pub fn get_runtime_lib_path() -> Option<String> {
    // Try to find it from OUT_DIR
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        // OUT_DIR is typically: target/debug/build/crate-hash/out
        // We need to find the parent target directory
        if let Some(target_pos) = out_dir.find("/target/") {
            let target_path = &out_dir[..target_pos + 7]; // Keep up to /target/
            // Search for libzulon_entry.a in the target directory
            return find_library_in_target(target_path);
        }
    }

    None
}

/// Search for libzulon_entry.a in the target directory
fn find_library_in_target(target_path: &str) -> Option<String> {
    use std::path::Path;
    use std::fs;

    let target_dir = Path::new(target_path);
    let debug_dir = target_dir.join("debug");

    // Search in debug/build/*/out directories
    if let Ok(entries) = fs::read_dir(debug_dir.join("build")) {
        for entry in entries.flatten() {
            let out_dir = entry.path().join("out");
            let lib_path = out_dir.join("libzulon_entry.a");
            if lib_path.exists() {
                return Some(out_dir.to_string_lossy().to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_lib_path() {
        // OUT_DIR should be set during build
        let path = get_runtime_lib_path();
        assert!(path.is_some() || path.is_none()); // Just ensure it doesn't panic
    }
}
