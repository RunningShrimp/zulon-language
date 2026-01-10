//! Async Effect Operations
//!
//! This module defines the async effect that ZULON programs will use
//! to perform async I/O operations without function coloring.

use std::fmt;

/// Async effect operations
///
/// These operations are called from ZULON code using the effect system:
/// ```zulon
/// effect Async {
///     fn read(path: string) -> string
/// }
///
/// fn fetch_data() -> string {
///     Async::read("data.txt")  // No async keyword needed!
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncOperation {
    /// Read file asynchronously
    FileRead {
        /// File path
        path: String,
    },

    /// Write file asynchronously
    FileWrite {
        /// File path
        path: String,
        /// Data to write
        data: Vec<u8>,
    },

    /// TCP connect
    TcpConnect {
        /// Host address
        host: String,
        /// Port number
        port: u16,
    },

    /// TCP read
    TcpRead {
        /// Socket file descriptor
        fd: i32,
        /// Number of bytes to read
        len: usize,
    },

    /// TCP write
    TcpWrite {
        /// Socket file descriptor
        fd: i32,
        /// Data to write
        data: Vec<u8>,
    },

    /// Sleep for duration
    Sleep {
        /// Duration in milliseconds
        duration_ms: u64,
    },
}

impl fmt::Display for AsyncOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileRead { path } => write!(f, "FileRead({})", path),
            Self::FileWrite { path, .. } => write!(f, "FileWrite({})", path),
            Self::TcpConnect { host, port } => write!(f, "TcpConnect({}:{})", host, port),
            Self::TcpRead { fd, len } => write!(f, "TcpRead(fd={}, len={})", fd, len),
            Self::TcpWrite { fd, data } => write!(f, "TcpWrite(fd={}, len={})", fd, data.len()),
            Self::Sleep { duration_ms } => write!(f, "Sleep({}ms)", duration_ms),
        }
    }
}

/// Result of an async operation
pub type AsyncResult<T> = Result<T, AsyncError>;

/// Error that can occur during async operations
#[derive(Debug, thiserror::Error)]
pub enum AsyncError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Operation timed out
    #[error("operation timed out")]
    Timeout,

    /// Connection refused
    #[error("connection refused")]
    ConnectionRefused,

    /// Invalid argument
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

/// Async effect trait
///
/// This trait is implemented by the async runtime to handle
/// effect operations from ZULON code.
pub trait AsyncEffect {
    /// Handle an async operation
    fn handle(&mut self, operation: AsyncOperation) -> AsyncResult<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_operation_display() {
        let op = AsyncOperation::FileRead {
            path: "test.txt".to_string(),
        };
        assert_eq!(format!("{}", op), "FileRead(test.txt)");
    }

    #[test]
    fn test_async_operation_clone() {
        let op1 = AsyncOperation::Sleep { duration_ms: 100 };
        let op2 = op1.clone();
        assert_eq!(op1, op2);
    }
}
