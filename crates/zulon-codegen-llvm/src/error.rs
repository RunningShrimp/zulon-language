// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Code generation errors

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CodegenError {
    #[error("type mapping error: {0}")]
    TypeError(String),

    #[error("instruction lowering error: {0}")]
    InstructionError(String),

    #[error("function lowering error: {0}")]
    FunctionError(String),

    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, CodegenError>;
