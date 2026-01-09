// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! MIR errors

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MirError {
    #[error("HIR lowering error: {0}")]
    LoweringError(String),

    #[error("type error: {0}")]
    TypeError(String),

    #[error("borrow checking error: {0}")]
    BorrowError(String),

    #[error("invalid MIR construction: {0}")]
    InvalidConstruction(String),

    #[error("invalid field access: field '{field}' - {reason}")]
    InvalidFieldAccess {
        field: String,
        reason: String,
    },
}

pub type Result<T> = std::result::Result<T, MirError>;
