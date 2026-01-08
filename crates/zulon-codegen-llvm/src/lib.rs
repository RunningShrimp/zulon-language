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

pub mod ty;
pub mod codegen;
pub mod error;
pub mod layout;
pub mod enum_layout;
pub mod abi;
pub mod optimize;

pub use ty::LlvmType;
pub use codegen::CodeGenerator;
pub use error::{CodegenError, Result};
pub use layout::{StructLayout, LayoutCache, FieldInfo};
pub use enum_layout::{EnumLayout, EnumLayoutCache, VariantInfo};
pub use abi::{CallingConvention, CallInfo, ArgLocation};
pub use optimize::{OptPassManager, OptConfig};
