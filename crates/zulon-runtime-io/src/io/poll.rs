// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Non-blocking I/O primitives
//!
//! This module provides types for non-blocking I/O operations.

use crate::error::{IoError, IoResult};
use std::os::unix::io::RawFd;

/// Set a file descriptor to non-blocking mode
///
/// # Arguments
///
/// * `fd` - File descriptor to set non-blocking
///
/// # Errors
///
/// Returns an error if fcntl fails.
pub fn set_nonblocking(fd: RawFd) -> IoResult<()> {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        let result = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        if result < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }
    }

    Ok(())
}

/// Check if a file descriptor is non-blocking
///
/// # Arguments
///
/// * `fd` - File descriptor to check
///
/// # Errors
///
/// Returns an error if fcntl fails.
pub fn is_nonblocking(fd: RawFd) -> IoResult<bool> {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        Ok((flags & libc::O_NONBLOCK) != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_nonblocking_pipe() {
        unsafe {
            let mut pipe_fds: [i32; 2] = [0, 0];
            let result = libc::pipe(pipe_fds.as_mut_ptr());
            assert_eq!(result, 0);

            // Set read end to non-blocking
            set_nonblocking(pipe_fds[0]).unwrap();

            // Verify it's non-blocking
            assert!(is_nonblocking(pipe_fds[0]).unwrap());

            // Clean up
            libc::close(pipe_fds[0]);
            libc::close(pipe_fds[1]);
        }
    }

    #[test]
    fn test_is_nonblocking_default_blocking() {
        unsafe {
            let mut pipe_fds: [i32; 2] = [0, 0];
            let result = libc::pipe(pipe_fds.as_mut_ptr());
            assert_eq!(result, 0);

            // Pipe is blocking by default
            assert!(!is_nonblocking(pipe_fds[0]).unwrap());

            // Clean up
            libc::close(pipe_fds[0]);
            libc::close(pipe_fds[1]);
        }
    }
}
