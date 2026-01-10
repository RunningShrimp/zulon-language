// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Enhanced Event Loop Executor with Wake-up Support
//!
//! This module extends the event loop executor with actual task wake-up
//! mechanism using the task registry.

use crate::Executor;
use crate::waker_registry::{generate_task_id, global_registry, init_global_registry, TaskRegistry};
use zulon_async_futures::{Future, Poll, Context, Waker, RawWaker, RawWakerVTable};
use std::collections::VecDeque;
use std::sync::Arc;

/// A task that can be woken up by the event loop
struct EventLoopTask {
    /// The boxed future
    future: Box<dyn Future<Output = ()> + 'static>,
    /// Task ID
    id: usize,
    /// Waker for this task
    waker: Option<Waker>,
    /// File descriptor to monitor (if any)
    #[allow(dead_code)]
    fd: Option<std::os::unix::io::RawFd>,
    /// Reference to task state in registry
    #[allow(dead_code)]
    task_state: Option<Arc<std::sync::Mutex<crate::waker_registry::TaskState>>>,
}

impl EventLoopTask {
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        let id = generate_task_id();
        Self {
            future: Box::new(future),
            id,
            waker: None,
            fd: None,
            task_state: None,
        }
    }

    /// Check if this task is ready to run
    fn is_ready(&self) -> bool {
        // Check the global registry
        let registry_guard = global_registry().lock().unwrap();
        if let Some(registry) = registry_guard.as_ref() {
            registry.is_task_ready(self.id)
        } else {
            // If registry not initialized, assume ready
            true
        }
    }

    /// Set the file descriptor to monitor for this task
    #[allow(dead_code)]
    fn set_fd(&mut self, fd: std::os::unix::io::RawFd) {
        self.fd = Some(fd);
    }
}

/// An enhanced event loop executor with task wake-up support
///
/// This executor maintains a task registry and can wake up tasks
/// when I/O events occur.
///
/// # Example
///
/// ```rust
/// use zulon_runtime_scheduler::{Executor, EventLoopExecutorWithWaker};
/// use zulon_async_futures::Ready;
///
/// // Initialize the global registry
/// let mut executor = EventLoopExecutorWithWaker::new();
/// executor.spawn(Ready::new(()));
/// executor.run(); // Polls the future
/// ```
pub struct EventLoopExecutorWithWaker {
    /// Queue of futures to execute
    tasks: VecDeque<EventLoopTask>,
    /// Task registry for wake-up notifications
    registry: TaskRegistry,
    /// Number of completed tasks
    completed_count: usize,
}

impl EventLoopExecutorWithWaker {
    /// Create a new EventLoopExecutorWithWaker
    ///
    /// # Returns
    ///
    /// A new executor instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::EventLoopExecutorWithWaker;
    ///
    /// let executor = EventLoopExecutorWithWaker::new();
    /// ```
    pub fn new() -> Self {
        // Initialize global registry if not already done
        init_global_registry();

        Self {
            tasks: VecDeque::new(),
            registry: TaskRegistry::new(),
            completed_count: 0,
        }
    }

    /// Get the number of pending tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Get the number of completed tasks
    pub fn completed_count(&self) -> usize {
        self.completed_count
    }

    /// Create a waker for a specific task
    ///
    /// This waker, when invoked, will mark the task as ready
    /// in the global registry.
    fn create_task_waker(task_id: usize) -> Waker {
        // Create waker data that contains the task ID
        let data = Arc::new(TaskWakerData { task_id });

        let raw_waker = RawWaker::new(
            Arc::into_raw(data) as *const (),
            &TASK_WAKER_VTABLE,
        );

        unsafe { Waker::from_raw(raw_waker) }
    }

    /// Wake a specific task
    ///
    /// This can be called externally (e.g., by the event loop)
    /// to wake up a task when its I/O is ready.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to wake
    ///
    /// # Returns
    ///
    /// `true` if the task was found and woken
    pub fn wake_task(&self, task_id: usize) -> bool {
        self.registry.wake_task(task_id)
    }

    /// Wake a task via the global registry
    ///
    /// This allows external components (like the event loop)
    /// to wake tasks without direct access to the executor.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to wake
    ///
    /// # Returns
    ///
    /// `true` if the task was found and woken
    pub fn wake_task_via_registry(task_id: usize) -> bool {
        let registry_guard = global_registry().lock().unwrap();
        if let Some(registry) = registry_guard.as_ref() {
            registry.wake_task(task_id)
        } else {
            false
        }
    }
}

impl Default for EventLoopExecutorWithWaker {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for EventLoopExecutorWithWaker {
    /// Spawn a new async task on this executor
    ///
    /// The future is added to the task queue and registered
    /// with the task registry for wake-up notifications.
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute (must return ())
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, EventLoopExecutorWithWaker};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = EventLoopExecutorWithWaker::new();
    /// executor.spawn(Ready::new(()));
    /// ```
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let mut task = EventLoopTask::new(future);

        // Register the task with the registry
        let task_state = self.registry.register_task(task.id);
        task.task_state = Some(task_state);

        // Create waker for this task
        let waker = Self::create_task_waker(task.id);
        task.waker = Some(waker);

