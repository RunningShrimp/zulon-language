// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Event Loop Abstraction
//!
//! This module provides platform-agnostic event loop traits and interfaces
//! for non-blocking I/O operations.
//!
//! ## Architecture
//!
//! The event loop system uses the Reactor pattern:
//! - **EventLoop**: Core event loop trait (implemented by epoll/IOCP/kqueue)
//! - **EventHandler**: Callback trait for handling events
//! - **EventSource**: Trait for sources of I/O events
//! - **Interest**: Describes which events to monitor
//!
//! ## Example
//!
//! ```rust
//! use zulon_runtime_io::event_loop::{EventLoop, EventHandler, Interest};
//!
//! struct MyHandler;
//!
//! impl EventHandler for MyHandler {
//!     fn readable(&mut self, token: Token) {
//!         println!("Readable event on {:?}", token);
//!     }
//!
//!     fn writable(&mut self, token: Token) {
//!         println!("Writable event on {:?}", token);
//!     }
//!
//!     fn error(&mut self, token: Token, err: IoError) {
//!         eprintln!("Error on {:?}: {}", token, err);
//!     }
//! }
//!
//! fn main() -> IoResult<()> {
//!     let mut event_loop = DefaultEventLoop::new()?;
//!     let token = event_loop.register(&my_source)?;
//!     event_loop.run()?;
//!     Ok(())
//! }
//! ```

mod token;
mod interest;

pub use token::Token;
pub use interest::Interest;

use crate::error::{IoError, IoResult};
use std::time::Duration;
use std::os::unix::io::RawFd;

/// Core event loop trait
///
/// This trait provides a platform-agnostic interface to the system's
/// event notification mechanism (epoll on Linux, IOCP on Windows,
/// kqueue on macOS/BSD).
///
/// # Thread Safety
///
/// Event loops are typically not thread-safe and should be run
/// from a single thread. Use channels for cross-thread communication.
pub trait EventLoop: Sized {
    /// Create a new event loop
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying system call fails
    /// (e.g., epoll_create1 on Linux).
    fn new() -> IoResult<Self>;

    /// Run the event loop
    ///
    /// This method blocks until `stop()` is called or an error occurs.
    /// It will continuously wait for events and dispatch them to
    /// the registered event handlers.
    ///
    /// # Errors
    ///
    /// Returns an error if a system call fails during event processing.
    fn run(&mut self) -> IoResult<()>;

    /// Register an I/O event source
    ///
    /// # Arguments
    ///
    /// * `source` - The event source to monitor (e.g., socket, file)
    ///
    /// # Returns
    ///
    /// A token that can be used to identify this event source.
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be registered
    /// (e.g., file descriptor already registered).
    fn register(&mut self, source: &dyn EventSource) -> IoResult<Token>;

    /// Reregister an event source with new interests
    ///
    /// # Arguments
    ///
    /// * `token` - The token returned by `register()`
    /// * `interest` - New interest set
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid or reregistration fails.
    fn reregister(&mut self, token: Token, interest: Interest) -> IoResult<()>;

    /// Deregister an event source
    ///
    /// # Arguments
    ///
    /// * `token` - The token to deregister
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid.
    fn deregister(&mut self, token: Token) -> IoResult<()>;

    /// Submit a timer
    ///
    /// # Arguments
    ///
    /// * `duration` - Timer duration
    ///
    /// # Returns
    ///
    /// A handle that can be used to cancel the timer.
    ///
    /// # Errors
    ///
    /// Returns an error if the timer cannot be created.
    fn submit_timer(&mut self, duration: Duration) -> IoResult<TimerHandle>;

    /// Stop the event loop
    ///
    /// This will cause `run()` to return after processing any
    /// currently pending events.
    fn stop(&mut self);
}

/// Event handler callback trait
///
/// Implement this trait to handle I/O events from an event loop.
/// Each registered event source should have a corresponding handler.
pub trait EventHandler: Send + Sync {
    /// Handle readable event
    ///
    /// Called when data is available to read from the event source.
    ///
    /// # Arguments
    ///
    /// * `token` - Identifies which event source is readable
    fn readable(&mut self, token: Token);

    /// Handle writable event
    ///
    /// Called when the event source is ready to accept data.
    ///
    /// # Arguments
    ///
    /// * `token` - Identifies which event source is writable
    fn writable(&mut self, token: Token);

    /// Handle error event
    ///
    /// Called when an error occurs on the event source.
    ///
    /// # Arguments
    ///
    /// * `token` - Identifies which event source has an error
    /// * `err` - The error that occurred
    fn error(&mut self, token: Token, err: IoError);
}

/// Source of I/O events
///
/// Implement this trait for types that can generate I/O events
/// (e.g., sockets, files, pipes, timers).
pub trait EventSource: Send + Sync {
    /// Raw file descriptor or handle
    ///
    /// On Unix systems, returns a file descriptor.
    /// On Windows, would return a HANDLE (TODO: add Windows support).
    fn raw_fd(&self) -> RawFd;

    /// Interest in events
    ///
    /// Specifies which events this source wants to be notified about.
    fn interest(&self) -> Interest;
}

/// Timer handle
///
/// Used to cancel or query a submitted timer.
#[derive(Debug, Clone, Copy)]
pub struct TimerHandle(pub usize);

impl TimerHandle {
    /// Create a new timer handle
    pub fn new(id: usize) -> Self {
        TimerHandle(id)
    }

    /// Get the timer ID
    pub fn id(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new(42);
        assert_eq!(token.index(), 42);
    }

    #[test]
    fn test_interest_creation() {
        let interest = Interest::READABLE | Interest::WRITABLE;
        assert!(interest.is_readable());
        assert!(interest.is_writable());
    }

    #[test]
    fn test_interest_combinations() {
        let interest = Interest::READABLE;
        assert!(interest.is_readable());
        assert!(!interest.is_writable());

        let combined = interest | Interest::WRITABLE;
        assert!(combined.is_readable());
        assert!(combined.is_writable());
    }
}
