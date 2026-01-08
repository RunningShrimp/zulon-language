// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Lowering errors

use thiserror::Error;
use zulon_parser::ast::Span;

#[derive(Debug, Clone, Error)]
pub enum LoweringError {
    #[error("type checking failed: {0}")]
    TypeError(#[from] zulon_typeck::TypeError),

    #[error("missing type annotation for: {name}")]
    MissingTypeAnnotation { name: String, span: Span },

    #[error("unsupported feature in HIR: {feature}")]
    UnsupportedFeature { feature: String, span: Span },

    #[error("invalid HIR construction: {message}")]
    InvalidConstruction { message: String, span: Span },
}

pub type Result<T> = std::result::Result<T, LoweringError>;
