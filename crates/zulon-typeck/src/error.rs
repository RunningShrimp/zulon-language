// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type checking errors
//!
//! This module defines the error types used in type checking.

use crate::ty::Ty;
use zulon_parser::ast::Span;
use thiserror::Error;

/// Type checking error
#[derive(Debug, Clone, Error)]
pub enum TypeError {
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: Ty,
        found: Ty,
        span: Span,
    },

    #[error("cannot find type {name} in this scope")]
    UndefinedType {
        name: String,
        span: Span,
    },

    #[error("cannot find value {name} in this scope")]
    UndefinedVariable {
        name: String,
        span: Span,
    },

    #[error("cannot find function {name} in this scope")]
    UndefinedFunction {
        name: String,
        span: Span,
    },

    #[error("cannot find effect {name} in this scope")]
    UndefinedEffect {
        name: String,
        span: Span,
    },

    #[error("cannot call non-function type")]
    NotCallable {
        ty: Ty,
        span: Span,
    },

    #[error("expected {expected} arguments, found {found}")]
    ArityMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("field {field} does not exist on type {ty}")]
    UnknownField {
        field: String,
        ty: Ty,
        span: Span,
    },

    #[error("type {ty} is not indexable")]
    NotIndexable {
        ty: Ty,
        span: Span,
    },

    #[error("cannot assign to immutable value")]
    CannotAssignImmutable {
        span: Span,
    },

    #[error("cannot borrow {ty} as mutable")]
    CannotBorrowMut {
        ty: Ty,
        span: Span,
    },

    #[error("inference error: {message}")]
    InferenceError {
        message: String,
        span: Span,
    },

    #[error("generic parameter {name} not provided")]
    MissingGenericParameter {
        name: String,
        span: Span,
    },

    #[error("trait bound not satisfied")]
    TraitBoundNotSatisfied {
        trait_name: String,
        ty: Ty,
        span: Span,
    },

    #[error("recursive type: {ty} contains itself")]
    RecursiveType {
        ty: Ty,
        span: Span,
    },

    #[error("integer literal too large")]
    IntegerOverflow {
        span: Span,
    },

    #[error("cannot convert {from} to {to}")]
    CannotConvert {
        from: Ty,
        to: Ty,
        span: Span,
    },
}

/// Result type for type checking
pub type Result<T> = std::result::Result<T, TypeError>;
