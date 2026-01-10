// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for executor implementations
//!
//! These tests verify that executors properly poll futures
//! and handle the async execution model.

use zulon_runtime_scheduler::{Executor, BasicExecutor, LocalExecutor, EventLoopExecutor};
use zulon_async_futures::{Future, Poll, Context, Ready};

/// A future that returns Pending once, then Ready
struct YieldOnce {
    yielded: bool,
}

impl YieldOnce {
    fn new() -> Self {
        Self { yielded: false }
    }
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        if !self.yielded {
            self.yielded = true;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

#[test]
fn test_basic_executor_polls_futures() {
    let mut executor = BasicExecutor::new();

    // Spawn a simple Ready future
    executor.spawn(Ready::new(()));

    // Run should complete immediately
    executor.run();
    assert!(!executor.has_pending_tasks());
}

#[test]
fn test_local_executor_polls_futures() {
    let mut executor = LocalExecutor::new();

    // Spawn a simple Ready future
    executor.spawn(Ready::new(()));

    // Run should complete immediately
    executor.run();
    assert!(!executor.has_pending_tasks());
}

#[test]
fn test_event_loop_executor_polls_futures() {
    let mut executor = EventLoopExecutor::new();

    // Spawn a simple Ready future
    executor.spawn(Ready::new(()));

    // Run should complete immediately
    executor.run();
    assert!(!executor.has_pending_tasks());
}

#[test]
fn test_local_executor_multiple_tasks() {
    let mut executor = LocalExecutor::new();

    // Spawn multiple tasks
    for _ in 0..5 {
        executor.spawn(Ready::new(()));
    }

    assert!(executor.has_pending_tasks());

    // Run should complete all tasks
    executor.run();
    assert!(!executor.has_pending_tasks());
}

#[test]
fn test_basic_executor_multiple_tasks() {
    let mut executor = BasicExecutor::new();

    // Spawn multiple tasks using spawn_send to actually spawn threads
    for _ in 0..3 {
        executor.spawn_send(Ready::new(()));
    }

    assert!(executor.has_pending_tasks());

    // Run should complete all tasks
    executor.run();
    assert!(!executor.has_pending_tasks());
}

#[test]
fn test_local_executor_with_yield_once() {
    let mut executor = LocalExecutor::new();

    // Spawn a future that yields once before completing
    executor.spawn(YieldOnce::new());

    // Run should complete after the future yields and then becomes ready
    executor.run();
    assert!(!executor.has_pending_tasks());
}

#[test]
fn test_event_loop_executor_with_yield_once() {
    let mut executor = EventLoopExecutor::new();

    // Spawn a future that yields once before completing
    executor.spawn(YieldOnce::new());

    // Run should complete after the future yields and then becomes ready
    executor.run();
    assert!(!executor.has_pending_tasks());
}

/// A future that returns a value
struct ValueFuture {
    value: i32,
}

impl ValueFuture {
    fn new(value: i32) -> Self {
        Self { value }
    }
}

impl Future for ValueFuture {
    type Output = i32;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.value)
    }
}

#[test]
fn test_task_wrapper_with_value() {
    use zulon_runtime_scheduler::Task;

    let mut task = Task::new(ValueFuture::new(42));
    assert!(!task.is_completed());

    // Poll the task
    let waker = zulon_async_futures::Waker::noop();
    let mut cx = Context::from_waker(&waker);

    match task.poll(&mut cx) {
        Poll::Ready(value) => assert_eq!(value, 42),
        Poll::Pending => panic!("Task should be ready"),
    }

    assert!(task.is_completed());
}
