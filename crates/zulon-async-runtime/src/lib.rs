// ZULON Async Runtime - Effect-Based Async I/O
//
// This crate provides an effect-based async runtime for the ZULON language.
// Unlike traditional async/await systems, ZULON uses effect handlers to
// manage asynchronous operations, eliminating function coloring.

#![deny(missing_docs)]
// Note: We allow unsafe_code in the platform layer because we need to call
// platform-specific syscalls (epoll_create1, epoll_ctl, epoll_wait, etc.)
// These are carefully controlled and documented.

//! ZULON Async Runtime - Effect-Based Async I/O
//!
//! This crate provides an effect-based async runtime for the ZULON language.
//! Unlike traditional async/await systems, ZULON uses effect handlers to
//! manage asynchronous operations, eliminating function coloring.
//!
//! # Key Features
//!
//! - **No Function Coloring**: Async operations use effect handlers, not colored functions
//! - **Type-Safe Effects**: Async operations are type-checked through the effect system
//! - **Composable Handlers**: Multiple effects can be combined and layered
//! - **Platform-Native**: Uses epoll (Linux), kqueue (macOS/BSD), or IOCP (Windows)
//!
//! # Architecture
//!
//! The runtime consists of:
//! - [`effect`] - Async effect operations (file I/O, network, timers)
//! - [`event_loop`] - Platform-agnostic event loop interface
//! - [`continuation`] - Continuation capture and restoration
//! - [`platform`] - Platform-specific event loop implementations
//!
//! # Example
//!
//! ```zulon
//! effect Async {
//!     fn read(path: string) -> string
//! }
//!
//! fn main() -> i32 {
//!     try {
//!         let data = Async::read("data.txt")
//!         printf("%s\n", data)
//!     } with Async {
//!         // Event loop handles the async operation
//!     }
//!     0
//! }
//! ```

pub mod continuation;
pub mod effect;
pub mod event_loop;
pub mod platform;
pub mod task;
pub mod thread_pool;

pub use continuation::{Continuation, ContinuationManager};
pub use effect::{AsyncEffect, AsyncOperation};
pub use event_loop::{EventHandler, EventLoop};
pub use platform::EventLoopFactory;
pub use task::{SharedTaskQueue, Task, TaskId, TaskQueue, TaskState};
pub use thread_pool::{ThreadPool, ThreadPoolConfig};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Async runtime builder
#[derive(Debug, Clone)]
pub struct RuntimeBuilder {
    /// Number of worker threads (for thread pool support)
    worker_threads: Option<usize>,
    /// Platform-specific configuration
    platform_config: platform::PlatformConfig,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self {
            worker_threads: None,
            platform_config: platform::PlatformConfig::default(),
        }
    }
}

impl RuntimeBuilder {
    /// Create a new runtime builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set number of worker threads
    pub fn worker_threads(mut self, count: usize) -> Self {
        self.worker_threads = Some(count);
        self
    }

    /// Build the async runtime
    pub fn build(self) -> Result<Runtime, BuildError> {
        let event_loop = EventLoopFactory::create(self.platform_config)?;

        let thread_pool = if let Some(num_threads) = self.worker_threads {
            let config = thread_pool::ThreadPoolConfig::new().num_threads(num_threads);
            Some(thread_pool::ThreadPool::new(config))
        } else {
            None
        };

        let task_queue = SharedTaskQueue::new();

        Ok(Runtime {
            event_loop,
            thread_pool,
            task_queue,
        })
    }
}

/// Async runtime instance
pub struct Runtime {
    event_loop: Box<dyn EventLoop>,
    thread_pool: Option<thread_pool::ThreadPool>,
    task_queue: SharedTaskQueue,
}

impl Runtime {
    /// Run the async runtime to completion
    pub fn block_on<F, R>(&mut self, fut: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _event_loop = self.event_loop_mut();

        let task_id = self.task_queue.spawn();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fut()));

        let result = match result {
            Ok(r) => r,
            Err(_) => {
                if let Some(mut task) = self.task_queue.get_task(task_id) {
                    task.panic();
                }
                std::panic::panic_any("task panicked");
            }
        };

        result
    }

    /// Spawn an async task
    pub fn spawn(&self, _f: impl FnOnce() + Send + 'static) -> TaskId {
        let task_id = self.task_queue.spawn();

        if let Some(_thread_pool) = &self.thread_pool {
            std::thread::spawn(move || {});
        }

        task_id
    }

    /// Get a reference to the event loop
    pub fn event_loop(&self) -> &dyn EventLoop {
        self.event_loop.as_ref()
    }

    /// Get a mutable reference to the event loop
    pub fn event_loop_mut(&mut self) -> &mut dyn EventLoop {
        self.event_loop.as_mut()
    }

    /// Get the task queue
    pub fn task_queue(&self) -> &SharedTaskQueue {
        &self.task_queue
    }

    /// Get a mutable reference to the task queue
    pub fn task_queue_mut(&mut self) -> &mut SharedTaskQueue {
        &mut self.task_queue
    }
}

/// Error that can occur when building the runtime
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// Failed to create event loop
    #[error("failed to create event loop: {0}")]
    EventLoopError(#[from] platform::EventLoopError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_builder() {
        let builder = RuntimeBuilder::new();
        assert!(builder.build().is_ok());
    }

    #[test]
    fn test_runtime_version() {
        assert!(!VERSION.is_empty());
    }
}
