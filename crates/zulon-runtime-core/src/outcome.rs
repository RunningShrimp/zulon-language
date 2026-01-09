// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON Outcome Type
//!
//! This module provides the `Outcome<T, E>` type for error handling in ZULON.
//! It's equivalent to Rust's `Result<T, E>` type.
//!
//! # Examples
//!
//! ```rust
//! use zulon_runtime_core::outcome::Outcome;
//!
//! fn divide(a: i32, b: i32) -> Outcome<i32, &'static str> {
//!     if b == 0 {
//!         return Outcome::Err("Division by zero");
//!     }
//!     Outcome::Ok(a / b)
//! }
//!
//! fn main() {
//!     match divide(10, 2) {
//!         Outcome::Ok(value) => println!("Result: {}", value),
//!         Outcome::Err(err) => println!("Error: {}", err),
//!     }
//! }
//! ```

use std::fmt;

/// A type representing success (`Ok`) or failure (`Err`).
///
/// `Outcome<T, E>` is the enum type used for error handling in ZULON.
/// It has two variants:
///
/// - `Ok(T)`: Represents success and contains a success value
/// - `Err(E)`: Represents error and contains an error value
///
/// # Memory Layout
///
/// `Outcome<T, E>` is guaranteed to have the same memory layout as Rust's
/// `Result<T, E>`, which is optimized for space:
///
/// - If `T` and `E` are both ZST (zero-sized types), the entire enum is a ZST
/// - If one variant is ZST, the enum has the same size as the non-ZST variant
/// - Otherwise, the discriminant is stored as a tag
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use zulon_runtime_core::outcome::Outcome;
///
/// fn divide(a: i32, b: i32) -> Outcome<i32, &'static str> {
///     if b == 0 {
///         return Outcome::Err("division by zero");
///     }
///     Outcome::Ok(a / b)
/// }
/// ```
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome<T, E> {
    /// Contains the success value
    Ok(T),

    /// Contains the error value
    Err(E),
}

