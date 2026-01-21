// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type adapter for Result <-> Outcome conversion
//!
//! This module provides type adapters to convert between zulon-runtime-io's
//! `Result<T, E>` and zulon-std-core's `Outcome<T, E>` types.
//!
//! ## Example
//!
//! ```rust
//! use zulon_std_core::Outcome;
//! use zulon_runtime_io::result::RuntimeResult;
//!
//! // Convert Outcome to RuntimeResult
//! let outcome: Outcome<String, IoError> = Outcome::Ok("hello".to_string());
//! let result: RuntimeResult<String, IoError> = outcome.into_runtime_result();
//! ```

use crate::fs::error::IoError;

/// Type alias for Result to make code clearer
pub type RuntimeResult<T, E> = Result<T, E>;

/// Adapter trait to convert Outcome to Result
pub trait IntoRuntimeResult<T, E> {
    /// Converts an Outcome into a RuntimeResult
    fn into_runtime_result(self) -> RuntimeResult<T, E>;
}

impl<T, E> IntoRuntimeResult<T, E> for crate::Outcome<T, E> {
    fn into_runtime_result(self) -> RuntimeResult<T, E> {
        match self {
            crate::Outcome::Ok(v) => Ok(v),
            crate::Outcome::Err(e) => Err(e),
        }
    }
}

/// Adapter trait to convert Result to Outcome
pub trait IntoOutcome<T, E> {
    /// Converts a RuntimeResult into an Outcome
    fn into_outcome(self) -> Outcome<T, E>;
}

impl<T, E> IntoOutcome<T, E> for Result<T, E> {
    fn into_outcome(self) -> Outcome<T, E> {
        self.map_err(IoError::from)
    }
}

// Extension traits to make conversion ergonomic
impl<T, E> crate::Outcome<T, E> {
    /// Converts Outcome to RuntimeResult with context
    pub fn to_runtime_result(self) -> RuntimeResult<T, E> {
        self.into_runtime_result()
    }

    /// Converts RuntimeResult to Result with context
    pub fn from_runtime_result(result: RuntimeResult<T, E>) -> Self {
        match result {
            Ok(v) => crate::Outcome::Ok(v),
            Err(e) => crate::Outcome::Err(e),
        }
    }
}

impl<T, E> Result<T, E> {
    /// Converts Result to Outcome with context
    pub fn to_outcome(self) -> Outcome<T, E> {
        self.map_err(IoError::from)
    }

    /// Converts Outcome to Result with context  
    pub fn from_outcome(outcome: crate::Outcome<T, E>) -> Self {
        match outcome {
            crate::Outcome::Ok(v) => Ok(v),
            crate::Outcome::Err(e) => Err(e),
        }
    }
}
