// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Suggestions for fixing diagnostics

use crate::span::Span;

/// A suggestion for fixing a problem
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The message explaining the suggestion
    pub message: String,
    /// The span to replace
    pub span: Span,
    /// The suggested replacement text
    pub replacement: String,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            replacement: replacement.into(),
        }
    }

    /// Create a suggestion with formatted message
    pub fn format<F>(span: Span, replacement: impl Into<String>, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        Self {
            message: f(),
            span,
            replacement: replacement.into(),
        }
    }

    /// Apply this suggestion to the source code
    pub fn apply(&self, source: &str) -> String {
        if self.span.is_dummy() {
            return source.to_string();
        }

        let start = self.span.lo.offset;
        let end = self.span.hi.offset;

        if start > source.len() || end > source.len() || start > end {
            return source.to_string();
        }

        let mut result = String::with_capacity(source.len() + self.replacement.len());
        result.push_str(&source[..start]);
        result.push_str(&self.replacement);
        result.push_str(&source[end..]);
        result
    }
}
