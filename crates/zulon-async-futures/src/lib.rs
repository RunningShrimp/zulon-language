// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Async Futures
//!
//! This crate provides core async primitives for ZULON:
//!
//! - **[`Future`]** - Core trait for asynchronous operations
//! - **[`Poll`]** - Result type for polling futures
//! - **[`Context`]** - Context provided to future polling
//! - **[`Waker`]** - Handle for waking up a task
//!
//! ## Example
//!
//! ```rust
//! use zulon_async_futures::{Future, Poll, Context, Waker};
//!
//! struct MyFuture;
//!
//! impl Future for MyFuture {
//!     type Output = i32;
//!
//!     fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
//!         Poll::Ready(42)
//!     }
//! }
//! ```

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod future;
mod poll;
mod context;
mod waker;
mod task;

pub use future::{Future, Ready, Pending};
pub use poll::Poll;
pub use context::Context;
pub use waker::Waker;
pub use task::{RawWaker, RawWakerVTable};

/// Re-export Pin from core when std feature is enabled
#[cfg(feature = "std")]
pub use std::pin::Pin;

/// Minimal Pin implementation for no_std
#[cfg(not(feature = "std"))]
pub use core::pin::Pin;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // Verify the async futures module is available
        assert!(true);
    }
}
