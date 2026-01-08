// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! High-Level Intermediate Representation (HIR)
//!
//! HIR is a typed, desugared representation of ZULON programs.
//! It preserves the high-level structure of the program while
//! making types explicit and resolving name binding.
//!
//! # Status
//!
//! This crate is currently under active development. The HIR type system
//! and node definitions are complete, but the lowering implementation
//! is being updated to match the actual parser AST structure.
//!
//! # Architecture
//!
//! HIR sits between type checking and MIR:
//!
//! ```text
//! AST → Type Checking → HIR → MIR → Code Gen
//! ```
//!
//! Key features:
//! - **Explicit Types**: Every node carries its type inline
//! - **Desugared**: Complex syntax simplified to canonical forms
//! - **Validated**: All type checking complete

pub mod ty;
pub mod hir;
pub mod error;
pub mod capture;
// pub mod lower;  // TEMPORARILY DISABLED - has compilation errors, using simple_lower instead
pub mod simple_lower;
pub mod test_discovery;

pub use ty::HirTy;
pub use hir::*;
pub use error::{LoweringError, Result};
pub use capture::{CaptureAnalyzer, CaptureAnalysis, analyze_captures, Environment, SimpleEnvironment};
// pub use lower::{LoweringContext, lower_ast};  // TEMPORARILY DISABLED
pub use simple_lower::{SimpleLoweringContext, lower_ast_simple};
pub use test_discovery::{discover_tests, DiscoveredTest};
