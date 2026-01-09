// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Low-Level Intermediate Representation (LIR)
//!
//! LIR is a low-level, SSA-based IR that comes after MIR.
//! Key characteristics:
//!
//! - **SSA Form**: Static Single Assignment for easier analysis
//! - **Virtual Registers**: Infinite virtual register space
//! - **Explicit Control Flow**: CFG with dominator tree
//! - **Optimization Ready**: Designed for classic optimizations
//!
//! ## Architecture
//!
//! ```text
//! HIR (high-level)
//!   ↓ Lowering
//! MIR (mid-level)
//!   ↓ Optimization & SSA
//! LIR (low-level) ← We are here
//!   ↓ Code Generation
//! LLVM IR
//!   ↓
//! Machine Code
//! ```

pub mod ty;
pub mod lir;
pub mod lower;
pub mod optimize;
pub mod error;

pub use ty::LirTy;
pub use lir::*;
pub use error::{LirError, Result};
pub use lower::LirLoweringContext;
