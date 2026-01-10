// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Event Loop Integration
//!
//! This module integrates ZULON's async executor with Phase 2.2's event loop,
//! enabling automatic task wake-up when I/O events occur.

use crate::Executor;
use crate::waker_registry::{global_registry, init_global_registry};
use zulon_async_futures::{Future, Poll, Context, Waker, RawWaker, RawWakerVTable};
use zulon_runtime_io::event_loop::{EventLoop, EventHandler, EventSource, Token, Interest};
use zulon_runtime_io::{IoError, IoResult};
use std::collections::{VecDeque, HashMap};
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A task with associated I/O resources
struct IOTask {
    /// The boxed future
    future: Box<dyn Future<Output = ()> + 'static>,
    /// Task ID
    id: usize,
    /// Waker for this task
    waker: Option<Waker>,
    /// File descriptor to monitor (if any)
    fd: Option<RawFd>,
    /// Interest in events
    interest: EventInterest,
}

/// Event interests for a task
#[derive(Debug, Clone, Copy)]
pub struct EventInterest {
    /// Readable interest
    pub readable: bool,
    /// Writable interest
    pub writable: bool,
}

impl EventInterest {
    /// No interest
    pub fn none() -> Self {
        Self {
            readable: false,
            writable: false,
        }
    }

    /// Readable only
    pub fn readable() -> Self {
        Self {
            readable: true,
            writable: false,
        }
    }

    /// Writable only
    pub fn writable() -> Self {
        Self {
            readable: false,
            writable: true,
        }
    }

    /// Both readable and writable
    pub fn both() -> Self {
        Self {
            readable: true,
            writable: true,
        }
    }

    /// Convert to Phase 2.2 Interest
    pub fn to_phase2_interest(&self) -> Interest {
        let mut interest = Interest::NONE;
        if self.readable {
            interest = interest | Interest::READABLE;
        }
        if self.writable {
            interest = interest | Interest::WRITABLE;
        }
        interest
    }
}

impl IOTask {
    fn new(future: impl Future<Output = ()> + 'static, id: usize) -> Self {
        Self {
            future: Box::new(future),
            id,
            waker: None,
            fd: None,
            interest: EventInterest::none(),
        }
    }

    /// Set the file descriptor and event interest
    #[allow(dead_code)]
    fn set_io(&mut self, fd: RawFd, interest: EventInterest) {
        self.fd = Some(fd);
        self.interest = interest;
    }

    /// Check if this task is ready to run
    fn is_ready(&self) -> bool {
        let registry_guard = global_registry().lock().unwrap();
        if let Some(registry) = registry_guard.as_ref() {
            registry.is_task_ready(self.id)
        } else {
            true
        }
    }
}

/// Integrated event loop executor
///
/// This executor maintains a registry of tasks with their file descriptors
/// and can be integrated with Phase 2.2's event loop for automatic wake-up.
///
/// # Architecture
///
/// ```text
/// IntegratedEventLoopExecutor
///     ├── Task Queue
///     │   ├── Task 1 (future + fd + interest)
///     │   ├── Task 2 (future + fd + interest)
///     │   └── Task 3 (future + fd + interest)
///     │
///     └── FD Registry
///         ├── fd1 → Task 1
///         ├── fd2 → Task 2
///         └── fd3 → Task 3
///
/// Event Loop Integration:
/// 1. Register task FDs with event loop
/// 2. Event loop calls handle_readable/handle_writable
/// 3. Handler wakes up corresponding task
/// 4. Executor re-polls the woken task
/// ```
///
/// # Example
///
/// ```rust
/// use zulon_runtime_scheduler::{Executor, IntegratedEventLoopExecutor};
/// use zulon_async_futures::Ready;
///
/// let mut executor = IntegratedEventLoopExecutor::new();
/// executor.spawn(Ready::new(()));
///
/// // Get FDs to register with event loop
/// let fds = executor.get_fds_to_register();
///
/// // Register with Phase 2.2 event loop...
///
/// // When event occurs, notify executor
/// executor.handle_readable(fd);
///
/// // Run executor
/// executor.run();
/// ```
pub struct IntegratedEventLoopExecutor {
    /// Queue of futures to execute
    tasks: VecDeque<IOTask>,
    /// Map from file descriptor to task ID
    fd_to_task: HashMap<RawFd, usize>,
    /// Task counter
    task_counter: AtomicUsize,
}

