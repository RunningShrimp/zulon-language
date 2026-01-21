// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! File system error types

use std::io as std_io;

/// Error type for file system operations
#[derive(Debug, PartialEq)]
pub enum IoError {
    /// Input/output error
    Io(std_io::ErrorKind),
    /// File not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Already exists
    AlreadyExists,
    /// Invalid input
    InvalidInput(String),
    /// Broken pipe
    BrokenPipe,
    /// Write zero
    WriteZero,
    /// Interrupted
    Interrupted,
    /// Timeout error
    Timeout,
    /// Path too long
    PathTooLong,
    /// Other error
    Other(String),
}

impl From<std_io::Error> for IoError {
    fn from(err: std_io::Error) -> Self {
        match err.kind() {
            std_io::ErrorKind::NotFound => IoError::NotFound,
            std_io::ErrorKind::PermissionDenied => IoError::PermissionDenied,
            std_io::ErrorKind::AlreadyExists => IoError::AlreadyExists,
            std_io::ErrorKind::InvalidInput => IoError::InvalidInput(err.to_string()),
            std_io::ErrorKind::BrokenPipe => IoError::BrokenPipe,
            std_io::ErrorKind::WriteZero => IoError::WriteZero,
            std_io::ErrorKind::Interrupted => IoError::Interrupted,
            std_io::ErrorKind::TimedOut => IoError::Timeout,
            std_io::ErrorKind::InvalidData => IoError::PathTooLong,
            _ => IoError::Io(err.kind()),
        }
    }
}

/// Result type for file operations
pub type IoResult<T> = crate::Outcome<T, IoError>;
