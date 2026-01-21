// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON Diagnostic System
//!
//! This crate provides rich error reporting and diagnostics for the ZULON compiler.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod diagnostic;
mod display;
mod error_codes;
mod label;
mod severity;
mod span;
mod suggestion;
mod type_display;

pub use diagnostic::Diagnostic;
pub use error_codes::{ErrorCategory, ErrorCode};
pub use label::Label;
pub use severity::Severity;
pub use span::{FileId, Loc, Span};
pub use suggestion::Suggestion;
pub use type_display::{format_type_list, format_type_mismatch, TypeDisplay};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        format_type_list, format_type_mismatch, Diagnostic, ErrorCategory, ErrorCode, FileId,
        Label, Loc, Severity, Span, Suggestion, TypeDisplay,
    };
}