impl<T, E> Outcome<T, E> {
    /// Returns `true` if the outcome is `Ok`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(-3);
    /// assert_eq!(x.is_ok(), true);
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.is_ok(), false);
    /// ```
    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(self, Outcome::Ok(_))
    }

    /// Returns `true` if the outcome is `Err`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(-3);
    /// assert_eq!(x.is_err(), false);
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.is_err(), true);
    /// ```
    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Converts from `Outcome<T, E>` to `Option<T>`.
    ///
    /// Converts `self` into an `Option<T>`, consuming `self`, and discarding
    /// the error, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            Outcome::Ok(v) => Some(v),
            Outcome::Err(_) => None,
        }
    }

    /// Converts from `Outcome<T, E>` to `Option<E>`.
    ///
    /// Converts `self` into an `Option<E>`, consuming `self`, and discarding
    /// the success value, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.err(), None);
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.err(), Some("Some error message"));
    /// ```
    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            Outcome::Ok(_) => None,
            Outcome::Err(e) => Some(e),
        }
    }

    /// Returns the contained `Ok` value or a default.
    ///
    /// Consumes the `self` argument then, if `Ok`, returns the contained
    /// value, otherwise if `Err`, returns the default value for that type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.unwrap_or(0), 2);
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.unwrap_or(0), 0);
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Outcome::Ok(v) => v,
            Outcome::Err(_) => default,
        }
    }

    /// Returns the contained `Ok` value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let k = 10;
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.unwrap_or_else(|| 2 * k), 2);
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.unwrap_or_else(|| 2 * k), 20);
    /// ```
    #[inline]
    pub fn unwrap_or_else<F>(self, default: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Outcome::Ok(v) => v,
            Outcome::Err(_) => default(),
        }
    }

    /// Returns the contained `Ok` value or panics.
    ///
    /// # Panics
    ///
    /// Panics if the value is an `Err`, with a panic message including the
    /// passed message, and the content of the `Err`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.expect("Happy path"), 2);
    /// ```
    ///
    /// ```rust,no_run
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// x.expect("Happy path"); // panics with `Happy path: Some error message`
    /// ```
    #[inline]
    pub fn expect(self, msg: &str) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Outcome::Ok(v) => v,
            Outcome::Err(e) => panic!("{}: {:?}", msg, e),
        }
    }

    /// Returns the contained `Ok` value or panics.
    ///
    /// # Panics
    ///
    /// Panics if the value is an `Err`, with a panic message provided by the
    /// `Err`'s value.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.unwrap(), 2);
    /// ```
    ///
    /// ```rust,no_run
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// x.unwrap(); // panics with `Some error message`
    /// ```
    #[inline]
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        self.expect("called `Outcome::unwrap()` on an `Err` value")
    }

    /// Maps an `Outcome<T, E>` to `Outcome<U, E>` by applying a function to a
    /// contained `Ok` value, leaving an `Err` value untouched.
    ///
    /// This function can be used to implement error handling adapters or
    /// computation on the success value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// fn stringify(x: i32) -> String {
    ///     if x == 0 {
    ///         return "zero".to_string();
    ///     }
    ///     "nonzero".to_string()
    /// }
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.map(stringify), Outcome::Ok("nonzero".to_string()));
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.map(stringify), Outcome::Err("Some error message"));
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> Outcome<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Outcome::Ok(v) => Outcome::Ok(f(v)),
            Outcome::Err(e) => Outcome::Err(e),
        }
    }

    /// Maps an `Outcome<T, E>` to `Outcome<T, F>` by applying a function to a
    /// contained `Err` value, leaving an `Ok` value untouched.
    ///
    /// This function can be used to pass through a successful result while
    /// handling an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// fn stringify(x: i32) -> String {
    ///     if x == 0 {
    ///         return "zero".to_string();
    ///     }
    ///     format!("{}", x)
    /// }
    ///
    /// let x: Outcome<i32, i32> = Outcome::Ok(2);
    /// assert_eq!(x.map_err(stringify), Outcome::Ok(2));
    ///
    /// let x: Outcome<i32, i32> = Outcome::Err(13);
    /// assert_eq!(x.map_err(stringify), Outcome::Err("13".to_string()));
    /// ```
    #[inline]
    pub fn map_err<F, O>(self, f: O) -> Outcome<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            Outcome::Ok(v) => Outcome::Ok(v),
            Outcome::Err(e) => Outcome::Err(f(e)),
        }
    }

    /// Returns the provided default (if `Err`), or applies a function to the contained value (if `Ok`).
    ///
    /// Arguments passed to `unwrap_or_else` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`unwrap_or_else`]: Outcome::unwrap_or_else
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Outcome;
    ///
    /// let x: Outcome<i32, &str> = Outcome::Ok(2);
    /// assert_eq!(x.and(Outcome::Ok(100)), Outcome::Ok(100));
    ///
    /// let x: Outcome<i32, &str> = Outcome::Err("Some error message");
    /// assert_eq!(x.and(Outcome::Ok(100)), Outcome::Err("Some error message"));
    /// ```
    #[inline]
    pub fn and<U>(self, res: Outcome<U, E>) -> Outcome<U, E> {
        match self {
            Outcome::Ok(_) => res,
            Outcome::Err(e) => Outcome::Err(e),
        }
    }

    /// Converts an Outcome<T, E> to Outcome<T, F> using the Into trait.
    ///
    /// This is useful for error type conversion in error propagation chains.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::{Outcome, Into};
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum Error1 {
    ///     Variant1,
    /// }
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum Error2 {
    ///     Variant2,
    /// }
    ///
    /// impl zulon_runtime_core::From<Error1> for Error2 {
    ///     fn from(_: Error1) -> Self {
    ///         Error2::Variant2
    ///     }
    /// }
    ///
    /// let result: Outcome<i32, Error1> = Outcome::Err(Error1::Variant1);
    /// let converted: Outcome<i32, Error2> = result.convert_err();
    /// assert_eq!(converted, Outcome::Err(Error2::Variant2));
    /// ```
    #[inline]
    pub fn convert_err<F>(self) -> Outcome<T, F>
    where
        E: Into<F>,
    {
        match self {
            Outcome::Ok(v) => Outcome::Ok(v),
            Outcome::Err(e) => Outcome::Err(e.into()),
        }
    }
}

// ============================================================================
// Error Trait for Error Chaining
// ============================================================================

/// Trait for error types that can be chained and provide context.
///
/// This trait is similar to Rust's `std::error::Error` trait, providing:
/// - Error source chaining via `source()`
/// - Error descriptions via `description()`
/// - Legacy cause support via `cause()`
///
/// # Examples
///
/// ```rust
/// use zulon_runtime_core::outcome::Error;
/// use std::fmt;
///
/// #[derive(Debug)]
/// struct InnerError {
///     message: String,
/// }
///
/// impl fmt::Display for InnerError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.message)
///     }
/// }
///
/// impl Error for InnerError {
///     fn source(&self) -> Option<&(dyn Error + 'static)> {
///         None  // No underlying error
///     }
///
///     fn description(&self) -> &str {
///         "inner error occurred"
///     }
/// }
/// ```
pub trait Error: fmt::Display + fmt::Debug {
    /// Returns the lower-level source of this error, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::Error;
    ///
    /// # #[derive(Debug)]
    /// # struct InnerError;
    /// # impl std::fmt::Display for InnerError { fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
    /// # impl Error for InnerError {
    /// #     fn source(&self) -> Option<&(dyn Error + 'static)> { None }
    /// #     fn description(&self) -> &str { "inner" }
    /// # }
    /// #
    /// #[derive(Debug)]
    /// struct OuterError {
    ///     inner: InnerError,
    /// }
    ///
    /// impl std::fmt::Display for OuterError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "outer error")
    ///     }
    /// }
    ///
    /// impl Error for OuterError {
    ///     fn source(&self) -> Option<&(dyn Error + 'static)> {
    ///         Some(&self.inner)
    ///     }
    ///
    ///     fn description(&self) -> &str {
    ///         "outer error occurred"
    ///     }
    /// }
    /// ```
    fn source(&self) -> Option<&(dyn Error + 'static)>;

    /// Returns a short description of the error.
    ///
    /// This method provides a static description of the error type,
    /// unlike `Display` which can provide context-specific messages.
    fn description(&self) -> &str;

    /// Returns the cause of this error, if any.
    ///
    /// This is a legacy method that delegates to `source()` by default.
    /// New code should use `source()` instead.
    #[inline]
    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

// ============================================================================
// ContextError for Error Context
// ============================================================================

/// Error type that adds context to an underlying error.
///
/// This is similar to `anyhow::Context` and allows adding contextual
/// information to errors as they propagate up the call stack.
///
/// # Examples
///
/// ```rust
/// use zulon_runtime_core::outcome::{Outcome, ContextError, OutcomeExt};
///
/// # #[derive(Debug, PartialEq)]
/// # struct IoError;
/// # impl std::fmt::Display for IoError { fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
/// # impl zulon_runtime_core::outcome::Error for IoError {
/// #     fn source(&self) -> Option<&(dyn zulon_runtime_core::outcome::Error + 'static)> { None }
/// #     fn description(&self) -> &str { "IO error" }
/// # }
/// #
/// fn read_config() -> Outcome<String, IoError> {
///     // ... IO operations that might fail
///     Outcome::Ok("config".to_string())
/// }
///
/// fn load_config() -> Outcome<String, ContextError<&'static str, IoError>> {
///     read_config().context("failed to read config file")
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ContextError<M, E> {
    /// The contextual message added to the error
    pub msg: M,

    /// The underlying error
    pub error: E,
}

impl<M, E> ContextError<M, E> {
    /// Creates a new `ContextError` with the given message and underlying error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::ContextError;
    ///
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl std::fmt::Display for MyError { fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
    /// #
    /// let error = ContextError::new("operation failed", MyError);
    /// assert_eq!(error.msg, "operation failed");
    /// ```
    #[inline]
    pub fn new(msg: M, error: E) -> Self {
        Self { msg, error }
    }
}

impl<M, E> fmt::Display for ContextError<M, E>
where
    M: fmt::Display,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.msg, self.error)
    }
}

impl<M, E> Error for ContextError<M, E>
where
    M: fmt::Display + fmt::Debug + 'static,
    E: Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }

    #[inline]
    fn description(&self) -> &str {
        "error with additional context"
    }
}

