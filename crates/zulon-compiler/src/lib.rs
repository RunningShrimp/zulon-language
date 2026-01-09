// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Compiler
//!
//! This crate provides the compiler driver that integrates all frontend components
//! and compiles ZULON source files to executables.
//!
//! ## Compilation Pipeline
//!
//! ```text
//! .zl source file
//!     ↓
//! [Lexer] → Tokens
//!     ↓
//! [Parser] → AST
//!     ↓
//! [Type Checker] → HIR
//!     ↓
//! [MIR] → MIR
//!     ↓
//! [LIR] → LIR
//!     ↓
//! [LLVM Codegen] → LLVM IR
//!     ↓
//! [Build Pipeline] → Executable
//! ```
//!
//! ## Example
//!
//! ```rust,no_run
//! use zulon_compiler::{Compiler, CompilerConfig};
//! use std::path::Path;
//!
//! let config = CompilerConfig::default();
//! let mut compiler = Compiler::new(config);
//!
//! // Compile a ZULON source file
//! let executable = compiler.compile_file(Path::new("hello.zl")).unwrap();
//! println!("Compiled to: {}", executable.display());
//! ```

pub mod compiler;
pub mod error;
pub mod macro_expander;

pub use compiler::{Compiler, CompilerConfig};
pub use error::{CompilerError, Result};
pub use macro_expander::MacroExpander;
