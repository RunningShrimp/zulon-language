// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Atomic Reference Counting (ARC) smart pointer
//!
//! `Arc<T>` provides shared ownership of a value across threads.
//! Multiple `Arc` pointers can point to the same data, and the data
//! is automatically freed when the last `Arc` is dropped.

use std::sync::atomic;
use std::sync::atomic::Ordering;
use std::fmt;

/// Internal data structure for Arc
///
/// Contains the reference counts and the actual data
pub(super) struct ArcData<T: ?Sized> {
    /// Number of strong references (Arc pointers)
    pub(super) strong: atomic::AtomicUsize,
    /// Number of weak references
    pub(super) weak: atomic::AtomicUsize,
    /// The actual data
    pub(super) data: T,
}

impl<T: ?Sized> ArcData<T> {
    /// Create new ArcData with initial reference counts
    fn new(data: T) -> Self
    where
        T: Sized,
    {
        ArcData {
            strong: atomic::AtomicUsize::new(1), // Start with 1 strong reference
            weak: atomic::AtomicUsize::new(1),   // Weak count includes strong holders
            data,
        }
    }

    /// Increment strong reference count
    fn inc_strong(&self) {
        self.strong.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement strong reference count
    /// Returns true if this was the last strong reference
    fn dec_strong(&self) -> bool {
        // Use AcqRel ordering to ensure all writes to data are visible
        // to the thread that deallocates it
        let prev = self.strong.fetch_sub(1, Ordering::AcqRel);

        if prev == 1 {
            // We were the last strong reference
            // Drop the data
            unsafe {
                std::ptr::drop_in_place(&self.data as *const T as *mut T);
            }

            // Now decrement weak count (was keeping the allocation alive)
            let prev_weak = self.weak.fetch_sub(1, Ordering::AcqRel);
            if prev_weak == 1 {
                // Last weak reference too, deallocate the ArcData
                // Note: data has already been dropped above
                return true;
            }
        }

        false
    }

    /// Increment weak reference count
    pub(super) fn inc_weak(&self) {
        self.weak.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement weak reference count
    /// Returns true if this was the last reference (strong or weak)
    pub(super) fn dec_weak(&self) -> bool {
        let prev = self.weak.fetch_sub(1, Ordering::AcqRel);
        prev == 1
    }

    /// Get current strong reference count
    pub(super) fn strong_count(&self) -> usize {
        self.strong.load(Ordering::Relaxed)
    }

    /// Get current weak reference count
    pub(super) fn weak_count(&self) -> usize {
        self.weak.load(Ordering::Relaxed)
    }
}

/// Atomic Reference Counted pointer
///
/// `Arc<T>` provides shared ownership of a value of type `T`.
/// Multiple `Arc` pointers can point to the same data, which is
/// automatically freed when the last `Arc` pointer is dropped.
///
/// # Thread Safety
///
/// `Arc<T>` uses atomic operations for thread-safe reference counting,
/// allowing safe sharing across threads.
///
/// # Example
///
/// ```rust
/// use zulon_runtime_memory::Arc;
///
/// let five = Arc::new(5);
///
/// // Clone creates another pointer to the same data
/// let five_clone = Arc::clone(&five);
///
/// // Both point to the same value
/// assert_eq!(*five, 5);
/// assert_eq!(*five_clone, 5);
/// ```
pub struct Arc<T: ?Sized> {
    /// Pointer to the ArcData
    pub(super) ptr: *const ArcData<T>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    /// Create a new Arc value
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// assert_eq!(*arc, 42);
    /// ```
    pub fn new(data: T) -> Self {
        // Allocate ArcData on the heap
        let boxed = Box::new(ArcData::new(data));

        Arc {
            ptr: Box::leak(boxed) as *const ArcData<T>,
        }
    }

    /// Convert the Arc into a unique owner
    ///
    /// This consumes the Arc and returns the owned value if there
    /// is exactly one strong reference. Otherwise, returns the Arc back.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// assert_eq!(Arc::try_unwrap(arc), Ok(42));
    /// ```
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        // Check if we're the only strong reference
        if Arc::strong_count(&this) == 1 {
            // Prevent the Arc from dropping the ArcData
            let ptr = this.ptr;
            std::mem::forget(this);

            // Extract the data and deallocate the ArcData
            unsafe {
                // Read the data out
                let data = (&(*ptr).data as *const T as *mut T).read();

                // Deallocate the ArcData without dropping its contents
                // (we already moved the data out)
                let layout = std::alloc::Layout::for_value(&*ptr);
                std::alloc::dealloc(ptr as *mut u8, layout);

                Ok(data)
            }
        } else {
            Err(this)
        }
    }
}

impl<T: ?Sized> Arc<T> {
    /// Get a reference to the inner data
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// assert_eq!(*Arc::inner(&arc), 42);
    /// ```
    pub fn inner(this: &Self) -> &T {
        unsafe { &(*this.ptr).data }
    }

