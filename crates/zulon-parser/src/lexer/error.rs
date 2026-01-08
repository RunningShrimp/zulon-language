// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Lexer error types

use std::fmt;

/// An error produced by the lexer
#[derive(Debug, Clone)]
pub struct LexError {
    /// The kind of error
    pub kind: LexErrorKind,
    /// The position where the error occurred
    pub position: super::token::Position,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.position, self.kind)
    }
}

impl std::error::Error for LexError {}

/// The kind of lexical error
#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    /// Invalid character in source
    InvalidCharacter(char),
    /// Unterminated string literal
    UnterminatedString,
    /// Unterminated template string
    UnterminatedTemplateString,
    /// Unterminated character literal
    UnterminatedChar,
    /// Invalid character literal (e.g., empty or too long)
    InvalidCharLiteral,
    /// Unterminated block comment
    UnterminatedBlockComment,
    /// Invalid number format (e.g., 12abc)
    InvalidNumber(String),
    /// Invalid escape sequence
    InvalidEscapeSequence(char),
    /// Unexpected end of file
    UnexpectedEof,
    /// Unterminated string interpolation (${...})
    UnterminatedInterpolation,
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::InvalidCharacter(c) => {
                write!(f, "invalid character '{}'", c)
            }
            LexErrorKind::UnterminatedString => {
                write!(f, "unterminated string literal")
            }
            LexErrorKind::UnterminatedTemplateString => {
                write!(f, "unterminated template string")
            }
            LexErrorKind::UnterminatedChar => {
                write!(f, "unterminated character literal")
            }
            LexErrorKind::InvalidCharLiteral => {
                write!(f, "invalid character literal")
            }
            LexErrorKind::UnterminatedBlockComment => {
                write!(f, "unterminated block comment")
            }
            LexErrorKind::InvalidNumber(s) => {
                write!(f, "invalid number format: '{}'", s)
            }
            LexErrorKind::InvalidEscapeSequence(c) => {
                write!(f, "invalid escape sequence '\\{}'", c)
            }
            LexErrorKind::UnexpectedEof => {
                write!(f, "unexpected end of file")
            }
            LexErrorKind::UnterminatedInterpolation => {
                write!(f, "unterminated string interpolation '${{...}}'")
            }
        }
    }
}
