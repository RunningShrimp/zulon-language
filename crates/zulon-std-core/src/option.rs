// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Optional type: Represents an optional value

use crate::traits::{PartialEq, Clone, Copy};

/// Represents a value that may or may not exist
#[derive(Debug)]
pub enum Optional<T> {
    /// Some value T exists
    Some(T),
    /// No value exists
    None,
}

impl<T: Copy> Copy for Optional<T> {}

impl<T: Clone> Clone for Optional<T> {
    fn clone(&self) -> Self {
        match self {
            Optional::Some(x) => Optional::Some(x.clone()),
            Optional::None => Optional::None,
        }
    }
}

impl<T: PartialEq> PartialEq for Optional<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => a.eq(b),
            (Optional::None, Optional::None) => true,
            _ => false,
        }
    }
}

impl<T> Optional<T> {
    pub fn is_some(&self) -> bool {
        match self {
            Optional::Some(_) => true,
            Optional::None => false,
        }
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn as_ref(&self) -> Optional<&T> {
        match self {
            Optional::Some(x) => Optional::Some(x),
            Optional::None => Optional::None,
        }
    }

    pub fn expect(self, msg: &str) -> T {
        match self {
            Optional::Some(x) => x,
            Optional::None => panic!("{}", msg),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Optional::Some(x) => x,
            Optional::None => panic!("called `Optional::unwrap()` on a `None` value"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Optional::Some(x) => x,
            Optional::None => default,
        }
    }

    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Optional::Some(x) => x,
            Optional::None => f(),
        }
    }

    pub fn map<U, F>(self, f: F) -> Optional<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Optional::Some(x) => Optional::Some(f(x)),
            Optional::None => Optional::None,
        }
    }

    pub fn and_then<U, F>(self, f: F) -> Optional<U>
    where
        F: FnOnce(T) -> Optional<U>,
    {
        match self {
            Optional::Some(x) => f(x),
            Optional::None => Optional::None,
        }
    }

    pub fn or(self, optb: Optional<T>) -> Optional<T> {
        match self {
            Optional::Some(_) => self,
            Optional::None => optb,
        }
    }

    pub fn or_else<F>(self, f: F) -> Optional<T>
    where
        F: FnOnce() -> Optional<T>,
    {
        match self {
            Optional::Some(_) => self,
            Optional::None => f(),
        }
    }

    pub fn filter<F>(self, predicate: F) -> Optional<T>
    where
        F: FnOnce(&T) -> bool,
    {
        match self {
            Optional::Some(x) if predicate(&x) => Optional::Some(x),
            _ => Optional::None,
        }
    }

    pub fn zip<U>(self, other: Optional<U>) -> Optional<(T, U)> {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => Optional::Some((a, b)),
            _ => Optional::None,
        }
    }
}
