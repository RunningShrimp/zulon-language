// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Task wrapper for async futures
//!
//! Tasks provide state management and polling capabilities for futures.

use zulon_async_futures::{Future, Poll, Context};
use std::pin::Pin;

/// A wrapper around a future that tracks its state
///
/// Tasks are used by executors to manage the lifecycle of async operations.
/// They track whether a future is still pending or has completed.
///
/// # Type Parameters
///
/// * `T` - The output type of the wrapped future
///
/// # Example
///
/// ```rust
/// use zulon_runtime_scheduler::Task;
/// use zulon_async_futures::{Future, Poll, Context, Ready};
///
/// let task = Task::new(Ready::new(42));
/// // The task can be polled using task.poll(cx)
/// ```
pub struct Task<T> {
    /// The wrapped future, if not yet completed
    future: Option<Pin<Box<dyn Future<Output = T> + 'static>>>,
}

impl<T> Task<T> {
    /// Create a new task from a future
    ///
    /// # Arguments
    ///
    /// * `future` - The async operation to wrap
    ///
    /// # Returns
    ///
    /// A new Task instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::Task;
    /// use zulon_async_futures::Ready;
    ///
    /// let task = Task::new(Ready::new(42));
    /// ```
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + 'static,
    {
        Self {
            future: Some(Box::pin(future)),
        }
    }

    /// Poll the task's future
    ///
    /// This method advances the async operation by calling its poll method.
    /// Once the future completes, subsequent calls will always return the
    /// completed result.
    ///
    /// # Arguments
    ///
    /// * `cx` - The context for this polling operation
    ///
    /// # Returns
    ///
    /// * `Poll::Pending` - The future is not ready yet
    /// * `Poll::Ready(value)` - The future completed with the given value
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::Task;
    /// use zulon_async_futures::{Future, Poll, Context, Waker, Ready};
    ///
    /// let mut task = Task::new(Ready::new(42));
    /// let waker = Waker::noop();
    /// let mut cx = Context::from_waker(&waker);
    ///
    /// match task.poll(&mut cx) {
    ///     Poll::Ready(value) => println!("Got: {}", value),
    ///     Poll::Pending => println!("Not ready yet"),
    /// }
    /// ```
    pub fn poll(&mut self, cx: &mut Context) -> Poll<T>
    where
        T: 'static,
    {
        if let Some(future) = &mut self.future {
            let result = future.as_mut().poll(cx);
            if result.is_ready() {
                // Future completed, take it out
                self.future.take();
            }
            result
        } else {
            // Future already completed - this should not happen in normal use
            // because completed tasks should be dropped by the executor
            panic!("Poll called on completed task");
        }
    }

    /// Check if the task has completed
    ///
    /// # Returns
    ///
    /// * `true` - The task's future has completed
    /// * `false` - The task is still pending
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::Task;
    /// use zulon_async_futures::Ready;
    ///
    /// let task = Task::new(Ready::new(42));
    /// assert!(!task.is_completed());
    /// // After polling to completion...
    /// // assert!(task.is_completed());
    /// ```
    pub fn is_completed(&self) -> bool {
        self.future.is_none()
    }
}
