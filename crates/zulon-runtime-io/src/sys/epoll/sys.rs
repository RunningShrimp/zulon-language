// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! epoll system call bindings
//!
//! This module provides low-level bindings to Linux epoll system calls.

use std::os::unix::io::RawFd;

/// epoll event structure
///
/// This mirrors the C struct epoll_event from <sys/epoll.h>.
#[repr(C)]
pub struct epoll_event {
    /// Epoll events (bitmask)
    pub events: u32,
    /// User data (typically token or file descriptor)
    pub u64: u64,
}

// Epoll control commands
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_MOD: i32 = 3;
pub const EPOLL_CTL_DEL: i32 = 2;

// Epoll event types
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;
pub const EPOLLPRI: u32 = 0x002;
pub const EPOLLRDHUP: u32 = 0x2000;

// Epoll flags
pub const EPOLLET: u32 = 1 << 31; // Edge-triggered mode
pub const EPOLLONESHOT: u32 = 1 << 30; // One-shot mode
pub const EPOLLWAKEUP: u32 = 1 << 29; // Wakeup support
pub const EPOLLEXCLUSIVE: u32 = 1 << 28; // Exclusive wake-up

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoll_event_size() {
        assert_eq!(std::mem::size_of::<epoll_event>(), 16);
    }

    #[test]
    fn test_epoll_constants() {
        assert_eq!(EPOLL_CTL_ADD, 1);
        assert_eq!(EPOLL_CTL_MOD, 3);
        assert_eq!(EPOLL_CTL_DEL, 2);
    }

    #[test]
    fn test_epoll_event_flags() {
        assert!(EPOLLIN > 0);
        assert!(EPOLLOUT > 0);
        assert!(EPOLLERR > 0);
        assert!(EPOLLET > 0);
        assert!(EPOLLONESHOT > 0);
    }
}
