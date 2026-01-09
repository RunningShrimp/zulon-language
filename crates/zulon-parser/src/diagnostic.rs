// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration with zulon-diagnostic

use crate::{LexError, LexErrorKind};
use zulon_diagnostic::{Diagnostic, Span, Suggestion, Loc};
use std::path::PathBuf;

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

impl LexError {
    /// Convert to a Diagnostic
    pub fn to_diagnostic(&self, source_code: &str) -> Diagnostic {
        let span = self.position_to_span(source_code);

        let (message, code) = match &self.kind {
            LexErrorKind::InvalidCharacter(c) => {
                (format!("invalid character '{}'", c), Some("E0001"))
            }
            LexErrorKind::UnterminatedString => {
                ("unterminated string literal".to_string(), Some("E0002"))
            }
            LexErrorKind::UnterminatedTemplateString => {
                ("unterminated template string".to_string(), Some("E0003"))
            }
            LexErrorKind::UnterminatedChar => {
                ("unterminated character literal".to_string(), Some("E0004"))
            }
            LexErrorKind::InvalidCharLiteral => {
                ("invalid character literal".to_string(), Some("E0005"))
            }
            LexErrorKind::UnterminatedBlockComment => {
                ("unterminated block comment".to_string(), Some("E0006"))
            }
            LexErrorKind::InvalidNumber(s) => {
                (format!("invalid number format: '{}'", s), Some("E0007"))
            }
            LexErrorKind::InvalidEscapeSequence(c) => {
                (format!("invalid escape sequence '\\{}'", c), Some("E0008"))
            }
            LexErrorKind::UnexpectedEof => {
                ("unexpected end of file".to_string(), Some("E0009"))
            }
            LexErrorKind::UnterminatedInterpolation => {
                ("unterminated string interpolation '${{...}}'".to_string(), Some("E0010"))
            }
        };

        let mut diagnostic = Diagnostic::error()
            .message(message)
            .span(span.clone());

        if let Some(code) = code {
            diagnostic = diagnostic.code(code);
        }

        // Add suggestions for common errors
        match &self.kind {
            LexErrorKind::InvalidCharacter(c) if c.is_whitespace() => {
                diagnostic = diagnostic.suggestion(Suggestion::new(
                    "consider removing this character",
                    span.clone(),
                    "",
                ));
            }
            LexErrorKind::UnterminatedString => {
                diagnostic = diagnostic.suggestion(Suggestion::new(
                    "close the string with a quote (\")",
                    span.clone(),
                    "\"",
                ));
            }
            _ => {}
        }

        diagnostic.build()
    }

    fn position_to_span(&self, source_code: &str) -> Span {
        // Convert Position to Span
        let file = Some(PathBuf::from("input.zl"));

        // Use the line/column from the error position
        // Note: offset is estimated as the byte offset at this position
        let offset = estimate_byte_offset(source_code, self.position.line, self.position.column);

        let loc = Loc::new(file, self.position.line, self.position.column, offset);
        Span::point(loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;
    use zulon_diagnostic::Severity;

    #[test]
    fn test_lex_error_to_diagnostic() {
        let source = "let x = 42";
        let error = LexError {
            kind: LexErrorKind::InvalidCharacter('@'),
            position: Position { line: 1, column: 9 },
        };

        let diagnostic = error.to_diagnostic(source);
        assert_eq!(diagnostic.severity, Severity::Error);
        assert!(diagnostic.code.is_some());
    }

    #[test]
    fn test_unterminated_string_diagnostic() {
        let source = "let x = \"hello";
        let error = LexError {
            kind: LexErrorKind::UnterminatedString,
            position: Position { line: 1, column: 16 },
        };

        let diagnostic = error.to_diagnostic(source);
        assert_eq!(diagnostic.message, "unterminated string literal");
        assert_eq!(diagnostic.code, Some("E0002".to_string()));
        assert!(diagnostic.suggestions.len() > 0);
    }
}
