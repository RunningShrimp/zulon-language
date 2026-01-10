// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Platform-specific event loop implementations
//!
//! This module provides platform-specific implementations of the EventLoop trait:
//! - **Linux**: epoll (edge-triggered mode)
//! - **macOS/BSD**: kqueue (edge-triggered mode)
//! - **Windows**: IOCP (future)

#[cfg(target_os = "linux")]
pub mod epoll;

#[cfg(target_os = "linux")]
pub use epoll::{EpollEventLoop, DEFAULT_MAX_EVENTS};

#[cfg(target_os = "linux")]
pub use epoll::sys as epoll_sys;

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub mod kqueue;

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub use kqueue::KqueueEventLoop;

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub use kqueue::sys as kqueue_sys;
