// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! End-to-end integration tests for Phase 2.3 Event Loop Integration
//!
//! These tests validate that IntegratedEventLoopExecutor correctly
//! integrates with Phase 2.2's EventLoop implementations (epoll/kqueue).

use zulon_runtime_scheduler::{Executor, IntegratedEventLoopExecutor};
use zulon_async_futures::{Future, Poll, Context};
use std::os::unix::io::RawFd;

/// A future that simulates async I/O by becoming Pending once, then Ready
///
/// This represents the common pattern where an async operation
/// (like reading from a socket) isn't ready immediately and needs
/// to be woken up by the event loop when data arrives.
struct SimulatedAsyncIO {
    polled_once: bool,
}

impl SimulatedAsyncIO {
    fn new() -> Self {
        Self {
            polled_once: false,
        }
    }
}

impl Future for SimulatedAsyncIO {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        if !self.polled_once {
            self.polled_once = true;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// A future that completes immediately
struct ImmediateCompletion;

impl Future for ImmediateCompletion {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}

/// A future that never completes (used to test event loop behavior)
#[allow(dead_code)]
struct NeverCompletes {
    _private: (),
}

#[allow(dead_code)]
impl NeverCompletes {
    fn new() -> Self {
        Self { _private: () }
    }
}

impl Future for NeverCompletes {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Pending
    }
}

#[test]
fn test_integrated_executor_basic_spawn() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn a task that completes immediately
    executor.spawn(ImmediateCompletion);

    assert_eq!(executor.task_count(), 1);

    // Run should complete the task
    executor.run();

    assert_eq!(executor.task_count(), 0, "Task should be completed and removed");
}

#[test]
fn test_integrated_executor_multiple_tasks() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn multiple tasks
    for _ in 0..5 {
        executor.spawn(ImmediateCompletion);
    }

    assert_eq!(executor.task_count(), 5);

    // Run should complete all tasks
    executor.run();

    assert_eq!(executor.task_count(), 0, "All tasks should be completed");
}

#[test]
fn test_integrated_executor_with_pending() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn a task that will pend once
    executor.spawn(SimulatedAsyncIO::new());

    assert_eq!(executor.task_count(), 1);

    // First run - task will pend
    executor.run();

    // Task should still be pending (not completed yet)
    assert_eq!(executor.task_count(), 1, "Task should still be pending after first poll");

    // Note: In a real scenario with event loop integration, the task would
    // be woken up by the event loop when I/O is ready. For this test, we're
    // just verifying that the executor properly handles pending tasks.
}

#[test]
fn test_integrated_executor_get_fds_to_register() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn a task
    executor.spawn(ImmediateCompletion);

    // Get FDs to register
    let fds = executor.get_fds_to_register();

    // Note: Currently, tasks don't automatically register their FDs.
    // In production, tasks would call set_io() during polling to register
    // their FDs. For now, this just tests the interface exists.
    assert!(fds.is_empty(), "No FDs registered yet");
}

#[test]
fn test_integrated_executor_handle_readable() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn tasks
    executor.spawn(ImmediateCompletion);
    executor.spawn(ImmediateCompletion);

    // Create a dummy FD
    let fd = 100 as RawFd;

    // Manually register an FD (simulating what event loop would do)
    executor.register_fd(fd, 0);

    // Handle readable event
    let handled = executor.handle_readable(fd);

    // Should return true (found the task)
    assert!(handled, "Should handle readable event for registered FD");
}

#[test]
fn test_integrated_executor_handle_writable() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn a task
    executor.spawn(ImmediateCompletion);

    // Create a dummy FD
    let fd = 200 as RawFd;

    // Manually register an FD
    executor.register_fd(fd, 0);

    // Handle writable event
    let handled = executor.handle_writable(fd);

    // Should return true
    assert!(handled, "Should handle writable event for registered FD");
}

#[test]
fn test_integrated_executor_handle_unknown_fd() {
    let executor = IntegratedEventLoopExecutor::new();

    // Don't spawn any tasks or register any FDs

    // Try to handle event for unknown FD
    let handled = executor.handle_readable(999);

    // Should return false (no task found)
    assert!(!handled, "Should not handle event for unregistered FD");
}

#[test]
fn test_event_interest_conversions() {
    use zulon_runtime_scheduler::EventInterest;

    // Test EventInterest::none()
    let none = EventInterest::none();
    assert!(!none.readable);
    assert!(!none.writable);
    let phase2_none = none.to_phase2_interest();
    assert!(!phase2_none.is_readable());
    assert!(!phase2_none.is_writable());
    assert!(!phase2_none.is_error());

    // Test EventInterest::readable()
    let readable = EventInterest::readable();
    assert!(readable.readable);
    assert!(!readable.writable);
    let phase2_readable = readable.to_phase2_interest();
    assert!(phase2_readable.is_readable());
    assert!(!phase2_readable.is_writable());

    // Test EventInterest::writable()
    let writable = EventInterest::writable();
    assert!(!writable.readable);
    assert!(writable.writable);
    let phase2_writable = writable.to_phase2_interest();
    assert!(!phase2_writable.is_readable());
    assert!(phase2_writable.is_writable());

    // Test EventInterest::both()
    let both = EventInterest::both();
    assert!(both.readable);
    assert!(both.writable);
    let phase2_both = both.to_phase2_interest();
    assert!(phase2_both.is_readable());
    assert!(phase2_both.is_writable());
}

