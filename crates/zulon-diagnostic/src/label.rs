// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Diagnostic labels for marking spans with messages

use crate::span::Span;

/// A label attached to a span
#[derive(Debug, Clone)]
pub struct Label {
    /// The span to label
    pub span: Span,
    /// The message for this label
    pub message: String,
}

impl Label {
    /// Create a new label
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    /// Create a label with a span and a formatted message
    pub fn format<F>(span: Span, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        Self {
            span,
            message: f(),
        }
    }
}
