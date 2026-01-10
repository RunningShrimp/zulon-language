//! Continuation Management
//!
//! This module provides continuation capture and restoration for async operations.
//! When an async operation is performed, the current execution state (continuation)
//! is captured and later restored when the operation completes.

use std::collections::HashMap;

/// Continuation ID type
pub type ContinuationId = u64;

/// Unique continuation ID counter
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_id() -> ContinuationId {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// A continuation represents the captured execution state at a point in time
///
/// In a full implementation, this would contain:
/// - Stack pointer
/// - Register state
/// - Instruction pointer
/// - Captured variables
///
/// For now, this is a placeholder that will be integrated with the MIR/LIR layers.
#[derive(Debug, Clone)]
pub struct Continuation {
    /// Unique ID for this continuation
    id: ContinuationId,
    /// Associated operation (if any)
    operation: Option<String>,
}

impl Continuation {
    /// Create a new continuation
    pub fn new(operation: Option<String>) -> Self {
        Self {
            id: next_id(),
            operation,
        }
    }

    /// Get the continuation ID
    pub fn id(&self) -> ContinuationId {
        self.id
    }

    /// Get the associated operation
    pub fn operation(&self) -> Option<&str> {
        self.operation.as_deref()
    }
}

/// Continuation manager
///
/// The continuation manager stores and retrieves continuations during async operations.
pub struct ContinuationManager {
    /// Active continuations indexed by ID
    continuations: HashMap<ContinuationId, Continuation>,
}

impl Default for ContinuationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuationManager {
    /// Create a new continuation manager
    pub fn new() -> Self {
        Self {
            continuations: HashMap::new(),
        }
    }

    /// Capture the current continuation
    ///
    /// This will be called from compiler-generated code when an async operation
    /// is performed.
    pub fn capture(&mut self, operation: Option<String>) -> ContinuationId {
        let continuation = Continuation::new(operation);
        let id = continuation.id();
        self.continuations.insert(id, continuation);
        id
    }

    /// Restore a continuation by ID
    ///
    /// This will be called when an async operation completes.
    pub fn restore(&mut self, id: ContinuationId) -> Option<Continuation> {
        self.continuations.remove(&id)
    }

    /// Check if a continuation exists
    pub fn contains(&self, id: ContinuationId) -> bool {
        self.continuations.contains_key(&id)
    }

    /// Get the number of active continuations
    pub fn len(&self) -> usize {
        self.continuations.len()
    }

    /// Check if there are no active continuations
    pub fn is_empty(&self) -> bool {
        self.continuations.is_empty()
    }

    /// Remove all continuations
    pub fn clear(&mut self) {
        self.continuations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuation_creation() {
        let cont = Continuation::new(Some("test_op".to_string()));
        assert!(cont.id() > 0);
        assert_eq!(cont.operation(), Some("test_op"));
    }

    #[test]
    fn test_continuation_manager() {
        let mut manager = ContinuationManager::new();
        assert!(manager.is_empty());

        let id1 = manager.capture(Some("op1".to_string()));
        let id2 = manager.capture(Some("op2".to_string()));

        assert_eq!(manager.len(), 2);
        assert!(manager.contains(id1));
        assert!(manager.contains(id2));

        let restored = manager.restore(id1);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().operation(), Some("op1"));
        assert_eq!(manager.len(), 1);

        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_unique_ids() {
        let mut manager = ContinuationManager::new();
        let id1 = manager.capture(None);
        let id2 = manager.capture(None);
        assert_ne!(id1, id2);
    }
}
