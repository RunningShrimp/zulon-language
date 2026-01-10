// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build functionality for yan build command

use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context};
use std::fs;
use std::io::Cursor;

/// Build a single ZULON source file to executable
/// Returns the path to the generated executable
pub fn build_zulon_file(source_file: &str, _release: bool) -> Result<String> {
    println!("🔨 Building ZULON file: {}", source_file);

    // Check if file exists
    if !Path::new(source_file).exists() {
        return Err(anyhow::anyhow!("Source file not found: {}", source_file));
    }

    // Read source code
    let source = fs::read_to_string(source_file)
        .with_context(|| format!("Failed to read source file: {}", source_file))?;

    println!("   Source: {} bytes", source.len());

    // Parse
    println!("\n   [1/5] Parsing...");
    let mut parser = zulon_parser::Parser::from_source(&source);
    let ast = parser.parse()
        .with_context(|| "Parsing failed")?;
    println!("      ✅ Parsed {} items", ast.items.len());

    // HIR
    println!("   [2/5] Lowering to HIR...");
    let hir = zulon_hir::lower_ast_simple(&ast)
        .with_context(|| "HIR lowering failed")?;
    println!("      ✅ HIR: {} items", hir.items.len());

    // MIR
    println!("   [3/5] Lowering to MIR...");
    let mir = zulon_mir::lower_hir(&hir)
        .with_context(|| "MIR lowering failed")?;
    println!("      ✅ MIR: {} functions", mir.functions.len());

    // LIR
    println!("   [4/5] Lowering to LIR...");
    let mut lir_ctx = zulon_lir::lower::LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)
        .with_context(|| "LIR lowering failed")?;
    println!("      ✅ LIR: {} functions", lir.functions.len());

    // LLVM IR Generation
    println!("   [5/5] Generating LLVM IR...");
    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = zulon_codegen_llvm::CodeGenerator::new(&mut buffer);
    codegen.generate_module_with_externals(&lir.functions, &lir.externals)
        .with_context(|| "LLVM IR generation failed")?;

    let llvm_ir = String::from_utf8(buffer.into_inner())?;
    println!("      ✅ LLVM IR: {} bytes", llvm_ir.len());

    // Determine output filename
    let output_base = source_file.strip_suffix(".zl").unwrap_or(source_file);
    let ll_file = format!("{}.ll", output_base);

    // Save LLVM IR
    fs::write(&ll_file, &llvm_ir)
        .with_context(|| format!("Failed to write LLVM IR to: {}", ll_file))?;
    println!("\n   ✅ Saved LLVM IR to: {}", ll_file);

    // Compile to assembly
    println!("\n   Compiling to assembly...");
    let s_file = format!("{}.s", output_base);
    let status = Command::new("llc")
        .arg(&ll_file)
        .arg("-o")
        .arg(&s_file)
        .status()
        .with_context(|| "Failed to run llc")?;

    if !status.success() {
        return Err(anyhow::anyhow!("llc compilation failed"));
    }
    println!("      ✅ Assembly: {}", s_file);

    // Assemble and link
    println!("   Assembling and linking...");
    let exe_file = output_base.to_string();
    let status = Command::new("clang")
        .arg(&s_file)
        .arg("-o")
        .arg(&exe_file)
        .status()
        .with_context(|| "Failed to run clang")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Linking failed"));
    }
    println!("      ✅ Executable: {}", exe_file);

    // Clean up intermediate files
    // TEMPORARILY DISABLED FOR DEBUGGING
    /*
    if fs::remove_file(&ll_file).is_ok() {
        println!("      🧹 Cleaned up: {}", ll_file);
    }
    if fs::remove_file(&s_file).is_ok() {
        println!("      🧹 Cleaned up: {}", s_file);
    }
    */

    println!("\n✅ Build complete!");
    println!("   Run with: ./{}", exe_file);

    Ok(exe_file)
}

/// Build a ZULON project
#[allow(dead_code)]
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
/// Returns the path to the generated example executable
pub fn build_example(example: &str, release: bool) -> Result<String> {
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

    if !status.success() {
        return Err(anyhow::anyhow!("Build failed with exit code: {:?}", status.code()));
    }

    // Determine the example executable path
    let profile = if release { "release" } else { "debug" };
    let exe_path = format!("target/{}/examples/{}", profile, example);

    println!();
    println!("✅ Example build successful!");

    Ok(exe_path)
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