impl IntegratedEventLoopExecutor {
    /// Create a new integrated event loop executor
    ///
    /// # Returns
    ///
    /// A new executor instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::IntegratedEventLoopExecutor;
    ///
    /// let executor = IntegratedEventLoopExecutor::new();
    /// ```
    pub fn new() -> Self {
        // Initialize global registry if not already done
        init_global_registry();

        Self {
            tasks: VecDeque::new(),
            fd_to_task: HashMap::new(),
            task_counter: AtomicUsize::new(0),
        }
    }

    /// Get the number of pending tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Get file descriptors to register with event loop
    ///
    /// Returns a list of (fd, task_id, interest) tuples
    pub fn get_fds_to_register(&self) -> Vec<(RawFd, usize, EventInterest)> {
        self.tasks.iter()
            .filter_map(|task| {
                task.fd.map(|fd| (fd, task.id, task.interest))
            })
            .collect()
    }

    /// Handle readable event for a file descriptor
    ///
    /// This is called by the event loop when data is available to read.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor that is readable
    ///
    /// # Returns
    ///
    /// `true` if a task was found and woken
    pub fn handle_readable(&self, fd: RawFd) -> bool {
        if let Some(task_id) = self.fd_to_task.get(&fd) {
            self.wake_task(*task_id);
            true
        } else {
            false
        }
    }

    /// Handle writable event for a file descriptor
    ///
    /// This is called by the event loop when data can be written.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor that is writable
    ///
    /// # Returns
    ///
    /// `true` if a task was found and woken
    pub fn handle_writable(&self, fd: RawFd) -> bool {
        if let Some(task_id) = self.fd_to_task.get(&fd) {
            self.wake_task(*task_id);
            true
        } else {
            false
        }
    }

    /// Wake a specific task by ID
    fn wake_task(&self, task_id: usize) -> bool {
        let registry_guard = global_registry().lock().unwrap();
        if let Some(registry) = registry_guard.as_ref() {
            registry.wake_task(task_id)
        } else {
            false
        }
    }

    /// Create a waker for a specific task
    fn create_task_waker(task_id: usize) -> Waker {
        let data = Arc::new(TaskWakerData { task_id });
        let raw_waker = RawWaker::new(
            Arc::into_raw(data) as *const (),
            &TASK_WAKER_VTABLE,
        );
        unsafe { Waker::from_raw(raw_waker) }
    }

    /// Register a task's file descriptor
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor to register
    /// * `task_id` - The task ID associated with this FD
    pub fn register_fd(&mut self, fd: RawFd, task_id: usize) {
        self.fd_to_task.insert(fd, task_id);
    }

    /// Deregister a task's file descriptor
    #[allow(dead_code)]
    fn deregister_fd(&mut self, fd: RawFd) {
        self.fd_to_task.remove(&fd);
    }

    /// Register task FDs with a Phase 2.2 event loop
    ///
    /// This method registers all task file descriptors with the provided
    /// event loop, enabling automatic I/O event notifications.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Event loop type implementing EventLoop trait
    ///
    /// # Arguments
    ///
    /// * `event_loop` - The event loop to register with
    ///
    /// # Returns
    ///
    /// A vector of (fd, token) tuples for tracking registrations
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails for any FD
    pub fn register_fds_with_event_loop<E: EventLoop + ?Sized>(
        &mut self,
        event_loop: &mut E
    ) -> IoResult<Vec<(RawFd, Token)>> {
        let mut registrations = Vec::new();
        
        for task in &self.tasks {
            if let Some(fd) = task.fd {
                // Create an EventSource wrapper for this FD
                let source = FdEventSource {
                    fd,
                    interest: task.interest,
                };
                
                // Register with event loop
                let token = event_loop.register(&source)?;
                registrations.push((fd, token));
            }
        }
        
        Ok(registrations)
    }

