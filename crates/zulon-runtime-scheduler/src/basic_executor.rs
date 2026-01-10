// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Basic executor implementation
//!
//! This module provides a simple thread-per-task executor for running async futures.

use crate::Executor;
use zulon_async_futures::{Future, Poll, Context, Waker, RawWaker, RawWakerVTable};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};

/// Create a dummy waker for polling
///
/// In a full implementation, this would integrate with the event loop
/// to wake tasks when I/O events occur.
fn create_dummy_waker() -> Waker {
    static VTABLE: RawWakerVTable = RawWakerVTable {
        clone: |data| RawWaker::new(data, &VTABLE),
        wake: |data| { let _ = data; },
        wake_by_ref: |data| { let _ = data; },
        drop: |data| { let _ = data; },
    };

    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    // SAFETY: The RawWaker methods are safe for null data pointer
    unsafe { Waker::from_raw(raw_waker) }
}

/// A simple thread-per-task executor
///
/// This executor spawns each async task on its own OS thread.
/// It's simple to understand but not the most efficient for large numbers of tasks.
///
/// # Example
///
/// ```rust
/// use zulon_runtime_scheduler::{Executor, BasicExecutor};
/// use zulon_async_futures::Ready;
///
/// let mut executor = BasicExecutor::new();
///
/// executor.spawn(Ready::new(()));
/// executor.spawn(Ready::new(()));
///
/// executor.run(); // Blocks until all tasks complete
/// ```
pub struct BasicExecutor {
    /// List of task handles for spawned threads
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl BasicExecutor {
    /// Create a new BasicExecutor
    ///
    /// # Returns
    ///
    /// A new executor instance with no tasks
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::BasicExecutor;
    ///
    /// let executor = BasicExecutor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawn a new async task on its own thread with Send bound
    ///
    /// This is the preferred way to spawn tasks on BasicExecutor as it
    /// enforces the Send requirement at compile time.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute (must return () and be Send + 'static)
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::BasicExecutor;
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// executor.spawn_send(Ready::new(()));
    /// ```
    pub fn spawn_send(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        let tasks = self.tasks.clone();

        let handle = thread::spawn(move || {
            // Poll the future to completion on this thread
            let mut future = Box::pin(future);
            let waker = create_dummy_waker();
            let mut cx = Context::from_waker(&waker);

            // Poll loop
            loop {
                match future.as_mut().poll(&mut cx) {
                    Poll::Ready(_) => break, // Task completed
                    Poll::Pending => {
                        // In a full implementation, we'd wait for the waker
                        // For now, we'll just busy-poll (inefficient but works)
                        std::thread::yield_now();
                    }
                }
            }
        });

        tasks.lock().unwrap().push(handle);
    }

    /// Spawn a non-Send future on the current thread
    ///
    /// This method runs the future to completion immediately on the current
    /// thread, blocking until it's done. Use this for futures that aren't Send.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute (must return ())
    ///
    /// # Note
    ///
    /// This blocks the current thread until the future completes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::BasicExecutor;
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// executor.spawn_local(Ready::new(())); // Runs immediately
    /// ```
    pub fn spawn_local(&mut self, future: impl Future<Output = ()> + 'static) {
        // Run the future to completion on the current thread
        let mut future = Box::pin(future);
        let waker = create_dummy_waker();
        let mut cx = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(_) => break,
                Poll::Pending => {
                    std::thread::yield_now();
                }
            }
        }
    }
}

impl Default for BasicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for BasicExecutor {
    /// Spawn a new async task
    ///
    /// This method runs the future to completion on the current thread.
    /// For true thread-per-task execution, use `spawn_send()` instead.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute (must return ())
    ///
    /// # Note
    ///
    /// The `Executor` trait cannot enforce `Send` bounds, so this method
    /// runs futures on the current thread. Use `spawn_send()` for actual
    /// thread-per-task execution with compile-time Send checking.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, BasicExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// executor.spawn(Ready::new(())); // Runs on current thread
    /// ```
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        // NOTE: For thread-per-task executor, we require Send but can't enforce it in trait.
        // Users should use spawn_send() for compile-time Send checking.
        // For the trait method, we use spawn_local() which runs on the current thread.
        // This is a limitation of the Executor trait not having Send bounds.
        //
        // TODO: Consider adding a separate ThreadExecutor trait with Send bounds.
        self.spawn_local(future);
    }

    /// Run the executor until all tasks are complete
    ///
    /// This method blocks the current thread until all spawned tasks
    /// have finished executing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::Executor;
    /// use zulon_runtime_scheduler::BasicExecutor;
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// executor.spawn(Ready::new(()));
    /// executor.run(); // Blocks until both tasks complete
    /// ```
    fn run(&mut self) {
        // Take ownership of all task handles
        let tasks = self.tasks.lock().unwrap().drain(..).collect::<Vec<_>>();

        // Wait for all tasks to complete
        for handle in tasks {
            handle.join().unwrap();
        }
    }

    /// Check if the executor has any pending tasks
    ///
    /// # Returns
    ///
    /// * `true` if there are pending tasks
    /// * `false` if all tasks have completed
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, BasicExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// assert!(!executor.has_pending_tasks());
    ///
    /// executor.spawn_send(Ready::new(()));
    /// assert!(executor.has_pending_tasks());
    ///
    /// executor.run();
    /// assert!(!executor.has_pending_tasks());
    /// ```
    fn has_pending_tasks(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_async_futures::Ready;

    #[test]
    fn test_executor_creation() {
        let executor = BasicExecutor::new();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_default() {
        let executor = BasicExecutor::default();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_spawn_increases_task_count() {
        let mut executor = BasicExecutor::new();
        executor.spawn_send(Ready::new(()));
        assert!(executor.has_pending_tasks());
    }

    #[test]
    fn test_run_clears_tasks() {
        let mut executor = BasicExecutor::new();
        executor.spawn_send(Ready::new(()));
        executor.run();
        assert!(!executor.has_pending_tasks());
    }
}
