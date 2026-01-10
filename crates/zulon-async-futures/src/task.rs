// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level task waking primitives
//!
//! This module contains the low-level building blocks for waking tasks.
//! Most users should use the Waker type instead of RawWaker directly.

/// A virtual function pointer table (vtable) for RawWaker operations
///
/// This vtable defines the behavior of a RawWaker. Each RawWaker contains
/// a data pointer and a pointer to this vtable.
///
/// # Safety
///
/// The functions in this vtable must uphold these contracts:
/// - `clone`: Creates a new RawWaker with the same data pointer
/// - `wake`: Wakes the task and consumes the RawWaker
/// - `wake_by_ref`: Wakes the task without consuming the RawWaker
/// - `drop`: Drops the RawWaker without waking
///
/// The data pointer must be valid for the lifetime of the RawWaker.
#[derive(Clone, Copy)]
pub struct RawWakerVTable {
    /// Clone this RawWaker
    ///
    /// # Safety
    ///
    /// This function must return a new RawWaker that points to the same
    /// conceptual task as the original.
    pub clone: unsafe fn(*const ()) -> RawWaker,

    /// Wake the task and consume the RawWaker
    ///
    /// # Safety
    ///
    /// This function must wake the task and clean up any resources
    /// associated with the RawWaker. The data pointer will not be used
    /// after this call.
    pub wake: unsafe fn(*const ()),

    /// Wake the task without consuming the RawWaker
    ///
    /// # Safety
    ///
    /// This function must wake the task but keep the RawWaker valid
    /// for future use.
    pub wake_by_ref: unsafe fn(*const ()),
    /// Drop the RawWaker without waking
    ///
    /// # Safety
    ///
    /// This function must clean up any resources associated with the
    /// RawWaker. The data pointer will not be used after this call.
    pub drop: unsafe fn(*const ()),
}

unsafe impl Send for RawWakerVTable {}
unsafe impl Sync for RawWakerVTable {}

/// A RawWaker is the low-level primitive for waking tasks
///
/// Most users should use Waker instead. RawWaker is useful for:
/// - Implementing custom executors
/// - Interfacing with foreign event loops
/// - Zero-allocation wakers
///
/// # Safety
///
/// The RawWaker must uphold the safety contracts of the vtable functions.
#[derive(Clone, Copy)]
pub struct RawWaker {
    /// Data pointer for the waker
    data: *const (),

    /// Vtable for the waker operations
    vtable: &'static RawWakerVTable,
}

impl RawWaker {
    /// Create a new RawWaker from a data pointer and vtable
    ///
    /// # Safety
    ///
    /// The data pointer must be valid for the lifetime of the RawWaker.
    /// The vtable functions must uphold their safety contracts when called
    /// with this data pointer.
    #[inline]
    pub const fn new(data: *const (), vtable: &'static RawWakerVTable) -> Self {
        Self { data, vtable }
    }

    /// Get the data pointer
    #[inline]
    pub const fn data(&self) -> *const () {
        self.data
    }

    /// Get the vtable
    #[inline]
    pub const fn vtable(&self) -> &'static RawWakerVTable {
        self.vtable
    }

    /// Clone this RawWaker
    ///
    /// # Safety
    ///
    /// Calls the clone function from the vtable. The caller must ensure
    /// the data pointer is still valid.
    #[inline]
    pub unsafe fn clone(&self) -> Self {
        unsafe { (self.vtable.clone)(self.data) }
    }

    /// Wake the task and consume the RawWaker
    ///
    /// # Safety
    ///
    /// Calls the wake function from the vtable. After this call, the
    /// RawWaker is consumed and should not be used again.
    #[inline]
    pub unsafe fn wake(self) {
        unsafe { (self.vtable.wake)(self.data) }
    }

    /// Wake the task without consuming the RawWaker
    ///
    /// # Safety
    ///
    /// Calls the wake_by_ref function from the vtable. The RawWaker
    /// remains valid after this call.
    #[inline]
    pub unsafe fn wake_by_ref(&self) {
        unsafe { (self.vtable.wake_by_ref)(self.data) }
    }

    /// Drop the RawWaker without waking
    ///
    /// # Safety
    ///
    /// Calls the drop function from the vtable. After this call, the
    /// RawWaker is consumed and should not be used again.
    #[inline]
    pub unsafe fn drop(self) {
        unsafe { (self.vtable.drop)(self.data) }
    }
}

unsafe impl Send for RawWaker {}
unsafe impl Sync for RawWaker {}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vtable functions
    unsafe fn clone_waker(data: *const ()) -> RawWaker {
        RawWaker::new(data, &TEST_VTABLE)
    }

    unsafe fn wake_waker(data: *const ()) {
        // In tests, we just do nothing
        let _ = data;
    }

    unsafe fn wake_by_ref_waker(_data: *const ()) {
        // In tests, we just do nothing
    }

    unsafe fn drop_waker(_data: *const ()) {
        // In tests, we just do nothing
    }

    static TEST_VTABLE: RawWakerVTable = RawWakerVTable {
        clone: clone_waker,
        wake: wake_waker,
        wake_by_ref: wake_by_ref_waker,
        drop: drop_waker,
    };

    #[test]
    fn test_raw_waker_creation() {
        let data = &42 as *const i32 as *const ();
        let waker = RawWaker::new(data, &TEST_VTABLE);

        assert_eq!(waker.data(), data);
        assert_eq!(waker.vtable() as *const RawWakerVTable, &TEST_VTABLE as *const RawWakerVTable);
    }

    #[test]
    fn test_raw_waker_clone() {
        let data = &42 as *const i32 as *const ();
        let waker = RawWaker::new(data, &TEST_VTABLE);

        unsafe {
            let cloned = waker.clone();
            assert_eq!(cloned.data(), data);
            assert_eq!(cloned.vtable() as *const RawWakerVTable, &TEST_VTABLE as *const RawWakerVTable);
        }
    }

    #[test]
    fn test_raw_waker_wake_by_ref() {
        let data = &42 as *const i32 as *const ();
        let waker = RawWaker::new(data, &TEST_VTABLE);

        unsafe {
            waker.wake_by_ref();
            // Waker should still be valid
            assert_eq!(waker.data(), data);
        }
    }

    #[test]
    fn test_raw_waker_drop() {
        let data = &42 as *const i32 as *const ();
        let waker = RawWaker::new(data, &TEST_VTABLE);

        unsafe {
            waker.drop();
            // Waker is consumed, no assertion needed
        }
    }
}
