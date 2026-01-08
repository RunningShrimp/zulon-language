// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build functionality for yan build command

use std::process::Command;
use anyhow::{Result, Context};

/// Build a ZULON project
pub fn build_project(release: bool, package: Option<&str>, jobs: usize) -> Result<()> {
    println!("🔨 Building ZULON project...");

    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
    }

    if let Some(pkg) = package {
        cmd.arg("-p").arg(pkg);
    }

    cmd.env("CARGO_BUILD_JOBS", jobs.to_string());

    println!("   Running: cargo build");
    if release {
        println!("   Mode: release (optimized)");
    } else {
        println!("   Mode: debug");
    }
    if let Some(pkg) = package {
        println!("   Package: {}", pkg);
    }
    println!("   Jobs: {}", jobs);
    println!();

    let status = cmd
        .status()
        .with_context(|| "Failed to execute cargo build".to_string())?;

    if status.success() {
        println!();
        println!("✅ Build successful!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Build failed with exit code: {:?}", status.code()))
    }
}

/// Build a specific example
pub fn build_example(example: &str, release: bool) -> Result<()> {
    println!("🔨 Building example: {}", example);

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.arg("-p").arg("zulon-build");  // Build examples from zulon-build package
    cmd.arg("--example").arg(example);

    if release {
        cmd.arg("--release");
    }

    println!("   Running: cargo build -p zulon-build --example {}", example);
    println!();

    let status = cmd
        .status()
        .with_context(|| "Failed to execute cargo build".to_string())?;

    if status.success() {
        println!();
        println!("✅ Example build successful!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Build failed with exit code: {:?}", status.code()))
    }
}

/// Check if the current directory is a valid ZULON project
pub fn check_project_dir() -> Result<()> {
    if !std::path::Path::new("Cargo.toml").exists() {
        return Err(anyhow::anyhow!(
            "Not a ZULON project (Cargo.toml not found). \
            Run this command in a project directory."
        ));
    }
    Ok(())
}
