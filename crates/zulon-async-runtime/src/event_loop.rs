//! Event Loop Abstraction
//!
//! This module defines the event loop interface that will be implemented
//! differently for each platform (epoll on Linux, kqueue on macOS/BSD, IOCP on Windows).

use crate::effect::AsyncOperation;
use crate::platform::EventLoopError;
use std::time::Duration;

/// File descriptor or handle type
pub type Fd = i32;

/// Event handler trait
///
/// Implementations of this trait handle I/O events from the event loop.
pub trait EventHandler: Send + Sync {
    /// Handle a read event
    fn on_read(&mut self, fd: Fd) -> Result<Vec<u8>, EventLoopError>;

    /// Handle a write event
    fn on_write(&mut self, fd: Fd, data: Vec<u8>) -> Result<(), EventLoopError>;

    /// Handle an error event
    fn on_error(&mut self, _fd: Fd, _error: EventLoopError);
}

/// Event loop trait
///
/// This trait provides a platform-agnostic interface to the event loop.
/// Implementations exist for epoll (Linux), kqueue (macOS/BSD), and IOCP (Windows).
pub trait EventLoop: Send + Sync {
    /// Register a file descriptor for read events
    fn register_read(&mut self, fd: Fd, handler: Box<dyn EventHandler>) -> Result<(), EventLoopError>;

    /// Register a file descriptor for write events
    fn register_write(&mut self, fd: Fd, handler: Box<dyn EventHandler>) -> Result<(), EventLoopError>;

    /// Deregister a file descriptor
    fn deregister(&mut self, fd: Fd) -> Result<(), EventLoopError>;

    /// Run the event loop for the specified duration
    ///
    /// Returns the number of events processed
    fn run_once(&mut self, timeout: Option<Duration>) -> Result<usize, EventLoopError>;

    /// Submit an async operation to the event loop
    fn submit(&mut self, operation: AsyncOperation) -> Result<Vec<u8>, EventLoopError>;

    /// Check if the event loop is empty (no registered handlers)
    fn is_empty(&self) -> bool;
}

/// Mock event loop for testing
#[cfg(test)]
pub struct MockEventLoop {
    handlers: std::collections::HashMap<Fd, Box<dyn EventHandler>>,
}

#[cfg(test)]
impl Default for MockEventLoop {
    fn default() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
impl EventLoop for MockEventLoop {
    fn register_read(&mut self, fd: Fd, handler: Box<dyn EventHandler>) -> Result<(), EventLoopError> {
        self.handlers.insert(fd, handler);
        Ok(())
    }

    fn register_write(&mut self, _fd: Fd, _handler: Box<dyn EventHandler>) -> Result<(), EventLoopError> {
        // Mock implementation
        Ok(())
    }

    fn deregister(&mut self, fd: Fd) -> Result<(), EventLoopError> {
        self.handlers.remove(&fd);
        Ok(())
    }

    fn run_once(&mut self, _timeout: Option<Duration>) -> Result<usize, EventLoopError> {
        Ok(self.handlers.len())
    }

    fn submit(&mut self, _operation: AsyncOperation) -> Result<Vec<u8>, EventLoopError> {
        Ok(vec![])
    }

    fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler;

    impl EventHandler for TestHandler {
        fn on_read(&mut self, _fd: Fd) -> Result<Vec<u8>, EventLoopError> {
            Ok(vec![1, 2, 3])
        }

        fn on_write(&mut self, _fd: Fd, _data: Vec<u8>) -> Result<(), EventLoopError> {
            Ok(())
        }

        fn on_error(&mut self, _fd: Fd, _error: EventLoopError) {}
    }

    #[test]
    fn test_mock_event_loop() {
        let mut loop_: MockEventLoop = MockEventLoop::default();
        assert!(loop_.is_empty());

        loop_
            .register_read(1, Box::new(TestHandler))
            .unwrap();
        assert!(!loop_.is_empty());
        assert_eq!(loop_.run_once(None).unwrap(), 1);

        loop_.deregister(1).unwrap();
        assert!(loop_.is_empty());
    }
}
