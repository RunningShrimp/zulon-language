// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Select mechanism for waiting on multiple channels
//!
//! This module provides a `select!` macro that allows waiting on multiple
//! channels simultaneously. When any channel becomes ready, the corresponding
//! branch is executed.
//!
//! # Example
//!
//! ```rust,no_run
//! use zulon_runtime_io::channel::{mpsc, oneshot};
//! use zulon_runtime_io::select;
//!
//! let (tx1, rx1) = mpsc::channel::<i32>();
//! let (tx2, rx2) = oneshot::channel::<i32>();
//!
//! select! {
//!     recv(rx1, value) => {
//!         println!("Received from rx1: {}", value);
//!     },
//!     recv(rx2, value) => {
//!         println!("Received from rx2: {}", value);
//!     },
//! }
//! ```

pub mod cases;

pub use cases::{RecvCase, SendCase, Case, SelectedValue, TryRecvError};

/// Select on multiple channels
///
/// # Macro Syntax
///
/// ```rust,no_run
/// use zulon_runtime_io::select;
///
/// select! {
///     recv(channel1, pattern) => {
///         // handle receive from channel1
///     },
///     recv(channel2, pattern) => {
///         // handle receive from channel2
///     },
///     send(channel3, value) => {
///         // handle send to channel3
///     },
/// }
/// ```
///
/// # Behavior
///
/// - Blocks until at least one channel is ready
/// - If multiple channels are ready, selects one fairly
/// - Executes exactly one branch
/// - Returns after the selected branch completes
#[macro_export]
macro_rules! select {
    // Entry point - normalize all branches
    (
        $($branch:tt)*
    ) => {{
        // Convert branches to cases and run select
        $crate::select::__select_internal!(
            branches: [$($branch)*]
        )
    }};
}

/// Internal macro for select implementation
#[macro_export]
#[doc(hidden)]
macro_rules! __select_internal {
    // Single recv branch
    (
        branches: [
            recv($rx:expr, $pattern:pat) => $handler:block
        ]
    ) => {{
        let rx = $rx;
        match rx.recv() {
            Ok($pattern) => $handler,
            Err($crate::channel::RecvError::Empty) => {
                // Block until value is available
                loop {
                    std::thread::yield_once();
                    match rx.recv() {
                        Ok($pattern) => break $handler,
                        Err($crate::channel::RecvError::Empty) => continue,
                        Err($crate::channel::RecvError::Disconnected) => {
                            panic!("select: channel disconnected");
                        }
                    }
                }
            },
            Err($crate::channel::RecvError::Disconnected) => {
                panic!("select: channel disconnected");
            }
        }
    }};

    // Two recv branches - simple polling with fairness
    (
        branches: [
            recv($rx1:expr, $pat1:pat) => $handler1:block,
            recv($rx2:expr, $pat2:pat) => $handler2:block
        ]
    ) => {{
        let rx1 = $rx1;
        let rx2 = $rx2;
        let mut round = 0u32;

        loop {
            // Alternate which channel to check first for fairness
            if round % 2 == 0 {
                // Check rx1 first
                match rx1.recv() {
                    Ok($pat1) => break $handler1,
                    Err($crate::channel::RecvError::Disconnected) => {
                        // rx1 is dead, try rx2
                        match rx2.recv() {
                            Ok($pat2) => break $handler2,
                            Err(_) => panic!("select: all channels disconnected"),
                        }
                    }
                    Err($crate::channel::RecvError::Empty) => {
                        // Try rx2
                        match rx2.recv() {
                            Ok($pat2) => break $handler2,
                            Err($crate::channel::RecvError::Disconnected) => {
                                // rx2 is dead but rx1 might still get data
                                round += 1;
                                std::thread::yield_once();
                                continue;
                            }
                            Err($crate::channel::RecvError::Empty) => {
                                round += 1;
                                std::thread::yield_once();
                                continue;
                            }
                        }
                    }
                }
            } else {
                // Check rx2 first
                match rx2.recv() {
                    Ok($pat2) => break $handler2,
                    Err($crate::channel::RecvError::Disconnected) => {
                        // rx2 is dead, try rx1
                        match rx1.recv() {
                            Ok($pat1) => break $handler1,
                            Err(_) => panic!("select: all channels disconnected"),
                        }
                    }
                    Err($crate::channel::RecvError::Empty) => {
                        // Try rx1
                        match rx1.recv() {
                            Ok($pat1) => break $handler1,
                            Err($crate::channel::RecvError::Disconnected) => {
                                // rx1 is dead but rx2 might still get data
                                round += 1;
                                std::thread::yield_once();
                                continue;
                            }
                            Err($crate::channel::RecvError::Empty) => {
                                round += 1;
                                std::thread::yield_once();
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }};

    // Generic case - not yet implemented
    (
        branches: [$($branches:tt)*]
    ) => {
        compile_error!("select! macro: this pattern is not yet implemented. Currently only 1-2 recv branches are supported.");
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::mpsc;

    #[test]
    fn test_recv_case_creation() {
        let (_tx, rx) = mpsc::channel::<i32>();
        let _case: RecvCase<_, i32> = RecvCase::new(&rx);
    }

    #[test]
    fn test_send_case_creation() {
        let (tx, _rx) = mpsc::channel::<i32>();
        let _case = SendCase::new(&tx, 42);
    }
}
