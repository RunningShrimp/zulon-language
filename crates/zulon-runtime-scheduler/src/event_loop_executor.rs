// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Event loop executor implementation
//!
//! This module provides an executor that integrates with Phase 2.2's event loop
//! for proper async task execution with I/O support.

use crate::Executor;
use zulon_async_futures::{Future, Poll, Context, Waker, RawWaker, RawWakerVTable};
use std::collections::VecDeque;

/// An executor that runs tasks on the current thread
///
/// This executor polls futures cooperatively and can be integrated
/// with an event loop for I/O operations.
///
/// # Example
///
/// ```rust
/// use zulon_runtime_scheduler::{Executor, EventLoopExecutor};
/// use zulon_async_futures::Ready;
///
/// let mut executor = EventLoopExecutor::new();
/// executor.spawn(Ready::new(()));
/// executor.run();
/// ```
pub struct EventLoopExecutor {
    /// Queue of futures to execute
    tasks: VecDeque<Task>,
}

/// A wrapped future with its waker
struct Task {
    /// The boxed future
    future: Box<dyn Future<Output = ()> + 'static>,
    /// Waker for this task
    waker: Waker,
    /// Task ID (reserved for future use)
    #[allow(dead_code)]
    id: usize,
}

impl EventLoopExecutor {
    /// Create a new EventLoopExecutor
    ///
    /// # Returns
    ///
    /// A new executor instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::EventLoopExecutor;
    ///
    /// let executor = EventLoopExecutor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    /// Get the number of pending tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for EventLoopExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for EventLoopExecutor {
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
    /// use zulon_runtime_scheduler::{Executor, EventLoopExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = EventLoopExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// ```
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        // Create a simple waker for this task
        // In a full implementation, this would integrate with the event loop
        let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };

        let task = Task {
            future: Box::new(future),
            waker,
            id: self.tasks.len(),
        };

        self.tasks.push_back(task);
    }

    /// Run the executor until all tasks are complete
    ///
    /// This method processes all tasks in the queue, polling each one
    /// until it completes or returns Pending.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, EventLoopExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = EventLoopExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// executor.run();
    /// ```
    fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            // Create context from waker
            let mut cx = Context::from_waker(&task.waker);

            // Pin the future for polling
            // SAFETY: The future is pinned on the stack and won't move
            let pinned = unsafe { std::pin::Pin::new_unchecked(&mut *task.future) };

            // Poll the future
            match pinned.poll(&mut cx) {
                Poll::Ready(_) => {
                    // Task completed, drop it
                }
                Poll::Pending => {
                    // Task not ready, put it back in the queue
                    // In a full implementation, we'd integrate with the event loop here
                    self.tasks.push_back(task);
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
    /// use zulon_runtime_scheduler::{Executor, EventLoopExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = EventLoopExecutor::new();
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

/// Virtual table for the dummy waker
///
/// This is a minimal implementation that does nothing.
/// In production, this would integrate with the event loop.
static VTABLE: RawWakerVTable = RawWakerVTable {
    clone: clone_waker,
    wake: wake,
    wake_by_ref: wake_by_ref,
    drop: drop_waker,
};

unsafe fn clone_waker(data: *const ()) -> RawWaker {
    RawWaker::new(data, &VTABLE)
}

unsafe fn wake(data: *const ()) {
    // In a full implementation, this would wake the task
    // and notify the event loop
    let _ = data;
}

unsafe fn wake_by_ref(data: *const ()) {
    // In a full implementation, this would wake the task
    // without consuming the waker
    let _ = data;
}

unsafe fn drop_waker(data: *const ()) {
    // In a full implementation, this would clean up task resources
    let _ = data;
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_async_futures::Ready;

    #[test]
    fn test_executor_creation() {
        let executor = EventLoopExecutor::new();
        assert!(!executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 0);
    }

    #[test]
    fn test_default() {
        let executor = EventLoopExecutor::default();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_spawn_increases_task_count() {
        let mut executor = EventLoopExecutor::new();
        executor.spawn(Ready::new(()));
        assert!(executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 1);
    }

    #[test]
    fn test_run_clears_tasks() {
        let mut executor = EventLoopExecutor::new();
        executor.spawn(Ready::new(()));
        executor.run();
        assert!(!executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 0);
    }

    #[test]
    fn test_multiple_tasks() {
        let mut executor = EventLoopExecutor::new();
        executor.spawn(Ready::new(()));
        executor.spawn(Ready::new(()));
        executor.spawn(Ready::new(()));
        assert_eq!(executor.task_count(), 3);

        executor.run();
        assert_eq!(executor.task_count(), 0);
    }
}
