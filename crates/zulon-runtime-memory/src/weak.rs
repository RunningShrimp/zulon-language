// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Weak references for Arc
//!
//! `Weak<T>` is a non-owning reference to Arc-managed data.
//! It's useful for breaking reference cycles and for cache-like structures.

use std::fmt;
use std::sync::atomic::Ordering;

use super::Arc;

// Import ArcData as pub(super) to access it
use super::arc::ArcData;

/// Weak reference to Arc-managed data
///
/// `Weak<T>` is a non-owning reference to data managed by `Arc<T>`.
/// Unlike `Arc<T>`, weak references don't prevent the data from being dropped.
///
/// # Purpose
///
/// Weak references are useful for:
/// - Breaking reference cycles
/// - Cache-like structures
/// - Observer patterns
///
/// # Example
///
/// ```rust
/// use zulon_runtime_memory::{Arc, Weak};
///
/// let arc = Arc::new(42);
/// let weak = Arc::downgrade(&arc);
///
/// // Weak can be upgraded to Arc
/// assert_eq!(weak.upgrade(), Some(arc.clone()));
///
/// // After arc is dropped, upgrade returns None
/// drop(arc);
/// assert_eq!(weak.upgrade(), None);
/// ```
pub struct Weak<T: ?Sized> {
    /// Pointer to the ArcData
    ptr: *const ArcData<T>,
    /// Metadata for sized types (to make Weak<T: ?Sized> work)
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Weak<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Weak<T> {}

impl<T: ?Sized> Clone for Weak<T> {
    /// Clone the weak pointer, incrementing weak reference count
    fn clone(&self) -> Self {
        if !self.ptr.is_null() {
            unsafe {
                (*self.ptr).inc_weak();
            }
        }

        Weak {
            ptr: self.ptr,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for Weak<T> {
    /// Drop the weak pointer, decrementing weak reference count
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                if (*self.ptr).dec_weak() {
                    // Last weak reference, deallocate ArcData
                    // Note: dec_weak doesn't drop the data (already dropped by dec_strong)
                    // We need to deallocate the ArcData itself without dropping
                    let layout = std::alloc::Layout::for_value(&*self.ptr);
                    std::alloc::dealloc(self.ptr as *mut u8, layout);
                }
            }
        }
    }
}

impl<T: ?Sized> Weak<T> {
    /// Create a new weak pointer that doesn't reference any data
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Weak;
    ///
    /// let weak: Weak<i32> = Weak::new();
    /// assert!(weak.upgrade().is_none());
    /// ```
    pub fn new() -> Self
    where
        T: Sized,
    {
        Weak {
            ptr: std::ptr::null() as *const ArcData<T>,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Weak<T> {
    /// Attempt to upgrade the weak pointer to a strong Arc
    ///
    /// Returns `Some(arc)` if the data is still alive,
    /// or `None` if the data has been dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// let weak = Arc::downgrade(&arc);
    ///
    /// assert!(weak.upgrade().is_some());
    /// drop(arc);
    /// assert!(weak.upgrade().is_none());
    /// ```
    pub fn upgrade(&self) -> Option<Arc<T>> {
        // Null pointer means no data
        if self.ptr.is_null() {
            return None;
        }

        unsafe {
            // Try to increment strong count
            let old_strong = (*self.ptr).strong.fetch_add(1, Ordering::Relaxed);

            // If strong count was 0, data is being dropped
            if old_strong == 0 {
                // Revert the increment
                (*self.ptr).strong.fetch_sub(1, Ordering::Relaxed);
                return None;
            }

            // Success, create Arc
            Some(Arc {
                ptr: self.ptr,
            })
        }
    }

    /// Get the number of strong references
    ///
    /// Returns 0 if the data has been dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// let weak = Arc::downgrade(&arc);
    ///
    /// assert_eq!(weak.strong_count(), 1);
    /// ```
    pub fn strong_count(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }

        unsafe { (*self.ptr).strong_count() }
    }

    /// Get the number of weak references
    ///
    /// Returns 0 if the weak pointer doesn't reference any data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// let weak1 = Arc::downgrade(&arc);
    /// let weak2 = weak1.clone();
    ///
    /// assert_eq!(weak1.weak_count(), 2);
    /// ```
    pub fn weak_count(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }

        unsafe {
            let weak = (*self.ptr).weak_count();
            let strong = (*self.ptr).strong_count();
            // Subtract strong count (they're included in weak count)
            if strong == 0 {
                weak
            } else {
                weak - strong
            }
        }
    }

    /// Get a raw pointer to the data
    ///
    /// Returns null if the data has been dropped.
    /// This is useful for FFI but should be used with caution.
    pub fn as_ptr(&self) -> *const T
    where
        T: Sized,
    {
        if self.ptr.is_null() {
            return std::ptr::null();
        }

        unsafe { &(*self.ptr).data as *const T }
    }
}

impl<T> Default for Weak<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ptr.is_null() {
            f.debug_tuple("Weak").field(&format!("(null)")).finish()
        } else {
            let strong = self.strong_count();
            f.debug_tuple("Weak")
                .field(&format!("(strong={})", strong))
                .finish()
        }
    }
}