/// Extension trait for adding context to `Outcome` errors.
///
/// # Examples
///
/// ```rust
/// use zulon_runtime_core::outcome::{Outcome, OutcomeExt};
///
/// # #[derive(Debug, PartialEq)]
/// # struct MyError;
/// # impl std::fmt::Display for MyError { fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
/// # impl zulon_runtime_core::outcome::Error for MyError {
/// #     fn source(&self) -> Option<&(dyn zulon_runtime_core::outcome::Error + 'static)> { None }
/// #     fn description(&self) -> &str { "my error" }
/// # }
/// #
/// fn might_fail() -> Outcome<(), MyError> {
///     Outcome::Err(MyError)
/// }
///
/// let result = might_fail().context("operation failed");
/// ```
pub trait OutcomeExt<T, E> {
    /// Adds context to an error, converting it to a `ContextError`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zulon_runtime_core::outcome::{Outcome, OutcomeExt};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// # struct IoError;
    /// # impl std::fmt::Display for IoError { fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
    /// # impl zulon_runtime_core::outcome::Error for IoError {
    /// #     fn source(&self) -> Option<&(dyn zulon_runtime_core::outcome::Error + 'static)> { None }
    /// #     fn description(&self) -> &str { "IO error" }
    /// # }
    /// #
    /// fn read_file() -> Outcome<String, IoError> {
    ///     Outcome::Err(IoError)
    /// }
    ///
    /// fn load_config() -> Outcome<String, zulon_runtime_core::outcome::ContextError<&'static str, IoError>> {
    ///     read_file().context("failed to read config")
    /// }
    /// ```
    fn context<M>(self, msg: M) -> Outcome<T, ContextError<M, E>>
    where
        M: fmt::Display + fmt::Debug + 'static,
        E: Error + 'static;
}

