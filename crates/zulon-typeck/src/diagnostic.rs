// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration with zulon-diagnostic

use crate::TypeError;
use zulon_diagnostic::{Diagnostic, Span, Suggestion, Loc};
use zulon_parser::Span as ParserSpan;
use std::path::PathBuf;

impl TypeError {
    /// Convert to a Diagnostic
    pub fn to_diagnostic(&self, source_code: &str) -> Diagnostic {
        match self {
            TypeError::TypeMismatch { expected, found, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                let mut diagnostic = Diagnostic::error()
                    .message(format!("type mismatch"))
                    .span(diagnostic_span.clone())
                    .code("E0308")
                    .label(diagnostic_span.clone(), &format!("expected {}", expected))
                    .label(diagnostic_span.clone(), &format!("found {}", found))
                    .note(&format!("expected type: {}", expected))
                    .note(&format!("found type: {}", found));

                // Add suggestions for common type mismatches
                if expected.is_integer() && found.is_integer() {
                    diagnostic = diagnostic.suggestion(Suggestion::new(
                        &format!("consider explicitly converting {} to {}", found, expected),
                        diagnostic_span.clone(),
                        &format!("{} as {}", found, expected),
                    ));
                }

                diagnostic.build()
            }

            TypeError::UndefinedType { name, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("cannot find type `{}` in this scope", name))
                    .span(diagnostic_span.clone())
                    .code("E0412")
                    .label(diagnostic_span.clone(), &format!("not found in this scope"))
                    .note(&format!("did you mean `{}`?", suggest_similar(name)))
                    .build()
            }

            TypeError::UndefinedVariable { name, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("cannot find value `{}` in this scope", name))
                    .span(diagnostic_span.clone())
                    .code("E0425")
                    .label(diagnostic_span.clone(), "not found in this scope")
                    .build()
            }

            TypeError::UndefinedFunction { name, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("cannot find function `{}` in this scope", name))
                    .span(diagnostic_span.clone())
                    .code("E0425")
                    .label(diagnostic_span.clone(), "not found in this scope")
                    .note("functions are declared with `fn`")
                    .build()
            }

            TypeError::NotCallable { ty, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("cannot call non-function type"))
                    .span(diagnostic_span.clone())
                    .code("E0618")
                    .label(diagnostic_span.clone(), &format!("{} is not a function", ty))
                    .build()
            }

            TypeError::ArityMismatch { expected, found, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("expected {} argument{}, found {}",
                        expected, if *expected == 1 { "" } else { "s" }, found))
                    .span(diagnostic_span.clone())
                    .code("E0061")
                    .label(diagnostic_span.clone(),
                        &format!("expected {} argument{}", expected,
                            if *expected == 1 { "" } else { "s" }))
                    .build()
            }

            TypeError::UnknownField { field, ty, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("field `{}` does not exist on type `{}`", field, ty))
                    .span(diagnostic_span.clone())
                    .code("E0609")
                    .label(diagnostic_span.clone(), "unknown field")
                    .note(&format!("available fields: ...")) // TODO: list actual fields
                    .build()
            }

            TypeError::NotIndexable { ty, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("type `{}` is not indexable", ty))
                    .span(diagnostic_span.clone())
                    .code("E0608")
                    .label(diagnostic_span.clone(), "not indexable")
                    .build()
            }

            TypeError::CannotAssignImmutable { span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message("cannot assign to immutable value")
                    .span(diagnostic_span.clone())
                    .code("E0384")
                    .label(diagnostic_span.clone(), "cannot assign twice to immutable variable")
                    .suggestion(Suggestion::new(
                        "consider using a mutable variable",
                        diagnostic_span.clone(),
                        "mut ",
                    ))
                    .build()
            }

            TypeError::CannotBorrowMut { ty, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("cannot borrow `{}` as mutable", ty))
                    .span(diagnostic_span.clone())
                    .code("E0596")
                    .label(diagnostic_span.clone(), "cannot borrow as mutable")
                    .build()
            }

            TypeError::InferenceError { message, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("type inference error: {}", message))
                    .span(diagnostic_span.clone())
                    .code("E0282")
                    .label(diagnostic_span.clone(), "cannot infer type")
                    .note("consider specifying the type explicitly")
                    .build()
            }

            TypeError::MissingGenericParameter { name, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("generic parameter `{}` not provided", name))
                    .span(diagnostic_span.clone())
                    .code("E0392")
                    .label(diagnostic_span.clone(), "generic parameter not provided")
                    .build()
            }

            TypeError::TraitBoundNotSatisfied { trait_name, ty, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("the trait `{}` is not implemented for `{}`", trait_name, ty))
                    .span(diagnostic_span.clone())
                    .code("E0277")
                    .label(diagnostic_span.clone(), &format!("{} doesn't implement {}", ty, trait_name))
                    .build()
            }

            TypeError::RecursiveType { ty, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("recursive type: {} contains itself", ty))
                    .span(diagnostic_span.clone())
                    .code("E0072")
                    .label(diagnostic_span.clone(), "recursive type")
                    .note("recursive types must be indirection")
                    .build()
            }

            TypeError::IntegerOverflow { span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message("integer literal too large")
                    .span(diagnostic_span.clone())
                    .code("E0200")
                    .label(diagnostic_span.clone(), "literal out of range")
                    .build()
            }

            TypeError::CannotConvert { from, to, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);

                Diagnostic::error()
                    .message(format!("cannot convert {} to {}", from, to))
                    .span(diagnostic_span.clone())
                    .code("E0604")
                    .label(diagnostic_span.clone(), &format!("cannot convert"))
                    .suggestion(Suggestion::new(
                        &format!("consider using `{} as {}`", from, to),
                        diagnostic_span.clone(),
                        &format!("{} as {}", from, to),
                    ))
                    .build()
            }
            TypeError::UndefinedEffect { name, span } => {
                let diagnostic_span = parser_span_to_diagnostic_span(span, source_code);
                Diagnostic::error()
                    .message(format!("cannot find effect {} in this scope", name))
                    .span(diagnostic_span.clone())
                    .code("E0605")
                    .label(diagnostic_span.clone(), &format!("undefined effect"))
                    .suggestion(Suggestion::new(
                        &format!("effect {} {{ ... }} must be declared before use", name),
                        diagnostic_span.clone(),
                        &format!("effect {} {{ ... }}", name),
                    ))
                    .build()
            }
        }
    }
}

