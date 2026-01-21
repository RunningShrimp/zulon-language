// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # LLVM IR Code Generator
//!
//! This module converts LIR to LLVM IR (text format).
//!
//! ## Architecture
//!
//! ```text
//! LIR (SSA form)
//!   ↓ Code Generation
//! LLVM IR (text)
//!   ↓ llc (LLVM compiler)
//! Machine Code
//!   ↓ ld (linker)
//! Executable
//! ```

pub mod abi;
pub mod codegen;
pub mod enum_layout;
pub mod error;
pub mod layout;
pub mod optimize;
pub mod ty;

pub use abi::{ArgLocation, CallInfo, CallingConvention};
pub use codegen::CodeGenerator;
pub use enum_layout::{EnumLayout, EnumLayoutCache, VariantInfo};
pub use error::{CodegenError, Result};
pub use layout::{FieldInfo, LayoutCache, StructLayout};
pub use optimize::{OptConfig, OptPassManager};
pub use ty::LlvmType;