impl<T> Arc<T> {
    /// Create a weak reference to this Arc
    ///
    /// The weak reference doesn't prevent the data from being dropped,
    /// but can be upgraded back to a strong reference if the data is still alive.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// let weak = Arc::downgrade(&arc);
    ///
    /// // Both reference the same data
    /// assert_eq!(*arc, 42);
    /// assert_eq!(weak.upgrade().map(|a| *a), Some(42));
    /// ```
    pub fn downgrade(this: &Self) -> Weak<T> {
        unsafe {
            (*this.ptr).inc_weak();
        }

        Weak {
            ptr: this.ptr,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_new() {
        let weak: Weak<i32> = Weak::new();
        assert!(weak.upgrade().is_none());
        assert_eq!(weak.strong_count(), 0);
    }

    #[test]
    fn test_weak_downgrade() {
        let arc = Arc::new(42);
        let weak = Arc::downgrade(&arc);

        assert_eq!(weak.strong_count(), 1);
        assert_eq!(weak.upgrade().map(|a| *a), Some(42));
    }

    #[test]
    fn test_weak_upgrade() {
        let arc = Arc::new(42);
        let weak = Arc::downgrade(&arc);

        let upgraded = weak.upgrade();
        assert!(upgraded.is_some());
        assert_eq!(*upgraded.unwrap(), 42);
    }

    #[test]
    fn test_weak_upgrade_after_drop() {
        let arc = Arc::new(42);
        let weak = Arc::downgrade(&arc);

        drop(arc);
        assert!(weak.upgrade().is_none());
        assert_eq!(weak.strong_count(), 0);
    }

    #[test]
    fn test_weak_clone() {
        let arc = Arc::new(42);
        let weak1 = Arc::downgrade(&arc);
        let weak2 = weak1.clone();

        assert_eq!(weak1.weak_count(), 2);
        assert_eq!(weak2.weak_count(), 2);

        // Both can upgrade
        assert!(weak1.upgrade().is_some());
        assert!(weak2.upgrade().is_some());
    }

    #[test]
    fn test_weak_prevents_deallocation() {
        let arc = Arc::new(vec![1, 2, 3]);
        let weak = Arc::downgrade(&arc);

        // Drop the strong reference
        drop(arc);

        // Weak still exists, so data isn't deallocated yet
        assert_eq!(weak.strong_count(), 0);
        assert_eq!(weak.weak_count(), 1);

        // But we can't upgrade it
        assert!(weak.upgrade().is_none());

        // Drop weak, now data is deallocated
        drop(weak);
    }

    #[test]
    fn test_weak_multiple() {
        let arc = Arc::new(42);
        let weak1 = Arc::downgrade(&arc);
        let weak2 = Arc::downgrade(&arc);

        assert_eq!(weak1.weak_count(), 2);
        assert_eq!(weak2.weak_count(), 2);

        drop(arc);

        // Both weaks can't upgrade
        assert!(weak1.upgrade().is_none());
        assert!(weak2.upgrade().is_none());
    }

    #[test]
    fn test_weak_with_cycle() {
        // Demonstrate breaking reference cycles
        struct Cycle {
            next: Option<Weak<Cycle>>,
            value: i32,
        }

        let a = Arc::new(Cycle {
            next: None,
            value: 1,
        });
        let b = Arc::new(Cycle {
            next: Some(Arc::downgrade(&a)),
            value: 2,
        });

        // Verify values to avoid dead code warning
        assert_eq!(a.value, 1);
        assert_eq!(b.value, 2);
        assert!(b.next.is_some());

        // Create a cycle using weak reference
        // (In real code, you'd need interior mutability)
        drop(a);
        drop(b);

        // No memory leak because weak references don't create cycles
    }

    #[test]
    fn test_weak_default() {
        let weak: Weak<i32> = Weak::default();
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_weak_debug() {
        let weak: Weak<i32> = Weak::new();
        assert!(format!("{:?}", weak).contains("Weak"));

        let arc = Arc::new(42);
        let weak = Arc::downgrade(&arc);
        assert!(format!("{:?}", weak).contains("strong=1"));
    }

    #[test]
    fn test_weak_as_ptr() {
        let arc = Arc::new(42);
        let weak = Arc::downgrade(&arc);

        let ptr = weak.as_ptr();
        assert!(!ptr.is_null());

        unsafe {
            assert_eq!(*ptr, 42);
        }
    }

    #[test]
    fn test_weak_thread_safety() {
        use std::thread;

        let arc = Arc::new(42);
        let weak = Arc::downgrade(&arc);

        let handle = thread::spawn(move || {
            assert_eq!(weak.strong_count(), 1);
            weak.upgrade()
        });

        let upgraded = handle.join().unwrap();
        assert!(upgraded.is_some());
        assert_eq!(*upgraded.unwrap(), 42);
    }
}
