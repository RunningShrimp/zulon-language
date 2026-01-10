// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Select case implementations
//!
//! This module provides case types for the select! macro, allowing
//! different channel operations to be combined.

use std::marker::PhantomData;

/// Value selected from a channel
#[derive(Debug, PartialEq, Eq)]
pub enum SelectedValue<T> {
    /// Channel has a value
    Value(T),
    /// Channel is disconnected
    Disconnected,
}

/// Error from try_recv in select context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    /// Channel is empty
    Empty,
}

/// Select case for receiving from a channel
///
/// This wraps a receiver with additional information needed
/// for select operations.
pub struct RecvCase<R, T> {
    receiver: R,
    _phantom: PhantomData<T>,
}

impl<R, T> RecvCase<R, T> {
    /// Create a new receive case
    pub fn new(receiver: R) -> Self {
        Self {
            receiver,
            _phantom: PhantomData,
        }
    }

    /// Get the inner receiver
    pub fn into_inner(self) -> R {
        self.receiver
    }

    /// Get a reference to the inner receiver
    pub fn get_ref(&self) -> &R {
        &self.receiver
    }
}

/// Select case for sending to a channel
///
/// This wraps a sender and value for select operations.
pub struct SendCase<S, T> {
    sender: S,
    value: Option<T>,
}

impl<S, T> SendCase<S, T> {
    /// Create a new send case
    pub fn new(sender: S, value: T) -> Self {
        Self {
            sender,
            value: Some(value),
        }
    }

    /// Get the inner sender
    pub fn into_inner(self) -> Option<(S, T)> {
        self.value.map(|v| (self.sender, v))
    }

    /// Get a reference to the inner sender
    pub fn get_ref(&self) -> &S {
        &self.sender
    }

    /// Get a reference to the value
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

/// Generic select case enum
///
/// Represents different types of operations in a select statement.
pub enum Case {
    /// Receive case
    Recv,
    /// Send case
    Send,
    /// Timeout case
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::mpsc;

    #[test]
    fn test_recv_case_creation() {
        let (_tx, rx) = mpsc::channel::<i32>();
        let _case: RecvCase<_, i32> = RecvCase::new(&rx);
    }

    #[test]
    fn test_recv_case_into_inner() {
        let (tx, rx) = mpsc::channel::<i32>();
        let case: RecvCase<_, i32> = RecvCase::new(rx);
        let _rx = case.into_inner();
        // Can still use tx
        tx.send(42).unwrap();
    }

    #[test]
    fn test_send_case_creation() {
        let (tx, _rx) = mpsc::channel::<i32>();
        let case = SendCase::new(&tx, 42);
        assert!(case.get_ref().has_receivers());
        assert_eq!(case.value(), Some(&42));
    }

    #[test]
    fn test_send_case_into_inner() {
        let (tx, rx) = mpsc::channel::<i32>();
        let case = SendCase::new(tx, 42);
        let (tx, value) = case.into_inner().unwrap();
        tx.send(value).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn test_selected_value() {
        let value = SelectedValue::Value(42);
        assert_eq!(value, SelectedValue::Value(42));

        let disconnected = SelectedValue::<i32>::Disconnected;
        assert_eq!(disconnected, SelectedValue::Disconnected);
    }

    #[test]
    fn test_try_recv_error() {
        let empty = TryRecvError::Empty;
        assert_eq!(empty, TryRecvError::Empty);
    }
}
