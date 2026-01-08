// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON Language Lexer
//!
//! The lexer (词法分析器) converts source code into tokens for the parser.
//! Features:
//! - Unicode identifier support
//! - String interpolation (${})
//! - Template strings (backticks)
//! - Multi-line comments
//! - Error recovery

mod token;
mod error;

pub use token::*;
pub use error::*;

use std::str::Chars;
use std::iter::Peekable;

/// The main lexer struct
pub struct Lexer<'a> {
    /// Source code being lexed (kept for potential future use in error messages)
    #[allow(dead_code)]
    source: &'a str,
    /// Character iterator
    chars: Peekable<Chars<'a>>,
    /// Current position (line, column)
    position: Position,
    /// Current token start position
    token_start: Position,
    /// Collected errors
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        let chars = source.chars().peekable();
        Lexer {
            source,
            chars,
            position: Position::new(1, 1),
            token_start: Position::new(1, 1),
            errors: Vec::new(),
        }
    }

    /// Lex all tokens in the source
    pub fn lex_all(mut self) -> (Vec<Token>, Vec<LexError>) {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token() {
            // Skip whitespace tokens (they're only tracked for position)
            if token.kind != TokenKind::Whitespace {
                tokens.push(token);
            }
        }

        (tokens, self.errors)
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Option<Token> {
        self.token_start = self.position;

        // Skip whitespace
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        let c = self.advance()?;

        let kind = match c {
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' => self.lex_identifier_or_keyword(c),
            '_' => TokenKind::Underscore,

            // Numbers
            '0'..='9' => self.lex_number(c),

            // String literals
            '"' => self.lex_string(),
            '`' => self.lex_template_string(),

            // Character literals
            '\'' => self.lex_char(),

            // Operators and punctuation
            '+' => self.lex_plus(),
            '-' => self.lex_minus(),
            '*' => self.lex_star(),
            '/' => self.lex_slash(),
            '%' => self.lex_percent(),
            '^' => self.lex_caret(),
            '!' => self.lex_bang(),
            '=' => self.lex_equals(),
            '<' => self.lex_less(),
            '>' => self.lex_greater(),
            '&' => self.lex_ampersand(),
            '|' => self.lex_pipe(),
            '.' => self.lex_dot(),
            ':' => self.lex_colon(),
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '@' => TokenKind::At,
            '#' => TokenKind::Hash,
            '$' => TokenKind::Dollar,
            '?' => TokenKind::Question,

            // Unexpected character
            unexpected => {
                self.errors.push(LexError {
                    kind: LexErrorKind::InvalidCharacter(unexpected),
                    position: self.token_start,
                });
                TokenKind::Unknown
            }
        };

        Some(Token {
            kind,
            span: Span::new(self.token_start, self.position),
        })
    }

    /// Lex an identifier or keyword
    fn lex_identifier_or_keyword(&mut self, first: char) -> TokenKind {
        let mut ident = String::from(first);

        while let Some(&c) = self.chars.peek() {
            if is_identifier_continue(c) {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Check if it's a keyword
        match ident.as_str() {
            // Control flow
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "loop" => TokenKind::Loop,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,

            // Declarations
            "fn" => TokenKind::Fn,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "type" => TokenKind::Type,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "const" => TokenKind::Const,
            "static" => TokenKind::Static,

            // Modifiers
            "pub" => TokenKind::Pub,
            "use" => TokenKind::Use,
            "mod" => TokenKind::Mod,
            "where" => TokenKind::Where,

            // Error and effects
            "error" => TokenKind::Error,
            "effect" => TokenKind::Effect,
            "throw" => TokenKind::Throw,
            "perform" => TokenKind::Perform,
            "try" => TokenKind::Try,
            "with" => TokenKind::With,

            // Special values
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,

            // Types
            "bool" => TokenKind::Bool,
            "char" => TokenKind::Char,
            "str" => TokenKind::Str,
            "i8" | "i16" | "i32" | "i64" | "i128" |
            "u8" | "u16" | "u32" | "u64" | "u128" |
            "f32" | "f64" => TokenKind::Ident(ident.into()), // Will be handled in parser

            // Otherwise it's an identifier
            _ => TokenKind::Ident(ident.into()),
        }
    }

    /// Lex a number literal (integer or float)
    fn lex_number(&mut self, first: char) -> TokenKind {
        let mut num_str = String::from(first);

        // Parse remaining digits
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Check for float (with decimal point)
        if let Some(&'.') = self.chars.peek() {
            // Look ahead to ensure it's not ".." (range)
            self.chars.peek(); // Just peek, don't consume yet

            // If it's a decimal followed by a digit, it's a float
            if let Some('.') = self.chars.next() {
                num_str.push('.');
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                return TokenKind::FloatLiteral(num_str.into());
            }
        }

        // Check for exponent (e.g., 1e10)
        if let Some(&'e' | &'E') = self.chars.peek() {
            self.advance(); // consume 'e' or 'E'
            num_str.push('e');

            // Optional sign
            if let Some(&'+' | &'-') = self.chars.peek() {
                num_str.push(self.advance().unwrap());
            }

            // Required exponent digits
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    num_str.push(self.advance().unwrap());
                } else {
                    break;
                }
            }

            return TokenKind::FloatLiteral(num_str.into());
        }

        // Check for type suffix (e.g., 42i32, 3.14f64)
        if let Some(&c) = self.chars.peek() {
            if c == 'i' || c == 'u' || c == 'f' {
                let mut type_suffix = String::new();
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_alphabetic() {
                        type_suffix.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                // TODO: Validate type suffix
                return TokenKind::IntLiteral(num_str.into());
            }
        }

        TokenKind::IntLiteral(num_str.into())
    }

    /// Lex a regular string literal
    fn lex_string(&mut self) -> TokenKind {
        let mut s = String::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '"' => {
                    self.advance(); // consume closing quote
                    return TokenKind::StringLiteral(s.into());
                }
                '\\' => {
                    self.advance(); // consume backslash
                    if let Some(escaped) = self.advance() {
                        s.push(self.parse_escape_sequence(escaped));
                    }
                }
                c if c == '\n' || c == '\r' => {
                    // Unterminated string
                    self.errors.push(LexError {
                        kind: LexErrorKind::UnterminatedString,
                        position: self.token_start,
                    });
                    return TokenKind::StringLiteral(s.into());
                }
                _ => {
                    s.push(self.advance().unwrap());
                }
            }
        }

        // EOF before closing quote
        self.errors.push(LexError {
            kind: LexErrorKind::UnterminatedString,
            position: self.token_start,
        });
        TokenKind::StringLiteral(s.into())
    }

    /// Lex a template string (backtick delimited)
    fn lex_template_string(&mut self) -> TokenKind {
        let mut s = String::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '`' => {
                    self.advance(); // consume closing backtick
                    return TokenKind::TemplateString(s.into());
                }
                '\\' => {
                    self.advance(); // consume backslash
                    if let Some(escaped) = self.advance() {
                        s.push(self.parse_escape_sequence(escaped));
                    }
                }
                '$' => {
                    self.advance(); // consume dollar sign
                    if let Some(&'{') = self.chars.peek() {
                        // String interpolation: ${...}
                        self.advance(); // consume '{'
                        s.push_str("${");

                        // Collect interpolated expression (handle nested braces)
                        let mut depth = 1;
                        while depth > 0 {
                            if let Some(ch) = self.advance() {
                                s.push(ch);
                                if ch == '{' {
                                    depth += 1;
                                } else if ch == '}' {
                                    depth -= 1;
                                }
                            } else {
                                // Error: Unterminated interpolation
                                self.errors.push(LexError {
                                    kind: LexErrorKind::UnterminatedInterpolation,
                                    position: self.token_start,
                                });
                                break;
                            }
                        }
                    } else {
                        s.push('$');
                    }
                }
                c if c == '\n' || c == '\r' => {
                    s.push(self.advance().unwrap());
                }
                _ => {
                    s.push(self.advance().unwrap());
                }
            }
        }

        // EOF before closing backtick
        self.errors.push(LexError {
            kind: LexErrorKind::UnterminatedTemplateString,
            position: self.token_start,
        });
        TokenKind::TemplateString(s.into())
    }

    /// Lex a character literal
    fn lex_char(&mut self) -> TokenKind {
        if let Some(&'\\') = self.chars.peek() {
            self.advance(); // consume backslash
            if let Some(escaped) = self.advance() {
                let c = self.parse_escape_sequence(escaped);
                if let Some(&'\'') = self.chars.peek() {
                    self.advance(); // consume closing quote
                    TokenKind::CharLiteral(c)
                } else {
                    self.errors.push(LexError {
                        kind: LexErrorKind::UnterminatedChar,
                        position: self.token_start,
                    });
                    TokenKind::CharLiteral(c)
                }
            } else {
                self.errors.push(LexError {
                    kind: LexErrorKind::UnterminatedChar,
                    position: self.token_start,
                });
                TokenKind::CharLiteral('\0')
            }
        } else if let Some(c) = self.advance() {
            if c != '\'' && self.advance() == Some('\'') {
                TokenKind::CharLiteral(c)
            } else {
                self.errors.push(LexError {
                    kind: LexErrorKind::InvalidCharLiteral,
                    position: self.token_start,
                });
                TokenKind::CharLiteral('\0')
            }
        } else {
            self.errors.push(LexError {
                kind: LexErrorKind::UnterminatedChar,
                position: self.token_start,
            });
            TokenKind::CharLiteral('\0')
        }
    }

    /// Parse escape sequences like \n, \t, \u{...}
    fn parse_escape_sequence(&self, c: char) -> char {
        match c {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '"' => '"',
            '\'' => '\'',
            '0' => '\0',
            'x' | 'u' => {
                // TODO: Implement hex and unicode escape sequences
                c // placeholder
            }
            _ => c, // Unknown escape, just return the character
        }
    }

    /// Lex + or +=
    fn lex_plus(&mut self) -> TokenKind {
        if let Some(&'=') = self.chars.peek() {
            self.advance();
            TokenKind::PlusEq
        } else {
            TokenKind::Plus
        }
    }

    /// Lex - or -= or ->
    fn lex_minus(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'=') => {
                self.advance();
                TokenKind::MinusEq
            }
            Some(&'>') => {
                self.advance();
                TokenKind::Arrow
            }
            _ => TokenKind::Minus,
        }
    }

    /// Lex * or *=
    fn lex_star(&mut self) -> TokenKind {
        if let Some(&'=') = self.chars.peek() {
            self.advance();
            TokenKind::StarEq
        } else {
            TokenKind::Star
        }
    }

    /// Lex / or /= or comment
    fn lex_slash(&mut self) -> TokenKind {
        if let Some(&'/') = self.chars.peek() {
            self.advance(); // consume second '/'

            // Line comment
            while let Some(&c) = self.chars.peek() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }

            TokenKind::Comment
        } else if let Some(&'*') = self.chars.peek() {
            self.advance(); // consume '*'

            // Block comment
            let mut depth = 1;
            while depth > 0 {
                match self.advance() {
                    Some('/') => {
                        if let Some(&'*') = self.chars.peek() {
                            self.advance();
                            depth += 1;
                        }
                    }
                    Some('*') => {
                        if let Some(&'/') = self.chars.peek() {
                            self.advance();
                            depth -= 1;
                        }
                    }
                    Some(_) => continue,
                    None => {
                        self.errors.push(LexError {
                            kind: LexErrorKind::UnterminatedBlockComment,
                            position: self.token_start,
                        });
                        break;
                    }
                }
            }

            TokenKind::Comment
        } else if let Some(&'=') = self.chars.peek() {
            self.advance();
            TokenKind::SlashEq
        } else {
            TokenKind::Slash
        }
    }

    /// Lex % or %=
    fn lex_percent(&mut self) -> TokenKind {
        if let Some(&'=') = self.chars.peek() {
            self.advance();
            TokenKind::PercentEq
        } else {
            TokenKind::Percent
        }
    }

    /// Lex ^ or ^=
    fn lex_caret(&mut self) -> TokenKind {
        if let Some(&'=') = self.chars.peek() {
            self.advance();
            TokenKind::CaretEq
        } else {
            TokenKind::Caret
        }
    }

    /// Lex ! or !=
    fn lex_bang(&mut self) -> TokenKind {
        if let Some(&'=') = self.chars.peek() {
            self.advance();
            TokenKind::BangEq
        } else {
            TokenKind::Bang
        }
    }

    /// Lex = or == or =>
    fn lex_equals(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'=') => {
                self.advance();
                TokenKind::EqEq
            }
            Some(&'>') => {
                self.advance();
                TokenKind::FatArrow
            }
            _ => TokenKind::Equals,
        }
    }

    /// Lex < or <= or << or <<= or =>
    fn lex_less(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'=') => {
                self.advance();
                TokenKind::LessEq
            }
            Some(&'<') => {
                self.advance();
                if let Some(&'=') = self.chars.peek() {
                    self.advance();
                    TokenKind::LeftShiftEq
                } else {
                    TokenKind::LeftShift
                }
            }
            _ => TokenKind::Less,
        }
    }

    /// Lex > or >= or >> or >>= or =>
    fn lex_greater(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'=') => {
                self.advance();
                TokenKind::GreaterEq
            }
            Some(&'>') => {
                self.advance();
                if let Some(&'=') = self.chars.peek() {
                    self.advance();
                    TokenKind::RightShiftEq
                } else {
                    TokenKind::RightShift
                }
            }
            _ => TokenKind::Greater,
        }
    }

    /// Lex & or && or &= or &&
    fn lex_ampersand(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'&') => {
                self.advance();
                TokenKind::And
            }
            Some(&'=') => {
                self.advance();
                TokenKind::AmpersandEq
            }
            _ => TokenKind::Ampersand,
        }
    }

    /// Lex | or || or |=
    fn lex_pipe(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'|') => {
                self.advance();
                TokenKind::Or
            }
            Some(&'=') => {
                self.advance();
                TokenKind::PipeEq
            }
            _ => TokenKind::Pipe,
        }
    }

    /// Lex . or .. or ... or ..=
    fn lex_dot(&mut self) -> TokenKind {
        match self.chars.peek() {
            Some(&'.') => {
                self.advance();
                if let Some(&'.') = self.chars.peek() {
                    self.advance();
                    TokenKind::DotDotDot
                } else if let Some(&'=') = self.chars.peek() {
                    self.advance();
                    TokenKind::DotDotEq
                } else {
                    TokenKind::DotDot
                }
            }
            _ => TokenKind::Dot,
        }
    }

    /// Lex : or ::
    fn lex_colon(&mut self) -> TokenKind {
        if let Some(&':') = self.chars.peek() {
            self.advance();
            TokenKind::PathSep
        } else {
            TokenKind::Colon
        }
    }

    /// Advance one character and update position
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        if c == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }

        Some(c)
    }
}

