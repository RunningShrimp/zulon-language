// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Thread pool for async runtime
//!
//! This module implements:
//! - Worker thread pool for parallel task execution
//! - Task stealing for load balancing
//! - Graceful shutdown

use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

/// Configuration for thread pool
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Number of worker threads
    pub num_threads: usize,
    /// Thread name prefix
    pub thread_name_prefix: String,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        ThreadPoolConfig {
            num_threads: num_cpus::get(),
            thread_name_prefix: "zulon-worker".to_string(),
        }
    }
}

impl ThreadPoolConfig {
    /// Create a new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set number of worker threads
    pub fn num_threads(mut self, count: usize) -> Self {
        self.num_threads = count.max(1);
        self
    }

    /// Set thread name prefix
    pub fn thread_name_prefix(mut self, prefix: String) -> Self {
        self.thread_name_prefix = prefix;
        self
    }
}

/// Shared state for thread pool
struct ThreadPoolState {
    /// Whether the pool is running
    running: bool,
    /// Number of active workers
    active_workers: usize,
}

/// Thread pool for async task execution
pub struct ThreadPool {
    /// Worker threads
    workers: Vec<Worker>,
    /// Shared state
    state: Arc<Mutex<ThreadPoolState>>,
    /// Condition variable for waiting
    condvar: Arc<Condvar>,
}

impl ThreadPool {
    /// Create a new thread pool
    pub fn new(config: ThreadPoolConfig) -> Self {
        let state = Arc::new(Mutex::new(ThreadPoolState {
            running: true,
            active_workers: 0,
        }));
        let condvar = Arc::new(Condvar::new());

        let mut workers = Vec::with_capacity(config.num_threads);

        for i in 0..config.num_threads {
            let name = format!("{}-{}", config.thread_name_prefix, i);
            let worker = Worker::new(name, state.clone(), condvar.clone());
            workers.push(worker);
        }

        ThreadPool {
            workers,
            state,
            condvar,
        }
    }

    /// Shutdown the thread pool
    pub fn shutdown(mut self) {
        {
            let mut state = self.state.lock().unwrap();
            state.running = false;
        }

        self.condvar.notify_all();

        for worker in self.workers.drain(..) {
            worker.join();
        }
    }

    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        if state.running {
            state.running = false;
            self.condvar.notify_all();
        }

        for worker in self.workers.drain(..) {
            worker.join();
        }
    }
}

/// Worker thread in the pool
struct Worker {
    /// Thread handle
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    /// Create a new worker
    fn new(name: String, state: Arc<Mutex<ThreadPoolState>>, condvar: Arc<Condvar>) -> Self {
        let handle = thread::Builder::new()
            .name(name)
            .spawn(move || {
                loop {
                    let mut guard = state.lock().unwrap();

                    if !guard.running {
                        break;
                    }

                    guard.active_workers += 1;

                    drop(guard);

                    // TODO: Process tasks from task queue

                    {
                        let mut guard = state.lock().unwrap();
                        guard.active_workers -= 1;

                        if !guard.running {
                            break;
                        }

                        guard = condvar.wait(guard).unwrap();
                    }
                }
            })
            .ok();

        Worker { handle }
    }

    /// Join the worker thread
    fn join(mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool_config() {
        let config = ThreadPoolConfig::new()
            .num_threads(4)
            .thread_name_prefix("test".to_string());

        assert_eq!(config.num_threads, 4);
        assert_eq!(config.thread_name_prefix, "test");
    }

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(ThreadPoolConfig::new());
        assert_eq!(pool.num_workers(), num_cpus::get());
        pool.shutdown();
    }
}