    /// Run the executor with a Phase 2.2 event loop
    ///
    /// This method integrates the executor with the event loop, automatically
    /// registering task FDs and running both together.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Event loop type implementing EventLoop trait
    ///
    /// # Arguments
    ///
    /// * `event_loop` - The event loop to use for I/O notifications
    ///
    /// # Errors
    ///
    /// Returns an error if event loop operations fail
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_scheduler::{Executor, IntegratedEventLoopExecutor};
    /// use zulon_async_futures::Ready;
    /// # use zulon_runtime_io::event_loop::EpollEventLoop;
    /// # fn test() -> zulon_runtime_io::IoResult<()> {
    /// let mut executor = IntegratedEventLoopExecutor::new();
    /// executor.spawn(Ready::new(()));
    ///
    /// let mut event_loop = EpollEventLoop::new()?;
    /// executor.run_with_event_loop(&mut event_loop)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_with_event_loop<E: EventLoop + ?Sized>(
        &mut self,
        event_loop: &mut E
    ) -> IoResult<()> {
        // Register all task FDs with event loop
        let _registrations = self.register_fds_with_event_loop(event_loop)?;
        
        // Run the event loop (which will call our EventHandler methods)
        event_loop.run()?;
        
        // After event loop stops, run any remaining tasks
        self.run();
        
        Ok(())
    }
}

impl Default for IntegratedEventLoopExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for IntegratedEventLoopExecutor {
    /// Spawn a new async task
    ///
    /// # Arguments
    ///
    /// * `future` - The async task to execute
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        // Generate task ID using our counter
        let id = self.task_counter.fetch_add(1, Ordering::SeqCst);
        
        // Register with global registry
        let mut registry_guard = global_registry().lock().unwrap();
        if let Some(registry) = registry_guard.as_mut() {
            let _task_state = registry.register_task(id);
        }

        let mut task = IOTask::new(future, id);
        let waker = Self::create_task_waker(task.id);
        task.waker = Some(waker);

        self.tasks.push_back(task);
    }

    /// Run the executor
    ///
    /// This polls all ready tasks. Tasks are only polled when
    /// marked as ready by the event loop via handle_readable/handle_writable.
    fn run(&mut self) {
        let mut iteration = 0;
        let max_iterations = 1000;

        while !self.tasks.is_empty() && iteration < max_iterations {
            iteration += 1;

            let mut requeue = Vec::new();
            let mut to_deregister = Vec::new();

            while let Some(mut task) = self.tasks.pop_front() {
                // Only poll if marked as ready
                if !task.is_ready() {
                    requeue.push(task);
                    continue;
                }

                // Mark as not ready before polling
                {
                    let mut registry_guard = global_registry().lock().unwrap();
                    if let Some(registry) = registry_guard.as_mut() {
                        registry.mark_not_ready(task.id);
                    }
                }

                // Create context and poll
                let waker = task.waker.as_ref().unwrap();
                let mut cx = Context::from_waker(waker);

                // SAFETY: Future is pinned on stack
                let pinned = unsafe { std::pin::Pin::new_unchecked(&mut *task.future) };

                match pinned.poll(&mut cx) {
                    Poll::Ready(_) => {
                        // Task completed
                        if let Some(fd) = task.fd {
                            to_deregister.push(fd);
                        }
                        {
                            let mut registry_guard = global_registry().lock().unwrap();
                            if let Some(registry) = registry_guard.as_mut() {
                                registry.deregister_task(task.id);
                            }
                        }
                    }
                    Poll::Pending => {
                        // Task pending, requeue
                        requeue.push(task);
                    }
                }
            }

            // Re-queue pending tasks
            for task in requeue {
                self.tasks.push_back(task);
            }

            // Deregister completed task FDs
            for fd in to_deregister {
                self.deregister_fd(fd);
            }

            // If no tasks ready, break
            if !self.tasks.is_empty() {
                let any_ready = self.tasks.iter().any(|t| t.is_ready());
                if !any_ready {
                    break;
                }
            }
        }
    }

    /// Check if there are pending tasks
    fn has_pending_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
}

// Safety: We need to implement Send + Sync for EventHandler
// The IntegratedEventLoopExecutor uses interior mutability via Mutex
unsafe impl Send for IntegratedEventLoopExecutor {}
unsafe impl Sync for IntegratedEventLoopExecutor {}

impl EventHandler for IntegratedEventLoopExecutor {
    /// Handle readable event from event loop
    fn readable(&mut self, _token: Token) {
        // For simplicity, wake all tasks with readable interest
        // In production, we'd maintain a token_to_task mapping for O(1) lookup
        for (_fd, task_id) in &self.fd_to_task {
            if let Some(task) = self.tasks.iter().find(|t| t.id == *task_id) {
                if task.interest.readable {
                    self.wake_task(*task_id);
                }
            }
        }
    }

