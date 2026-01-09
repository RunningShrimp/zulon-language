// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ZULON Type Checker
//!
//! This crate provides type checking and inference for the ZULON programming language.
//!
//! ## Type System
//!
//! ZULON uses a static type system with:
//! - Type inference
//! - Generic types
//! - Trait bounds
//! - Algebraic data types (structs and enums)
//!
//! ## Usage
//!
//! ```rust
//! use zulon_typeck::TypeChecker;
//! use zulon_parser::Parser;
//!
//! let source = r#"
//!     fn add(a: i32, b: i32) -> i32 {
//!     a + b
//!     }
//! "#;
//!
//! let mut parser = Parser::from_source(source);
//! let ast = parser.parse().unwrap();
//!
//! let mut checker = TypeChecker::new();
//! match checker.check(&ast) {
//!     Ok(()) => println!("Type checking passed!"),
//!     Err(e) => eprintln!("Type error: {}", e),
//! }
//! ```

pub mod ty;
pub mod env;
pub mod error;
pub mod checker;
pub mod infer;
pub mod diagnostic;

pub use ty::{Ty, TyVarId, GenericParam, TraitBound, subst_ty};
pub use env::Env;
pub use error::{TypeError, Result};
pub use checker::TypeChecker;
pub use infer::{Substitution, unify};