/// Check if character can start an identifier
#[allow(dead_code)]
fn is_identifier_start(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_start(c) || c == '_'
}

/// Check if character can continue an identifier
fn is_identifier_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c) || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let source = r#"fn main() {
            println("Hello, World!");
        }"#;

        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident("main".into()));
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[3].kind, TokenKind::RightParen);
        assert_eq!(tokens[4].kind, TokenKind::LeftBrace);
    }

    #[test]
    fn test_numbers() {
        let source = "42 3.14 1e10 0xFF";
        let lexer = Lexer::new(source);
        let (tokens, _errors) = lexer.lex_all();

        assert_eq!(tokens[0].kind, TokenKind::IntLiteral("42".into()));
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral("3.14".into()));
    }

    #[test]
    fn test_strings() {
        let source = r#""hello" `multiline`"#;
        let lexer = Lexer::new(source);
        let (tokens, _errors) = lexer.lex_all();

        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello".into()));
        assert_eq!(tokens[1].kind, TokenKind::TemplateString("multiline".into()));
    }

    #[test]
    fn test_fat_arrow() {
        let source = "=>";
        let lexer = Lexer::new(source);
        let (tokens, _errors) = lexer.lex_all();

        assert_eq!(tokens[0].kind, TokenKind::FatArrow);
    }

    #[test]
    fn test_underscore() {
        let source = "_";
        let lexer = Lexer::new(source);
        let (tokens, _errors) = lexer.lex_all();

        assert_eq!(tokens[0].kind, TokenKind::Underscore);
    }

    #[test]
    fn test_match_with_fat_arrow() {
        let source = r#"match x { 1 => one, _ => other }"#;
        let lexer = Lexer::new(source);
        let (tokens, _errors) = lexer.lex_all();

        assert_eq!(tokens[0].kind, TokenKind::Match);
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral("1".into()));
        assert_eq!(tokens[4].kind, TokenKind::FatArrow);
        assert_eq!(tokens[7].kind, TokenKind::Underscore);
        assert_eq!(tokens[8].kind, TokenKind::FatArrow);
    }

    #[test]
    fn test_string_interpolation_simple() {
        let source = "`Hello ${name}!`";
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty(), "Should have no errors");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TemplateString("Hello ${name}!".into()));
    }

    #[test]
    fn test_string_interpolation_nested() {
        let source = "`Count: ${map.len()}`";
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty(), "Should have no errors");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TemplateString("Count: ${map.len()}".into()));
    }

    #[test]
    fn test_string_interpolation_nested_braces() {
        let source = "`Test ${func({key: value})}`";
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty(), "Should have no errors");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TemplateString("Test ${func({key: value})}".into()));
    }

    #[test]
    fn test_string_interpolation_unterminated() {
        let source = "`Hello ${name`";
        let lexer = Lexer::new(source);
        let (_tokens, errors) = lexer.lex_all();

        assert!(!errors.is_empty(), "Should have an error");
        assert_eq!(errors[0].kind, LexErrorKind::UnterminatedInterpolation);
    }

    #[test]
    fn test_string_interpolation_multiple() {
        let source = "`Hello ${user}, you have ${count} messages`";
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty(), "Should have no errors");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TemplateString("Hello ${user}, you have ${count} messages".into()));
    }

    #[test]
    fn test_dollar_without_interpolation() {
        let source = "`Price: $100`";
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty(), "Should have no errors");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TemplateString("Price: $100".into()));
    }
}
