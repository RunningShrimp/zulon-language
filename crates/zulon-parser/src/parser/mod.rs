// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON Parser (语法分析器)
//!
//! The parser converts tokens into an Abstract Syntax Tree (AST).

use crate::lexer::{Lexer, Token, TokenKind, Span};
use crate::ast::*;
use std::iter::Peekable;
use std::vec::IntoIter;

/// Parser error
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
        span: Span,
    },

    #[error("unexpected end of input")]
    UnexpectedEof {
        span: Span,
    },

    #[error("invalid syntax: {message}")]
    InvalidSyntax {
        message: String,
        span: Span,
    },

    #[error("error in module")]
    ModuleError {
        #[source]
        source: Box<ParseError>,
    },
}

/// Parser result type
pub type ParseResult<T> = Result<T, ParseError>;

/// The ZULON parser
pub struct Parser {
    /// Tokens from the lexer
    tokens: Peekable<IntoIter<Token>>,
    /// Current token
    current: Option<Token>,
}

impl Parser {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens = tokens.into_iter().peekable();
        let current = tokens.next();

        Parser {
            tokens,
            current,
        }
    }

    /// Create a parser from source code
    pub fn from_source(source: &str) -> Self {
        let lexer = Lexer::new(source);
        let (tokens, _lex_errors) = lexer.lex_all();

        // Filter out whitespace and comments
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Whitespace | TokenKind::Comment))
            .collect();

        Self::new(tokens)
    }

    /// Parse an entire compilation unit
    pub fn parse(&mut self) -> ParseResult<Ast> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            if let Some(item) = self.parse_item()? {
                items.push(item);
            }
        }

        Ok(Ast::new(items))
    }

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool {
        self.current.is_none()
    }

    /// Get the current token kind
    fn current_kind(&self) -> Option<&TokenKind> {
        self.current.as_ref().map(|t| &t.kind)
    }

    /// Peek at the next token kind without consuming it
    fn peek_kind(&mut self) -> Option<&TokenKind> {
        self.tokens.peek().map(|t| &t.kind)
    }

    /// Get the current span
    fn current_span(&self) -> Span {
        self.current
            .as_ref()
            .map(|t| t.span)
            .unwrap_or_else(|| Span {
                start: crate::lexer::Position { line: 0, column: 0 },
                end: crate::lexer::Position { line: 0, column: 0 },
            })
    }

    /// Advance to the next token
    fn advance(&mut self) -> Option<Token> {
        let token = self.current.take();
        self.current = self.tokens.next();
        token
    }

    /// Consume the current token if it matches the expected kind
    fn consume(&mut self, kind: TokenKind) -> ParseResult<Token> {
        if let Some(token) = self.current.clone() {
            if std::mem::discriminant(&token.kind) == std::mem::discriminant(&kind) {
                return self.advance().ok_or(ParseError::UnexpectedEof {
                    span: self.current_span(),
                });
            }
        }

        Err(ParseError::UnexpectedToken {
            expected: format!("{:?}", kind),
            found: self.current_kind().cloned().unwrap_or(TokenKind::Unknown),
            span: self.current_span(),
        })
    }

    /// Check if current token matches kind
    fn check(&self, kind: &TokenKind) -> bool {
        match &self.current {
            None => false,
            Some(token) => {
                std::mem::discriminant(&token.kind) == std::mem::discriminant(kind)
            }
        }
    }

    /// Parse an item (top-level declaration)
    fn parse_item(&mut self) -> ParseResult<Option<Item>> {
        let span = self.current_span();

        // Skip empty statements
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            return Ok(None);
        }

        // Parse attributes before the item (e.g., #[test])
        let mut attributes = Vec::new();
        while self.check(&TokenKind::Hash) {
            attributes.push(self.parse_attribute()?);
        }

        // Check for visibility modifier
        let _visibility = if self.check(&TokenKind::Pub) {
            self.advance();
            Some(Visibility::Public)
        } else {
            Some(Visibility::Private)
        };

        let kind = match self.current_kind() {
            Some(TokenKind::Extern) => {
                self.advance();
                if self.check(&TokenKind::Fn) {
                    // Parse extern function declaration
                    self.consume(TokenKind::Fn)?;
                    let name = self.parse_identifier()?;

                    // Parse generics
                    let generics = if self.check(&TokenKind::Less) {
                        Some(self.parse_generics()?)
                    } else {
                        None
                    };

                    self.consume(TokenKind::LeftParen)?;

                    // Parse parameters
                    let mut params = Vec::new();
                    let mut is_variadic = false;

                    while !self.check(&TokenKind::RightParen) {
                        params.push(self.parse_param()?);

                        if !self.check(&TokenKind::RightParen) {
                            self.consume(TokenKind::Comma)?;
                        }

                        // Check for variadic argument marker ... after the comma
                        if self.check(&TokenKind::DotDotDot) {
                            self.advance();
                            is_variadic = true;
                            break;
                        }
                    }

                    self.consume(TokenKind::RightParen)?;

                    // Parse return type
                    let return_type = if self.check(&TokenKind::Arrow) {
                        self.advance();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };

                    // Parse error type
                    let error_type = if self.check(&TokenKind::Pipe) {
                        self.advance();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };

                    // Parse effects
                    let mut effects = Vec::new();
                    if error_type.is_some() && self.check(&TokenKind::Pipe) {
                        self.advance();
                        effects.push(self.parse_type()?);
                        while self.check(&TokenKind::Plus) {
                            self.advance();
                            effects.push(self.parse_type()?);
                        }
                    }

                    // Extern functions end with semicolon, not a block
                    self.consume(TokenKind::Semicolon)?;

                    let func = Function {
                        name,
                        generics,
                        params,
                        return_type,
                        error_type,
                        effects,
                        is_variadic,
                        body: Block {
                            statements: Vec::new(),
                            trailing_expr: None,
                            span: self.current_span(),
                        },
                        is_async: false,
                        is_unsafe: false,
                        attributes,
                    };

                    ItemKind::ExternFunction(func)
                } else {
                    return Err(ParseError::InvalidSyntax {
                        message: "expected 'fn' after 'extern'".to_string(),
                        span: self.current_span(),
                    });
                }
            }
            Some(TokenKind::Fn) => {
                let mut func = self.parse_function()?;
                // Add attributes parsed before the item
                func.attributes.extend(attributes);
                ItemKind::Function(func)
            }
            Some(TokenKind::Struct) => {
                let struct_def = self.parse_struct()?;
                ItemKind::Struct(struct_def)
            }
            Some(TokenKind::Enum) => {
                let enum_def = self.parse_enum()?;
                ItemKind::Enum(enum_def)
            }
            Some(TokenKind::Trait) => {
                let trait_def = self.parse_trait()?;
                ItemKind::Trait(trait_def)
            }
            Some(TokenKind::Impl) => {
                let impl_def = self.parse_impl()?;
                ItemKind::Impl(impl_def)
            }
            Some(TokenKind::Type) => {
                let type_alias = self.parse_type_alias()?;
                ItemKind::TypeAlias(type_alias)
            }
            Some(TokenKind::Const) => {
                let const_def = self.parse_const()?;
                ItemKind::Const(const_def)
            }
            Some(TokenKind::Static) => {
                let static_def = self.parse_static()?;
                ItemKind::Static(static_def)
            }
            Some(TokenKind::Mod) => {
                let module = self.parse_module()?;
                ItemKind::Module(module)
            }
            Some(TokenKind::Use) => {
                let use_stmt = self.parse_use()?;
                ItemKind::Use(use_stmt)
            }
            Some(TokenKind::Effect) => {
                let effect = self.parse_effect()?;
                ItemKind::Effect(effect)
            }
            _ => {
                return Err(ParseError::InvalidSyntax {
                    message: format!("expected item declaration, found {:?}", self.current_kind()),
                    span,
                });
            }
        };

        Ok(Some(Item { span, kind }))
    }

    /// Parse a function definition
    fn parse_function(&mut self) -> ParseResult<Function> {
        self.consume(TokenKind::Fn)?;

        let name = self.parse_identifier()?;

        // Parse generics
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftParen)?;

        // Parse parameters
        let mut params = Vec::new();
        let mut is_variadic = false;

        while !self.check(&TokenKind::RightParen) {
            params.push(self.parse_param()?);

            if !self.check(&TokenKind::RightParen) {
                self.consume(TokenKind::Comma)?;
            }

            // Check for variadic argument marker ... after the comma
            if self.check(&TokenKind::DotDotDot) {
                self.advance();
                is_variadic = true;
                break;
            }
        }

        self.consume(TokenKind::RightParen)?;

        // Parse return type and optional error type/effects
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse error type and effects with | separator
        // Syntax: -> Type | Error | Effect1 + Effect2
        //         -> Type | Effect1 + Effect2  (no error type)
        //         -> Type | Error  (no effects)
        let mut error_type = None;
        let mut effects = Vec::new();

        while self.check(&TokenKind::Pipe) {
            self.advance();  // consume |

            // Parse the type after |
            let ty = self.parse_type()?;

            // Check if this is an error type or effect
            // For now, we'll use a simple heuristic: if it's a known type name like "Error", treat it as error type
            // Otherwise, treat it as an effect
            let is_error_type = match &ty {
                Type::Simple(ident) => {
                    ident.name == "Error" || ident.name.ends_with("Error")
                }
                _ => false,
            };

            if is_error_type && error_type.is_none() {
                // First type that looks like an error type becomes the error type
                error_type = Some(ty);
            } else {
                // Everything else is an effect
                effects.push(ty);

                // Parse additional effects with + separator
                while self.check(&TokenKind::Plus) {
                    self.advance();  // consume +
                    effects.push(self.parse_type()?);
                }
            }
        }

        // Parse body
        let body = self.parse_block()?;

        Ok(Function {
            name,
            generics,
            params,
            return_type,
            error_type,
            effects,
            is_variadic,
            body,
            is_async: false, // TODO: Parse async modifier
            is_unsafe: false, // TODO: Parse unsafe modifier
            attributes: Vec::new(), // Will be populated by parse_item
        })
    }

    /// Parse a function parameter
    fn parse_param(&mut self) -> ParseResult<Param> {
        let span = self.current_span();
        let name = self.parse_identifier()?;

        let type_annotation = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let default_value = if self.check(&TokenKind::Equals) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(Param {
            span,
            name,
            type_annotation,
            default_value,
        })
    }

    /// Parse a block statement
    fn parse_block(&mut self) -> ParseResult<Block> {
        let span = self.current_span();
        self.consume(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        let mut trailing_expr = None;

        while !self.check(&TokenKind::RightBrace) {
            let stmt = self.parse_statement()?;

            // Check if this is a trailing expression (no semicolon)
            match stmt.kind {
                StatementKind::Expr(ref expr) => {
                    // If next token is semicolon, consume it and continue
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                        statements.push(stmt);
                    } else if self.check(&TokenKind::RightBrace) {
                        // No semicolon and next is RightBrace → trailing expression
                        trailing_expr = Some(Box::new(expr.clone()));
                        break;
                    } else {
                        // No semicolon and not RightBrace → error or expression statement without semicolon
                        statements.push(stmt);
                    }
                }
                _ => {
                    statements.push(stmt);
                }
            }
        }

        self.consume(TokenKind::RightBrace)?;

        Ok(Block {
            span,
            statements,
            trailing_expr,
        })
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        let span = self.current_span();

        let kind = match self.current_kind() {
            Some(TokenKind::Let) => {
                let local = self.parse_local()?;
                StatementKind::Local(local)
            }
            Some(TokenKind::Defer) => {
                // Defer statement: defer expr_or_statement;
                self.advance();
                let stmt = Box::new(self.parse_statement()?);
                StatementKind::Defer(stmt)
            }
            Some(TokenKind::Fn | TokenKind::Struct | TokenKind::Enum | TokenKind::Trait |
                 TokenKind::Impl | TokenKind::Type | TokenKind::Const | TokenKind::Static |
                 TokenKind::Mod | TokenKind::Use) => {
                let item = Box::new(self.parse_item()?.unwrap());
                StatementKind::Item(item)
            }
            _ => {
                let expr = self.parse_expression()?;
                StatementKind::Expr(expr)  // Don't consume semicolon here - let parse_block handle it
            }
        };

        Ok(Statement { span, kind })
    }

    /// Parse a local variable declaration
    fn parse_local(&mut self) -> ParseResult<Local> {
        self.consume(TokenKind::Let)?;

        let is_mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.parse_identifier()?;

        let type_annotation = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.check(&TokenKind::Equals) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume(TokenKind::Semicolon)?;

        Ok(Local {
            name,
            type_annotation,
            init,
            is_mutable,
        })
    }

    /// Parse an expression (using precedence climbing)
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment()
    }

    /// Parse assignment expressions
    fn parse_assignment(&mut self) -> ParseResult<Expression> {
        let left = self.parse_or()?;

        if self.check(&TokenKind::Equals) {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_expression()?);

            return Ok(Expression {
                span,
                kind: ExpressionKind::Assign(Box::new(left), right),
            });
        }

        // Compound assignment operators
        if let Some(op) = self.match_assign_op() {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_expression()?);

            return Ok(Expression {
                span,
                kind: ExpressionKind::AssignOp(op, Box::new(left), right),
            });
        }

        Ok(left)
    }

    /// Match compound assignment operators
    fn match_assign_op(&self) -> Option<BinaryOp> {
        match self.current_kind() {
            Some(TokenKind::PlusEq) => Some(BinaryOp::Add),
            Some(TokenKind::MinusEq) => Some(BinaryOp::Sub),
            Some(TokenKind::StarEq) => Some(BinaryOp::Mul),
            Some(TokenKind::SlashEq) => Some(BinaryOp::Div),
            Some(TokenKind::PercentEq) => Some(BinaryOp::Mod),
            Some(TokenKind::CaretEq) => Some(BinaryOp::BitXor),
            Some(TokenKind::AmpersandEq) => Some(BinaryOp::BitAnd),
            Some(TokenKind::PipeEq) => Some(BinaryOp::BitOr),
            Some(TokenKind::LeftShiftEq) => Some(BinaryOp::LeftShift),
            Some(TokenKind::RightShiftEq) => Some(BinaryOp::RightShift),
            _ => None,
        }
    }

    /// Parse logical OR
    fn parse_or(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_and()?;

        while self.check(&TokenKind::Or) {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_and()?);

            left = Expression {
                span,
                kind: ExpressionKind::Binary(BinaryOp::Or, Box::new(left), right),
            };
        }

        Ok(left)
    }

    /// Parse logical AND
    fn parse_and(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::And) {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_equality()?);

            left = Expression {
                span,
                kind: ExpressionKind::Binary(BinaryOp::And, Box::new(left), right),
            };
        }

        Ok(left)
    }

    /// Parse equality comparisons
    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_comparison()?;

        while let Some(op) = self.match_equality_op() {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_comparison()?);

            left = Expression {
                span,
                kind: ExpressionKind::Binary(op, Box::new(left), right),
            };
        }

        Ok(left)
    }

    /// Match equality operators
    fn match_equality_op(&self) -> Option<BinaryOp> {
        match self.current_kind() {
            Some(TokenKind::EqEq) => Some(BinaryOp::Eq),
            Some(TokenKind::BangEq) => Some(BinaryOp::NotEq),
            _ => None,
        }
    }

    /// Parse comparison operators
    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_term()?;

        while let Some(op) = self.match_comparison_op() {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_term()?);

            left = Expression {
                span,
                kind: ExpressionKind::Binary(op, Box::new(left), right),
            };
        }

        Ok(left)
    }

    /// Match comparison operators
    fn match_comparison_op(&self) -> Option<BinaryOp> {
        match self.current_kind() {
            Some(TokenKind::Less) => Some(BinaryOp::Less),
            Some(TokenKind::LessEq) => Some(BinaryOp::LessEq),
            Some(TokenKind::Greater) => Some(BinaryOp::Greater),
            Some(TokenKind::GreaterEq) => Some(BinaryOp::GreaterEq),
            _ => None,
        }
    }

    /// Parse term (addition/subtraction)
    fn parse_term(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_factor()?;

        while let Some(op) = self.match_additive_op() {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_factor()?);

            left = Expression {
                span,
                kind: ExpressionKind::Binary(op, Box::new(left), right),
            };
        }

        Ok(left)
    }

    /// Match additive operators
    fn match_additive_op(&self) -> Option<BinaryOp> {
        match self.current_kind() {
            Some(TokenKind::Plus) => Some(BinaryOp::Add),
            Some(TokenKind::Minus) => Some(BinaryOp::Sub),
            _ => None,
        }
    }

    /// Parse factor (multiplication/division)
    fn parse_factor(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.match_multiplicative_op() {
            let span = self.current_span();
            self.advance();
            let right = Box::new(self.parse_unary()?);

            left = Expression {
                span,
                kind: ExpressionKind::Binary(op, Box::new(left), right),
            };
        }

        Ok(left)
    }

    /// Match multiplicative operators
    fn match_multiplicative_op(&self) -> Option<BinaryOp> {
        match self.current_kind() {
            Some(TokenKind::Star) => Some(BinaryOp::Mul),
            Some(TokenKind::Slash) => Some(BinaryOp::Div),
            Some(TokenKind::Percent) => Some(BinaryOp::Mod),
            _ => None,
        }
    }

    /// Parse unary expressions
    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if let Some(op) = self.match_unary_op() {
            let span = self.current_span();
            self.advance();
            let operand = Box::new(self.parse_unary()?);

            return Ok(Expression {
                span,
                kind: ExpressionKind::Unary(op, operand),
            });
        }

        self.parse_primary()
    }

    /// Match unary operators
    fn match_unary_op(&self) -> Option<UnaryOp> {
        match self.current_kind() {
            Some(TokenKind::Minus) => Some(UnaryOp::Neg),
            Some(TokenKind::Bang) => Some(UnaryOp::Not),
            Some(TokenKind::Caret) => Some(UnaryOp::BitNot),
            Some(TokenKind::Star) => Some(UnaryOp::Deref),
            Some(TokenKind::Ampersand) => Some(UnaryOp::Ref),
            _ => None,
        }
    }

    /// Parse postfix operators (calls, field access, indexing)
    fn parse_postfix(&mut self, mut expr: Expression) -> ParseResult<Expression> {
        loop {
            let span = self.current_span();

            match self.current_kind() {
                // Function call: func(args)
                Some(TokenKind::LeftParen) => {
                    self.advance();

                    let mut args = Vec::new();

                    while !self.check(&TokenKind::RightParen) {
                        args.push(Box::new(self.parse_expression()?));

                        if !self.check(&TokenKind::RightParen) {
                            self.consume(TokenKind::Comma)?;
                        }
                    }

                    self.consume(TokenKind::RightParen)?;

                    expr = Expression {
                        span,
                        kind: ExpressionKind::Call(Box::new(expr), args),
                    };
                }

                // Field access: obj.field
                Some(TokenKind::Dot) => {
                    self.advance();

                    let field_name = self.parse_identifier()?;

                    expr = Expression {
                        span,
                        kind: ExpressionKind::FieldAccess(Box::new(expr), field_name),
                    };
                }

                // Array indexing: arr[index]
                Some(TokenKind::LeftBracket) => {
                    self.advance();

                    let index = Box::new(self.parse_expression()?);

                    self.consume(TokenKind::RightBracket)?;

                    expr = Expression {
                        span,
                        kind: ExpressionKind::Index(Box::new(expr), index),
                    };
                }

                // Question mark operator: expr?
                Some(TokenKind::Question) => {
                    self.advance();

                    expr = Expression {
                        span,
                        kind: ExpressionKind::QuestionMark(Box::new(expr)),
                    };
                }

                // No more postfix operators
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse primary expressions
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_primary_base()?;

        // Parse postfix operators (calls, field access, indexing)
        self.parse_postfix(expr)
    }

    /// Parse base primary expressions
    fn parse_primary_base(&mut self) -> ParseResult<Expression> {
        let span = self.current_span();

        match self.current_kind() {
            // Control flow: if expression
            Some(TokenKind::If) => {
                self.advance();
                let condition = Box::new(self.parse_expression()?);
                let then_block = self.parse_block()?;

                let else_block = if self.check(&TokenKind::Else) {
                    self.advance();

                    if self.check(&TokenKind::If) {
                        // else if - parse the if expression and extract block
                        let if_expr = self.parse_primary_base()?;
                        match if_expr.kind {
                            ExpressionKind::If(_, else_then_block, _) => {
                                Some(else_then_block)
                            }
                            _ => None,
                        }
                    } else {
                        // else { block }
                        Some(self.parse_block()?)
                    }
                } else {
                    None
                };

                Ok(Expression {
                    span,
                    kind: ExpressionKind::If(condition, then_block, else_block),
                })
            }

            // Control flow: loop expression
            Some(TokenKind::Loop) => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Loop(body, None), // TODO: Parse label
                })
            }

            // Control flow: while loop
            Some(TokenKind::While) => {
                self.advance();
                let condition = Box::new(self.parse_expression()?);
                let body = self.parse_block()?;
                Ok(Expression {
                    span,
                    kind: ExpressionKind::While(condition, body, None), // TODO: Parse label
                })
            }

            // Control flow: for loop
            Some(TokenKind::For) => {
                self.advance();
                let span = self.current_span();

                // Parse pattern (currently only identifier)
                let name = self.parse_identifier()?;

                self.consume(TokenKind::In)?;

                // Parse iterator expression
                let iter = self.parse_expression()?;

                // Parse body
                let body = self.parse_block()?;

                Ok(Expression {
                    span,
                    kind: ExpressionKind::For(
                        Local {
                            name,
                            type_annotation: None,
                            init: None,
                            is_mutable: false,
                        },
                        Box::new(iter),
                        body,
                        None, // TODO: Parse label
                    ),
                })
            }

            // Control flow: match expression
            Some(TokenKind::Match) => {
                self.advance();
                let scrutinee = Box::new(self.parse_expression()?);
                self.consume(TokenKind::LeftBrace)?;

                let mut arms = Vec::new();

                while !self.check(&TokenKind::RightBrace) {
                    let arm_span = self.current_span();

                    // Parse patterns
                    let mut patterns = Vec::new();
                    patterns.push(self.parse_pattern()?);

                    while self.check(&TokenKind::Pipe) {
                        self.advance();
                        patterns.push(self.parse_pattern()?);
                    }

                    // Parse guard (if condition)
                    let guard = if self.check(&TokenKind::If) {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };

                    self.consume(TokenKind::FatArrow)?;

                    // Parse arm body (expression or block)
                    let body = if self.check(&TokenKind::LeftBrace) {
                        let block = self.parse_block()?;
                        Box::new(Expression {
                            span: arm_span,
                            kind: ExpressionKind::Block(block),
                        })
                    } else {
                        Box::new(self.parse_expression()?)
                    };

                    arms.push(MatchArm {
                        span: arm_span,
                        patterns,
                        guard,
                        body,
                    });

                    if !self.check(&TokenKind::RightBrace) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::RightBrace)?;

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Match(scrutinee, arms),
                })
            }

            // Control flow: break
            Some(TokenKind::Break) => {
                self.advance();
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Break(None), // TODO: Parse label
                })
            }

            // Control flow: continue
            Some(TokenKind::Continue) => {
                self.advance();
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Continue(None), // TODO: Parse label
                })
            }

            // Control flow: return
            Some(TokenKind::Return) => {
                self.advance();

                let value = if !self.check(&TokenKind::Semicolon) && !self.check(&TokenKind::RightBrace) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Return(value),
                })
            }

            // Error handling: throw statement
            Some(TokenKind::Throw) => {
                self.advance();

                let error = Box::new(self.parse_expression()?);

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Throw(error),
                })
            }

            // Effect handling: try block with handlers
            Some(TokenKind::Try) => {
                self.advance();

                // Parse the try block body
                let try_block = Box::new(self.parse_block()?);

                // Parse handlers: with Effect1 { ... } with Effect2 { ... }
                let mut handlers = Vec::new();

                while self.check(&TokenKind::With) {
                    self.advance();

                    // Parse effect name
                    let effect_name = self.parse_identifier()?;

                    self.consume(TokenKind::LeftBrace)?;

                    // Parse handler methods
                    let mut methods = Vec::new();

                    while !self.check(&TokenKind::RightBrace) {
                        // Parse: fn name(params) { body }
                        let name = self.parse_identifier()?;

                        self.consume(TokenKind::LeftParen)?;

                        // Parse parameters
                        let mut params = Vec::new();
                        while !self.check(&TokenKind::RightParen) {
                            params.push(self.parse_param()?);

                            if !self.check(&TokenKind::RightParen) {
                                self.consume(TokenKind::Comma)?;
                            }
                        }

                        self.consume(TokenKind::RightParen)?;

                        // Note: Effect methods in handlers don't have explicit return types
                        // The return type is inferred from the effect signature

                        // Parse method body
                        let body = self.parse_block()?;

                        methods.push(EffectMethod {
                            name,
                            params,
                            body,
                        });

                        // Methods can be separated by commas or semicolons
                        if !self.check(&TokenKind::RightBrace) {
                            self.consume_one_of(&[TokenKind::Comma, TokenKind::Semicolon])?;
                        }
                    }

                    self.consume(TokenKind::RightBrace)?;

                    handlers.push(EffectHandler {
                        effect_name,
                        methods,
                    });
                }

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Try(try_block, handlers),
                })
            }

            // Closure: |params| body or |params: Type| -> Type { body }
            Some(TokenKind::Pipe) => {
                self.advance(); // consume first pipe

                // Parse parameters
                let mut params = Vec::new();

                // Check for empty parameter list: ||
                if !self.check(&TokenKind::Pipe) {
                    // Parse first parameter
                    params.push(self.parse_closure_param()?);

                    // Parse additional parameters separated by commas
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        params.push(self.parse_closure_param()?);
                    }
                }

                self.consume(TokenKind::Pipe)?; // consume closing pipe

                // Parse optional return type: -> Type
                let return_type = if self.check(&TokenKind::Arrow) {
                    self.advance(); // consume ->
                    Some(self.parse_type()?)
                } else {
                    None
                };

                // Parse closure body
                // Body can be a block expression or a simple expression
                let body = if self.check(&TokenKind::LeftBrace) {
                    // Block body: |...| { statements }
                    let block = self.parse_block()?;
                    Box::new(Expression {
                        span: block.span,
                        kind: ExpressionKind::Block(block),
                    })
                } else {
                    // Expression body: |...| expression
                    Box::new(self.parse_expression()?)
                };

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Closure {
                        params,
                        return_type,
                        body,
                    },
                })
            }

            // Literals
            Some(TokenKind::IntLiteral(_)) => {
                let token = self.advance().unwrap();
                // Extract the integer value from IntLiteral
                let value = if let TokenKind::IntLiteral(s) = &token.kind {
                    s.parse().unwrap_or(0)
                } else {
                    0
                };
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Literal(Literal::Int(value)),
                })
            }
            Some(TokenKind::FloatLiteral(_)) => {
                let token = self.advance().unwrap();
                // Extract the float value from FloatLiteral
                let value = if let TokenKind::FloatLiteral(s) = &token.kind {
                    s.parse().unwrap_or(0.0)
                } else {
                    0.0
                };
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Literal(Literal::Float(value)),
                })
            }
            Some(TokenKind::StringLiteral(_)) => {
                let token = self.advance().unwrap();
                if let TokenKind::StringLiteral(s) = token.kind {
                    Ok(Expression {
                        span,
                        kind: ExpressionKind::Literal(Literal::String(s.to_string())),
                    })
                } else {
                    unreachable!()
                }
            }
            Some(TokenKind::TemplateString(_)) => {
                let token = self.advance().unwrap();
                if let TokenKind::TemplateString(template) = &token.kind {
                    // Parse template string with interpolation
                    let parts = self.parse_template_string_parts(template, &token.span)?;

                    Ok(Expression {
                        span,
                        kind: ExpressionKind::TemplateString(
                            TemplateString { parts }
                        ),
                    })
                } else {
                    unreachable!()
                }
            }
            Some(TokenKind::CharLiteral(_)) => {
                let token = self.advance().unwrap();
                if let TokenKind::CharLiteral(c) = token.kind {
                    Ok(Expression {
                        span,
                        kind: ExpressionKind::Literal(Literal::Char(c)),
                    })
                } else {
                    unreachable!()
                }
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Literal(Literal::Bool(true)),
                })
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Literal(Literal::Bool(false)),
                })
            }
            Some(TokenKind::Null) => {
                self.advance();
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Literal(Literal::Null),
                })
            }

            // Parenthesized expression or tuple
            Some(TokenKind::LeftParen) => {
                self.advance();

                // Try to parse as tuple (multiple expressions)
                let mut elements = Vec::new();
                let first_expr = self.parse_expression()?;

                if self.check(&TokenKind::Comma) {
                    // It's a tuple
                    elements.push(Box::new(first_expr));

                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::RightParen) {
                            break; // Trailing comma
                        }
                        elements.push(Box::new(self.parse_expression()?));
                    }

                    self.consume(TokenKind::RightParen)?;

                    Ok(Expression {
                        span,
                        kind: ExpressionKind::Tuple(elements),
                    })
                } else {
                    // It's a parenthesized expression
                    self.consume(TokenKind::RightParen)?;
                    Ok(first_expr)
                }
            }

            // Identifier or path or macro invocation
            Some(TokenKind::Ident(_)) => {
                // Check if this is a macro invocation (identifier followed by !)
                if let Some(TokenKind::Bang) = self.peek_kind() {
                    // Parse as macro invocation
                    let macro_name = self.parse_identifier()?;
                    return self.parse_macro_invocation(macro_name, span);
                }

                // Otherwise parse as path
                let path = self.parse_path()?;
                Ok(Expression {
                    span,
                    kind: ExpressionKind::Path(path),
                })
            }

            // Path starting with :: (e.g., ::__builtin_function)
            Some(TokenKind::PathSep) => {
                let mut path = Vec::new();
                // Add empty identifier for the leading ::
                path.push(Identifier::new(span, String::new()));
                self.advance();

                // Parse the rest of the path
                loop {
                    match self.current_kind() {
                        Some(TokenKind::Ident(_)) => {
                            path.push(self.parse_identifier()?);
                        }
                        _ => break,
                    }

                    if !self.check(&TokenKind::PathSep) {
                        break;
                    }
                    self.advance();
                }

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Path(path),
                })
            }

            // Array literal
            Some(TokenKind::LeftBracket) => {
                self.advance();
                let mut elements = Vec::new();

                while !self.check(&TokenKind::RightBracket) {
                    elements.push(Box::new(self.parse_expression()?));

                    if !self.check(&TokenKind::RightBracket) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::RightBracket)?;

                Ok(Expression {
                    span,
                    kind: ExpressionKind::Array(elements),
                })
            }

            _ => Err(ParseError::InvalidSyntax {
                message: format!("unexpected token in expression: {:?}", self.current_kind()),
                span,
            }),
        }
    }

    /// Parse a path (identifier or qualified path)
    fn parse_path(&mut self) -> ParseResult<Vec<Identifier>> {
        let mut path = Vec::new();

        path.push(self.parse_identifier()?);

        while self.check(&TokenKind::PathSep) {
            self.advance();
            path.push(self.parse_identifier()?);
        }

        Ok(path)
    }

    /// Parse a macro invocation: macro_name!(args), macro_name! {args}, or macro_name![args]
    fn parse_macro_invocation(&mut self, macro_name: Identifier, span: Span) -> ParseResult<Expression> {
        use crate::ast::{MacroDelimiter, ExpressionKind};

        // Consume the !
        self.consume(TokenKind::Bang)?;

        // Determine the delimiter and parse arguments
        let delimiter = match self.current_kind() {
            Some(TokenKind::LeftParen) => {
                self.advance();
                MacroDelimiter::Paren
            }
            Some(TokenKind::LeftBrace) => {
                self.advance();
                MacroDelimiter::Brace
            }
            Some(TokenKind::LeftBracket) => {
                self.advance();
                MacroDelimiter::Bracket
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "macro delimiter (, {, or [".to_string(),
                    found: self.current_kind().cloned().unwrap_or(TokenKind::Unknown),
                    span: self.current_span(),
                })
            }
        };

        // Parse macro arguments (comma-separated expressions)
        let mut args = Vec::new();
        while !self.check(&Self::closing_delimiter(delimiter)) {
            args.push(Box::new(self.parse_expression()?));

            if !self.check(&Self::closing_delimiter(delimiter)) {
                self.consume(TokenKind::Comma)?;
            }
        }

        // Consume the closing delimiter
        self.consume(Self::closing_delimiter(delimiter))?;

        Ok(Expression {
            span,
            kind: ExpressionKind::MacroInvocation {
                macro_name,
                args,
                delimiter,
            },
        })
    }

    /// Get the closing delimiter for a macro invocation
    fn closing_delimiter(opening: MacroDelimiter) -> TokenKind {
        match opening {
            MacroDelimiter::Paren => TokenKind::RightParen,
            MacroDelimiter::Brace => TokenKind::RightBrace,
            MacroDelimiter::Bracket => TokenKind::RightBracket,
        }
    }

    /// Parse an identifier
    fn parse_identifier(&mut self) -> ParseResult<Identifier> {
        let span = self.current_span();

        match self.current_kind() {
            Some(TokenKind::Ident(_)) => {
                let token = self.advance().unwrap();
                if let TokenKind::Ident(s) = token.kind {
                    Ok(Identifier::new(span, s.to_string()))
                } else {
                    unreachable!()
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current_kind().cloned().unwrap_or(TokenKind::Unknown),
                span,
            }),
        }
    }

    /// Parse a closure parameter (name or name: Type)
    fn parse_closure_param(&mut self) -> ParseResult<Local> {
        let name = self.parse_identifier()?;

        // Optional type annotation
        let type_annotation = if self.check(&TokenKind::Colon) {
            self.advance(); // consume :
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(Local {
            name,
            type_annotation,
            init: None,
            is_mutable: false, // Closure params are immutable by default
        })
    }

    /// Parse a type
    fn parse_type(&mut self) -> ParseResult<Type> {
        let span = self.current_span();

        // Reference type: &T or &mut T
        if self.check(&TokenKind::Ampersand) {
            self.advance();
            let is_mutable = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            let inner = Box::new(self.parse_type()?);
            return Ok(Type::Ref(inner, is_mutable));
        }

        // Pointer type: *T (C-style pointer)
        if self.check(&TokenKind::Star) {
            self.advance();
            let inner = Box::new(self.parse_type()?);
            // Treat *T as &T (reference) for now
            return Ok(Type::Ref(inner, false));
        }

        // Simple type or path (with optional generic arguments)
        if let Some(TokenKind::Ident(_)) = self.current_kind() {
            let path = self.parse_path()?;

            // Check for generic arguments: Outcome<i32, Error>
            let generic_args = if self.check(&TokenKind::Less) {
                self.advance();
                let mut args = Vec::new();

                while !self.check(&TokenKind::Greater) {
                    args.push(self.parse_type()?);

                    if !self.check(&TokenKind::Greater) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::Greater)?;
                Some(args)
            } else {
                None
            };

            if path.len() == 1 && generic_args.is_none() {
                return Ok(Type::Simple(path[0].clone()));
            } else {
                return Ok(Type::PathGeneric(path, generic_args));
            }
        }

        // Tuple type
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let mut types = Vec::new();

            while !self.check(&TokenKind::RightParen) {
                types.push(self.parse_type()?);

                if !self.check(&TokenKind::RightParen) {
                    self.consume(TokenKind::Comma)?;
                }
            }

            self.consume(TokenKind::RightParen)?;

            return Ok(Type::Tuple(types));
        }

        // Array type
        if self.check(&TokenKind::LeftBracket) {
            self.advance();
            let inner = Box::new(self.parse_type()?);

            let size = if self.check(&TokenKind::Semicolon) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.consume(TokenKind::RightBracket)?;

            return Ok(Type::Array(inner, size));
        }

        Err(ParseError::InvalidSyntax {
            message: format!("expected type, found {:?}", self.current_kind()),
            span,
        })
    }

    /// Parse generics
    fn parse_generics(&mut self) -> ParseResult<Generics> {
        let span = self.current_span();
        self.consume(TokenKind::Less)?;

        let mut params = Vec::new();

        while !self.check(&TokenKind::Greater) {
            let name = self.parse_identifier()?;
            params.push(GenericParam::Type(name));

            if !self.check(&TokenKind::Greater) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::Greater)?;

        Ok(Generics {
            span,
            params,
            where_clause: Vec::new(), // TODO: Parse where clause
        })
    }

    /// Parse a struct definition
    fn parse_struct(&mut self) -> ParseResult<Struct> {
        self.consume(TokenKind::Struct)?;

        let name = self.parse_identifier()?;

        // Parse generics
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace)?;

        let mut fields = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            let span = self.current_span();
            let field_name = self.parse_identifier()?;

            self.consume(TokenKind::Colon)?;

            let field_type = self.parse_type()?;

            let default_value = if self.check(&TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            fields.push(StructField {
                span,
                name: field_name,
                type_annotation: field_type,
                default_value,
            });

            if !self.check(&TokenKind::RightBrace) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightBrace)?;

        Ok(Struct {
            name,
            generics,
            fields,
        })
    }

    /// Parse an enum definition
    fn parse_enum(&mut self) -> ParseResult<Enum> {
        self.consume(TokenKind::Enum)?;

        let name = self.parse_identifier()?;

        // Parse generics
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace)?;

        let mut variants = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            let span = self.current_span();
            let variant_name = self.parse_identifier()?;

            let mut fields = Vec::new();

            // Check if this variant has data
            if self.check(&TokenKind::LeftBrace) {
                // Struct-style variant: Variant { field: Type, ... }
                self.advance();

                while !self.check(&TokenKind::RightBrace) {
                    let field_name = self.parse_identifier()?;

                    self.consume(TokenKind::Colon)?;

                    let field_type = self.parse_type()?;

                    fields.push(VariantField::Named(field_name, field_type));

                    if !self.check(&TokenKind::RightBrace) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::RightBrace)?;
            } else if self.check(&TokenKind::LeftParen) {
                // Tuple-style variant: Variant(Type1, Type2, ...)
                self.advance();

                while !self.check(&TokenKind::RightParen) {
                    let field_type = self.parse_type()?;
                    fields.push(VariantField::Unnamed(field_type));

                    if !self.check(&TokenKind::RightParen) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::RightParen)?;
            }
            // else: unit variant (no data)

            variants.push(EnumVariant {
                span,
                name: variant_name,
                fields,
            });

            if !self.check(&TokenKind::RightBrace) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightBrace)?;

        Ok(Enum {
            name,
            generics,
            variants,
        })
    }

    /// Parse a trait definition
    fn parse_trait(&mut self) -> ParseResult<Trait> {
        self.consume(TokenKind::Trait)?;

        let name = self.parse_identifier()?;

        // Parse generics
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        // Parse super traits: trait Name: Super1 + Super2
        let mut super_traits = Vec::new();

        if self.check(&TokenKind::Colon) {
            self.advance();

            loop {
                super_traits.push(self.parse_type()?);

                if !self.check(&TokenKind::Plus) {
                    break;
                }

                self.advance();
            }
        }

        self.consume(TokenKind::LeftBrace)?;

        let mut items = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            let span = self.current_span();

            // Check for method or associated type or const
            if self.check(&TokenKind::Fn) {
                let func = Box::new(self.parse_function()?);
                items.push(TraitItem::Method(func));
            } else if self.check(&TokenKind::Type) {
                let _token = self.advance();

                let type_name = self.parse_identifier()?;

                // Parse type bounds
                let mut bounds = Vec::new();
                if self.check(&TokenKind::Colon) {
                    let _token2 = self.advance();

                    loop {
                        let bound_type = self.parse_type()?;
                        bounds.push(bound_type);

                        if !self.check(&TokenKind::Plus) {
                            break;
                        }

                        let _token3 = self.advance();
                    }
                }

                self.consume(TokenKind::Semicolon)?;

                items.push(TraitItem::AssociatedType(type_name, bounds));
            } else if self.check(&TokenKind::Const) {
                let const_def = Box::new(self.parse_const()?);
                items.push(TraitItem::Const(const_def));
            } else {
                return Err(ParseError::InvalidSyntax {
                    message: format!("expected trait item, found {:?}", self.current_kind()),
                    span,
                });
            }
        }

        self.consume(TokenKind::RightBrace)?;

        Ok(Trait {
            name,
            generics,
            super_traits,
            items,
        })
    }

    /// Parse a trait implementation
    fn parse_impl(&mut self) -> ParseResult<Impl> {
        let impl_span = self.current_span();
        self.consume(TokenKind::Impl)?;

        // Parse generics
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        // Parse trait name: impl Trait for Type
        let trait_name = if self.check(&TokenKind::For) {
            // We have: impl Trait for Type
            let trait_type = self.parse_type()?;
            self.consume(TokenKind::For)?;
            Some(trait_type)
        } else {
            // We have: impl Type (inherent impl)
            None
        };

        // Parse self type
        let self_type = self.parse_type()?;

        self.consume(TokenKind::LeftBrace)?;

        let mut items = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            let func = Box::new(self.parse_function()?);
            items.push(func);
        }

        self.consume(TokenKind::RightBrace)?;

        Ok(Impl {
            impl_span,
            generics,
            trait_name,
            self_type,
            items,
        })
    }

    /// Parse a constant definition
    fn parse_const(&mut self) -> ParseResult<Const> {
        self.consume(TokenKind::Const)?;

        let name = self.parse_identifier()?;
        self.consume(TokenKind::Colon)?;

        let type_annotation = self.parse_type()?;
        self.consume(TokenKind::Equals)?;

        let value = self.parse_expression()?;

        self.consume(TokenKind::Semicolon)?;

        Ok(Const {
            name,
            type_annotation,
            value,
            is_mutable: false,
        })
    }

    /// Parse a static variable definition
    fn parse_static(&mut self) -> ParseResult<Static> {
        self.consume(TokenKind::Static)?;

        let is_mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.parse_identifier()?;
        self.consume(TokenKind::Colon)?;

        let type_annotation = self.parse_type()?;
        self.consume(TokenKind::Equals)?;

        let value = self.parse_expression()?;

        self.consume(TokenKind::Semicolon)?;

        Ok(Static {
            name,
            type_annotation,
            value,
            is_mutable,
        })
    }

    /// Parse a type alias
    fn parse_type_alias(&mut self) -> ParseResult<TypeAlias> {
        self.consume(TokenKind::Type)?;

        let name = self.parse_identifier()?;

        // Parse generics
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::Equals)?;

        let type_annotation = self.parse_type()?;

        self.consume(TokenKind::Semicolon)?;

        Ok(TypeAlias {
            name,
            generics,
            type_annotation,
        })
    }

    /// Parse a module declaration
    fn parse_module(&mut self) -> ParseResult<Module> {
        self.consume(TokenKind::Mod)?;

        let name = self.parse_identifier()?;

        let items = if self.check(&TokenKind::LeftBrace) {
            // Inline module: mod name { ... }
            let _token = self.advance();

            let mut module_items = Vec::new();

            while !self.check(&TokenKind::RightBrace) {
                if let Some(item) = self.parse_item()? {
                    module_items.push(Box::new(item));
                }
            }

            self.consume(TokenKind::RightBrace)?;

            Some(module_items)
        } else {
            // Module declaration: mod name;
            self.consume(TokenKind::Semicolon)?;
            None
        };

        Ok(Module {
            name,
            items,
        })
    }

    /// Parse a use statement
    fn parse_use(&mut self) -> ParseResult<Use> {
        self.consume(TokenKind::Use)?;

        let is_pub = false; // TODO: Parse pub use

        // Parse the path
        let mut path_segments = Vec::new();

        loop {
            path_segments.push(self.parse_identifier()?);

            if !self.check(&TokenKind::PathSep) {
                break;
            }

            let _token = self.advance();
        }

        let path = if self.check(&TokenKind::LeftBrace) {
            // Use list: use path::{a, b, c}
            let _token2 = self.advance();

            let mut items = Vec::new();

            while !self.check(&TokenKind::RightBrace) {
                items.push(self.parse_identifier()?);

                if !self.check(&TokenKind::RightBrace) {
                    self.consume(TokenKind::Comma)?;
                }
            }

            self.consume(TokenKind::RightBrace)?;

            UsePath::List(path_segments, items)
        } else if self.check(&TokenKind::Star) {
            // Glob use: use path::*
            let _token3 = self.advance();
            UsePath::Glob(path_segments)
        } else {
            // Simple use: use path::to::item
            UsePath::Simple(path_segments)
        };

        self.consume(TokenKind::Semicolon)?;

        Ok(Use {
            path,
            is_pub,
            alias: None, // TODO: Parse `as` alias
        })
    }

    /// Parse a pattern (for match expressions, let bindings, etc.)
    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        let _span = self.current_span();

        match self.current_kind() {
            // Wildcard pattern: _
            Some(TokenKind::Underscore) => {
                self.advance();
                Ok(Pattern::Wildcard)
            }

            // Literal pattern
            Some(TokenKind::IntLiteral(_) | TokenKind::FloatLiteral(_) |
                 TokenKind::StringLiteral(_) | TokenKind::CharLiteral(_) |
                 TokenKind::True | TokenKind::False) => {
                // We'll create a literal pattern directly
                match self.current_kind() {
                    Some(TokenKind::IntLiteral(_)) => {
                        let token = self.advance().unwrap();
                        if let TokenKind::IntLiteral(s) = &token.kind {
                            let value = s.to_string().parse().unwrap_or(0);
                            Ok(Pattern::Literal(Literal::Int(value)))
                        } else {
                            unreachable!()
                        }
                    }
                    Some(TokenKind::True) => {
                        self.advance();
                        Ok(Pattern::Literal(Literal::Bool(true)))
                    }
                    Some(TokenKind::False) => {
                        self.advance();
                        Ok(Pattern::Literal(Literal::Bool(false)))
                    }
                    _ => Ok(Pattern::Wildcard), // TODO: Handle other literals
                }
            }

            // Identifier pattern or Struct pattern (including path patterns like Outcome::Ok)
            Some(TokenKind::Ident(_)) => {
                // Parse path (could be single identifier or path like Outcome::Ok)
                let mut path = Vec::new();
                path.push(self.parse_identifier()?);

                // Check for path separators (::)
                while self.check(&TokenKind::PathSep) {
                    self.advance();
                    path.push(self.parse_identifier()?);
                }

                if self.check(&TokenKind::LeftBrace) {
                    // Struct pattern: Point { x, y } or mod::Point { x, y }
                    self.advance();

                    let mut fields = Vec::new();

                    while !self.check(&TokenKind::RightBrace) {
                        let field_name = self.parse_identifier()?;

                        let field = if self.check(&TokenKind::Colon) {
                            self.advance();
                            let pattern = Box::new(self.parse_pattern()?);
                            StructPatternField::Field(field_name, pattern)
                        } else {
                            StructPatternField::Shorthand(field_name)
                        };

                        fields.push(field);

                        if !self.check(&TokenKind::RightBrace) {
                            self.consume(TokenKind::Comma)?;
                        }
                    }

                    self.consume(TokenKind::RightBrace)?;

                    Ok(Pattern::Struct(path, fields))
                } else if self.check(&TokenKind::LeftParen) {
                    // Tuple-like enum variant pattern: Outcome::Ok(value), Some(x)
                    self.advance();

                    let mut patterns = Vec::new();

                    while !self.check(&TokenKind::RightParen) {
                        patterns.push(self.parse_pattern()?);

                        if !self.check(&TokenKind::RightParen) {
                            self.consume(TokenKind::Comma)?;
                        }
                    }

                    self.consume(TokenKind::RightParen)?;

                    Ok(Pattern::TupleVariant(path, patterns))
                } else if path.len() == 1 {
                    // Single identifier pattern
                    Ok(Pattern::Identifier(path[0].clone()))
                } else {
                    // Path pattern like Outcome::Ok or Outcome::Err (no fields)
                    // Treat as struct pattern with no fields (enum variant pattern)
                    Ok(Pattern::Struct(path, vec![]))
                }
            }

            // Tuple pattern: (a, b, c)
            Some(TokenKind::LeftParen) => {
                self.advance();

                let mut patterns = Vec::new();

                while !self.check(&TokenKind::RightParen) {
                    patterns.push(self.parse_pattern()?);

                    if !self.check(&TokenKind::RightParen) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::RightParen)?;

                Ok(Pattern::Tuple(patterns))
            }

            // Array pattern: [a, b, c]
            Some(TokenKind::LeftBracket) => {
                self.advance();

                let mut patterns = Vec::new();

                while !self.check(&TokenKind::RightBracket) {
                    patterns.push(self.parse_pattern()?);

                    if !self.check(&TokenKind::RightBracket) {
                        self.consume(TokenKind::Comma)?;
                    }
                }

                self.consume(TokenKind::RightBracket)?;

                Ok(Pattern::Array(patterns))
            }

            _ => {
                // For now, treat anything else as wildcard
                // TODO: Add proper wildcard token (_)
                self.advance();
                Ok(Pattern::Wildcard)
            }
        }
    }

    /// Parse an attribute: #[attribute] or #[attribute(arg)] or #[attribute(key = value)]
    fn parse_attribute(&mut self) -> ParseResult<Attribute> {
        // Consume #
        self.consume(TokenKind::Hash)?;

        // Consume [
        self.consume(TokenKind::LeftBracket)?;

        // Parse attribute name
        let name = self.parse_identifier()?;

        // Parse attribute arguments (optional)
        let mut args = Vec::new();

        // Check for ( ... ) - note: this is NOT part of standard ZULON attributes
        // but we allow it for compatibility with testing framework syntax like #[test()]
        if self.check(&TokenKind::LeftParen) {
            self.advance();

            // Parse arguments inside parentheses
            while !self.check(&TokenKind::RightParen) {
                match &self.current {
                    Some(token) => match &token.kind {
                        TokenKind::StringLiteral(s) => {
                            // String literal argument
                            args.push(AttributeArg::String(s.to_string()));
                            self.advance();
                        }
                        TokenKind::Ident(ident_str) => {
                            // Identifier or key-value pair
                            let key = Identifier::new(token.span, ident_str.as_ref());
                            self.advance();

                            if self.check(&TokenKind::Equals) {
                                // Key-value pair: key = "value"
                                self.advance();

                                let value = match &self.current {
                                    Some(t) if matches!(t.kind, TokenKind::StringLiteral(_)) => {
                                        if let TokenKind::StringLiteral(s) = &t.kind {
                                            s.to_string()
                                        } else {
                                            unreachable!()
                                        }
                                    }
                                    Some(t) => {
                                        return Err(ParseError::UnexpectedToken {
                                            expected: "string literal".to_string(),
                                            found: t.kind.clone(),
                                            span: self.current_span(),
                                        });
                                    }
                                    None => {
                                        return Err(ParseError::UnexpectedEof {
                                            span: self.current_span(),
                                        });
                                    }
                                };

                                args.push(AttributeArg::KeyValue { key, value });
                                self.advance();
                            } else {
                                // Just an identifier
                                args.push(AttributeArg::Ident(key));
                            }
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "identifier or string literal".to_string(),
                                found: self.current_kind().cloned().unwrap_or(TokenKind::Unknown),
                                span: self.current_span(),
                            });
                        }
                    },
                    None => {
                        return Err(ParseError::UnexpectedEof {
                            span: self.current_span(),
                        });
                    }
                }

                // Check for comma separator
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    // Continue parsing next argument
                } else {
                    break;
                }
            }

            // Consume )
            self.consume(TokenKind::RightParen)?;
        }

        // Consume ]
        self.consume(TokenKind::RightBracket)?;

        Ok(Attribute {
            name,
            args,
        })
    }

    /// Parse an effect declaration: `effect Name { operations }`
    fn parse_effect(&mut self) -> ParseResult<Effect> {
        self.consume(TokenKind::Effect)?;

        let name = self.parse_identifier()?;

        // Parse generics (optional)
        let generics = if self.check(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace)?;

        // Parse effect operations
        let mut operations = Vec::new();
        while !self.check(&TokenKind::RightBrace) {
            operations.push(self.parse_effect_operation()?);

            // Operations are separated by commas or semicolons
            if !self.check(&TokenKind::RightBrace) {
                self.consume_one_of(&[TokenKind::Comma, TokenKind::Semicolon])?;
            }
        }

        self.consume(TokenKind::RightBrace)?;

        Ok(Effect {
            name,
            generics,
            operations,
        })
    }

    /// Parse an effect operation: `fn name(params) -> ReturnType`
    fn parse_effect_operation(&mut self) -> ParseResult<EffectOperation> {
        let name = self.parse_identifier()?;

        self.consume(TokenKind::LeftParen)?;

        // Parse parameters
        let mut params = Vec::new();
        while !self.check(&TokenKind::RightParen) {
            params.push(self.parse_param()?);

            if !self.check(&TokenKind::RightParen) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightParen)?;

        // Parse return type (optional)
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(EffectOperation {
            name,
            params,
            return_type,
        })
    }

    /// Consume one of the given tokens
    fn consume_one_of(&mut self, kinds: &[TokenKind]) -> ParseResult<()> {
        if let Some(kind) = self.current_kind() {
            if kinds.iter().any(|k| std::mem::discriminant(k) == std::mem::discriminant(kind)) {
                self.advance();
                return Ok(());
            }
        }

        Err(ParseError::UnexpectedToken {
            expected: format!("one of {:?}", kinds),
            found: self.current_kind().cloned().unwrap_or(TokenKind::Unknown),
            span: self.current_span(),
        })
    }

    /// Parse template string parts, splitting static text from interpolated expressions
    fn parse_template_string_parts(&mut self, template: &str, span: &Span) -> ParseResult<Vec<TemplateStringPart>> {
        use crate::ast::TemplateStringPart;

        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = template.chars().peekable();
        let mut in_interpolation = false;
        let mut brace_depth = 0;

        while let Some(c) = chars.next() {
            if in_interpolation {
                if c == '{' {
                    brace_depth += 1;
                    current.push(c);
                } else if c == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        // End of interpolation
                        let expr_str = current[1..].to_string(); // Skip the opening '$'
                        current.clear();

                        // Parse the interpolated expression
                        // We need to create a temporary lexer to tokenize just this expression
                        let lexer = Lexer::new(&expr_str);
                        let (tokens, errors) = lexer.lex_all();

                        if !errors.is_empty() {
                            return Err(ParseError::InvalidSyntax {
                                message: format!("Failed to lex interpolated expression: {}", expr_str),
                                span: span.clone(),
                            });
                        }

                        // Parse the expression using the tokens
                        let old_tokens = std::mem::replace(&mut self.tokens, tokens.into_iter().peekable());
                        let _old_current = std::mem::replace(&mut self.current, self.tokens.next());

                        let expr_result = self.parse_expression();

                        // Restore the original token stream BEFORE checking expr_result
                        self.tokens = old_tokens;
                        // Don't restore old_current - we've moved past it
                        // Instead, get the next token from the restored stream
                        self.current = self.tokens.next();

                        parts.push(TemplateStringPart::Expr(expr_result?));
                        in_interpolation = false;
                    } else {
                        current.push(c);
                    }
                } else {
                    current.push(c);
                }
            } else {
                if c == '$' {
                    // Check if next char is '{'
                    if let Some(&'{') = chars.peek() {
                        chars.next(); // consume '{'

                        // Save the static part before interpolation
                        if !current.is_empty() {
                            parts.push(TemplateStringPart::Static(std::mem::take(&mut current)));
                        }

                        // Start interpolation
                        current.push('$');
                        current.push('{');
                        in_interpolation = true;
                        brace_depth = 1;
                    } else {
                        current.push(c);
                    }
                } else {
                    current.push(c);
                }
            }
        }

        // Add remaining static text
        if !current.is_empty() && !in_interpolation {
            parts.push(TemplateStringPart::Static(current));
        }

        // If we're still in interpolation at EOF, that's an error
        if in_interpolation {
            return Err(ParseError::InvalidSyntax {
                message: "Unterminated interpolation in template string".to_string(),
                span: span.clone(),
            });
        }

        Ok(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let source = "";
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 0);
    }

    #[test]
    fn test_function_definition() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);

        match &ast.items[0].kind {
            ItemKind::Function(func) => {
                assert_eq!(func.name.name, "main");
                assert_eq!(func.params.len(), 0);
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_arithmetic_expression() {
        let source = "fn test() { let x = 1 + 2 * 3; }";
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_function_call() {
        let source = r#"
            fn test() {
                let x = add(1, 2);
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_struct_definition() {
        let source = r#"
            struct Point {
                x: i32,
                y: i32,
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].kind {
            ItemKind::Struct(s) => {
                assert_eq!(s.name.name, "Point");
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn test_enum_definition() {
        let source = r#"
            enum Option {
                Some(T),
                None,
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].kind {
            ItemKind::Enum(e) => {
                assert_eq!(e.name.name, "Option");
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn test_if_expression() {
        let source = r#"
            fn test() {
                if x > 0 {
                    positive()
                } else {
                    negative()
                }
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let source = r#"
            fn test() {
                while condition {
                    do_work();
                }
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_for_loop() {
        let source = r#"
            fn test() {
                for item in items {
                    process(item);
                }
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_complex_program() {
        let source = r#"
            struct Point {
                x: i32,
                y: i32,
            }

            fn add_points(a: Point, b: Point) -> Point {
                a
            }

            fn main() {
                let x = 42;
            }
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 3);
    }

    // TODO: Fix trait and impl tests when full signature parsing is implemented
    // #[test]
    // fn test_trait_definition() { ... }

    // #[test]
    // fn test_impl_block() { ... }

    #[test]
    fn test_const_definition() {
        let source = r#"
            const MAX_SIZE: i32 = 100;
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].kind {
            ItemKind::Const(const_def) => {
                assert_eq!(const_def.name.name, "MAX_SIZE");
            }
            _ => panic!("expected const"),
        }
    }

    #[test]
    fn test_module_declaration() {
        let source = r#"
            mod my_module;
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].kind {
            ItemKind::Module(mod_def) => {
                assert_eq!(mod_def.name.name, "my_module");
            }
            _ => panic!("expected module"),
        }
    }

    #[test]
    fn test_use_statement() {
        let source = r#"
            use std::collections::HashMap;
        "#;
        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].kind {
            ItemKind::Use(use_stmt) => {
                match &use_stmt.path {
                    UsePath::Simple(path) => {
                        assert_eq!(path.len(), 3);
                    }
                    _ => panic!("expected simple path"),
                }
            }
            _ => panic!("expected use"),
        }
    }

    // TODO: Implement struct instantiation syntax
    // Deferred due to ambiguity with block expressions in control flow
    // #[test]
    // fn test_struct_instantiation() { ... }

    // ========== New End-to-End Tests ==========

    #[test]
    fn test_complex_expressions() {
        let source = r#"
            fn test() {
                let x = (a + b) * c / d;
                let y = func1(func2(x)).method();
                let z = a > b && c != d || e == f;
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_generic_function() {
        let source = r#"
            fn identity<T>(x: T) -> T {
                x
            }

            fn main() {
                let x = identity(42);
                let y = identity("hello");
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 2);

        match &ast.items[0].kind {
            ItemKind::Function(f) => {
                assert_eq!(f.name.name, "identity");
                assert!(f.generics.is_some());
                if let Some(g) = &f.generics {
                    assert!(g.params.len() > 0);
                }
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_path_expressions() {
        let source = r#"
            mod a {
                mod b {
                    fn func() {}
                }
            }

            fn test() {
                a::b::func();
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        assert_eq!(ast.items.len(), 2); // mod a, fn test
    }
}
