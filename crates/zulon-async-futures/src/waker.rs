// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Waker type for waking up tasks
//!
//! The Waker provides a handle for waking up a task. It's a safe wrapper
//! around RawWaker that ensures proper memory management.

use crate::task::{RawWaker, RawWakerVTable};

/// A handle for waking up a task
///
/// Wakers are passed to futures through the Context and allow the future
/// to request that the executor poll it again when it's ready to make progress.
///
/// # Cloning
///
/// Wakers are cheap to clone and each clone refers to the same task.
///
/// # Example
///
/// ```rust
/// use zulon_async_futures::Waker;
///
/// // Create a waker from a raw waker
/// // (normally you'd get this from an executor)
/// ```
#[repr(transparent)]
pub struct Waker {
    raw: RawWaker,
}

unsafe impl Send for Waker {}
unsafe impl Sync for Waker {}

impl Waker {
    /// Create a Waker from a RawWaker
    ///
    /// # Safety
    ///
    /// The RawWaker must be valid and uphold the safety contracts of its vtable.
    #[inline]
    pub const unsafe fn from_raw(raw: RawWaker) -> Self {
        Self { raw }
    }

    /// Get the RawWaker inside this Waker
    #[inline]
    pub const fn as_raw(&self) -> &RawWaker {
        &self.raw
    }

    /// Wake the task and consume the Waker
    ///
    /// This will notify the executor that the task is ready to be polled again.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Waker;
    ///
    /// // waker.wake(); // Wakes the task
    /// ```
    #[inline]
    pub fn wake(self) {
        // The waker will be consumed after this call
        unsafe { self.raw.wake() };
    }

    /// Wake the task without consuming the Waker
    ///
    /// This is useful when you want to wake the task but keep using the Waker.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Waker;
    ///
    /// // waker.wake_by_ref(); // Wakes the task
    /// // waker.wake_by_ref(); // Can still use the waker
    /// ```
    #[inline]
    pub fn wake_by_ref(&self) {
        unsafe { self.raw.wake_by_ref() };
    }

    /// Create a Waker that will never wake
    ///
    /// This is useful for dummy wakers in tests or placeholders.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Waker;
    ///
    /// let waker = Waker::noop();
    /// waker.wake(); // Does nothing
    /// ```
    pub fn noop() -> Self {
        // Create a dummy waker that does nothing
        const NOOP_VTABLE: RawWakerVTable = RawWakerVTable {
            clone: const_noop_clone,
            wake: const_noop_wake,
            wake_by_ref: const_noop_wake_by_ref,
            drop: const_noop_drop,
        };

        unsafe { Self::from_raw(RawWaker::new(core::ptr::null(), &NOOP_VTABLE)) }
    }

    /// Check if this is a noop waker
    pub fn is_noop(&self) -> bool {
        core::ptr::null() == self.raw.data()
    }
}

impl Clone for Waker {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            Self::from_raw(self.raw.clone())
        }
    }
}

impl Drop for Waker {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.raw.clone().drop(); // Drop a clone, not self (to avoid self-referential drop)
        }
    }
}

// Const functions for the noop waker
const unsafe fn const_noop_clone(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &NOOP_RAW_VTABLE)
}

const unsafe fn const_noop_wake(_: *const ()) {
    // Do nothing
}

const unsafe fn const_noop_wake_by_ref(_: *const ()) {
    // Do nothing
}

const unsafe fn const_noop_drop(_: *const ()) {
    // Do nothing
}

static NOOP_RAW_VTABLE: RawWakerVTable = RawWakerVTable {
    clone: const_noop_clone,
    wake: const_noop_wake,
    wake_by_ref: const_noop_wake_by_ref,
    drop: const_noop_drop,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waker_noop() {
        let waker = Waker::noop();
        assert!(waker.is_noop());

        // These should not panic
        waker.wake();

        let waker2 = Waker::noop();
        waker2.wake_by_ref();
    }

    #[test]
    fn test_waker_clone() {
        let waker1 = Waker::noop();
        let waker2 = waker1.clone();

        assert!(waker1.is_noop());
        assert!(waker2.is_noop());
    }
}
