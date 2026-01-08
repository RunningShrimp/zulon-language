// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Language Parser
//!
//! This crate provides the lexer and parser for the ZULON programming language.
//!
//! ## Lexer
//!
//! The lexer (词法分析器) converts source code into tokens:
//!
//! ```rust
//! use zulon_parser::lexer::Lexer;
//!
//! let source = r#"fn main() { println("Hello, World!"); }"#;
//! let lexer = Lexer::new(source);
//! let (tokens, errors) = lexer.lex_all();
//!
//! for token in tokens {
//!     println!("{:?}", token);
//! }
//! ```
//!
//! ## Features
//!
//! - Unicode identifier support
//! - String interpolation with `${}`
//! - Template strings with backticks
//! - Multi-line comments
//! - Error recovery

pub mod lexer;
pub mod ast;
pub mod parser;

pub use lexer::{Lexer, Token, TokenKind, Span, Position, LexError, LexErrorKind};
pub use ast::*;
pub use parser::{Parser, ParseError, ParseResult};
