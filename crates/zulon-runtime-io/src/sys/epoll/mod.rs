// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Linux epoll event loop implementation
//!
//! This module provides an epoll-based event loop for Linux systems.
//! It uses edge-triggered mode (EPOLLET) for better performance and
//! supports EPOLLONESHOT for thread-safe event handling.
//!
//! ## Architecture
//!
//! - **Edge-triggered mode**: Events are only delivered when state changes
//! - **EPOLLONESHOT**: Each event is delivered to only one thread
//! - **Efficient**: O(1) event notification regardless of registered fds
//!
//! ## Example
//!
//! ```rust,no_run
//! use zulon_runtime_io::event_loop::{EventLoop, EventHandler, Interest};
//! use zulon_runtime_io::sys::EpollEventLoop;
//!
//! struct MyHandler;
//!
//! impl EventHandler for MyHandler {
//!     fn readable(&mut self, token: Token) {
//!         println!("Readable: {:?}", token);
//!     }
//!
//!     fn writable(&mut self, token: Token) {
//!         println!("Writable: {:?}", token);
//!     }
//!
//!     fn error(&mut self, token: Token, err: IoError) {
//!         eprintln!("Error: {:?} - {}", token, err);
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut event_loop = EpollEventLoop::new()?;
//!     // Register event sources...
//!     event_loop.run()?;
//!     Ok(())
//! }
//! ```

mod sys;

use crate::event_loop::{EventLoop, EventHandler, EventSource, Token, Interest, TimerHandle};
use crate::error::{IoError, IoResult};

use std::os::unix::io::RawFd;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Default maximum number of events to process per epoll_wait call
pub const DEFAULT_MAX_EVENTS: usize = 1024;

/// Epoll-based event loop for Linux
///
/// This implementation uses edge-triggered mode (EPOLLET) for optimal
/// performance and supports EPOLLONESHOT for thread-safe operation.
pub struct EpollEventLoop {
    /// Epoll file descriptor
    epoll_fd: RawFd,

    /// Event handlers indexed by token
    handlers: Vec<Option<Box<dyn EventHandler>>>,

    /// Map from file descriptor to token
    fd_to_token: HashMap<RawFd, Token>,

    /// Next token index
    next_token: AtomicUsize,

    /// Running flag
    running: AtomicBool,

    /// Maximum events per wait
    max_events: usize,
}

impl EpollEventLoop {
    /// Create a new epoll event loop
    ///
    /// # Errors
    ///
    /// Returns an error if epoll_create1 fails.
    pub fn new() -> IoResult<Self> {
        Self::with_capacity(DEFAULT_MAX_EVENTS)
    }

    /// Create a new epoll event loop with specified capacity
    ///
    /// # Arguments
    ///
    /// * `max_events` - Maximum number of events to process per wait
    ///
    /// # Errors
    ///
    /// Returns an error if epoll_create1 fails.
    pub fn with_capacity(max_events: usize) -> IoResult<Self> {
        let epoll_fd = unsafe {
            libc::epoll_create1(libc::O_CLOEXEC)
        };

        if epoll_fd < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        Ok(EpollEventLoop {
            epoll_fd,
            handlers: Vec::new(),
            fd_to_token: HashMap::new(),
            next_token: AtomicUsize::new(0),
            running: AtomicBool::new(false),
            max_events,
        })
    }

    /// Allocate a new token
    fn allocate_token(&self) -> Token {
        let index = self.next_token.fetch_add(1, Ordering::Relaxed);
        Token(index)
    }

    /// Convert Interest to epoll events
    fn interest_to_epoll_events(interest: &Interest) -> u32 {
        let mut events = 0;

        if interest.is_readable() {
            events |= sys::EPOLLIN;
        }

        if interest.is_writable() {
            events |= sys::EPOLLOUT;
        }

        if interest.is_error() {
            events |= sys::EPOLLERR;
        }

        // Always use edge-triggered mode
        events |= sys::EPOLLET;

        // Use EPOLLONESHOT for thread safety
        events |= sys::EPOLLONESHOT;

        events
    }

    /// Convert epoll events to Interest
    fn epoll_events_to_interest(events: u32) -> Interest {
        Interest::new(
            (events & sys::EPOLLIN) != 0,
            (events & sys::EPOLLOUT) != 0,
            (events & sys::EPOLLERR) != 0,
        )
    }
}

impl EventLoop for EpollEventLoop {
    fn new() -> IoResult<Self> {
        EpollEventLoop::new()
    }

    fn run(&mut self) -> IoResult<()> {
        self.running.store(true, Ordering::Relaxed);

        let mut events = vec![sys::epoll_event { u64: 0, events: 0 }; self.max_events];

        while self.running.load(Ordering::Relaxed) {
            // Wait for events
            let nfds = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr(),
                    events.len() as i32,
                    -1, // Infinite timeout
                )
            };