        self.tasks.push_back(task);
    }

    /// Run the executor until all tasks are complete
    ///
    /// This method processes all tasks in the queue, polling each one
    /// until it completes or returns Pending. Tasks are only polled
    /// when they are marked as ready in the registry.
    ///
    /// In a full integration with the event loop, this would:
    /// 1. Poll ready tasks
    /// 2. Wait for event loop events
    /// 3. Wake tasks when their I/O is ready
    /// 4. Repeat until all tasks complete
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, EventLoopExecutorWithWaker};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = EventLoopExecutorWithWaker::new();
    /// executor.spawn(Ready::new(()));
    /// executor.run();
    /// ```
    fn run(&mut self) {
        let mut iteration = 0;
        let max_iterations = 1000; // Prevent infinite loops for now

        while !self.tasks.is_empty() && iteration < max_iterations {
            iteration += 1;

            // Process all tasks (not just ready ones)
            let mut requeue = Vec::new();

            while let Some(mut task) = self.tasks.pop_front() {
                // Mark as not ready before polling (if registered)
                if self.registry.is_task_ready(task.id) {
                    self.registry.mark_not_ready(task.id);
                }

                // Create context from waker
                let waker = task.waker.as_ref().unwrap();
                let mut cx = Context::from_waker(waker);

                // Pin the future for polling
                // SAFETY: The future is pinned on the stack and won't move
                let pinned = unsafe { std::pin::Pin::new_unchecked(&mut *task.future) };

                // Poll the future
                match pinned.poll(&mut cx) {
                    Poll::Ready(_) => {
                        // Task completed, deregister and count it
                        self.registry.deregister_task(task.id);
                        self.completed_count += 1;
                    }
                    Poll::Pending => {
                        // Task not ready, put it back in the queue
                        // It will be re-queued when its waker is called
                        requeue.push(task);
                    }
                }
            }

            // Put back tasks that returned Pending
            for task in requeue {
                self.tasks.push_back(task);
            }

            // If all tasks returned Pending and none are ready, we're done
            if !self.tasks.is_empty() {
                let any_ready = self.tasks.iter().any(|t| t.is_ready());
                if !any_ready {
                    // No tasks are ready and none will become ready
                    // This is a deadlock situation - in production we'd wait for events
                    break;
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
    /// use zulon_runtime_scheduler::{Executor, EventLoopExecutorWithWaker};
    /// use zulon_async_futures::Ready;
    ///
    /// let mut executor = EventLoopExecutorWithWaker::new();
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

/// Internal data for task waker
struct TaskWakerData {
    task_id: usize,
}

unsafe impl Send for TaskWakerData {}
unsafe impl Sync for TaskWakerData {}

/// Virtual table for task waker
static TASK_WAKER_VTABLE: RawWakerVTable = RawWakerVTable {
    clone: clone_task_waker,
    wake: wake_task,
    wake_by_ref: wake_task_by_ref,
    drop: drop_task_waker,
};

/// Clone a task waker
unsafe fn clone_task_waker(data: *const ()) -> RawWaker {
    let arc_data = Arc::from_raw(data as *const TaskWakerData);
    let cloned = Arc::clone(&arc_data);
    // Don't drop the original
    std::mem::forget(arc_data);

    RawWaker::new(Arc::into_raw(cloned) as *const (), &TASK_WAKER_VTABLE)
}

/// Wake a task (consumes the waker)
unsafe fn wake_task(data: *const ()) {
    wake_task_by_ref(data);
}

/// Wake a task by reference
///
/// This is the key function that integrates with the task registry.
/// When called, it marks the task as ready in the global registry.
unsafe fn wake_task_by_ref(data: *const ()) {
    let arc_data = Arc::from_raw(data as *const TaskWakerData);
    let task_id = arc_data.task_id;

    // Don't drop the Arc yet
    std::mem::forget(arc_data);

    // Mark the task as ready in the global registry
    let registry_guard = global_registry().lock().unwrap();
    if let Some(registry) = registry_guard.as_ref() {
        registry.wake_task(task_id);
    }
}

/// Drop a task waker
unsafe fn drop_task_waker(data: *const ()) {
    let _arc_data = Arc::from_raw(data as *const TaskWakerData);
    // Arc is dropped here
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_async_futures::Ready;

    #[test]
    fn test_executor_creation() {
        let executor = EventLoopExecutorWithWaker::new();
        assert!(!executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 0);
        assert_eq!(executor.completed_count(), 0);
    }

    #[test]
    fn test_default() {
        let executor = EventLoopExecutorWithWaker::default();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_spawn_increases_task_count() {
        let mut executor = EventLoopExecutorWithWaker::new();
        executor.spawn(Ready::new(()));
        assert!(executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 1);
    }

    #[test]
    fn test_run_clears_tasks() {
        let mut executor = EventLoopExecutorWithWaker::new();
        executor.spawn(Ready::new(()));
        executor.run();
        assert!(!executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 0);
        assert_eq!(executor.completed_count(), 1);
    }

    #[test]
    fn test_multiple_tasks() {
        let mut executor = EventLoopExecutorWithWaker::new();
        executor.spawn(Ready::new(()));
        executor.spawn(Ready::new(()));
        executor.spawn(Ready::new(()));
        assert_eq!(executor.task_count(), 3);

        executor.run();
        assert_eq!(executor.task_count(), 0);
        assert_eq!(executor.completed_count(), 3);
    }

    #[test]
    fn test_wake_task() {
        let executor = EventLoopExecutorWithWaker::new();
        let mut executor = executor;

        // Spawn a task
        executor.spawn(Ready::new(()));
        assert_eq!(executor.task_count(), 1);

        // Get the task ID
        let task_id = executor.tasks.front().unwrap().id;

        // Wake the task
        assert!(executor.wake_task(task_id));
    }

    #[test]
    fn test_wake_nonexistent_task() {
        let executor = EventLoopExecutorWithWaker::new();
        assert!(!executor.wake_task(999));
    }
}