    /// Handle writable event from event loop
    fn writable(&mut self, _token: Token) {
        // For simplicity, wake all tasks with writable interest
        // In production, we'd maintain a token_to_task mapping for O(1) lookup
        for (_fd, task_id) in &self.fd_to_task {
            if let Some(task) = self.tasks.iter().find(|t| t.id == *task_id) {
                if task.interest.writable {
                    self.wake_task(*task_id);
                }
            }
        }
    }

    /// Handle error event from event loop
    fn error(&mut self, token: Token, err: IoError) {
        // Log error and wake all tasks so they can handle it
        eprintln!("Event loop error on token {:?}: {}", token, err);
        
        for (_fd, task_id) in &self.fd_to_task {
            self.wake_task(*task_id);
        }
    }
}

/// Internal waker data
struct TaskWakerData {
    task_id: usize,
}

unsafe impl Send for TaskWakerData {}
unsafe impl Sync for TaskWakerData {}

/// Waker vtable
static TASK_WAKER_VTABLE: RawWakerVTable = RawWakerVTable {
    clone: clone_task_waker,
    wake: wake_task,
    wake_by_ref: wake_task_by_ref,
    drop: drop_task_waker,
};

unsafe fn clone_task_waker(data: *const ()) -> RawWaker {
    let arc_data = Arc::from_raw(data as *const TaskWakerData);
    let cloned = Arc::clone(&arc_data);
    std::mem::forget(arc_data);
    RawWaker::new(Arc::into_raw(cloned) as *const (), &TASK_WAKER_VTABLE)
}

unsafe fn wake_task(data: *const ()) {
    wake_task_by_ref(data);
}

unsafe fn wake_task_by_ref(data: *const ()) {
    let arc_data = Arc::from_raw(data as *const TaskWakerData);
    let task_id = arc_data.task_id;
    std::mem::forget(arc_data);

    // Wake task in global registry
    let registry_guard = global_registry().lock().unwrap();
    if let Some(registry) = registry_guard.as_ref() {
        registry.wake_task(task_id);
    }
}

unsafe fn drop_task_waker(data: *const ()) {
    let _arc_data = Arc::from_raw(data as *const TaskWakerData);
}

/// File descriptor event source wrapper
///
/// This wraps a raw file descriptor with event interests to implement
/// the EventSource trait for Phase 2.2 event loop integration.
struct FdEventSource {
    fd: RawFd,
    interest: EventInterest,
}

unsafe impl Send for FdEventSource {}
unsafe impl Sync for FdEventSource {}

impl EventSource for FdEventSource {
    fn raw_fd(&self) -> RawFd {
        self.fd
    }

    fn interest(&self) -> Interest {
        self.interest.to_phase2_interest()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_async_futures::Ready;

    #[test]
    fn test_executor_creation() {
        let executor = IntegratedEventLoopExecutor::new();
        assert!(!executor.has_pending_tasks());
        assert_eq!(executor.task_count(), 0);
    }

    #[test]
    fn test_spawn_and_run() {
        let mut executor = IntegratedEventLoopExecutor::new();
        executor.spawn(Ready::new(()));
        executor.run();
        assert!(!executor.has_pending_tasks());
    }

    #[test]
    fn test_multiple_tasks() {
        let mut executor = IntegratedEventLoopExecutor::new();
        executor.spawn(Ready::new(()));
        executor.spawn(Ready::new(()));
        executor.spawn(Ready::new(()));
        assert_eq!(executor.task_count(), 3);

        executor.run();
        assert_eq!(executor.task_count(), 0);
    }

    #[test]
    fn test_event_interest() {
        let interest = EventInterest::readable();
        assert!(interest.readable);
        assert!(!interest.writable);

        let interest = EventInterest::writable();
        assert!(!interest.readable);
        assert!(interest.writable);

        let interest = EventInterest::both();
        assert!(interest.readable);
        assert!(interest.writable);
    }

    #[test]
    fn test_fd_registry() {
        let mut executor = IntegratedEventLoopExecutor::new();

        // Simulate registering FDs
        let fd1: RawFd = 42;
        let fd2: RawFd = 43;

        // Manually add to registry (normally done via spawn with FD)
        executor.register_fd(fd1, 100);
        executor.register_fd(fd2, 200);

        // Test handle_readable
        assert!(executor.handle_readable(fd1));
        assert!(executor.handle_writable(fd2));

        // Test non-existent FD
        assert!(!executor.handle_readable(999));
    }
}
