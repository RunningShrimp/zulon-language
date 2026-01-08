// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LIR errors

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum LirError {
    #[error("MIR lowering error: {0}")]
    LoweringError(String),

    #[error("SSA construction error: {0}")]
    SsaError(String),

    #[error("optimization error: {0}")]
    OptimizationError(String),

    #[error("invalid LIR construction: {0}")]
    InvalidConstruction(String),
}

pub type Result<T> = std::result::Result<T, LirError>;
