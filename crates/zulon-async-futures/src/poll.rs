// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Poll type for future results
//!
//! The Poll enum represents the result of polling a Future.

/// The result of polling a future
///
/// This type is returned by the [`Future::poll`] method to indicate
/// whether the future is complete or still pending.
///
/// # Variants
///
/// - `Ready(T)` - The future has completed and produces a value
/// - `Pending` - The future is not ready yet
///
/// # Example
///
/// ```rust
/// use zulon_async_futures::Poll;
///
/// fn check_ready() -> Poll<i32> {
///     Poll::Ready(42)
/// }
///
/// fn check_pending() -> Poll<i32> {
///     Poll::Pending
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Poll<T> {
    /// The future is complete and produces this value
    Ready(T),

    /// The future is not ready yet
    ///
    /// When a future returns Pending, the executor will wait for the
    /// associated Waker to be called before polling again.
    Pending,
}

impl<T> Poll<T> {
    /// Map the value inside a Ready variant
    ///
    /// If the poll is Ready, apply the function to the value.
    /// If the poll is Pending, return Pending unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// let poll: Poll<i32> = Poll::Ready(5);
    /// let poll = poll.map(|x| x * 2);
    /// assert_eq!(poll, Poll::Ready(10));
    ///
    /// let pending: Poll<i32> = Poll::Pending;
    /// assert_eq!(pending.map(|x| x * 2), Poll::Pending);
    /// ```
    pub fn map<U, F>(self, f: F) -> Poll<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Ready(value) => Poll::Ready(f(value)),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Check if the poll is Ready
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// assert!(Poll::Ready(42).is_ready());
    /// assert!(!Poll::<i32>::Pending.is_ready());
    /// ```
    pub fn is_ready(&self) -> bool {
        matches!(self, Poll::Ready(_))
    }

    /// Check if the poll is Pending
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// assert!(Poll::<i32>::Pending.is_pending());
    /// assert!(!Poll::Ready(42).is_pending());
    /// ```
    pub fn is_pending(&self) -> bool {
        matches!(self, Poll::Pending)
    }

    /// Extract the value from a Ready poll, or panic if Pending
    ///
    /// # Panics
    ///
    /// Panics if the poll is Pending
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// let poll = Poll::Ready(42);
    /// assert_eq!(poll.unwrap(), 42);
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("called unwrap on Pending poll"),
        }
    }

    /// Extract the value from a Ready poll, or return a default
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// let poll = Poll::Ready(42);
    /// assert_eq!(poll.unwrap_or(0), 42);
    ///
    /// let pending: Poll<i32> = Poll::Pending;
    /// assert_eq!(pending.unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Poll::Ready(value) => value,
            Poll::Pending => default,
        }
    }
}

impl<T, E> Poll<Result<T, E>> {
    /// Map the Ok value inside a Ready(Ok(_)) variant
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// let poll: Poll<Result<i32, &str>> = Poll::Ready(Ok(5));
    /// let poll = poll.map_ok(|x| x * 2);
    /// assert_eq!(poll, Poll::Ready(Ok(10)));
    /// ```
    pub fn map_ok<U, F>(self, f: F) -> Poll<Result<U, E>>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(f(value))),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Map the Err value inside a Ready(Err(_)) variant
    ///
    /// # Example
    ///
    /// ```rust
    /// use zulon_async_futures::Poll;
    ///
    /// let poll: Poll<Result<i32, &str>> = Poll::Ready(Err("error"));
    /// let poll = poll.map_err(|e| e.len());
    /// assert_eq!(poll, Poll::Ready(Err(5)));
    /// ```
    pub fn map_err<F, G>(self, f: G) -> Poll<Result<T, F>>
    where
        G: FnOnce(E) -> F,
    {
        match self {
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(value)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(f(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_ready() {
        let poll: Poll<i32> = Poll::Ready(42);
        assert!(poll.is_ready());
        assert!(!poll.is_pending());
        assert_eq!(poll.unwrap(), 42);
    }

    #[test]
    fn test_poll_pending() {
        let poll: Poll<i32> = Poll::Pending;
        assert!(poll.is_pending());
        assert!(!poll.is_ready());
        assert_eq!(poll.unwrap_or(0), 0);
    }

    #[test]
    fn test_poll_map() {
        let poll: Poll<i32> = Poll::Ready(5);
        assert_eq!(poll.map(|x| x * 2), Poll::Ready(10));

        let pending: Poll<i32> = Poll::Pending;
        assert_eq!(pending.map(|x| x * 2), Poll::Pending);
    }

    #[test]
    #[should_panic(expected = "called unwrap on Pending poll")]
    fn test_poll_unwrap_panic() {
        let poll: Poll<i32> = Poll::Pending;
        poll.unwrap();
    }

    #[test]
    fn test_poll_map_ok() {
        let poll: Poll<Result<i32, &str>> = Poll::Ready(Ok(5));
        assert_eq!(poll.map_ok(|x| x * 2), Poll::Ready(Ok(10)));

        let poll: Poll<Result<i32, &str>> = Poll::Ready(Err("error"));
        assert_eq!(poll.map_ok(|x| x * 2), Poll::Ready(Err("error")));
    }

    #[test]
    fn test_poll_map_err() {
        let poll: Poll<Result<i32, &str>> = Poll::Ready(Err("error"));
        assert_eq!(poll.map_err(|e| e.len()), Poll::Ready(Err(5)));

        let poll: Poll<Result<i32, &str>> = Poll::Ready(Ok(5));
        assert_eq!(poll.map_err(|e| e.len()), Poll::Ready(Ok(5)));
    }
}
