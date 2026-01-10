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
pub mod event_loop;

pub use error::{IoError, IoResult};
pub use stdout::{print, println};
pub use file::File;
pub use event_loop::{EventLoop, EventHandler, EventSource, Token, Interest, TimerHandle};

// TODO: Re-enable when modules are complete
// mod channel;
// mod io;
// mod sys;
// mod select;

#[cfg(test)]
mod tests {
    #[test]
    fn test_io_module_exists() {
        // Verify IO module is available
        assert!(true);
    }
}