    /// Get the number of strong references to this data
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// assert_eq!(Arc::strong_count(&arc), 1);
    ///
    /// let _clone = arc.clone();
    /// assert_eq!(Arc::strong_count(&arc), 2);
    /// ```
    pub fn strong_count(this: &Self) -> usize {
        unsafe { (*this.ptr).strong_count() }
    }

    /// Get the number of weak references to this data
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// assert_eq!(Arc::weak_count(&arc), 0);
    /// ```
    pub fn weak_count(this: &Self) -> usize {
        unsafe {
            let weak = (*this.ptr).weak_count();
            let strong = (*this.ptr).strong_count();
            // Subtract strong count (they're included in weak count)
            // Use saturating_sub to handle edge cases
            weak.saturating_sub(strong)
        }
    }

    /// Get a raw pointer to the data
    ///
    /// This is useful for FFI but should be used with caution.
    pub fn as_ptr(this: &Self) -> *const T {
        unsafe { &(*this.ptr).data as *const T }
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    /// Create a new Arc pointer to the same data
    ///
    /// This increments the strong reference count.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// let clone = arc.clone();
    ///
    /// assert_eq!(Arc::strong_count(&arc), 2);
    /// assert_eq!(*arc, *clone);
    /// ```
    fn clone(&self) -> Self {
        unsafe {
            (*self.ptr).inc_strong();
        }

        Arc {
            ptr: self.ptr,
        }
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    /// Drop the Arc, decrementing the strong reference count
    ///
    /// If this is the last strong reference, the data is dropped.
    fn drop(&mut self) {
        unsafe {
            if (*self.ptr).dec_strong() {
                // Last reference, deallocate ArcData
                // Note: dec_strong already dropped the data
                // We need to deallocate the ArcData itself without dropping data again
                let layout = std::alloc::Layout::for_value(&*self.ptr);
                std::alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T: ?Sized> std::ops::Deref for Arc<T> {
    type Target = T;

    /// Dereference to access the inner data
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_runtime_memory::Arc;
    ///
    /// let arc = Arc::new(42);
    /// assert_eq!(*arc, 42);
    /// ```
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.ptr).data }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        // Compare by value, not by pointer
        **self == **other
    }
}

impl<T: ?Sized + Eq> Eq for Arc<T> {}

impl<T: ?Sized + fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Arc")
            .field(&&**self)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_arc_new() {
        let arc = Arc::new(42);
        assert_eq!(*arc, 42);
        assert_eq!(Arc::strong_count(&arc), 1);
    }

    #[test]
    fn test_arc_clone() {
        let arc = Arc::new(42);
        assert_eq!(Arc::strong_count(&arc), 1);

        let clone = arc.clone();
        assert_eq!(Arc::strong_count(&arc), 2);
        assert_eq!(Arc::strong_count(&clone), 2);

        assert_eq!(*arc, *clone);
    }

    #[test]
    fn test_arc_drop_decrements_count() {
        let arc = Arc::new(42);
        let clone = arc.clone();
        assert_eq!(Arc::strong_count(&arc), 2);

        drop(clone);
        assert_eq!(Arc::strong_count(&arc), 1);
    }

    #[test]
    fn test_arc_try_unwrap_success() {
        let arc = Arc::new(42);
        assert_eq!(Arc::try_unwrap(arc), Ok(42));
    }

    #[test]
    fn test_arc_try_unwrap_failure() {
        let arc = Arc::new(42);
        let _clone = arc.clone();

        assert!(Arc::try_unwrap(arc).is_err());
    }

    #[test]
    fn test_arc_weak_count() {
        let arc = Arc::new(42);
        assert_eq!(Arc::weak_count(&arc), 0);

        let _clone = arc.clone();
        assert_eq!(Arc::weak_count(&arc), 0);
    }

    #[test]
    fn test_arc_thread_safety() {
        let arc = Arc::new(42);
        let arc_clone = arc.clone();

        let handle = thread::spawn(move || {
            assert_eq!(*arc_clone, 42);
            Arc::strong_count(&arc_clone)
        });

        assert_eq!(handle.join().unwrap(), 2);
    }

    #[test]
    fn test_arc_equality() {
        let arc1 = Arc::new(42);
        let arc2 = Arc::new(42);
        assert_eq!(arc1, arc2);
    }

    #[test]
    fn test_arc_display() {
        let arc = Arc::new(42);
        assert_eq!(format!("{}", arc), "42");
    }

    #[test]
    fn test_arc_debug() {
        let arc = Arc::new(42);
        assert_eq!(format!("{:?}", arc), "Arc(42)");
    }

    #[test]
    fn test_arc_with_vec() {
        let vec = Arc::new(vec![1, 2, 3]);
        assert_eq!(*vec, vec![1, 2, 3]);

        let clone = vec.clone();
        assert_eq!(Arc::strong_count(&vec), 2);
        assert_eq!(*clone, vec![1, 2, 3]);
    }

    #[test]
    fn test_arc_as_ptr() {
        let arc = Arc::new(42);
        let ptr = Arc::as_ptr(&arc);

        unsafe {
            assert_eq!(*ptr, 42);
        }
    }
}
