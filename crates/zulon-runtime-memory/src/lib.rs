// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Runtime Memory Management
//!
//! This crate provides memory management primitives for ZULON programs:
//!
//! - **`Arc<T>`** - Atomic Reference Counting for shared ownership
//! - **`Weak<T>`** - Weak reference for breaking reference cycles
//! - **Memory safety** - Tree borrows model integration
//!
//! ## Architecture
//!
//! ZULON uses Automatic Reference Counting (ARC) for memory management:
//! - Each `Arc<T>` instance points to dynamically allocated data
//! - A reference counter tracks how many `Arc` pointers exist
//! - When the last `Arc` is dropped, the data is automatically freed
//! - `Weak<T>` references don't increment the strong count
//!
//! ## Thread Safety
//!
//! `Arc<T>` uses atomic operations for thread-safe reference counting:
//! - Multiple threads can safely share `Arc<T>` instances
//! - Reference counts use atomic fetch_add/fetch_sub operations
//! - Compatible with multi-threaded ZULON programs
//!
//! ## Example
//!
//! ```rust
//! use zulon_runtime_memory::Arc;
//!
//! // Create a new Arc
//! let arc = Arc::new(42);
//!
//! // Clone creates another pointer to the same data
//! let another = arc.clone();
//!
//! // Both pointers access the same value
//! assert_eq!(*arc, 42);
//! assert_eq!(*another, 42);
//!
//! // Data is freed when both Arcs are dropped
//! ```

mod arc;
mod weak;

pub use arc::Arc;
pub use weak::Weak;

/// Error type for Arc operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArcError {
    /// Reference count overflow
    Overflow,
    /// Attempted to upgrade a Weak pointer that was already dropped
    InvalidWeak,
}

impl std::fmt::Display for ArcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArcError::Overflow => write!(f, "Arc reference count overflow"),
            ArcError::InvalidWeak => write!(f, "Weak pointer was already dropped"),
        }
    }
}

impl std::error::Error for ArcError {}

/// Result type for Arc operations
pub type Result<T> = std::result::Result<T, ArcError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_error_display() {
        let err = ArcError::Overflow;
        assert_eq!(format!("{}", err), "Arc reference count overflow");

        let err = ArcError::InvalidWeak;
        assert_eq!(format!("{}", err), "Weak pointer was already dropped");
    }
}
