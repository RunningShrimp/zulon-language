// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Task management for async runtime
//!
//! This module implements:
//! - Task queue for managing async tasks
//! - Task scheduling and execution
//! - Task wakeup and completion tracking

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub usize);

impl TaskId {
    /// Create a new task ID
    pub fn new(id: usize) -> Self {
        TaskId(id)
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run
    Ready,
    /// Task is waiting on an event
    Waiting,
    /// Task has completed
    Completed,
    /// Task has panicked
    Panicked,
}

/// Task in the async runtime
pub struct Task {
    /// Unique task ID
    pub id: TaskId,
    /// Current task state
    pub state: TaskState,
    /// Result of task (set when completed)
    pub result: Option<Box<dyn std::any::Any + Send>>,
}

impl Clone for Task {
    fn clone(&self) -> Task {
        Task {
            id: self.id,
            state: self.state,
            result: None,
        }
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("result", &self.result.as_ref().map(|_| "<opaque>"))
            .finish()
    }
}

impl Task {
    /// Create a new task
    pub fn new(id: TaskId) -> Self {
        Task {
            id,
            state: TaskState::Ready,
            result: None,
        }
    }

    /// Mark task as completed
    pub fn complete(&mut self, result: Box<dyn std::any::Any + Send>) {
        self.state = TaskState::Completed;
        self.result = Some(result);
    }

    /// Mark task as panicked
    pub fn panic(&mut self) {
        self.state = TaskState::Panicked;
    }

    /// Check if task is ready to run
    pub fn is_ready(&self) -> bool {
        self.state == TaskState::Ready
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        self.state == TaskState::Completed
    }
}

/// Thread-safe task queue
#[derive(Debug)]
pub struct TaskQueue {
    /// Queue of ready tasks
    ready_queue: VecDeque<TaskId>,
    /// All tasks
    tasks: Vec<Option<Task>>,
    /// Next task ID
    next_id: usize,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new() -> Self {
        TaskQueue {
            ready_queue: VecDeque::new(),
            tasks: Vec::new(),
            next_id: 0,
        }
    }

    /// Spawn a new task
    pub fn spawn(&mut self) -> TaskId {
        let id = TaskId::new(self.next_id);
        self.next_id += 1;

        let task = Task::new(id);

        if self.next_id > self.tasks.len() {
            self.tasks.push(Some(task));
        } else {
            self.tasks[id.0] = Some(task);
        }

        self.ready_queue.push_back(id);
        id
    }

    /// Get the next ready task
    pub fn pop_ready(&mut self) -> Option<TaskId> {
        self.ready_queue.pop_front()
    }

    /// Push a task to the ready queue
    pub fn push_ready(&mut self, task_id: TaskId) {
        if let Some(task) = self.tasks.get_mut(task_id.0) {
            if let Some(task) = task {
                if task.state == TaskState::Waiting {
                    task.state = TaskState::Ready;
                }
            }
        }
        self.ready_queue.push_back(task_id);
    }

    /// Get a mutable reference to a task
    pub fn get_task_mut(&mut self, task_id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(task_id.0)?.as_mut()
    }

    /// Get an immutable reference to a task
    pub fn get_task(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.get(task_id.0)?.as_ref()
    }