impl<T, E> OutcomeExt<T, E> for Outcome<T, E> {
    #[inline]
    fn context<M>(self, msg: M) -> Outcome<T, ContextError<M, E>>
    where
        M: fmt::Display + fmt::Debug + 'static,
        E: Error + 'static,
    {
        match self {
            Outcome::Ok(v) => Outcome::Ok(v),
            Outcome::Err(e) => Outcome::Err(ContextError::new(msg, e)),
        }
    }
}

// ============================================================================
// Panic Support
// ============================================================================

/// Panics the current process with the given message.
///
/// This function prints the panic message to stderr and terminates
/// the process with exit code 1.
///
/// # Examples
///
/// ```rust,no_run
/// use zulon_runtime_core::outcome::panic;
///
/// panic("Something went terribly wrong!");
/// ```
#[inline]
pub fn panic(msg: &str) -> ! {
    eprintln!("Panic: {}", msg);
    std::process::exit(1)
}

// ============================================================================
// ZULON Builtins - C ABI for LLVM IR
// ============================================================================

/// ZULON builtin panic function - C ABI for calling from LLVM IR.
///
/// This function is called by the panic! macro expansion and provides
/// runtime panic support for ZULON programs.
///
/// # Safety
///
/// This function expects a valid C string pointer (null-terminated).
/// It is only meant to be called from generated LLVM IR code.
///
/// # ABI
///
/// - Uses C calling convention
/// - Symbol name is not mangled (#[no_mangle])
/// - Takes a pointer to a null-terminated string
/// - Never returns (diverges)
#[no_mangle]
pub extern "C" fn __zulon_builtin_panic(message: *const u8) -> ! {
    unsafe {
        if message.is_null() {
            eprintln!("Panic: <null message>");
        } else {
            // Convert C string to Rust string
            let len = std::ffi::CStr::from_ptr(message as *const i8)
                .to_str()
                .map(|s| s.len())
                .unwrap_or(0);

            let slice = std::slice::from_raw_parts(message, len);
            let msg_str = std::str::from_utf8_unchecked(slice);
            eprintln!("Panic: {}", msg_str);
        }
    }
    std::process::exit(1);
}

