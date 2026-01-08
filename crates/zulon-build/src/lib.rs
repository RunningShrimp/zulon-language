// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON build system
//!
//! This crate provides the build pipeline for compiling ZULON programs to executables.

pub mod error;
pub mod pipeline;

pub use error::{BuildError, Result};
pub use pipeline::{BuildConfig, BuildPipeline};