            if nfds < 0 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue; // Interrupted by signal, retry
                }
                return Err(IoError::from(err));
            }

            // Process events
            for i in 0..nfds as usize {
                let event = &events[i];
                let token = Token(event.u64 as usize);

                // Get handler
                if let Some(Some(handler)) = self.handlers.get(token.index()) {
                    // Check event type
                    if (event.events & sys::EPOLLIN) != 0 {
                        handler.readable(token);
                    }

                    if (event.events & sys::EPOLLOUT) != 0 {
                        handler.writable(token);
                    }

                    if (event.events & sys::EPOLLERR) != 0 {
                        handler.error(token, IoError::new(crate::error::IoErrorKind::Other));
                    }
                }
            }
        }

        Ok(())
    }

    fn register(&mut self, source: &dyn EventSource) -> IoResult<Token> {
        let fd = source.raw_fd();
        let interest = source.interest();

        // Allocate token
        let token = self.allocate_token();

        // Ensure handlers vector is large enough
        if token.index() >= self.handlers.len() {
            self.handlers.resize(token.index() + 1, None);
        }

        // Create epoll event
        let mut epoll_event = sys::epoll_event {
            events: Self::interest_to_epoll_events(&interest),
            u64: token.index() as u64,
        };

        // Register with epoll
        let result = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                sys::EPOLL_CTL_ADD,
                fd,
                &mut epoll_event,
            )
        };

        if result < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Store mapping
        self.fd_to_token.insert(fd, token);

        Ok(token)
    }

    fn reregister(&mut self, token: Token, interest: Interest) -> IoResult<()> {
        // Find file descriptor for this token
        let fd = self.fd_to_token
            .iter()
            .find(|(_, &t)| t == token)
            .map(|(&fd, _)| fd)
            .ok_or_else(|| IoError::new(crate::error::IoErrorKind::NotFound))?;

        // Create epoll event
        let mut epoll_event = sys::epoll_event {
            events: Self::interest_to_epoll_events(&interest),
            u64: token.index() as u64,
        };

        // Reregister with epoll
        let result = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                sys::EPOLL_CTL_MOD,
                fd,
                &mut epoll_event,
            )
        };

        if result < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    fn deregister(&mut self, token: Token) -> IoResult<()> {
        // Find file descriptor for this token
        let fd = self.fd_to_token
            .remove(&token)
            .ok_or_else(|| IoError::new(crate::error::IoErrorKind::NotFound))?;

        // Remove from epoll
        let result = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                sys::EPOLL_CTL_DEL,
                fd,
                std::ptr::null_mut(),
            )
        };

        if result < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Remove handler
        if token.index() < self.handlers.len() {
            self.handlers[token.index()] = None;
        }

        Ok(())
    }

    fn submit_timer(&mut self, duration: Duration) -> IoResult<TimerHandle> {
        // Create timerfd
        let timer_fd = unsafe {
            libc::timerfd_create(
                libc::CLOCK_MONOTONIC,
                libc::TFD_NONBLOCK | libc::TFD_CLOEXEC,
            )
        };

        if timer_fd < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Calculate expiration time
        let mut spec = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: duration.as_secs() as i64,
                tv_nsec: duration.subsec_nanos() as i64,
            },
        };

        // Set timer
        let result = unsafe {
            libc::timerfd_settime(timer_fd, 0, &spec, std::ptr::null_mut())
        };

        if result < 0 {
            unsafe { libc::close(timer_fd) };
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Create timer event source
        let timer_source = TimerEventSource::new(timer_fd);
        let token = self.register(&timer_source)?;

        Ok(TimerHandle(token.index()))
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for EpollEventLoop {
    fn drop(&mut self) {
        // Close epoll fd
        if self.epoll_fd >= 0 {
            unsafe {
                libc::close(self.epoll_fd);
            }
        }
    }
}

/// Timer event source
///
/// Wraps a timerfd for use with the event loop.
struct TimerEventSource {
    fd: RawFd,
}

impl TimerEventSource {
    fn new(fd: RawFd) -> Self {
        TimerEventSource { fd }
    }
}

impl EventSource for TimerEventSource {
    fn raw_fd(&self) -> RawFd {
        self.fd
    }

    fn interest(&self) -> Interest {
        Interest::READABLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoll_event_loop_creation() {
        let event_loop = EpollEventLoop::new();
        assert!(event_loop.is_ok());
    }

    #[test]
    fn test_epoll_event_loop_with_capacity() {
        let event_loop = EpollEventLoop::with_capacity(512);
        assert!(event_loop.is_ok());
    }

    #[test]
    fn test_interest_to_epoll_events() {
        let interest = Interest::READABLE;
        let events = EpollEventLoop::interest_to_epoll_events(&interest);
        assert!(events & sys::EPOLLIN != 0);
        assert!(events & sys::EPOLLET != 0); // Edge-triggered
        assert!(events & sys::EPOLLONESHOT != 0); // One-shot
    }

    #[test]
    fn test_interest_to_epoll_events_combined() {
        let interest = Interest::READABLE | Interest::WRITABLE;
        let events = EpollEventLoop::interest_to_epoll_events(&interest);
        assert!(events & sys::EPOLLIN != 0);
        assert!(events & sys::EPOLLOUT != 0);
    }

    #[test]
    fn test_timer_event_source() {
        let timer_source = TimerEventSource::new(42);
        assert_eq!(timer_source.raw_fd(), 42);
        assert_eq!(timer_source.interest(), Interest::READABLE);
    }
}
