// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Result type: Represents either success (Ok) or error (Err)

use crate::traits::{PartialEq, Clone, Copy};

/// Represents a result that can be either success (Ok) or error (Err)
#[derive(Debug)]
pub enum Outcome<T, E> {
    /// Success with value T
    Ok(T),
    /// Error with value E
    Err(E),
}

impl<T: Copy, E: Copy> Copy for Outcome<T, E> {}

impl<T: Clone, E: Clone> Clone for Outcome<T, E> {
    fn clone(&self) -> Self {
        match self {
            Outcome::Ok(x) => Outcome::Ok(x.clone()),
            Outcome::Err(e) => Outcome::Err(e.clone()),
        }
    }
}

impl<T: PartialEq, E: PartialEq> PartialEq for Outcome<T, E> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Outcome::Ok(a), Outcome::Ok(b)) => a.eq(b),
            (Outcome::Err(e1), Outcome::Err(e2)) => e1.eq(e2),
            _ => false,
        }
    }
}

impl<T, E> Outcome<T, E> {
    pub fn is_ok(&self) -> bool {
        match self {
            Outcome::Ok(_) => true,
            Outcome::Err(_) => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn as_ref(&self) -> Outcome<&T, &E> {
        match self {
            Outcome::Ok(x) => Outcome::Ok(x),
            Outcome::Err(e) => Outcome::Err(e),
        }
    }

    pub fn expect(self, msg: &str) -> T {
        match self {
            Outcome::Ok(x) => x,
            Outcome::Err(_) => panic!("{}", msg),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Outcome::Ok(x) => x,
            Outcome::Err(_) => panic!("called `Outcome::unwrap()` on an `Err` value"),
        }
    }

    pub fn unwrap_err(self) -> E {
        match self {
            Outcome::Ok(_) => panic!("called `Outcome::unwrap_err()` on an `Ok` value"),
            Outcome::Err(e) => e,
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Outcome::Ok(x) => x,
            Outcome::Err(_) => default,
        }
    }

    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Outcome::Ok(x) => x,
            Outcome::Err(e) => f(e),
        }
    }

    pub fn map<U, F>(self, f: F) -> Outcome<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Outcome::Ok(x) => Outcome::Ok(f(x)),
            Outcome::Err(e) => Outcome::Err(e),
        }
    }

    pub fn map_err<F, O>(self, f: F) -> Outcome<T, O>
    where
        F: FnOnce(E) -> O,
    {
        match self {
            Outcome::Ok(x) => Outcome::Ok(x),
            Outcome::Err(e) => Outcome::Err(f(e)),
        }
    }

    pub fn and_then<U, F>(self, f: F) -> Outcome<U, E>
    where
        F: FnOnce(T) -> Outcome<U, E>,
    {
        match self {
            Outcome::Ok(x) => f(x),
            Outcome::Err(e) => Outcome::Err(e),
        }
    }

    pub fn or<F>(self, resb: Outcome<T, F>) -> Outcome<T, F> {
        match self {
            Outcome::Ok(_) => Outcome::Ok(self.unwrap()),
            Outcome::Err(_) => resb,
        }
    }

    pub fn or_else<F, O>(self, f: F) -> Outcome<T, O>
    where
        F: FnOnce(E) -> Outcome<T, O>,
    {
        match self {
            Outcome::Ok(x) => Outcome::Ok(x),
            Outcome::Err(e) => f(e),
        }
    }
}
