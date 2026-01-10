// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Executor trait for async task execution

use zulon_async_futures::Future;
use crate::task::Task;

/// Core trait for executing async tasks
///
/// Executors are responsible for managing and polling async tasks to completion.
/// Different executor implementations provide different scheduling strategies:
/// - Thread-per-task: Each task runs on its own OS thread
/// - Work-stealing: Tasks are distributed across a thread pool
/// - Single-threaded: All tasks run on one thread with cooperative scheduling
pub trait Executor {
    /// Spawn a new async task on this executor
    ///
    /// The executor takes ownership of the future and will poll it until completion.
    /// The future's Output type must be () (unit type) for tasks spawned this way.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, BasicExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// ```
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static);

    /// Run the executor until all tasks are complete
    ///
    /// This method blocks until all spawned tasks have completed.
    /// It should only be called once per executor instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, BasicExecutor};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// executor.spawn(Ready::new(()));
    /// executor.spawn(Ready::new(()));
    /// executor.run(); // Blocks until both tasks complete
    /// ```
    fn run(&mut self);

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
    /// println!("Has tasks: {}", executor.has_pending_tasks()); // false
    /// executor.spawn(Ready::new(()));
    /// println!("Has tasks: {}", executor.has_pending_tasks()); // true
    /// ```
    fn has_pending_tasks(&self) -> bool;
}

/// Extension trait for spawning futures with non-unit output
///
/// This trait allows spawning futures that return values, not just ().
/// The spawned future is wrapped in a Task that ignores the output.
pub trait ExecutorExt: Executor {
    /// Spawn a future and return a handle to its result
    ///
    /// This is useful when you want to execute a future and get its result,
    /// but the result processing happens outside the executor context.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute
    ///
    /// # Returns
    ///
    /// A task handle that can be used to poll the future manually
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, BasicExecutor, ExecutorExt};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = BasicExecutor::new();
    /// let handle = executor.spawn_with_output(Ready::new(42));
    /// // Use the handle to get the result later
    /// ```
    fn spawn_with_output<F>(&mut self, future: F) -> Task<F::Output>
    where
        F: Future + 'static,
        F::Output: Send + 'static;
}

// Blanket implementation for all executors
impl<E: Executor> ExecutorExt for E {
    fn spawn_with_output<F>(&mut self, future: F) -> Task<F::Output>
    where
        F: Future + 'static,
        F::Output: Send + 'static,
    {
        Task::new(future)
    }
}
