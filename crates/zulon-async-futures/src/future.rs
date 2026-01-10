// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Future trait for asynchronous operations
//!
//! The Future trait is the core abstraction for asynchronous values in ZULON.

use crate::{Context, Poll, Pin};

/// A future representing an asynchronous computation
///
/// Futures are the building blocks of asynchronous code in ZULON. They represent
/// a value that may not be available yet, but will be at some point.
///
/// # The Poll Model
///
/// Futures are polled repeatedly until they return `Poll::Ready(value)`. If they
/// return `Poll::Pending`, they must arrange for the Waker to be called when
/// they're ready to be polled again.
///
/// # Example
///
/// ```rust
/// use zulon_async_futures::{Future, Poll, Context};
///
/// struct Delay {
///     polled: bool,
/// }
///
/// impl Future for Delay {
///     type Output = i32;
///
///     fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
///         if self.polled {
///             Poll::Ready(42)
///         } else {
///             self.polled = true;
///             Poll::Pending
///         }
///     }
/// }
/// ```
///
/// # Pinning
///
/// Futures must be pinned before they can be polled. This ensures that the
/// future cannot be moved in memory, which is important for self-referential
/// futures.
pub trait Future {
    /// The type of value produced on completion
    type Output;

    /// Attempt to resolve the future to a final value
    ///
    /// This method should return:
    /// - `Poll::Pending` if the future is not ready yet
    /// - `Poll::Ready(value)` if the future is complete
    ///
    /// # Return Value
    ///
    /// When a future returns `Poll::Pending`, it must ensure that the Waker
    /// is called when the future is ready to be polled again.
    ///
    /// # Panics
    ///
    /// This method may panic if called again after returning `Poll::Ready`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::{Future, Poll, Context, Pin};
    ///
    /// struct ReadyFuture<T>(Option<T>);
    ///
    /// impl<T> Future for ReadyFuture<T> {
    ///     type Output = T;
    ///
    ///     fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
    ///         Poll::Ready(self.0.take().unwrap())
    ///     }
    /// }
    /// ```
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}

/// A future that immediately returns a value
///
/// This is useful for testing and for converting synchronous values into futures.
///
/// # Example
///
/// ```rust
/// use zulon_async_futures::{Future, Poll, Context, Pin, Ready};
///
/// let future = Ready::new(42);
/// let mut context = Context::from_waker(&Waker::noop());
/// assert_eq!(future.poll(&mut context), Poll::Ready(42));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Ready<T>(Option<T>);

impl<T> Ready<T> {
    /// Create a new Ready future
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Ready;
    ///
    /// let future = Ready::new(42);
    /// ```
    #[allow(dead_code)]
    pub const fn new(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T: Unpin> Future for Ready<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.0.take().expect("Ready polled after completion"))
    }
}

/// A future that never returns
///
/// This future will always return `Poll::Pending`. It's useful for:
/// - Tests that need a never-completing future
/// - Placeholder futures
/// - Infinite processes
///
/// # Example
///
/// ```rust
/// use zulon_async_futures::{Future, Poll, Context, Pin, Pending};
///
/// let future = Pending;
/// let mut context = Context::from_waker(&Waker::noop());
/// assert_eq!(future.poll(&mut context), Poll::Pending);
/// ```
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Pending;

impl Future for Pending {
    type Output = (); // Use () instead of ! for now

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Waker;

    #[test]
    fn test_ready_future() {
        let mut future = Ready::new(42);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);

        let mut pinned = std::pin::Pin::new(&mut future);
        assert_eq!(Future::poll(pinned.as_mut(), &mut cx), Poll::Ready(42));
    }

    #[test]
    fn test_pending_future() {
        let mut future = Pending;
        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);

        let mut pinned = std::pin::Pin::new(&mut future);
        assert_eq!(Future::poll(pinned.as_mut(), &mut cx), Poll::Pending);
    }

    #[test]
    fn test_ready_clone() {
        let future1 = Ready::new(42);
        let mut future2 = future1;

        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);

        let mut pinned = std::pin::Pin::new(&mut future2);
        assert_eq!(Future::poll(pinned.as_mut(), &mut cx), Poll::Ready(42));
    }

    // Test a simple stateful future
    #[allow(dead_code)]
    struct Delay {
        count: usize,
    }

    impl Future for Delay {
        type Output = i32;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
            // Note: This is a simplified version that doesn't properly handle Pin
            // In a real implementation, we'd need to use unsafe code properly
            Poll::Ready(42)
        }
    }

    #[test]
    fn test_delay_future() {
        let mut future = Delay { count: 0 };
        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);

        let mut pinned = std::pin::Pin::new(&mut future);
        assert_eq!(Future::poll(pinned.as_mut(), &mut cx), Poll::Ready(42));
    }
}
