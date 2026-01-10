// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Multi-producer single-consumer channel
//!
//! This module provides a channel that allows multiple senders
//! but only one receiver. This is useful for fan-in scenarios
//! where multiple tasks need to send data to a single consumer.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::VecDeque;

/// Create a new multi-producer single-consumer channel
///
/// # Returns
///
/// Returns a tuple of (Sender, Receiver)
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::channel::mpsc;
///
/// let (tx, rx) = mpsc::channel::<i32>();
///
/// // Clone sender to have multiple producers
/// let tx2 = tx.clone();
///
/// tx.send(1).unwrap();
/// tx2.send(2).unwrap();
/// ```
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(ChannelInner {
        queue: std::sync::Mutex::new(VecDeque::new()),
        senders: AtomicUsize::new(1),
        receivers: AtomicUsize::new(1),
    });

    (
        Sender { inner: inner.clone() },
        Receiver { inner },
    )
}

/// Sending end of the channel
///
/// Can be cloned to have multiple senders.
/// Implements Send + Sync so it can be shared between threads.
pub struct Sender<T> {
    inner: Arc<ChannelInner<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.senders.fetch_add(1, Ordering::Relaxed);
        Sender { inner: self.inner.clone() }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.senders.fetch_sub(1, Ordering::Relaxed);
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
    /// - `Err(SendError(value))` if all receivers have been dropped
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_io::channel::mpsc;
    ///
    /// let (tx, rx) = mpsc::channel();
    /// tx.send(42).unwrap();
    /// ```
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Lock first to ensure atomicity with receiver check
        let mut queue = self.inner.queue.lock().unwrap();

        // Check if receiver still exists (while holding lock)
        if self.inner.receivers.load(Ordering::Relaxed) == 0 {
            return Err(SendError(value));
        }

        // Push to queue
        queue.push_back(value);
        Ok(())
    }

    /// Check if there are any receivers still connected
    ///
    /// # Returns
    ///
    /// `true` if at least one receiver exists
    pub fn has_receivers(&self) -> bool {
        self.inner.receivers.load(Ordering::Relaxed) > 0
    }
}

/// Receiving end of the channel
///
/// There can only be one receiver for a channel.
pub struct Receiver<T> {
    inner: Arc<ChannelInner<T>>,
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.receivers.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<T: Send> Receiver<T> {
    /// Try to receive a value from the channel
    ///
    /// # Returns
    ///
    /// - `Ok(value)` if a value was received
    /// - `Err(RecvError::Disconnected)` if all senders have been dropped and queue is empty
    /// - `Err(RecvError::Empty)` if no value is available but senders still exist
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_io::channel::mpsc;
    ///
    /// let (tx, rx) = mpsc::channel::<i32>();
    /// tx.send(42).unwrap();
    ///
    /// match rx.recv() {
    ///     Ok(value) => println!("Received: {}", value),
    ///     Err(mpsc::RecvError::Disconnected) => println!("Disconnected"),
    ///     Err(mpsc::RecvError::Empty) => println!("No data"),
    /// }
    /// ```
    pub fn recv(&self) -> Result<T, RecvError> {
        let mut queue = self.inner.queue.lock().unwrap();

        // Try to pop from queue
        if let Some(value) = queue.pop_front() {
            return Ok(value);
        }

        // Queue is empty, check if senders still exist
        if self.inner.senders.load(Ordering::Relaxed) == 0 {
            Err(RecvError::Disconnected)
        } else {
            Err(RecvError::Empty)
        }
    }

    /// Check if there are any senders still connected
    ///
    /// # Returns
    ///
    /// `true` if at least one sender exists
    pub fn has_senders(&self) -> bool {
        self.inner.senders.load(Ordering::Relaxed) > 0
    }

    /// Check if the channel is empty
    ///
    /// # Returns
    ///
    /// `true` if there are no values in the queue
    pub fn is_empty(&self) -> bool {
        let queue = self.inner.queue.lock().unwrap();
        queue.is_empty()
    }

    /// Get the number of values in the queue
    ///
    /// # Returns
    ///
    /// The number of pending values
    pub fn len(&self) -> usize {
        let queue = self.inner.queue.lock().unwrap();
        queue.len()
    }
}

/// Inner channel state shared between senders and receiver
struct ChannelInner<T> {
    /// Message queue
    queue: std::sync::Mutex<VecDeque<T>>,

    /// Number of active senders
    senders: AtomicUsize,

    /// Number of active receivers (always 0 or 1 for mpsc)
    receivers: AtomicUsize,
}

/// Error returned when sending fails
///
/// Contains the value that failed to send.
#[derive(Debug, PartialEq, Eq)]
pub struct SendError<T>(pub T);

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "send failed: no receivers")
    }
}

impl<T: std::fmt::Debug> std::error::Error for SendError<T> {}

/// Error returned when receiving fails
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// All senders have been disconnected and queue is empty
    Disconnected,
    /// No value available but senders still exist
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
        assert!(tx.has_receivers());
        assert!(rx.has_senders());
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
        assert!(rx.has_senders());
    }

    #[test]
    fn test_send_after_receiver_dropped() {
        let (tx, rx) = channel::<i32>();
        drop(rx);
        assert_eq!(tx.send(42), Err(SendError(42)));
        assert!(!tx.has_receivers());
    }

    #[test]
    fn test_recv_after_sender_dropped() {
        let (tx, rx) = channel::<i32>();
        drop(tx);
        assert_eq!(rx.recv(), Err(RecvError::Disconnected));
        assert!(!rx.has_senders());
    }

    #[test]
    fn test_multiple_senders() {
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        tx1.send(1).unwrap();
        tx2.send(2).unwrap();

        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
    }

    #[test]
    fn test_sender_count() {
        let (tx1, rx): (Sender<i32>, Receiver<i32>) = channel();
        let tx2 = tx1.clone();
        let tx3 = tx1.clone();

        // Initially 3 senders
        assert!(rx.has_senders());

        // Drop one
        drop(tx3);
        assert!(rx.has_senders());

        // Drop all
        drop(tx1);
        drop(tx2);
        assert!(!rx.has_senders());
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