/// Integration test with actual Phase 2.2 EventLoop
///
/// Note: This test is platform-specific and requires the appropriate
/// event loop implementation (epoll on Linux, kqueue on macOS/BSD).
#[cfg(test)]
#[cfg(target_os = "linux")]
mod with_real_event_loop {
    use super::*;
    use zulon_runtime_io::event_loop::{EpollEventLoop, EventLoop, EventHandler};

    /// A simple test that validates executor can be integrated with real event loop
    #[test]
    fn test_executor_with_epoll_event_loop() {
        // Create executor
        let executor = IntegratedEventLoopExecutor::new();

        // Spawn a simple task
        executor.spawn(ImmediateCompletion);

        // Create epoll event loop
        let mut event_loop = match EpollEventLoop::new() {
            Ok(el) => el,
            Err(_) => {
                // Skip test if epoll cannot be created
                eprintln!("Skipping test: cannot create epoll event loop");
                return;
            }
        };

        // Register FDs (none in this simple case)
        let result = executor.register_fds_with_event_loop(&mut event_loop);
        assert!(result.is_ok(), "Should successfully register FDs");

        // Note: We don't run the event loop here because it would block
        // In production, you'd call executor.run_with_event_loop(&mut event_loop)
        // which would run both the executor and event loop together
    }

    /// Test EventHandler implementation for IntegratedEventLoopExecutor
    #[test]
    fn test_event_handler_implementation() {
        use zulon_runtime_io::event_loop::{Token, EventHandler};
        use zulon_runtime_io::IoError;

        let mut executor = IntegratedEventLoopExecutor::new();

        // Spawn tasks and register FDs
        executor.spawn(ImmediateCompletion);
        let fd = 42 as RawFd;
        executor.register_fd(fd, 0);

        // Create a dummy token
        let token = Token::new(0);

        // Test EventHandler::readable
        executor.readable(token);

        // Test EventHandler::writable
        executor.writable(token);

        // Test EventHandler::error
        let dummy_error = IoError::new(std::io::ErrorKind::Other, "test error");
        executor.error(token, dummy_error);

        // If we get here without panicking, the EventHandler implementation works
    }
}
/// Integration test with actual Phase 2.2 EventLoop
///
/// Note: This test is platform-specific and requires the appropriate
/// event loop implementation (epoll on Linux, kqueue on macOS/BSD).
#[cfg(test)]
#[cfg(target_os = "linux")]
mod with_real_event_loop {
    use super::*;
    use zulon_runtime_io::event_loop::{EpollEventLoop, EventLoop, EventHandler};

    /// A simple test that validates executor can be integrated with real event loop
    #[test]
    fn test_executor_with_epoll_event_loop() {
        // Create executor
        let executor = IntegratedEventLoopExecutor::new();

        // Spawn a simple task
        executor.spawn(ImmediateCompletion);

        // Create epoll event loop
        let mut event_loop = match EpollEventLoop::new() {
            Ok(el) => el,
            Err(_) => {
                // Skip test if epoll cannot be created
                eprintln!("Skipping test: cannot create epoll event loop");
                return;
            }
        };

        // Register FDs (none in this simple case)
        let result = executor.register_fds_with_event_loop(&mut event_loop);
        assert!(result.is_ok(), "Should successfully register FDs");

        // Note: We don't run the event loop here because it would block
        // In production, you'd call executor.run_with_event_loop(&mut event_loop)
        // which would run both the executor and event loop together
    }

    /// Test EventHandler implementation for IntegratedEventLoopExecutor
    #[test]
    fn test_event_handler_implementation() {
        use zulon_runtime_io::event_loop::{Token, EventHandler};
        use zulon_runtime_io::IoError;

        let mut executor = IntegratedEventLoopExecutor::new();

        // Spawn tasks and register FDs
        executor.spawn(ImmediateCompletion);
        let fd = 42 as RawFd;
        executor.register_fd(fd, 0);

        // Create a dummy token
        let token = Token::new(0);

        // Test EventHandler::readable
        executor.readable(token);

        // Test EventHandler::writable
        executor.writable(token);

        // Test EventHandler::error
        let dummy_error = IoError::new(std::io::ErrorKind::Other, "test error");
        executor.error(token, dummy_error);

        // If we get here without panicking, the EventHandler implementation works
    }
}

/// Performance test: measure executor overhead
#[test]
fn test_executor_performance() {
    let mut executor = IntegratedEventLoopExecutor::new();

    let start = std::time::Instant::now();

    // Spawn many tasks
    for _ in 0..1000 {
        executor.spawn(ImmediateCompletion);
    }

    let spawn_time = start.elapsed();

    // Run all tasks
    let run_start = std::time::Instant::now();
    executor.run();
    let run_time = run_start.elapsed();

    // Performance assertions
    assert!(spawn_time.as_millis() < 100, "Spawning 1000 tasks should take < 100ms");
    assert!(run_time.as_millis() < 100, "Running 1000 tasks should take < 100ms");

    println!("Spawn time: {:?}", spawn_time);
    println!("Run time: {:?}", run_time);
}

/// Stress test: many pending tasks
#[test]
fn test_many_pending_tasks() {
    let mut executor = IntegratedEventLoopExecutor::new();

    // Spawn many tasks that will pend
    for _ in 0..100 {
        executor.spawn(SimulatedAsyncIO::new());
    }

    assert_eq!(executor.task_count(), 100);

    // Run will poll them once (they'll all pend)
    executor.run();

    // All should still be pending
    assert_eq!(executor.task_count(), 100);
}
