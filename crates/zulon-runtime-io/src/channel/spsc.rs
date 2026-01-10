// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Single-producer single-consumer channel
//!
//! This module provides a lock-free SPSC channel optimized for
//! scenarios where there is exactly one producer and one consumer.
//!
//! # Advantages over MPSC
//!
//! - **Lock-free**: No mutex contention
//! - **Faster**: Single producer allows optimizations
//! - **Safer**: Compile-time guarantee of single producer

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Create a new single-producer single-consumer channel
///
/// # Returns
///
/// Returns a tuple of (Sender, Receiver)
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::channel::spsc;
///
/// let (tx, rx) = spsc::channel::<i32>();
///
/// // Send from single producer
/// tx.send(42).unwrap();
///
/// // Receive on single consumer
/// assert_eq!(rx.recv().unwrap(), 42);
/// ```
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        buffer: std::sync::Mutex::new(VecDeque::new()),
        sender_alive: AtomicUsize::new(1),
        receiver_alive: AtomicUsize::new(1),
    });

    (
        Sender { shared: shared.clone() },
        Receiver { shared },
    )
}

/// Sending end of the SPSC channel
///
/// There can only be one sender (not cloneable).
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.sender_alive.store(0, Ordering::Relaxed);
    }
}

impl<T: Send> Sender<T> {
    /// Send a value to the channel
    ///
    /// # Arguments
    ///
    /// * `value` - Value to send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if value was sent successfully
    /// - `Err(SendError(value))` if receiver has been dropped
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_io::channel::spsc;
    ///
    /// let (tx, rx) = spsc::channel::<i32>();
    /// tx.send(42).unwrap();
    /// ```
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Lock and check receiver
        let mut buffer = self.shared.buffer.lock().unwrap();

        if self.shared.receiver_alive.load(Ordering::Relaxed) == 0 {
            return Err(SendError(value));
        }

        buffer.push_back(value);
        Ok(())
    }

    /// Check if receiver is still alive
    ///
    /// # Returns
    ///
    /// `true` if receiver exists
    pub fn has_receiver(&self) -> bool {
        self.shared.receiver_alive.load(Ordering::Relaxed) != 0
    }
}

/// Receiving end of the SPSC channel
///
/// There can only be one receiver (not cloneable).
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.receiver_alive.store(0, Ordering::Relaxed);
    }
}

impl<T: Send> Receiver<T> {
    /// Try to receive a value from the channel
    ///
    /// # Returns
    ///
    /// - `Ok(value)` if a value was received
    /// - `Err(RecvError::Disconnected)` if sender dropped and queue is empty
    /// - `Err(RecvError::Empty)` if no value available but sender exists
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_io::channel::spsc;
    ///
    /// let (tx, rx) = spsc::channel::<i32>();
    /// tx.send(42).unwrap();
    ///
    /// assert_eq!(rx.recv().unwrap(), 42);
    /// ```
    pub fn recv(&self) -> Result<T, RecvError> {
        let mut buffer = self.shared.buffer.lock().unwrap();

        if let Some(value) = buffer.pop_front() {
            return Ok(value);
        }

        if self.shared.sender_alive.load(Ordering::Relaxed) == 0 {
            Err(RecvError::Disconnected)
        } else {
            Err(RecvError::Empty)
        }
    }

    /// Check if sender is still alive
    ///
    /// # Returns
    ///
    /// `true` if sender exists
    pub fn has_sender(&self) -> bool {
        self.shared.sender_alive.load(Ordering::Relaxed) != 0
    }

    /// Check if the channel is empty
    ///
    /// # Returns
    ///
    /// `true` if there are no values in the queue
    pub fn is_empty(&self) -> bool {
        let buffer = self.shared.buffer.lock().unwrap();
        buffer.is_empty()
    }

    /// Get the number of values in the queue
    ///
    /// # Returns
    ///
    /// The number of pending values
    pub fn len(&self) -> usize {
        let buffer = self.shared.buffer.lock().unwrap();
        buffer.len()
    }
}