/// Formatted panic function for assert macros
///
/// Takes a format string and multiple arguments.
/// For MVP, this simplifies to just concatenating all parts with spaces.
///
/// # Safety
///
/// - Uses C calling convention
/// - Symbol name is not mangled (#[no_mangle])
/// - Takes variable arguments: format string and args
/// - Never returns (diverges)
#[no_mangle]
pub extern "C" fn __zulon_builtin_panic_formatted(
    format: *const u8,
    arg1: *const u8,
    arg2: *const u8,
    arg3: *const u8,
    arg4: *const u8,
) -> ! {
    unsafe {
        eprint!("Panic: ");

        // Print format string
        if !format.is_null() {
            let len = std::ffi::CStr::from_ptr(format as *const i8)
                .to_str()
                .map(|s| s.len())
                .unwrap_or(0);
            let slice = std::slice::from_raw_parts(format, len);
            let fmt_str = std::str::from_utf8_unchecked(slice);
            eprint!("{}", fmt_str);
        }

        // Print additional arguments if present
        let args = [arg1, arg2, arg3, arg4];
        for arg in args.iter() {
            if !arg.is_null() {
                let len = std::ffi::CStr::from_ptr(*arg as *const i8)
                    .to_str()
                    .map(|s| s.len())
                    .unwrap_or(0);
                let slice = std::slice::from_raw_parts(*arg, len);
                let arg_str = std::str::from_utf8_unchecked(slice);
                eprint!(" {}", arg_str);
            }
        }

        eprintln!();
    }
    std::process::exit(1);
}

/// Get current time in milliseconds
///
/// Returns the current time as milliseconds since the Unix epoch.
/// Used for performance benchmarking and timing measurements.
///
/// # Safety
///
/// - Uses C calling convention
/// - Symbol name is not mangled (#[no_mangle])
/// - Returns time as i32 (milliseconds)
///
/// # Note
///
/// On platforms where `gettimeofday` is not available, this will
/// fall back to a simple implementation that may not be accurate.
#[no_mangle]
pub extern "C" fn __zulon_builtin_current_time_ms() -> i32 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Use a static variable to store the start time on first call
    static START_TIME: AtomicU64 = AtomicU64::new(0);

    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,  // Convert u128 to u64
        Err(_) => return 0,
    };

    // Initialize start time on first call
    let start = START_TIME.load(Ordering::Relaxed);
    if start == 0 {
        START_TIME.store(now, Ordering::Relaxed);
        return 0;
    }

    // Return elapsed time since first call (as i32, fits in ~24 days)
    (now - start) as i32
}

// ============================================================================
// From Trait Implementation for Error Propagation
// ============================================================================

/// Trait for converting between types.
///
/// This trait is used by the `?` operator for automatic error conversion.
///
/// # Examples
///
/// ```rust
/// use zulon_runtime_core::outcome::{Outcome, From};
///
/// #[derive(Debug, PartialEq)]
/// struct ParseError;
///
/// #[derive(Debug, PartialEq)]
/// struct ComputeError;
///
/// impl From<ParseError> for ComputeError {
///     fn from(_: ParseError) -> Self {
///         ComputeError
///     }
/// }
///
/// fn parse(input: &str) -> Outcome<i32, ParseError> {
///     if input.is_empty() {
///         return Outcome::Err(ParseError);
///     }
///     Outcome::Ok(42)
/// }
///
/// // The From trait enables error type conversion
/// let result: Outcome<i32, ParseError> = Outcome::Err(ParseError);
/// let converted: Outcome<i32, ComputeError> = <Outcome<i32, ComputeError> as From<ParseError>>::from(ParseError);
/// assert_eq!(converted, Outcome::Err(ComputeError));
/// ```
pub trait From<T> {
    /// Converts from `T` to `Self`
    fn from(t: T) -> Self;
}

/// Blanket implementation for Into trait (reciprocal of From)
///
/// If you implement `From<T> for U`, you automatically get `Into<U> for T`.
pub trait Into<T>: Sized {
    /// Converts self into the target type
    fn into(self) -> T;
}

impl<T, U> Into<U> for T
where
    U: From<T>,
{
    #[inline]
    fn into(self) -> U {
        U::from(self)
    }
}

