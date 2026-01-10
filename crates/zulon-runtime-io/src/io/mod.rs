// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Non-blocking I/O primitives
//!
//! This module provides types and functions for non-blocking I/O operations.

pub mod poll;
pub mod tcp;
pub mod async_io;

pub use poll::{set_nonblocking, is_nonblocking};
pub use tcp::{TcpStream, TcpListener, Shutdown, AsyncPending};
pub use async_io::{AsyncTcpStream, AsyncTcpListener};
