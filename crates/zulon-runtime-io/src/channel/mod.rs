// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Channel types for inter-task communication
//!
//! This module provides various channel implementations for communication
//! between different tasks or threads:
//!
//! - **mpsc**: Multi-producer single-consumer channel
//! - **spsc**: Single-producer single-consumer channel
//! - **oneshot**: Single-use channel
//!
//! ## Example
//!
//! ```rust,no_run
//! use zulon_runtime_io::channel::mpsc;
//!
//! let (tx, rx) = mpsc::channel();
//!
//! // Send a message
//! tx.send(42);
//!
//! // Receive a message
//! match rx.recv() {
//!     Ok(value) => println!("Received: {}", value),
//!     Err(mpsc::RecvError::Disconnected) => println!("Sender disconnected"),
//!     Err(mpsc::RecvError::Empty) => println!("No message available"),
//! }
//! ```

pub mod mpsc;
pub mod spsc;
pub mod oneshot;

// Re-export MPSC types (most commonly used)
pub use mpsc::{channel, Sender, Receiver, SendError, RecvError};
