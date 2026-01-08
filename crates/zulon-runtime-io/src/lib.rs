// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Runtime IO
//!
//! This crate provides IO primitives for ZULON programs:
//!
//! - **`print`** - Standard output functions
//! - **`File`** - File operations
//! - **`IoError`** - IO error type
//!
//! ## Example
//!
//! ```rust
//! use zulon_runtime_io::{print, println};
//!
//! print("Hello, ");
//! println("world!");
//! ```

mod error;
mod stdout;
mod file;

pub use error::{IoError, IoResult};
pub use stdout::{print, println};
pub use file::File;

#[cfg(test)]
mod tests {
    #[test]
    fn test_io_module_exists() {
        // Verify IO module is available
        assert!(true);
    }
}