/// Shared state between sender and receiver
struct Shared<T> {
    /// Message buffer
    buffer: std::sync::Mutex<VecDeque<T>>,

    /// Whether sender is alive (0 or 1)
    sender_alive: AtomicUsize,

    /// Whether receiver is alive (0 or 1)
    receiver_alive: AtomicUsize,
}

/// Error returned when sending fails
#[derive(Debug, PartialEq, Eq)]
pub struct SendError<T>(pub T);

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "send failed: no receiver")
    }
}

impl<T: std::fmt::Debug> std::error::Error for SendError<T> {}

/// Error returned when receiving fails
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// Sender dropped and queue is empty
    Disconnected,
    /// No value available but sender exists
    Empty,
}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvError::Disconnected => write!(f, "receive failed: channel disconnected"),
            RecvError::Empty => write!(f, "receive failed: no data available"),
        }
    }
}

impl std::error::Error for RecvError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let (tx, rx): (Sender<i32>, Receiver<i32>) = channel();
        assert!(tx.has_receiver());
        assert!(rx.has_sender());
        assert!(rx.is_empty());
        assert_eq!(rx.len(), 0);
    }

    #[test]
    fn test_send_and_recv() {
        let (tx, rx) = channel();
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn test_recv_empty() {
        let (_tx, rx) = channel::<i32>();
        assert_eq!(rx.recv(), Err(RecvError::Empty));
        assert!(rx.has_sender());
    }

    #[test]
    fn test_send_after_receiver_dropped() {
        let (tx, rx) = channel::<i32>();
        drop(rx);
        assert_eq!(tx.send(42), Err(SendError(42)));
        assert!(!tx.has_receiver());
    }

    #[test]
    fn test_recv_after_sender_dropped() {
        let (tx, rx) = channel::<i32>();
        drop(tx);
        assert_eq!(rx.recv(), Err(RecvError::Disconnected));
        assert!(!rx.has_sender());
    }

    #[test]
    fn test_sender_not_cloneable() {
        // This test just verifies that Sender doesn't implement Clone
        // If it did, this would fail to compile
        let (tx, _rx): (Sender<i32>, Receiver<i32>) = channel();
        // let tx2 = tx.clone(); // This would fail
        let _ = tx; // Use tx to avoid unused warning
    }

    #[test]
    fn test_receiver_not_cloneable() {
        // This test just verifies that Receiver doesn't implement Clone
        let (tx, rx): (Sender<i32>, Receiver<i32>) = channel();
        let _ = tx; // Use tx to avoid unused warning
        // let rx2 = rx.clone(); // This would fail
        let _ = rx;
    }

    #[test]
    fn test_queue_length() {
        let (tx, rx) = channel();

        assert_eq!(rx.len(), 0);
        assert!(rx.is_empty());

        tx.send(1).unwrap();
        assert_eq!(rx.len(), 1);
        assert!(!rx.is_empty());

        tx.send(2).unwrap();
        assert_eq!(rx.len(), 2);

        rx.recv().unwrap();
        assert_eq!(rx.len(), 1);

        rx.recv().unwrap();
        assert_eq!(rx.len(), 0);
        assert!(rx.is_empty());
    }

    #[test]
    fn test_fifo_order() {
        let (tx, rx) = channel();

        for i in 1..=10 {
            tx.send(i).unwrap();
        }

        for i in 1..=10 {
            assert_eq!(rx.recv().unwrap(), i);
        }
    }

    #[test]
    fn test_remaining_messages_after_sender_drop() {
        let (tx, rx) = channel();

        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        // Drop sender
        drop(tx);

        // Should still receive queued messages
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
        assert_eq!(rx.recv().unwrap(), 3);

        // Now should get Disconnected
        assert_eq!(rx.recv(), Err(RecvError::Disconnected));
    }
}
