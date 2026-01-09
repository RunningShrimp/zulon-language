// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Compiler error types

use std::path::PathBuf;
use thiserror::Error;

/// Compiler result type
pub type Result<T> = std::result::Result<T, CompilerError>;

/// Compiler error
#[derive(Error, Debug)]
pub enum CompilerError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Lexical error
    #[error("Lexical error: {0}")]
    Lexical(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Type checking error
    #[error("Type error: {0}")]
    TypeCheck(String),

    /// HIR lowering error
    #[error("HIR lowering error: {0}")]
    HirLowering(String),

    /// MIR lowering error
    #[error("MIR lowering error: {0}")]
    MirLowering(String),

    /// LIR lowering error
    #[error("LIR lowering error: {0}")]
    LirLowering(String),

    /// Pipeline not yet implemented
    #[error("Pipeline stage not yet implemented: {0}")]
    NotImplemented(String),

    /// Code generation error
    #[error("Code generation error: {0}")]
    CodeGen(String),

    /// Build error
    #[error("Build error: {0}")]
    Build(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Multiple errors
    #[error("Multiple errors occurred:\n{0}")]
    Multiple(String),

    /// Link error
    #[error("Link error: {0}")]
    Link(String),

    /// Macro expansion error
    #[error("Macro expansion error: {0}")]
    MacroExpansion(String),
}

impl CompilerError {
    /// Create a lexical error
    pub fn lexical(msg: impl Into<String>) -> Self {
        Self::Lexical(msg.into())
    }

    /// Create a parse error
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Create a type checking error
    pub fn type_check(msg: impl Into<String>) -> Self {
        Self::TypeCheck(msg.into())
    }

    /// Create a code generation error
    pub fn code_gen(msg: impl Into<String>) -> Self {
        Self::CodeGen(msg.into())
    }

    /// Create a build error
    pub fn build(msg: impl Into<String>) -> Self {
        Self::Build(msg.into())
    }

    /// Create a link error
    pub fn link(msg: impl Into<String>) -> Self {
        Self::Link(msg.into())
    }

    /// Create a macro expansion error
    pub fn macro_expansion(msg: impl Into<String>) -> Self {
        Self::MacroExpansion(msg.into())
    }
}
