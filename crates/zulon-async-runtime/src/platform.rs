//! Platform Abstraction Layer
//!
//! This module provides platform-specific event loop implementations.
//! It selects the appropriate implementation at compile time.

use crate::event_loop::EventLoop;

/// Error that can occur when creating or using the event loop
#[derive(Debug, thiserror::Error)]
pub enum EventLoopError {
    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Operation not supported on this platform
    #[error("operation not supported on this platform")]
    NotSupported,

    /// Invalid file descriptor
    #[error("invalid file descriptor: {0}")]
    InvalidFd(i32),

    /// Event loop full
    #[error("event loop at capacity")]
    Full,

    /// Other error
    #[error("event loop error: {0}")]
    Other(String),
}

/// Platform-specific configuration
#[derive(Debug, Clone, Default)]
pub struct PlatformConfig {
    /// Maximum number of events per poll
    pub max_events: usize,
    /// Enable edge-triggered mode (epoll only)
    pub edge_triggered: bool,
    /// Enable one-shot mode (epoll only)
    pub one_shot: bool,
}

/// Event loop factory
///
/// Creates the appropriate event loop implementation for the current platform.
pub struct EventLoopFactory;

impl EventLoopFactory {
    /// Create an event loop for the current platform
    pub fn create(config: PlatformConfig) -> Result<Box<dyn EventLoop>, EventLoopError> {
        #[cfg(target_os = "linux")]
        {
            Ok(Box::new(linux::EpollEventLoop::new(config)?))
        }

        #[cfg(target_os = "macos")]
        {
            Ok(Box::new(macos::KqueueEventLoop::new(config)?))
        }

        #[cfg(target_os = "windows")]
        {
            Ok(Box::new(windows::IocpEventLoop::new(config)?))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(EventLoopError::NotSupported)
        }
    }
}

