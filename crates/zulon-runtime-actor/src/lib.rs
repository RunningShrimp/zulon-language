// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Actor Model for ZULON
//!
//! Lightweight actor runtime with message passing.
//!
//! # Overview
//!
//! This module provides a simple, efficient actor model implementation for ZULON.
//! Actors are isolated units of computation that communicate via message passing.
//!
//! # Components
//!
//! - [`ActorId`]: Unique identifier for actors
//! - [`ActorState`]: State of an actor (Running, Stopped)
//! - [`Message`]: Trait that all messages must implement
//! - [`Actor`]: Trait that users implement for their actors
//! - [`ActorRuntime`]: Runtime that manages actors and message processing
//!
//! # Example
//!
//! ```no_run
//! use zulon_runtime_actor::{Actor, ActorRuntime, ActorId, Message, ActorState};
//! use std::sync::Arc;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//!
//! struct CounterMessage {
//!     count: Arc<AtomicUsize>,
//! }
//!
//! impl Message for CounterMessage {
//!     fn handle(&self, runtime: &mut ActorRuntime) -> Result<(), ActorError> {
//!         self.count.fetch_add(1, Ordering::SeqCst);
//!         Ok(())
//!     }
//! }
//!
//! struct MyActor;
//!
//! impl Actor for MyActor {
//!     fn started(&mut self, _runtime: &mut ActorRuntime) {
//!     // Actor started
//!     println!("Actor started");
//!     _runtime = runtime;
//!     }
//!
//!     fn stopped(&mut self) {
//!         // Actor stopped
//!         println!("Actor stopped");
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut runtime = ActorRuntime::new();
//!     let actor_id = runtime.spawn(MyActor)?;
//!
//!     let count = Arc::new(AtomicUsize::new(0));
//!     let msg = Box::new(CounterMessage { count });
//!     runtime.send(actor_id, msg)?;
//!     runtime.process_messages()?;
//!
//!     runtime.stop(actor_id)?;
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for an actor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId(pub Uuid);

impl ActorId {
    pub fn new() -> Self {
        ActorId(Uuid::new_v4())
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Actor error types
#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Actor {0} not found")]
    NotFound(ActorId),
    #[error("Actor {0} is not running")]
    NotRunning(ActorId),
    #[error("Actor mailbox is full")]
    MailboxFull,
    #[error("Message handling failed: {0}")]
    MessageHandlingFailed(String),
}

/// Actor state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    Running,
    Stopped,
}

/// Message trait - all messages must implement this
pub trait Message: Send + 'static {
    /// Handle the message for the given actor runtime
    fn handle(&self, runtime: &mut ActorRuntime) -> Result<(), ActorError>;
}

/// Actor trait - users implement this for their actors
pub trait Actor: Send + 'static {
    /// Called when actor is spawned
    fn started(&mut self, runtime: &mut ActorRuntime);
    /// Called when actor is stopped
    fn stopped(&mut self);
}

/// Actor wrapper - stores actor instance and message queue
#[derive(Debug)]
struct ActorWrapper {
    id: ActorId,
    state: ActorState,
    sender: crossbeam_channel::Sender<Box<dyn Message>>,
    receiver: crossbeam_channel::Receiver<Box<dyn Message>>,
}

impl ActorWrapper {
    pub fn new(id: ActorId) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        ActorWrapper {
            id,
            state: ActorState::Running,
            sender,
            receiver,
        }
    }

    pub fn send(&self, msg: Box<dyn Message>) -> Result<(), ActorError> {
        if self.state != ActorState::Running {
            return Err(ActorError::NotRunning(self.id));
        }
        self.sender.send(msg).map_err(|_| ActorError::MailboxFull)
    }

    pub fn stop(&mut self) {
        self.state = ActorState::Stopped;
    }

    pub fn try_recv(&self) -> Result<Box<dyn Message>, crossbeam_channel::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Actor runtime
#[derive(Debug)]
pub struct ActorRuntime {
    actors: HashMap<ActorId, ActorWrapper>,
}

impl ActorRuntime {
    pub fn new() -> Self {
        ActorRuntime {
            actors: HashMap::new(),
        }
    }

    pub fn spawn<A: Actor + 'static>(&mut self, mut actor: A) -> Result<ActorId, ActorError> {
        let id = ActorId::new();
        actor.started(self);
        let wrapper = ActorWrapper::new(id);
        self.actors.insert(id, wrapper);
        Ok(id)
    }

    pub fn send(&self, actor_id: ActorId, msg: Box<dyn Message>) -> Result<(), ActorError> {
        let actor = self
            .actors
            .get(&actor_id)
            .ok_or_else(|| ActorError::NotFound(actor_id))?;
        actor.send(msg)
    }

    pub fn stop(&mut self, actor_id: ActorId) -> Result<(), ActorError> {
        let actor = self
            .actors
            .get_mut(&actor_id)
            .ok_or_else(|| ActorError::NotFound(actor_id))?;
        actor.stop();
        Ok(())
    }

    pub fn process_messages(&mut self) -> Result<usize, ActorError> {
        let mut processed = 0;
        let mut messages = Vec::new();

        // Collect all messages from all running actors
        for actor in self.actors.values() {
            if actor.state == ActorState::Running {
                while let Ok(msg) = actor.try_recv() {
                    messages.push(msg);
                }
            }
        }

        // Process all collected messages
        for msg in messages {
            msg.handle(self)?;
            processed += 1;
        }

        Ok(processed)
    }
}