    /// Check if there are ready tasks
    pub fn has_ready(&self) -> bool {
        !self.ready_queue.is_empty()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared task queue
#[derive(Debug, Clone)]
pub struct SharedTaskQueue {
    inner: Arc<Mutex<TaskQueue>>,
}

impl SharedTaskQueue {
    /// Create a new shared task queue
    pub fn new() -> Self {
        SharedTaskQueue {
            inner: Arc::new(Mutex::new(TaskQueue::new())),
        }
    }

    /// Spawn a new task
    pub fn spawn(&self) -> TaskId {
        let mut queue = self.inner.lock().unwrap();
        queue.spawn()
    }

    /// Get the next ready task
    pub fn pop_ready(&self) -> Option<TaskId> {
        let mut queue = self.inner.lock().unwrap();
        queue.pop_ready()
    }

    /// Push a task to the ready queue
    pub fn push_ready(&self, task_id: TaskId) {
        let mut queue = self.inner.lock().unwrap();
        queue.push_ready(task_id);
    }

    /// Get a task
    pub fn get_task(&self, task_id: TaskId) -> Option<Task> {
        let queue = self.inner.lock().unwrap();
        queue.get_task(task_id).cloned()
    }

    /// Check if there are ready tasks
    pub fn has_ready(&self) -> bool {
        let queue = self.inner.lock().unwrap();
        queue.has_ready()
    }
}

impl Default for SharedTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_new() {
        let id = TaskId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_task_new() {
        let id = TaskId::new(0);
        let task = Task::new(id);
        assert_eq!(task.id, id);
        assert_eq!(task.state, TaskState::Ready);
        assert!(task.result.is_none());
    }

    #[test]
    fn test_task_complete() {
        let id = TaskId::new(0);
        let mut task = Task::new(id);
        let result: Box<i32> = Box::new(42);
        task.complete(result);
        assert_eq!(task.state, TaskState::Completed);
        assert!(task.result.is_some());
    }

    #[test]
    fn test_task_panic() {
        let id = TaskId::new(0);
        let mut task = Task::new(id);
        task.panic();
        assert_eq!(task.state, TaskState::Panicked);
    }

    #[test]
    fn test_task_is_ready() {
        let id = TaskId::new(0);
        let task = Task::new(id);
        assert!(task.is_ready());
    }

    #[test]
    fn test_task_is_completed() {
        let id = TaskId::new(0);
        let mut task = Task::new(id);
        assert!(!task.is_completed());

        let result: Box<i32> = Box::new(42);
        task.complete(result);
        assert!(task.is_completed());
    }

    #[test]
    fn test_task_clone() {
        let id = TaskId::new(0);
        let mut task = Task::new(id);
        let result: Box<i32> = Box::new(42);
        task.complete(result);

        let cloned = task.clone();
        assert_eq!(cloned.id, task.id);
        assert_eq!(cloned.state, task.state);
        assert!(cloned.result.is_none());
    }

    #[test]
    fn test_task_queue_new() {
        let queue = TaskQueue::new();
        assert!(!queue.has_ready());
        assert_eq!(queue.next_id, 0);
    }

    #[test]
    fn test_task_queue_spawn() {
        let mut queue = TaskQueue::new();
        let id = queue.spawn();
        assert_eq!(id.0, 0);
        assert_eq!(queue.next_id, 1);
        assert!(queue.has_ready());
    }

    #[test]
    fn test_task_queue_spawn_multiple() {
        let mut queue = TaskQueue::new();
        let id1 = queue.spawn();
        let id2 = queue.spawn();
        let id3 = queue.spawn();

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(id3.0, 2);
        assert_eq!(queue.next_id, 3);
    }

    #[test]
    fn test_task_queue_pop_ready() {
        let mut queue = TaskQueue::new();
        let id = queue.spawn();

        let popped = queue.pop_ready();
        assert_eq!(popped, Some(id));
        assert!(!queue.has_ready());

        let popped_again = queue.pop_ready();
        assert_eq!(popped_again, None);
    }

    #[test]
    fn test_task_queue_fifo() {
        let mut queue = TaskQueue::new();
        let id1 = queue.spawn();
        let id2 = queue.spawn();
        let id3 = queue.spawn();

        assert_eq!(queue.pop_ready(), Some(id1));
        assert_eq!(queue.pop_ready(), Some(id2));
        assert_eq!(queue.pop_ready(), Some(id3));
        assert_eq!(queue.pop_ready(), None);
    }

    #[test]
    fn test_task_queue_push_ready() {
        let mut queue = TaskQueue::new();
        let id = queue.spawn();

        queue.pop_ready();
        assert!(!queue.has_ready());

        if let Some(task) = queue.get_task_mut(id) {
            task.state = TaskState::Waiting;
        }

        queue.push_ready(id);
        assert!(queue.has_ready());

        let task = queue.get_task(id).unwrap();
        assert_eq!(task.state, TaskState::Ready);
    }

    #[test]
    fn test_task_queue_get_task() {
        let mut queue = TaskQueue::new();
        let id = queue.spawn();

        let task = queue.get_task(id);
        assert!(task.is_some());
        assert_eq!(task.unwrap().id, id);
    }

    #[test]
    fn test_task_queue_get_task_mut() {
        let mut queue = TaskQueue::new();
        let id = queue.spawn();

        let task = queue.get_task_mut(id);
        assert!(task.is_some());

        let task = task.unwrap();
        assert_eq!(task.id, id);
        assert!(task.is_ready());

        task.state = TaskState::Waiting;
        assert_eq!(task.state, TaskState::Waiting);
    }

    #[test]
    fn test_task_queue_get_nonexistent_task() {
        let mut queue = TaskQueue::new();
        let fake_id = TaskId::new(999);

        let task = queue.get_task(fake_id);
        assert!(task.is_none());

        let task = queue.get_task_mut(fake_id);
        assert!(task.is_none());
    }

    #[test]
    fn test_task_queue_default() {
        let queue = TaskQueue::default();
        assert!(!queue.has_ready());
        assert_eq!(queue.next_id, 0);
    }

    #[test]
    fn test_shared_task_queue_spawn_and_pop() {
        let queue = SharedTaskQueue::new();
        let id = queue.spawn();

        let popped = queue.pop_ready();
        assert_eq!(popped, Some(id));
    }

    #[test]
    fn test_shared_task_queue_push_ready() {
        let queue = SharedTaskQueue::new();
        let id = queue.spawn();

        queue.pop_ready();
        queue.push_ready(id);
        let popped = queue.pop_ready();
        assert_eq!(popped, Some(id));
    }

    #[test]
    fn test_shared_task_queue_spawn() {
        let queue = SharedTaskQueue::new();
        let id = queue.spawn();
        assert_eq!(id.0, 0);
    }

    #[test]
    fn test_shared_task_queue_get_task() {
        let queue = SharedTaskQueue::new();
        let id = queue.spawn();

        let task = queue.get_task(id);
        assert!(task.is_some());
        assert_eq!(task.unwrap().id, id);
    }

    #[test]
    fn test_shared_task_queue_thread_safety() {
        use std::thread;
        use std::time::Duration;

        let queue = std::sync::Arc::new(SharedTaskQueue::new());
        let mut handles = vec![];

        for _ in 0..4 {
            let queue_clone = queue.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    queue_clone.spawn();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        thread::sleep(Duration::from_millis(10));

        assert!(queue.has_ready());

        let mut count = 0;
        while queue.pop_ready().is_some() {
            count += 1;
        }

        assert_eq!(count, 40);
    }

    #[test]
    fn test_shared_task_queue_default() {
        let queue = SharedTaskQueue::default();
        assert!(!queue.has_ready());
    }

    #[test]
    fn test_shared_task_queue_concurrent_spawn_and_pop() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;
        use std::time::Duration;

        let queue = std::sync::Arc::new(SharedTaskQueue::new());
        let spawned_count = Arc::new(AtomicUsize::new(0));
        let popped_count = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for _ in 0..4 {
            let queue_clone = queue.clone();
            let spawned_count_clone = spawned_count.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    queue_clone.spawn();
                    spawned_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        for _ in 0..4 {
            let queue_clone = queue.clone();
            let popped_count_clone = popped_count.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    if queue_clone.pop_ready().is_some() {
                        popped_count_clone.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        thread::sleep(Duration::from_millis(10));

        assert_eq!(spawned_count.load(Ordering::SeqCst), 40);
    }
}