// Linux implementation using epoll
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::event_loop::{EventHandler, Fd};
    use std::os::unix::io::AsRawFd;
    use std::time::Duration;

    /// Epoll-based event loop for Linux
    pub struct EpollEventLoop {
        epoll_fd: i32,
        max_events: usize,
        edge_triggered: bool,
        one_shot: bool,
        handlers: std::collections::HashMap<Fd, Box<dyn EventHandler>>,
    }

    impl EpollEventLoop {
        /// Create a new epoll event loop
        pub fn new(config: PlatformConfig) -> Result<Self, EventLoopError> {
            // Use epoll_create1 with EPOLL_CLOEXEC
            let epoll_fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };

            if epoll_fd < 0 {
                return Err(EventLoopError::Io(std::io::Error::last_os_error()));
            }

            Ok(Self {
                epoll_fd,
                max_events: config.max_events.max(1),
                edge_triggered: config.edge_triggered,
                one_shot: config.one_shot,
                handlers: std::collections::HashMap::new(),
            })
        }

        /// Convert configuration to epoll events flags
        fn epoll_events(&self, for_read: bool) -> u32 {
            let mut events = if for_read {
                libc::EPOLLIN
            } else {
                libc::EPOLLOUT
            };

            if self.edge_triggered {
                events |= libc::EPOLLET;
            }

            if self.one_shot {
                events |= libc::EPOLLONESHOT;
            }

            events as u32
        }
    }

    impl Drop for EpollEventLoop {
        fn drop(&mut self) {
            if self.epoll_fd >= 0 {
                unsafe { libc::close(self.epoll_fd) };
            }
        }
    }

    impl EventLoop for EpollEventLoop {
        fn register_read(&mut self, fd: Fd, handler: Box<dyn EventHandler>) -> Result<(), EventLoopError> {
            let mut event = libc::epoll_event {
                events: self.epoll_events(true),
                u64: fd as u64,
            };

            let ret = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) };

            if ret < 0 {
                return Err(EventLoopError::Io(std::io::Error::last_os_error()));
            }

            self.handlers.insert(fd, handler);
            Ok(())
        }

        fn register_write(&mut self, fd: Fd, handler: Box<dyn EventHandler>) -> Result<(), EventLoopError> {
            let mut event = libc::epoll_event {
                events: self.epoll_events(false),
                u64: fd as u64,
            };

            let ret = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) };

            if ret < 0 {
                return Err(EventLoopError::Io(std::io::Error::last_os_error()));
            }

            self.handlers.insert(fd, handler);
            Ok(())
        }

        fn deregister(&mut self, fd: Fd) -> Result<(), EventLoopError> {
            let ret = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };

            if ret < 0 {
                return Err(EventLoopError::Io(std::io::Error::last_os_error()));
            }

            self.handlers.remove(&fd);
            Ok(())
        }

        fn run_once(&mut self, timeout: Option<Duration>) -> Result<usize, EventLoopError> {
            let timeout_ms = timeout
                .map(|d| d.as_millis() as i32)
                .unwrap_or(-1);

            let mut events = Vec::with_capacity(self.max_events);
            let ret = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr() as *mut libc::epoll_event,
                    self.max_events as i32,
                    timeout_ms,
                )
            };

            if ret < 0 {
                return Err(EventLoopError::Io(std::io::Error::last_os_error()));
            }

            unsafe { events.set_len(ret as usize) };

            let mut events_processed = 0;

            // Process events
            for event in &events {
                let fd = event.u64 as Fd;
                if let Some(handler) = self.handlers.get_mut(&fd) {
                    let mut has_error = false;

                    // Handle read events
                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        match handler.on_read(fd) {
                            Ok(_) => {},
                            Err(e) => {
                                handler.on_error(fd, e);
                                has_error = true;
                            }
                        }
                    }

                    // Handle write events
                    if (event.events & libc::EPOLLOUT as u32) != 0 && !has_error {
                        // For write events, we'd typically have buffered data
                        // For now, this is a placeholder
                    }

                    // Handle errors
                    if (event.events & (libc::EPOLLERR as u32 | libc::EPOLLHUP as u32)) != 0 {
                        handler.on_error(fd, EventLoopError::Io(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "epoll error or hangup",
                        )));
                    }

                    events_processed += 1;
                }
            }

            Ok(events_processed)
        }

        fn submit(&mut self, operation: crate::effect::AsyncOperation) -> Result<Vec<u8>, EventLoopError> {
            use crate::effect::AsyncOperation;

            match operation {
                AsyncOperation::Sleep { duration_ms } => {
                    // For sleep operations, we can use the event loop timeout
                    // This is a simplified implementation - a real one would use timers
                    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
                    Ok(vec![])
                }
                _ => Err(EventLoopError::NotSupported),
            }
        }

        fn is_empty(&self) -> bool {
            self.handlers.is_empty()
        }
    }
}

