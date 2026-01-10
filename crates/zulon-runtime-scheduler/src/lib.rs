// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Task Scheduler
//!
//! This crate provides task scheduling and execution for async futures in ZULON:
//!
//! - **[`Executor`]** - Core trait for executing async tasks
//! - **[`BasicExecutor`]** - Simple thread-per-task executor
//! - **[`Task`]** - Wrapper around futures with state management
//! - **[`LocalExecutor`]** - Thread-local executor for single-threaded execution
//!
//! ## Example
//!
//! ```rust
//! use zulon_runtime_scheduler::{Executor, BasicExecutor};
//! use zulon_async_futures::{Future, Poll};
//!
//! // Create an executor
//! let mut executor = BasicExecutor::new();
//! use zulon_async_futures::Ready;
//!
//! // Spawn an async task
//! executor.spawn(Ready::new(()));
//!
//! // Run the executor
//! executor.run();
//! ```

pub mod executor;
pub mod task;
pub mod basic_executor;
pub mod local_executor;
pub mod event_loop_executor;
pub mod event_loop_waker;
pub mod waker_registry;
pub mod executor_with_waker;
pub mod event_loop_integration;

pub use executor::{Executor, ExecutorExt};
pub use task::Task;
pub use basic_executor::BasicExecutor;
pub use local_executor::LocalExecutor;
pub use event_loop_executor::EventLoopExecutor;
pub use event_loop_waker::EventLoopExecutor as EventLoopWakerExecutor;
pub use executor_with_waker::EventLoopExecutorWithWaker;
pub use event_loop_integration::{IntegratedEventLoopExecutor, EventInterest};
