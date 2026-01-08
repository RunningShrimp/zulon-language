// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build system errors

/// Build system error type
#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildError {
    /// Code generation error
    #[error("code generation error: {0}")]
    CodeGeneration(String),

    /// LLVM tool not found
    #[error("LLVM tool not found: {0}")]
    ToolNotFound(String),

    /// LLVM assembly failed
    #[error("llvm-as failed: {0}")]
    LlvmAsFailed(String),

    /// LLVM compiler failed
    #[error("llc failed: {0}")]
    LlcFailed(String),

    /// Linker failed
    #[error("linker failed: {0}")]
    LinkerFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Invalid output
    #[error("invalid output: {0}")]
    InvalidOutput(String),

    /// Compilation failed
    #[error("compilation failed: {0}")]
    CompilationFailed(String),
}

pub type Result<T> = std::result::Result<T, BuildError>;
