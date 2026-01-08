// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! IO error types

use std::fmt;

/// IO error kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoErrorKind {
    /// Entity not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Connection reset
    ConnectionReset,
    /// Connection aborted
    ConnectionAborted,
    /// Not connected
    NotConnected,
    /// Broken pipe
    BrokenPipe,
    /// Would block
    WouldBlock,
    /// Invalid input
    InvalidInput,
    /// Timed out
    TimedOut,
    /// Other error
    Other,
}

impl fmt::Display for IoErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoErrorKind::NotFound => write!(f, "entity not found"),
            IoErrorKind::PermissionDenied => write!(f, "permission denied"),
            IoErrorKind::ConnectionReset => write!(f, "connection reset"),
            IoErrorKind::ConnectionAborted => write!(f, "connection aborted"),
            IoErrorKind::NotConnected => write!(f, "not connected"),
            IoErrorKind::BrokenPipe => write!(f, "broken pipe"),
            IoErrorKind::WouldBlock => write!(f, "operation would block"),
            IoErrorKind::InvalidInput => write!(f, "invalid input"),
            IoErrorKind::TimedOut => write!(f, "timed out"),
            IoErrorKind::Other => write!(f, "other error"),
        }
    }
}

/// IO error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoError {
    kind: IoErrorKind,
    message: Option<String>,
}

impl IoError {
    /// Create a new IO error
    pub fn new(kind: IoErrorKind) -> Self {
        IoError {
            kind,
            message: None,
        }
    }

    /// Create a new IO error with a message
    pub fn with_message(kind: IoErrorKind, message: String) -> Self {
        IoError {
            kind,
            message: Some(message),
        }
    }

    /// Get the error kind
    pub fn kind(&self) -> IoErrorKind {
        self.kind
    }

    /// Get the error message
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.message {
            write!(f, "{}: {}", self.kind, msg)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl std::error::Error for IoError {}

/// IO result type
pub type IoResult<T> = Result<T, IoError>;

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        let kind = match err.kind() {
            std::io::ErrorKind::NotFound => IoErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
            std::io::ErrorKind::ConnectionReset => IoErrorKind::ConnectionReset,
            std::io::ErrorKind::ConnectionAborted => IoErrorKind::ConnectionAborted,
            std::io::ErrorKind::NotConnected => IoErrorKind::NotConnected,
            std::io::ErrorKind::BrokenPipe => IoErrorKind::BrokenPipe,
            std::io::ErrorKind::WouldBlock => IoErrorKind::WouldBlock,
            std::io::ErrorKind::InvalidInput => IoErrorKind::InvalidInput,
            std::io::ErrorKind::TimedOut => IoErrorKind::TimedOut,
            _ => IoErrorKind::Other,
        };

        IoError::with_message(kind, err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_creation() {
        let err = IoError::new(IoErrorKind::NotFound);
        assert_eq!(err.kind(), IoErrorKind::NotFound);
        assert!(err.message().is_none());
    }

    #[test]
    fn test_io_error_with_message() {
        let err = IoError::with_message(IoErrorKind::NotFound, "file not found".to_string());
        assert_eq!(err.kind(), IoErrorKind::NotFound);
        assert_eq!(err.message(), Some("file not found"));
    }

    #[test]
    fn test_io_error_display() {
        let err = IoError::new(IoErrorKind::NotFound);
        assert_eq!(format!("{}", err), "entity not found");

        let err_with_msg = IoError::with_message(IoErrorKind::NotFound, "test.txt".to_string());
        assert_eq!(format!("{}", err_with_msg), "entity not found: test.txt");
    }
}
