// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Diagnostic severity levels

use std::fmt;

/// The severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    /// An error - compilation will fail
    Error,
    /// A warning - compilation succeeds but may indicate a problem
    Warning,
    /// A note - additional information
    Note,
    /// A help suggestion - advice for fixing the problem
    Help,
}

impl Severity {
    /// Get the color code for this severity (for terminal output)
    pub fn color_code(&self) -> &str {
        match self {
            Severity::Error => "\x1b[31m",     // Red
            Severity::Warning => "\x1b[33m",   // Yellow
            Severity::Note => "\x1b[36m",      // Cyan
            Severity::Help => "\x1b[32m",      // Green
        }
    }

    /// Get the reset color code
    pub fn reset_code() -> &'static str {
        "\x1b[0m"
    }

    /// Get the display name of this severity
    pub fn name(&self) -> &str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
