// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Mid-Level Intermediate Representation (MIR)
//!
//! MIR is a simplified, control-flow explicit IR that comes after HIR.
//! Key characteristics:
//!
//! - **Explicit Control Flow**: All branches and loops are explicit
//! - **Basic Blocks**: Sequences of instructions without control flow
//! - **Temporary Variables**: All nested expressions are flattened
//! - **Borrow Checking Ready**: Designed for Tree Borrows model
//!
//! ## Architecture
//!
//! ```text
//! HIR (high-level)
//!   ↓ Lowering
//! MIR (mid-level) ← We are here
//!   ↓ Optimization
//! LIR (low-level)
//!   ↓ Code Generation
//! Machine Code
//! ```

pub mod ty;
pub mod mir;
pub mod lower;
pub mod error;
pub mod borrow;
pub mod effect;

pub use ty::MirTy;
pub use mir::*;
pub use error::{MirError, Result};
pub use lower::{lower_hir, MirLoweringContext};
pub use borrow::{check_borrows, BorrowKind, Permission};
pub use effect::{check_effects, Effect, EffectSet};
