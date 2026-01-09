// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Token types for the ZULON language

use std::fmt;

/// A token produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The span (location) of the token in source
    pub span: Span,
}

/// The kind of token
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === Keywords ===

    // Control flow
    If,
    Else,
    Match,
    Loop,
    While,
    For,
    In,
    Break,
    Continue,
    Return,
    Defer,

    // Declarations
    Extern,
    Fn,
    Struct,
    Enum,
    Trait,
    Impl,
    Type,
    Let,
    Mut,
    Const,
    Static,

    // Modifiers
    Pub,
    Use,
    Mod,
    Where,

    // Error and effects
    Error,
    Effect,
    Throw,
    Perform,
    Try,
    With,

    // Special values
    True,
    False,
    Null,

    // Types
    Bool,
    Char,
    Str,
    // Int/Float types are represented as Ident with special names
    // e.g., Ident("i32"), Ident("f64")

    // === Identifiers and Literals ===

    /// Identifier or keyword (e.g., variable names, function names)
    Ident(Box<str>),
    /// Integer literal (e.g., 42, 0xFF, 1_000_000)
    IntLiteral(Box<str>),
    /// Float literal (e.g., 3.14, 1e10, 1.5f64)
    FloatLiteral(Box<str>),
    /// String literal (e.g., "hello")
    StringLiteral(Box<str>),
    /// Character literal (e.g., 'a', '\n')
    CharLiteral(char),
    /// Template string (backtick delimited, with interpolation)
    TemplateString(Box<str>),

    // === Operators ===

    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,

    // Assignment operators
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    CaretEq,
    AmpersandEq,
    PipeEq,
    LeftShiftEq,
    RightShiftEq,

    // Comparison
    Equals,      // =
    EqEq,        // ==
    BangEq,      // !=
    Less,        // <
    LessEq,      // <=
    Greater,     // >
    GreaterEq,   // >=

    // Logical
    And,
    Or,
    Bang,

    // Bitwise
    Ampersand,   // &
    Pipe,        // |

    // Shift operators
    LeftShift,   // <<
    RightShift,  // >>

    // Other operators
    Arrow,       // ->
    FatArrow,    // =>
    Dot,         // .
    DotDot,      // ..
    DotDotDot,   // ...
    DotDotEq,    // ..=
    PathSep,     // ::
    Question,    // ?
    Underscore,  // _

    // === Delimiters ===

    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Semicolon,    // ;
    Comma,        // ,
    At,           // @
    Hash,         // #
    Dollar,       // $

    // === Other ===

    /// Whitespace (only tracked for position, not returned to parser)
    Whitespace,
    /// Comment (line or block)
    Comment,

    /// Unknown/invalid token
    Unknown,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(s) => write!(f, "identifier({})", s),
            TokenKind::IntLiteral(n) => write!(f, "integer({})", n),
            TokenKind::FloatLiteral(n) => write!(f, "float({})", n),
            TokenKind::StringLiteral(s) => write!(f, "string(\"{}\")", s),
            TokenKind::CharLiteral(c) => write!(f, "char('{}')", c),
            TokenKind::TemplateString(s) => write!(f, "template({})", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// A position in source code (1-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub const fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.start.line, self.start.column)
    }
}
