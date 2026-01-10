// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! One-shot channel for single-use synchronization
//!
//! This module provides a channel that can send exactly one message.
//! It's useful for one-time results or simple synchronization.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Create a new one-shot channel
///
/// # Returns
///
/// Returns a tuple of (Sender, Receiver)
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::channel::oneshot;
///
/// let (tx, rx) = oneshot::channel::<i32>();
///
/// // Send once
/// tx.send(42).unwrap();
///
/// // Receive once
/// assert_eq!(rx.recv().unwrap(), 42);
///
/// // Second receive fails
/// assert!(rx.recv().is_err());
/// ```
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        value: std::sync::Mutex::new(None),
        sender_alive: AtomicBool::new(true),
        receiver_alive: AtomicBool::new(true),
        consumed: AtomicBool::new(false),
    });

    (
        Sender { shared: shared.clone() },
        Receiver { shared },
    )
}

/// Sending end of the one-shot channel
///
/// Can send exactly one message.
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.sender_alive.store(false, Ordering::Relaxed);
    }
}

impl<T: Send> Sender<T> {
    /// Send a value through the one-shot channel
    ///
    /// # Arguments
    ///
    /// * `value` - Value to send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if value was sent successfully
    /// - `Err(SendError(value))` if receiver was already dropped or consumed
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_io::channel::oneshot;
    ///
    /// let (tx, rx) = oneshot::channel();
    /// tx.send(42).unwrap();
    /// ```
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Lock and check state
        let mut slot = self.shared.value.lock().unwrap();

        // Check if receiver is still alive
        if !self.shared.receiver_alive.load(Ordering::Relaxed) {
            return Err(SendError(value));
        }

        // Check if already consumed
        if slot.is_some() {
            return Err(SendError(value));
        }

        // Store value
        *slot = Some(value);
        Ok(())
    }

    /// Check if receiver is still alive
    ///
    /// # Returns
    ///
    /// `true` if receiver exists and hasn't consumed the value
    pub fn has_receiver(&self) -> bool {
        self.shared.receiver_alive.load(Ordering::Relaxed) &&
        !self.shared.consumed.load(Ordering::Relaxed)
    }
}

/// Receiving end of the one-shot channel
///
/// Can receive exactly one message.
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.receiver_alive.store(false, Ordering::Relaxed);
    }
}

impl<T: Send> Receiver<T> {
    /// Try to receive the value from the one-shot channel
    ///
    /// # Returns
    ///
    /// - `Ok(value)` if value was received
    /// - `Err(RecvError::Disconnected)` if sender dropped without sending
    /// - `Err(RecvError::Empty)` if no value available yet but sender exists
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_io::channel::oneshot;
    ///
    /// let (tx, rx) = oneshot::channel();
    /// tx.send(42).unwrap();
    ///
    /// assert_eq!(rx.recv().unwrap(), 42);
    ///
    /// // Second receive fails
    /// assert!(rx.recv().is_err());
    /// ```
    pub fn recv(&self) -> Result<T, RecvError> {
        let mut slot = self.shared.value.lock().unwrap();

        if let Some(value) = slot.take() {
            // Mark as consumed
            self.shared.consumed.store(true, Ordering::Relaxed);
            return Ok(value);
        }

        // Check if already consumed or sender gone
        if self.shared.consumed.load(Ordering::Relaxed) ||
           !self.shared.sender_alive.load(Ordering::Relaxed) {
            Err(RecvError::Disconnected)
        } else {
            Err(RecvError::Empty)
        }
    }

    /// Check if sender is still alive
    ///
    /// # Returns
    ///
    /// `true` if sender exists and hasn't sent yet
    pub fn has_sender(&self) -> bool {
        self.shared.sender_alive.load(Ordering::Relaxed) &&
        self.shared.value.lock().unwrap().is_none()
    }

    /// Check if value has been sent
    ///
    /// # Returns
    ///
    /// `true` if a value is available (even if not yet received)
    pub fn is_ready(&self) -> bool {
        self.shared.value.lock().unwrap().is_some()
    }

    /// Check if value has been consumed
    ///
    /// # Returns
    ///
    /// `true` if recv() has already been called successfully
    pub fn is_consumed(&self) -> bool {
        self.shared.consumed.load(Ordering::Relaxed)
    }
}

/// Shared state for one-shot channel
struct Shared<T> {
    /// The value (None = not sent, Some = sent)
    value: std::sync::Mutex<Option<T>>,

    /// Whether sender is alive
    sender_alive: AtomicBool,

    /// Whether receiver is alive
    receiver_alive: AtomicBool,

    /// Whether value has been consumed
    consumed: AtomicBool,
}

/// Error returned when sending fails
#[derive(Debug, PartialEq, Eq)]
pub struct SendError<T>(pub T);

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "send failed: receiver gone or already consumed")
    }
}

impl<T: std::fmt::Debug> std::error::Error for SendError<T> {}

/// Error returned when receiving fails
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// Sender dropped without sending
    Disconnected,
    /// No value available yet but sender exists
    Empty,
}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvError::Disconnected => write!(f, "receive failed: sender disconnected"),
            RecvError::Empty => write!(f, "receive failed: no value available"),
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
        assert!(!rx.is_ready());
        assert!(!rx.is_consumed());
    }

    #[test]
    fn test_send_and_recv() {
        let (tx, rx) = channel();
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
        assert!(rx.is_consumed());
    }

    #[test]
    fn test_recv_before_send() {
        let (tx, rx) = channel();
        assert_eq!(rx.recv(), Err(RecvError::Empty));
        assert!(rx.has_sender());

        // Now send
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
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
    fn test_second_recv_fails() {
        let (tx, rx) = channel();
        tx.send(42).unwrap();

        assert_eq!(rx.recv().unwrap(), 42);
        assert!(rx.is_consumed());

        // Second recv fails
        assert_eq!(rx.recv(), Err(RecvError::Disconnected));
    }

    #[test]
    fn test_second_send_fails() {
        let (tx, rx) = channel();
        tx.send(42).unwrap();

        // Second send fails
        assert_eq!(tx.send(100), Err(SendError(100)));

        // But first value is still there
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn test_is_ready() {
        let (tx, rx) = channel();

        assert!(!rx.is_ready());

        tx.send(42).unwrap();
        assert!(rx.is_ready());

        rx.recv().unwrap();
        assert!(!rx.is_ready()); // Consumed
    }

    #[test]
    fn test_sender_not_cloneable() {
        let (tx, _rx): (Sender<i32>, Receiver<i32>) = channel();
        // let tx2 = tx.clone(); // Would fail
        let _ = tx;
    }

    #[test]
    fn test_receiver_not_cloneable() {
        let (tx, rx): (Sender<i32>, Receiver<i32>) = channel();
        let _ = tx;
        // let rx2 = rx.clone(); // Would fail
        let _ = rx;
    }
}
