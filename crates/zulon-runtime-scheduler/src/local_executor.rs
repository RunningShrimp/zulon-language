// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Local (single-threaded) executor implementation
//!
//! This module provides a single-threaded executor for running async futures
//! on the current thread with cooperative scheduling.

use crate::Executor;
use zulon_async_futures::{Future, Poll, Context, Waker, RawWaker, RawWakerVTable};
use std::collections::VecDeque;
use std::pin::Pin;

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

/// A single-threaded executor that runs tasks cooperatively
///
/// This executor runs all tasks on the current thread, switching between
/// them cooperatively when they yield. It's efficient for I/O-bound workloads
/// and doesn't require thread synchronization.
///
/// # Example
///
/// ```rust
/// use zulon_runtime_scheduler::{Executor, LocalExecutor};
/// use zulon_async_futures::Ready;
///
/// let mut executor = LocalExecutor::new();
///
/// executor.spawn(Ready::new(()));
/// executor.spawn(Ready::new(()));
///
/// executor.run(); // Runs both tasks cooperatively
/// ```
pub struct LocalExecutor {
    /// Queue of futures to execute
    tasks: VecDeque<Box<dyn Future<Output = ()> + 'static>>,
}

impl LocalExecutor {
    /// Create a new LocalExecutor
    ///
    /// # Returns
    ///
    /// A new single-threaded executor instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::LocalExecutor;
    ///
    /// let executor = LocalExecutor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }
}

impl Default for LocalExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for LocalExecutor {
    /// Spawn a new async task on this executor
    ///
    /// The future is added to the task queue and will be executed when
    /// the executor is run.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute (must return ())
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, LocalExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = LocalExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// ```
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        self.tasks.push_back(Box::new(future));
    }

    /// Run the executor until all tasks are complete
    ///
    /// This method processes all tasks in the queue, polling each one
    /// until it completes. Tasks are executed cooperatively on the current thread.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::Executor;
    /// use zulon_runtime_scheduler::LocalExecutor;
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = LocalExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// executor.spawn(Ready::new(()));
    /// executor.run(); // Runs both tasks cooperatively
    /// ```
    fn run(&mut self) {
        // Process all tasks until they complete
        let waker = create_dummy_waker();
        let mut cx = Context::from_waker(&waker);

        while let Some(mut future) = self.tasks.pop_front() {
            // Pin the future for polling
            // SAFETY: The future is pinned on the stack and won't move
            let pinned = unsafe { Pin::new_unchecked(&mut *future) };

            match pinned.poll(&mut cx) {
                Poll::Ready(_) => {
                    // Task completed, drop it
                }
                Poll::Pending => {
                    // Task not ready, put it back in the queue
                    // In a full implementation with event loop, we'd
                    // wait for the waker before re-polling
                    self.tasks.push_back(future);
                }
            }
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
    /// use zulon_runtime_scheduler::Executor;
    /// use zulon_runtime_scheduler::LocalExecutor;
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = LocalExecutor::new();
    /// assert!(!executor.has_pending_tasks());
    ///
    /// executor.spawn(Ready::new(()));
    /// assert!(executor.has_pending_tasks());
    ///
    /// executor.run();
    /// assert!(!executor.has_pending_tasks());
    /// ```
    fn has_pending_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_async_futures::Ready;

    #[test]
    fn test_executor_creation() {
        let executor = LocalExecutor::new();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_default() {
        let executor = LocalExecutor::default();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_spawn_increases_task_count() {
        let mut executor = LocalExecutor::new();
        executor.spawn(Ready::new(()));
        assert!(executor.has_pending_tasks());
    }

    #[test]
    fn test_run_clears_tasks() {
        let mut executor = LocalExecutor::new();
        executor.spawn(Ready::new(()));
        executor.run();
        assert!(!executor.has_pending_tasks());
    }
}