impl Default for ActorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test message
    struct TestMessage;

    impl Message for TestMessage {
        fn handle(&self, _runtime: &mut ActorRuntime) -> Result<(), ActorError> {
            Ok(())
        }
    }

    /// Counter message - increments a counter
    struct CounterMessage {
        count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl Message for CounterMessage {
        fn handle(&self, _runtime: &mut ActorRuntime) -> Result<(), ActorError> {
            self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    /// Simple test actor
    struct TestActor;

    impl Actor for TestActor {
        fn started(&mut self, _runtime: &mut ActorRuntime) {
            // Actor started callback
        }

        fn stopped(&mut self) {
            // Actor stopped callback
        }
    }

    #[test]
    fn test_actor_id_display() {
        let id = ActorId::new();
        let formatted = format!("{}", id);
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_actor_spawn() {
        let mut runtime = ActorRuntime::new();
        let actor = TestActor;
        let result = runtime.spawn(actor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_actor_lifecycle() {
        let mut runtime = ActorRuntime::new();
        let actor = TestActor;
        let actor_id = runtime.spawn(actor).unwrap();

        // Spawn should create a running actor
        assert!(runtime.actors.contains_key(&actor_id));

        // Stop the actor
        runtime.stop(actor_id).unwrap();

        // Verify the actor exists in the runtime
        assert!(runtime.actors.contains_key(&actor_id));
    }

    #[test]
    fn test_send_message() {
        let mut runtime = ActorRuntime::new();
        let actor = TestActor;
        let actor_id = runtime.spawn(actor).unwrap();

        let msg: Box<dyn Message> = Box::new(TestMessage);
        let result = runtime.send(actor_id, msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_send_to_nonexistent_actor() {
        let runtime = ActorRuntime::new();
        let msg: Box<dyn Message> = Box::new(TestMessage);
        let fake_id = ActorId::new();
        let result = runtime.send(fake_id, msg);
        assert!(matches!(result, Err(ActorError::NotFound(_))));
    }

    #[test]
    fn test_stop_nonexistent_actor() {
        let mut runtime = ActorRuntime::new();
        let fake_id = ActorId::new();
        let result = runtime.stop(fake_id);
        assert!(matches!(result, Err(ActorError::NotFound(_))));
    }

    #[test]
    fn test_send_to_stopped_actor() {
        let mut runtime = ActorRuntime::new();
        let actor = TestActor;
        let actor_id = runtime.spawn(actor).unwrap();

        // Stop the actor
        runtime.stop(actor_id).unwrap();

        // Try to send a message
        let msg: Box<dyn Message> = Box::new(TestMessage);
        let result = runtime.send(actor_id, msg);
        assert!(matches!(result, Err(ActorError::NotRunning(_))));
    }

    #[test]
    fn test_process_messages() {
        let mut runtime = ActorRuntime::new();
        let actor = TestActor;
        let actor_id = runtime.spawn(actor).unwrap();

        // Send multiple messages
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..5 {
            let msg: Box<dyn Message> = Box::new(CounterMessage {
                count: count.clone(),
            });
            runtime.send(actor_id, msg).unwrap();
        }

        // Process messages
        let processed = runtime.process_messages().unwrap();
        assert_eq!(processed, 5);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 5);
    }

    #[test]
    fn test_process_messages_empty() {
        let mut runtime = ActorRuntime::new();
        let actor = TestActor;
        runtime.spawn(actor).unwrap();

        // Process with no messages
        let processed = runtime.process_messages().unwrap();
        assert_eq!(processed, 0);
    }

    #[test]
    fn test_runtime_default() {
        let runtime = ActorRuntime::default();
        assert_eq!(runtime.actors.len(), 0);
    }

    #[test]
    fn test_multiple_actors() {
        let mut runtime = ActorRuntime::new();

        // Spawn multiple actors
        let id1 = runtime.spawn(TestActor).unwrap();
        let id2 = runtime.spawn(TestActor).unwrap();
        let id3 = runtime.spawn(TestActor).unwrap();

        // Verify all actors exist
        assert!(runtime.actors.contains_key(&id1));
        assert!(runtime.actors.contains_key(&id2));
        assert!(runtime.actors.contains_key(&id3));
        assert_eq!(runtime.actors.len(), 3);

        // Stop one actor
        runtime.stop(id2).unwrap();
        assert!(runtime.actors.contains_key(&id2)); // Still exists, just stopped
        assert_eq!(runtime.actors.len(), 3);
    }
}
