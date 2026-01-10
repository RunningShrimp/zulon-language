// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Interest type for event notification

use std::ops::{BitOr, BitOrAssign};

/// Interest in events
///
/// Describes which events an event source wants to be notified about.
/// This can be combined with bitwise OR to monitor multiple event types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interest {
    readable: bool,
    writable: bool,
    error: bool,
}

impl Interest {
    /// No interest (placeholder)
    pub const NONE: Interest = Interest {
        readable: false,
        writable: false,
        error: false,
    };

    /// Interest in readable events
    pub const READABLE: Interest = Interest {
        readable: true,
        writable: false,
        error: false,
    };

    /// Interest in writable events
    pub const WRITABLE: Interest = Interest {
        readable: false,
        writable: true,
        error: false,
    };

    /// Interest in error events
    pub const ERROR: Interest = Interest {
        readable: false,
        writable: false,
        error: true,
    };

    /// Interest in all events
    pub const ALL: Interest = Interest {
        readable: true,
        writable: true,
        error: true,
    };

    /// Check if interested in readable events
    #[inline]
    pub const fn is_readable(&self) -> bool {
        self.readable
    }

    /// Check if interested in writable events
    #[inline]
    pub const fn is_writable(&self) -> bool {
        self.writable
    }

    /// Check if interested in error events
    #[inline]
    pub const fn is_error(&self) -> bool {
        self.error
    }

    /// Create a new interest
    ///
    /// # Arguments
    ///
    /// * `readable` - Interest in readable events
    /// * `writable` - Interest in writable events
    /// * `error` - Interest in error events
    #[inline]
    pub const fn new(readable: bool, writable: bool, error: bool) -> Self {
        Interest {
            readable,
            writable,
            error,
        }
    }
}

impl BitOr for Interest {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        Interest {
            readable: self.readable || other.readable,
            writable: self.writable || other.writable,
            error: self.error || other.error,
        }
    }
}

impl BitOrAssign for Interest {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.readable = self.readable || other.readable;
        self.writable = self.writable || other.writable;
        self.error = self.error || other.error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_constants() {
        assert!(Interest::READABLE.is_readable());
        assert!(!Interest::READABLE.is_writable());

        assert!(Interest::WRITABLE.is_writable());
        assert!(!Interest::WRITABLE.is_readable());

        assert!(Interest::ERROR.is_error());
    }

    #[test]
    fn test_interest_bitor() {
        let interest = Interest::READABLE | Interest::WRITABLE;
        assert!(interest.is_readable());
        assert!(interest.is_writable());
        assert!(!interest.is_error());
    }

    #[test]
    fn test_interest_all() {
        assert!(Interest::ALL.is_readable());
        assert!(Interest::ALL.is_writable());
        assert!(Interest::ALL.is_error());
    }

    #[test]
    fn test_interest_new() {
        let interest = Interest::new(true, true, false);
        assert!(interest.is_readable());
        assert!(interest.is_writable());
        assert!(!interest.is_error());
    }

    #[test]
    fn test_interest_bitor_assign() {
        let mut interest = Interest::READABLE;
        interest |= Interest::WRITABLE;
        assert!(interest.is_readable());
        assert!(interest.is_writable());
    }
}
