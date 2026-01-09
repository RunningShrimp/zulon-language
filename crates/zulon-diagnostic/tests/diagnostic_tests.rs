// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Diagnostic system tests

use zulon_diagnostic::prelude::*;
use std::path::PathBuf;

#[test]
fn test_span_creation() {
    let file = Some(PathBuf::from("test.zl"));
    let lo = Loc::new(file.clone(), 5, 10, 100);
    let hi = Loc::new(file, 5, 15, 105);
    let span = Span::new(lo, hi);

    assert_eq!(span.lo.line, 5);
    assert_eq!(span.lo.column, 10);
    assert_eq!(span.hi.column, 15);
    assert_eq!(span.len(), 5);
}

#[test]
fn test_span_merge() {
    let file = Some(PathBuf::from("test.zl"));
    let span1 = Span::new(
        Loc::new(file.clone(), 1, 1, 0),
        Loc::new(file.clone(), 1, 10, 9),
    );
    let span2 = Span::new(
        Loc::new(file.clone(), 2, 1, 10),
        Loc::new(file, 2, 20, 29),
    );

    let merged = span1.merge(span2);
    assert_eq!(merged.lo.line, 1);
    assert_eq!(merged.hi.line, 2);
}

#[test]
fn test_diagnostic_builder() {
    let file = Some(PathBuf::from("test.zl"));
    let span = Span::new(
        Loc::new(file.clone(), 5, 10, 100),
        Loc::new(file, 5, 15, 105),
    );

    let diagnostic = Diagnostic::error()
        .message("test error")
        .span(span)
        .code("E0001")
        .build();

    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.message, "test error");
    assert_eq!(diagnostic.code, Some("E0001".to_string()));
    assert!(diagnostic.span.is_some());
}

#[test]
fn test_diagnostic_with_labels() {
    let file = Some(PathBuf::from("test.zl"));
    let span = Span::new(
        Loc::new(file.clone(), 5, 10, 100),
        Loc::new(file.clone(), 5, 15, 105),
    );

    let diagnostic = Diagnostic::error()
        .message("type mismatch")
        .span(span.clone())
        .label(span, "expected i32")
        .build();

    assert_eq!(diagnostic.labels.len(), 1);
    assert_eq!(diagnostic.labels[0].message, "expected i32");
}

#[test]
fn test_diagnostic_with_notes() {
    let diagnostic = Diagnostic::error()
        .message("test error")
        .note("this is a note")
        .note("another note")
        .build();

    assert_eq!(diagnostic.notes.len(), 2);
    assert_eq!(diagnostic.notes[0], "this is a note");
}

#[test]
fn test_diagnostic_with_suggestions() {
    let file = Some(PathBuf::from("test.zl"));
    let span = Span::new(
        Loc::new(file.clone(), 5, 10, 100),
        Loc::new(file.clone(), 5, 15, 105),
    );

    let suggestion = Suggestion::new(
        "try this instead",
        span,
        "fixed_code",
    );

    let diagnostic = Diagnostic::error()
        .message("test error")
        .suggestion(suggestion)
        .build();

    assert_eq!(diagnostic.suggestions.len(), 1);
    assert_eq!(diagnostic.suggestions[0].message, "try this instead");
}

#[test]
fn test_suggestion_apply() {
    let source = "let x = old_value";
    let file = Some(PathBuf::from("test.zl"));

    // "old_value" starts at offset 8 and ends at offset 17 (length 9)
    let span = Span::new(
        Loc::new(file.clone(), 1, 9, 8),
        Loc::new(file, 1, 18, 17),
    );

    let suggestion = Suggestion::new(
        "use new_value",
        span,
        "new_value",
    );

    let result = suggestion.apply(source);
    assert_eq!(result, "let x = new_value");
}

#[test]
fn test_diagnostic_display() {
    let file = Some(PathBuf::from("test.zl"));
    let span = Span::new(
        Loc::new(file.clone(), 5, 10, 100),
        Loc::new(file, 5, 15, 105),
    );

    let diagnostic = Diagnostic::error()
        .message("test error")
        .span(span)
        .code("E0001")
        .build();

    let source = "fn main() {}\nfn test() { error_code }\n";
    let output = diagnostic.display_with_context(source, false);

    assert!(output.contains("error"));
    assert!(output.contains("E0001"));
    assert!(output.contains("test error"));
}

#[test]
fn test_severity_display() {
    assert_eq!(Severity::Error.name(), "error");
    assert_eq!(Severity::Warning.name(), "warning");
    assert_eq!(Severity::Note.name(), "note");
    assert_eq!(Severity::Help.name(), "help");
}

#[test]
fn test_diagnostic_chaining() {
    let file = Some(PathBuf::from("test.zl"));
    let span = Span::new(
        Loc::new(file.clone(), 5, 10, 100),
        Loc::new(file.clone(), 5, 15, 105),
    );

    let related = Diagnostic::note()
        .message("related info")
        .span(span.clone())
        .build();

    let main = Diagnostic::error()
        .message("main error")
        .span(span)
        .related(related)
        .build();

    assert_eq!(main.related.len(), 1);
    assert_eq!(main.related[0].message, "related info");
}

#[test]
fn test_multiline_diagnostic() {
    let file = Some(PathBuf::from("test.zl"));
    let span1 = Span::new(
        Loc::new(file.clone(), 5, 10, 100),
        Loc::new(file.clone(), 5, 15, 105),
    );
    let span2 = Span::new(
        Loc::new(file.clone(), 6, 5, 110),
        Loc::new(file, 6, 10, 115),
    );

    let diagnostic = Diagnostic::error()
        .message("multi-line error")
        .span(span1.clone())
        .label(span1, "first issue")
        .label(span2, "second issue")
        .build();

    assert_eq!(diagnostic.labels.len(), 2);
}