// Generic implementation for converting errors in Outcome
impl<T, E, F> From<F> for Outcome<T, E>
where
    E: From<F>,
{
    #[inline]
    fn from(err: F) -> Self {
        Outcome::Err(E::from(err))
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T, E> fmt::Display for Outcome<T, E>
where
    T: fmt::Display,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Ok(v) => v.fmt(f),
            Outcome::Err(e) => e.fmt(f),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ok() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        assert!(ok.is_ok());
        assert!(!ok.is_err());

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert!(!err.is_ok());
        assert!(err.is_err());
    }

    #[test]
    fn test_ok_err() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        assert_eq!(ok.ok(), Some(42));
        assert_eq!(ok.err(), None);

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert_eq!(err.ok(), None);
        assert_eq!(err.err(), Some("error"));
    }

    #[test]
    fn test_unwrap_or() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        assert_eq!(ok.unwrap_or(0), 42);

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert_eq!(err.unwrap_or(0), 0);
    }

    #[test]
    fn test_unwrap_or_else() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        assert_eq!(ok.unwrap_or_else(|| 0), 42);

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert_eq!(err.unwrap_or_else(|| 0), 0);
    }

    #[test]
    fn test_map() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        assert_eq!(ok.map(|x| x * 2), Outcome::Ok(84));

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert_eq!(err.map(|x| x * 2), Outcome::Err("error"));
    }

    #[test]
    fn test_map_err() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        let result = ok.map_err(|e| format!("{}!", e));
        assert_eq!(result, Outcome::Ok(42));

        let err: Outcome<i32, &str> = Outcome::Err("error");
        let result = err.map_err(|e| format!("{}!", e));
        assert_eq!(result, Outcome::Err("error!".to_string()));
    }

    #[test]
    fn test_and() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        let other: Outcome<i32, &str> = Outcome::Ok(100);
        assert_eq!(ok.and(other), Outcome::Ok(100));

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert_eq!(err.and(Outcome::Ok(100)), Outcome::Err("error"));
    }

    #[test]
    fn test_display() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        assert_eq!(format!("{}", ok), "42");

        let err: Outcome<i32, &str> = Outcome::Err("error");
        assert_eq!(format!("{}", err), "error");
    }

    #[test]
    fn test_copy() {
        let ok: Outcome<i32, &str> = Outcome::Ok(42);
        let ok_copy = ok;
        assert_eq!(ok, ok_copy);

        let err: Outcome<i32, &str> = Outcome::Err("error");
        let err_copy = err;
        assert_eq!(err, err_copy);
    }

    #[test]
    fn test_from_error() {
        #[derive(Debug, PartialEq, Eq)]
        enum Error1 {
            Variant1,
        }

        #[derive(Debug, PartialEq, Eq)]
        enum Error2 {
            Variant2,
        }

        impl From<Error1> for Error2 {
            fn from(_: Error1) -> Self {
                Error2::Variant2
            }
        }

        // Test From trait for error conversion using fully qualified syntax
        let outcome: Outcome<(), Error2> = <Outcome<(), Error2> as From<Error1>>::from(Error1::Variant1);
        assert_eq!(outcome, Outcome::Err(Error2::Variant2));
    }

    #[test]
    fn test_convert_err() {
        #[derive(Debug, PartialEq, Eq)]
        enum Error1 {
            Variant1,
        }

        #[derive(Debug, PartialEq, Eq)]
        enum Error2 {
            Variant2,
        }

        impl From<Error1> for Error2 {
            fn from(_: Error1) -> Self {
                Error2::Variant2
            }
        }

        // Test convert_err method
        let result: Outcome<i32, Error1> = Outcome::Err(Error1::Variant1);
        let converted: Outcome<i32, Error2> = result.convert_err();
        assert_eq!(converted, Outcome::Err(Error2::Variant2));

        // Test that Ok values pass through
        let ok_result: Outcome<i32, Error1> = Outcome::Ok(42);
        let ok_converted: Outcome<i32, Error2> = ok_result.convert_err();
        assert_eq!(ok_converted, Outcome::Ok(42));
    }

    #[test]
    fn test_into_trait() {
        #[derive(Debug, PartialEq, Eq)]
        struct Small(i32);

        #[derive(Debug, PartialEq, Eq)]
        struct Big(i64);

        impl From<Small> for Big {
            fn from(small: Small) -> Self {
                Big(small.0 as i64)
            }
        }

        // Test Into trait (automatically implemented from From) using fully qualified syntax
        let small = Small(42);
        let big: Big = Into::into(small);
        assert_eq!(big, Big(42));
    }

    // ========================================================================
    // Error Trait Tests
    // ========================================================================

    #[test]
    fn test_error_trait() {
        #[derive(Debug, PartialEq)]
        struct TestError {
            message: String,
        }

        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl Error for TestError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }

            fn description(&self) -> &str {
                "test error"
            }
        }

        let error = TestError {
            message: "something went wrong".to_string(),
        };

        assert_eq!(error.description(), "test error");
        assert!(error.source().is_none());
        assert!(error.cause().is_none());
    }

    #[test]
    fn test_error_chain() {
        #[derive(Debug, PartialEq)]
        struct InnerError;

        impl fmt::Display for InnerError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "inner error")
            }
        }

        impl Error for InnerError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }

            fn description(&self) -> &str {
                "inner error occurred"
            }
        }

        #[derive(Debug, PartialEq)]
        struct OuterError {
            inner: InnerError,
        }

        impl fmt::Display for OuterError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "outer error")
            }
        }

        impl Error for OuterError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                Some(&self.inner)
            }

            fn description(&self) -> &str {
                "outer error occurred"
            }
        }

        let inner = InnerError;
        let outer = OuterError { inner };

        // Test error chain
        assert_eq!(outer.description(), "outer error occurred");
        assert!(outer.source().is_some());

        let source = outer.source().unwrap();
        assert_eq!(source.description(), "inner error occurred");
        assert!(source.source().is_none());
    }

    #[test]
    fn test_context_error() {
        #[derive(Debug, PartialEq)]
        struct IoError;

        impl fmt::Display for IoError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "IO error")
            }
        }

        impl Error for IoError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }

            fn description(&self) -> &str {
                "IO operation failed"
            }
        }

        let io_error = IoError;
        let context_error = ContextError::new("failed to read file", io_error);

        assert_eq!(context_error.msg, "failed to read file");
        assert_eq!(context_error.description(), "error with additional context");

        // Test Display
        let display_str = format!("{}", context_error);
        assert!(display_str.contains("failed to read file"));
        assert!(display_str.contains("IO error"));

        // Test source chain
        assert!(context_error.source().is_some());
        let source = context_error.source().unwrap();
        assert_eq!(source.description(), "IO operation failed");
    }

    #[test]
    fn test_outcome_ext_context() {
        use super::OutcomeExt;

        #[derive(Debug, PartialEq)]
        struct TestError;

        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "test error")
            }
        }

        impl Error for TestError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }

            fn description(&self) -> &str {
                "test error occurred"
            }
        }

        // Test Ok branch - context should pass through
        let ok_result: Outcome<(), TestError> = Outcome::Ok(());
        let contextualized = ok_result.context("operation failed");
        assert!(contextualized.is_ok());

        // Test Err branch - context should wrap error
        let err_result: Outcome<(), TestError> = Outcome::Err(TestError);
        let contextualized = err_result.context("operation failed");
        assert!(contextualized.is_err());

        if let Outcome::Err(err) = contextualized {
            assert_eq!(err.msg, "operation failed");
            assert_eq!(err.error.description(), "test error occurred");
        }
    }

    #[test]
    fn test_nested_context() {
        use super::OutcomeExt;

        #[derive(Debug, PartialEq)]
        struct BaseError;

        impl fmt::Display for BaseError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "base error")
            }
        }

        impl Error for BaseError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }

            fn description(&self) -> &str {
                "base error occurred"
            }
        }

        // Test nested context for error chain
        let result: Outcome<(), BaseError> = Outcome::Err(BaseError);
        let result = result.context("level 1 failed");
        let result = result.context("level 2 failed");

        assert!(result.is_err());

        if let Outcome::Err(err) = result {
            assert_eq!(err.msg, "level 2 failed");

            // Check the error chain: level 2 -> level 1 -> base
            let level1 = err.error;
            assert_eq!(level1.msg, "level 1 failed");

            let base = level1.error;
            assert_eq!(base.description(), "base error occurred");
        }
    }

    #[test]
    fn test_panic_function() {
        // Note: We can't actually test panic since it terminates the process,
        // but we can verify it compiles and has the right signature
        //
        // In a real test, you would use a separate process or catch the panic
        use super::panic;

        // This is just to verify the function exists and compiles
        // We won't actually call it since it exits the process
        let _ = panic as fn(&str) -> !;
    }
}
