// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Context type for futures
//!
//! The Context is passed to futures during polling and provides access
//! to the Waker.

use crate::Waker;

/// Context provided to the [`Future::poll`] method
///
/// The context provides access to the Waker, which allows futures to
/// request that the executor poll them again when they're ready to make progress.
///
/// # Example
///
/// ```rust
/// use zulon_async_futures::{Future, Poll, Context, Waker};
///
/// struct MyFuture;
///
/// impl Future for MyFuture {
///     type Output = i32;
///
///     fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
///         // Get the waker from the context
///         let waker: &Waker = cx.waker();
///
///         // ... do some work ...
///
///         Poll::Pending
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Context<'a> {
    waker: &'a Waker,
}

impl<'a> Context<'a> {
    /// Create a new Context from a Waker reference
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::{Context, Waker};
    ///
    /// let waker = Waker::noop();
    /// let cx = Context::from_waker(&waker);
    /// ```
    #[inline]
    pub const fn from_waker(waker: &'a Waker) -> Self {
        Self { waker }
    }

    /// Get a reference to the Waker
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::{Context, Waker};
    ///
    /// let waker = Waker::noop();
    /// let cx = Context::from_waker(&waker);
    /// assert_eq!(cx.waker() as *const Waker, &waker as *const Waker);
    /// ```
    #[inline]
    pub const fn waker(&self) -> &Waker {
        self.waker
    }

    /// Create a builder-style context with a different waker
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::{Context, Waker};
    ///
    /// let waker1 = Waker::noop();
    /// let waker2 = Waker::noop();
    ///
    /// let cx = Context::from_waker(&waker1);
    /// let cx2 = cx.with_waker(&waker2);
    /// ```
    #[inline]
    pub const fn with_waker<'b>(&self, waker: &'b Waker) -> Context<'b> {
        Context { waker }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_waker() {
        let waker = Waker::noop();
        let cx = Context::from_waker(&waker);

        assert_eq!(cx.waker() as *const Waker, &waker as *const Waker);
    }

    #[test]
    fn test_context_with_waker() {
        let waker1 = Waker::noop();
        let waker2 = Waker::noop();

        let cx = Context::from_waker(&waker1);
        let cx2 = cx.with_waker(&waker2);

        assert_eq!(cx2.waker() as *const Waker, &waker2 as *const Waker);
    }

    #[test]
    fn test_context_clone() {
        let waker = Waker::noop();
        let cx = Context::from_waker(&waker);
        let cx2 = cx;

        assert_eq!(cx2.waker() as *const Waker, &waker as *const Waker);
    }

    #[test]
    fn test_context_copy() {
        let waker = Waker::noop();
        let cx = Context::from_waker(&waker);
        let cx2 = cx; // Context is Copy, so this moves

        // Can still use cx2 because it was moved, not copied
        assert_eq!(cx2.waker() as *const Waker, &waker as *const Waker);

        // Actually, since it's Copy, we can also still use cx
        let cx3 = cx;
        assert_eq!(cx3.waker() as *const Waker, &waker as *const Waker);
    }
}