/// Convert Parser Span to Diagnostic Span
fn parser_span_to_diagnostic_span(span: &ParserSpan, source_code: &str) -> Span {
    let file = Some(PathBuf::from("input.zl"));

    // Calculate offsets from line/column
    let start_offset = estimate_byte_offset(source_code, span.start.line, span.start.column);
    let end_offset = estimate_byte_offset(source_code, span.end.line, span.end.column);

    let lo = Loc::new(file.clone(), span.start.line, span.start.column, start_offset);
    let hi = Loc::new(file, span.end.line, span.end.column, end_offset);

    Span { lo, hi }
}

/// Estimate byte offset from line and column (1-indexed)
fn estimate_byte_offset(source_code: &str, line: usize, column: usize) -> usize {
    let mut current_line = 1;
    let mut offset = 0;

    for (char_offset, c) in source_code.char_indices() {
        if c == '\n' {
            current_line += 1;
        }

        if current_line >= line {
            // We're at or past the target line
            if current_line == line {
                // On the target line, add column-1 for 1-indexed column
                return offset + (column - 1);
            } else {
                // Past the target line, return current offset
                return offset;
            }
        }

        offset = char_offset + c.len_utf8();
    }

    // If we reach here, the position is past the end of the file
    source_code.len()
}

/// Suggest a similar name (placeholder for now)
fn suggest_similar(_name: &str) -> String {
    // TODO: Implement actual fuzzy matching
    format!("a similar type")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ty;

    #[test]
    fn test_type_error_to_diagnostic() {
        let source = "let x: i32 = \"hello\";";
        let span = ParserSpan::new(
            zulon_parser::Position { line: 1, column: 9 },
            zulon_parser::Position { line: 1, column: 18 },
        );

        let error = TypeError::TypeMismatch {
            expected: Ty::I32,
            found: Ty::String,
            span,
        };

        let diagnostic = error.to_diagnostic(source);
        assert_eq!(diagnostic.code, Some("E0308".to_string()));
        assert!(diagnostic.message.contains("type mismatch"));
    }

    #[test]
    fn test_undefined_variable_diagnostic() {
        let source = "undefined_var = 42";
        let span = ParserSpan::new(
            zulon_parser::Position { line: 1, column: 1 },
            zulon_parser::Position { line: 1, column: 14 },
        );

        let error = TypeError::UndefinedVariable {
            name: "undefined_var".to_string(),
            span,
        };

        let diagnostic = error.to_diagnostic(source);
        assert_eq!(diagnostic.code, Some("E0425".to_string()));
        assert!(diagnostic.message.contains("undefined_var"));
    }
}
