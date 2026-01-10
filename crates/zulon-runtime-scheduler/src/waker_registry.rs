// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Task Wake-up Mechanism
//!
//! This module provides the wake-up mechanism that allows tasks to be
//! marked as ready when I/O events occur.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global task registry for wake-up notifications
///
/// This registry allows wakers to communicate with the executor
/// by marking tasks as ready when I/O events occur.
///
/// # Thread Safety
///
/// The registry uses Arc<Mutex<>> for thread-safe access.
pub struct TaskRegistry {
    /// Map from task ID to task state
    tasks: HashMap<usize, Arc<Mutex<TaskState>>>,
}

/// State of a task in the registry
pub struct TaskState {
    /// Whether the task is ready to run
    pub ready: bool,
    /// Whether the task is still active
    pub active: bool,
}

impl TaskRegistry {
    /// Create a new task registry
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    /// Register a task with the registry
    ///
    /// # Arguments
    ///
    /// * `task_id` - Unique task identifier
    ///
    /// # Returns
    ///
    /// A handle to the registered task state
    pub fn register_task(&mut self, task_id: usize) -> Arc<Mutex<TaskState>> {
        let state = Arc::new(Mutex::new(TaskState {
            ready: true,  // Initially ready
            active: true,
        }));

        self.tasks.insert(task_id, state.clone());
        state
    }

    /// Wake a task by ID
    ///
    /// This marks the task as ready to be polled.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to wake
    ///
    /// # Returns
    ///
    /// `true` if the task was found and woken, `false` otherwise
    pub fn wake_task(&self, task_id: usize) -> bool {
        if let Some(state) = self.tasks.get(&task_id) {
            let mut state_guard = state.lock().unwrap();
            if state_guard.active {
                state_guard.ready = true;
                return true;
            }
        }
        false
    }

    /// Check if a task is ready
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to check
    ///
    /// # Returns
    ///
    /// `true` if the task is ready to be polled
    pub fn is_task_ready(&self, task_id: usize) -> bool {
        if let Some(state) = self.tasks.get(&task_id) {
            let state_guard = state.lock().unwrap();
            state_guard.active && state_guard.ready
        } else {
            false
        }
    }

    /// Mark a task as not ready
    ///
    /// This is called before polling the task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to mark as not ready
    pub fn mark_not_ready(&mut self, task_id: usize) {
        if let Some(state) = self.tasks.get(&task_id) {
            let mut state_guard = state.lock().unwrap();
            state_guard.ready = false;
        }
    }

    /// Deregister a task
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task to deregister
    pub fn deregister_task(&mut self, task_id: usize) {
        self.tasks.remove(&task_id);
    }

    /// Check if a task is registered
    pub fn has_task(&self, task_id: usize) -> bool {
        self.tasks.contains_key(&task_id)
    }

    /// Get the number of registered tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global task registry instance
///
/// In a full implementation, this would be per-executor rather than global.
/// For now, we use a global registry for simplicity.
static GLOBAL_REGISTRY: Mutex<Option<TaskRegistry>> = Mutex::new(None);

/// Initialize the global task registry
pub fn init_global_registry() {
    let mut registry = GLOBAL_REGISTRY.lock().unwrap();
    if registry.is_none() {
        *registry = Some(TaskRegistry::new());
    }
}

/// Get the global task registry
///
/// # Panics
///
/// Panics if the registry hasn't been initialized
pub fn global_registry() -> &'static Mutex<Option<TaskRegistry>> {
    &GLOBAL_REGISTRY
}

/// Task ID counter
static TASK_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a new unique task ID
pub fn generate_task_id() -> usize {
    TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = TaskRegistry::new();
        assert_eq!(registry.task_count(), 0);
    }

    #[test]
    fn test_task_registration() {
        let mut registry = TaskRegistry::new();
        let task_id = 1;

        let state = registry.register_task(task_id);
        assert_eq!(registry.task_count(), 1);

        // Check initial state
        let state_guard = state.lock().unwrap();
        assert!(state_guard.ready);
        assert!(state_guard.active);
    }

    #[test]
    fn test_wake_task() {
        let mut registry = TaskRegistry::new();
        let task_id = 1;

        registry.register_task(task_id);

        // Mark as not ready
        registry.mark_not_ready(task_id);
        assert!(!registry.is_task_ready(task_id));

        // Wake the task
        assert!(registry.wake_task(task_id));
        assert!(registry.is_task_ready(task_id));
    }

    #[test]
    fn test_wake_nonexistent_task() {
        let registry = TaskRegistry::new();
        assert!(!registry.wake_task(999));
    }

    #[test]
    fn test_deregister_task() {
        let mut registry = TaskRegistry::new();
        let task_id = 1;

        registry.register_task(task_id);
        assert_eq!(registry.task_count(), 1);

        registry.deregister_task(task_id);
        assert_eq!(registry.task_count(), 0);
        assert!(!registry.is_task_ready(task_id));
    }

    #[test]
    fn test_mark_not_ready() {
        let mut registry = TaskRegistry::new();
        let task_id = 1;

        registry.register_task(task_id);
        assert!(registry.is_task_ready(task_id));

        registry.mark_not_ready(task_id);
        assert!(!registry.is_task_ready(task_id));
    }

    #[test]
    fn test_generate_task_id() {
        let id1 = generate_task_id();
        let id2 = generate_task_id();
        let id3 = generate_task_id();

        assert!(id1 < id2);
        assert!(id2 < id3);
    }
}
