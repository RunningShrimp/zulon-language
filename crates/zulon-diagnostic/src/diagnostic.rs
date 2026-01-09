// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Diagnostic type

use crate::label::Label;
use crate::severity::Severity;
use crate::span::Span;
use crate::suggestion::Suggestion;

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The severity level
    pub severity: Severity,
    /// The primary error message
    pub message: String,
    /// The primary code location
    pub span: Option<Span>,
    /// Additional labels for related locations
    pub labels: Vec<Label>,
    /// Additional notes
    pub notes: Vec<String>,
    /// Suggestions for fixing
    pub suggestions: Vec<Suggestion>,
    /// Related diagnostics
    pub related: Vec<Diagnostic>,
    /// Error code (e.g., "E0308")
    pub code: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error() -> DiagnosticBuilder {
        DiagnosticBuilder::new(Severity::Error)
    }

    /// Create a new warning diagnostic
    pub fn warning() -> DiagnosticBuilder {
        DiagnosticBuilder::new(Severity::Warning)
    }

    /// Create a new note diagnostic
    pub fn note() -> DiagnosticBuilder {
        DiagnosticBuilder::new(Severity::Note)
    }

    /// Create a new help diagnostic
    pub fn help() -> DiagnosticBuilder {
        DiagnosticBuilder::new(Severity::Help)
    }

    /// Create a diagnostic from a builder
    pub fn from_builder(builder: DiagnosticBuilder) -> Self {
        builder.build()
    }

    /// Display this diagnostic with source code context
    pub fn display(&self, _source: &str) -> String {
        // This will be implemented in the display module
        format!("{}: {}", self.severity, self.message)
    }

    /// Add a label to this diagnostic
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::new(span, message));
        self
    }

    /// Add a note to this diagnostic
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a suggestion to this diagnostic
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add a related diagnostic
    pub fn with_related(mut self, related: Diagnostic) -> Self {
        self.related.push(related);
        self
    }

    /// Set the error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Builder for creating diagnostics
pub struct DiagnosticBuilder {
    diagnostic: Diagnostic,
}

impl DiagnosticBuilder {
    fn new(severity: Severity) -> Self {
        Self {
            diagnostic: Diagnostic {
                severity,
                message: String::new(),
                span: None,
                labels: Vec::new(),
                notes: Vec::new(),
                suggestions: Vec::new(),
                related: Vec::new(),
                code: None,
            },
        }
    }

    /// Set the message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.diagnostic.message = message.into();
        self
    }

    /// Set the primary span
    pub fn span(mut self, span: Span) -> Self {
        self.diagnostic.span = Some(span);
        self
    }

    /// Add a label
    pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.diagnostic.labels.push(Label::new(span, message));
        self
    }

    /// Add a note
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.diagnostic.notes.push(note.into());
        self
    }

    /// Add a suggestion
    pub fn suggestion(mut self, suggestion: Suggestion) -> Self {
        self.diagnostic.suggestions.push(suggestion);
        self
    }

    /// Add a related diagnostic
    pub fn related(mut self, related: Diagnostic) -> Self {
        self.diagnostic.related.push(related);
        self
    }

    /// Set the error code
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.diagnostic.code = Some(code.into());
        self
    }

    /// Build the diagnostic
    pub fn build(self) -> Diagnostic {
        self.diagnostic
    }
}