// macOS implementation using kqueue
#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::event_loop::{EventHandler, EventLoop, Fd};
    use std::time::Duration;

    /// Kqueue-based event loop for macOS/BSD
    pub struct KqueueEventLoop {
        kqueue_fd: i32,
        handlers: std::collections::HashMap<i32, Box<dyn EventHandler>>,
    }

    impl KqueueEventLoop {
        /// Create a new kqueue event loop
        pub fn new(_config: PlatformConfig) -> Result<Self, EventLoopError> {
            let kqueue_fd = unsafe { libc::kqueue() };

            if kqueue_fd < 0 {
                return Err(EventLoopError::Io(std::io::Error::last_os_error()));
            }

            Ok(Self {
                kqueue_fd,
                handlers: std::collections::HashMap::new(),
            })
        }
    }

    impl Drop for KqueueEventLoop {
        fn drop(&mut self) {
            if self.kqueue_fd >= 0 {
                unsafe { libc::close(self.kqueue_fd) };
            }
        }
    }

    impl EventLoop for KqueueEventLoop {
        fn register_read(&mut self, fd: Fd, handler: Box<dyn EventHandler>) -> Result<(), EventLoopError> {
            // TODO: Implement kqueue EVFILT_READ registration
            self.handlers.insert(fd, handler);
            Ok(())
        }

        fn register_write(&mut self, _fd: Fd, _handler: Box<dyn EventHandler>) -> Result<(), EventLoopError> {
            // TODO: Implement kqueue EVFILT_WRITE registration
            Ok(())
        }

        fn deregister(&mut self, fd: Fd) -> Result<(), EventLoopError> {
            self.handlers.remove(&fd);
            Ok(())
        }

        fn run_once(&mut self, _timeout: Option<Duration>) -> Result<usize, EventLoopError> {
            // TODO: Implement kqueue event waiting
            Ok(self.handlers.len())
        }

        fn submit(&mut self, operation: crate::effect::AsyncOperation) -> Result<Vec<u8>, EventLoopError> {
            use crate::effect::AsyncOperation;

            match operation {
                AsyncOperation::Sleep { duration_ms } => {
                    // For sleep operations, we can use the event loop timeout
                    // This is a simplified implementation - a real one would use timers
                    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
                    Ok(vec![])
                }
                _ => Err(EventLoopError::NotSupported),
            }
        }

        fn is_empty(&self) -> bool {
            self.handlers.is_empty()
        }
    }
}

// Windows implementation using IOCP
#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    /// IOCP-based event loop for Windows
    pub struct IocpEventLoop {
        iocp_handle: winapi::um::winnt::HANDLE,
        handlers: std::collections::HashMap<i32, Box<dyn super::EventHandler>>,
    }

    impl IocpEventLoop {
        /// Create a new IOCP event loop
        pub fn new(_config: PlatformConfig) -> Result<Self, EventLoopError> {
            // TODO: Create IOCP port
            Err(EventLoopError::NotSupported)
        }
    }

    // TODO: Implement EventLoop trait for IocpEventLoop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_factory() {
        let config = PlatformConfig::default();
        let result = EventLoopFactory::create(config);

        // On supported platforms, this should succeed
        // On unsupported platforms, it should fail with NotSupported
        if cfg!(any(target_os = "linux", target_os = "macos", target_os = "windows")) {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(EventLoopError::NotSupported)));
        }
    }

    #[test]
    fn test_platform_config_default() {
        let config = PlatformConfig::default();
        assert_eq!(config.max_events, 0); // Default value
        assert!(!config.edge_triggered);
        assert!(!config.one_shot);
    }

    #[test]
    fn test_platform_config_custom() {
        let config = PlatformConfig {
            max_events: 1024,
            edge_triggered: true,
            one_shot: true,
        };

        assert_eq!(config.max_events, 1024);
        assert!(config.edge_triggered);
        assert!(config.one_shot);
    }

    #[test]
    fn test_sleep_operation() {
        let config = PlatformConfig::default();
        if let Ok(mut event_loop) = EventLoopFactory::create(config) {
            use crate::effect::AsyncOperation;

            let start = std::time::Instant::now();
            let result = event_loop.submit(AsyncOperation::Sleep { duration_ms: 10 });
            let elapsed = start.elapsed();

            assert!(result.is_ok());
            assert!(elapsed >= std::time::Duration::from_millis(10));
            assert!(elapsed < std::time::Duration::from_millis(100)); // Should not be too slow
        }
    }

    #[test]
    fn test_unsupported_operation() {
        let config = PlatformConfig::default();
        if let Ok(mut event_loop) = EventLoopFactory::create(config) {
            use crate::effect::AsyncOperation;

            // File operations should not be supported yet
            let result = event_loop.submit(AsyncOperation::FileRead {
                path: "/tmp/test.txt".to_string(),
            });

            assert!(matches!(result, Err(EventLoopError::NotSupported)));
        }
    }
}
