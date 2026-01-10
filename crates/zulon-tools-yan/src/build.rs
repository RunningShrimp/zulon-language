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
pub fn build_zulon_file(source_file: &str, release: bool) -> Result<String> {
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

    // Register all user-defined and standard library types
    use zulon_codegen_llvm::layout::{FieldInfo, StructLayout};
    use zulon_lir::LirTy;
    use zulon_hir::HirItem;

    // Helper function to get type size in bytes
    fn get_type_size(ty: &zulon_hir::HirTy) -> u64 {
        match ty {
            zulon_hir::HirTy::I32 => 4,
            zulon_hir::HirTy::I64 => 8,
            zulon_hir::HirTy::Bool => 1,
            zulon_hir::HirTy::Unit => 0,
            _ => 4, // Default to 4 for complex types
        }
    }

    // Helper function to get type alignment in bytes
    fn get_type_align(ty: &zulon_hir::HirTy) -> u64 {
        match ty {
            zulon_hir::HirTy::I64 => 8,
            _ => 4, // Most types are 4-byte aligned
        }
    }

    // Collect and register all structs from HIR
    let mut structs_to_register = Vec::new();

    for item in &hir.items {
        if let HirItem::Struct(struct_def) = item {
            let struct_name = struct_def.name.clone();
            let mut fields = Vec::new();
            let mut current_offset = 0u64;
            let mut max_align = 1u64;

            for field in &struct_def.fields {
                let field_size = get_type_size(&field.ty);
                let field_align = get_type_align(&field.ty);

                // Align current offset to field's alignment
                let aligned_offset = (current_offset + field_align - 1) / field_align * field_align;

                // Convert HirTy to LirTy
                let lir_ty = match &field.ty {
                    zulon_hir::HirTy::I32 => LirTy::I32,
                    zulon_hir::HirTy::I64 => LirTy::I64,
                    zulon_hir::HirTy::Bool => LirTy::Bool,
                    zulon_hir::HirTy::Unit => LirTy::Unit,
                    _ => LirTy::I32, // Default for complex types
                };

                fields.push(FieldInfo {
                    name: field.name.clone(),
                    ty: lir_ty,
                    offset: aligned_offset,
                    size: field_size,
                    align: field_align,
                });

                current_offset = aligned_offset + field_size;
                max_align = max_align.max(field_align);
            }

            // Calculate total size (pad to max alignment)
            let struct_size = (current_offset + max_align - 1) / max_align * max_align;
            let tail_padding = struct_size - current_offset;

            structs_to_register.push(StructLayout {
                name: struct_name,
                fields,
                size: struct_size,
                align: max_align,
                tail_padding,
            });
        }
    }

    // Register all collected structs
    for struct_layout in structs_to_register {
        codegen.register_struct(struct_layout);
    }

    // Register Outcome struct if not already present (for error handling)
    let outcome_layout = StructLayout {
        name: "Outcome".to_string(),
        fields: vec![
            FieldInfo {
                name: "discriminant".to_string(),
                ty: LirTy::I32,
                offset: 0,
                size: 4,
                align: 4,
            },
            FieldInfo {
                name: "data".to_string(),
                ty: LirTy::I32,
                offset: 4,
                size: 4,
                align: 4,
            },
        ],
        size: 8,
        align: 4,
        tail_padding: 0,
    };
    codegen.register_struct(outcome_layout);

    // Auto-inject common C standard library externals (printf, scanf)
    // These are used implicitly by ZULON programs
    let mut externals = lir.externals.clone();

    // Check if printf is already declared
    let has_printf = externals.iter().any(|e| e.name == "printf");
    if !has_printf {
        externals.push(zulon_lir::LirExternal {
            name: "printf".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I32))], // format string pointer
            return_type: LirTy::I32,
            variadic: true,
        });
    }

    // Check if scanf is already declared
    let has_scanf = externals.iter().any(|e| e.name == "scanf");
    if !has_scanf {
        externals.push(zulon_lir::LirExternal {
            name: "scanf".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I32))], // format string pointer
            return_type: LirTy::I32,
            variadic: true,
        });
    }

    // Check if string_concat is already declared (for template string interpolation)
    let has_string_concat = externals.iter().any(|e| e.name == "string_concat");
    if !has_string_concat {
        externals.push(zulon_lir::LirExternal {
            name: "string_concat".to_string(),
            param_types: vec![
                LirTy::Ptr(Box::new(LirTy::U8)), // str1: *u8
                LirTy::Ptr(Box::new(LirTy::U8)), // str2: *u8
            ],
            return_type: LirTy::Ptr(Box::new(LirTy::U8)), // returns *u8
            variadic: false,
        });
    }

    // Async runtime external declarations (Phase 1: External Declarations)
    // These functions will be provided by the async runtime crate

    // async_sleep(duration_ms: i64) -> ()
    let has_async_sleep = externals.iter().any(|e| e.name == "async_sleep");
    if !has_async_sleep {
        externals.push(zulon_lir::LirExternal {
            name: "async_sleep".to_string(),
            param_types: vec![LirTy::I64], // duration_ms: i64
            return_type: LirTy::Unit,      // returns ()
            variadic: false,
        });
    }

    // async_file_read(path: *u8) -> *u8
    let has_async_file_read = externals.iter().any(|e| e.name == "async_file_read");
    if !has_async_file_read {
        externals.push(zulon_lir::LirExternal {
            name: "async_file_read".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::U8))], // path: *u8
            return_type: LirTy::Ptr(Box::new(LirTy::U8)),      // returns *u8 (file contents)
            variadic: false,
        });
    }

    // async_file_write(path: *u8, data: *u8) -> ()
    let has_async_file_write = externals.iter().any(|e| e.name == "async_file_write");
    if !has_async_file_write {
        externals.push(zulon_lir::LirExternal {
            name: "async_file_write".to_string(),
            param_types: vec![
                LirTy::Ptr(Box::new(LirTy::U8)), // path: *u8
                LirTy::Ptr(Box::new(LirTy::U8)), // data: *u8
            ],
            return_type: LirTy::Unit, // returns ()
            variadic: false,
        });
    }

    // async_tcp_connect(host: *u8, port: i16) -> i32
    let has_async_tcp_connect = externals.iter().any(|e| e.name == "async_tcp_connect");
    if !has_async_tcp_connect {
        externals.push(zulon_lir::LirExternal {
            name: "async_tcp_connect".to_string(),
            param_types: vec![
                LirTy::Ptr(Box::new(LirTy::U8)), // host: *u8
                LirTy::I16,                     // port: i16
            ],
            return_type: LirTy::I32, // returns socket fd
            variadic: false,
        });
    }

    println!("      ✅ Injected {} async runtime external declarations", 4);

    codegen.generate_module_with_externals(&lir.functions, &externals)
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

    // Determine optimization level
    let opt_level = if release { "-O2" } else { "-O0" };
    println!("      Optimization level: {}", opt_level);

    let mut llc_cmd = Command::new("llc");
    llc_cmd
        .arg(&ll_file)
        .arg("-o")
        .arg(&s_file)
        .arg(opt_level);  // Add optimization flag

    // Strip debug symbols in release mode for smaller binaries
    if release {
        println!("      Stripping debug symbols...");
        // We'll strip after linking instead
    }

    let status = llc_cmd
        .status()
        .with_context(|| "Failed to run llc")?;

    if !status.success() {
        return Err(anyhow::anyhow!("llc compilation failed"));
    }
    println!("      ✅ Assembly: {}", s_file);

    // Compile runtime library
    println!("   Compiling runtime library...");
    let runtime_dir = Path::new("runtime");
    let runtime_c = runtime_dir.join("zulon_runtime.c");
    let runtime_o = runtime_dir.join("zulon_runtime.o");

    if !runtime_c.exists() {
        return Err(anyhow::anyhow!("Runtime source not found: {:?}", runtime_c));
    }

    // Compile runtime C code to object file
    let mut gcc_cmd = Command::new("clang");
    gcc_cmd
        .arg("-c")
        .arg(&runtime_c)
        .arg("-o")
        .arg(&runtime_o)
        .arg("-O2");

    let gcc_status = gcc_cmd
        .status()
        .with_context(|| "Failed to compile runtime library")?;

    if !gcc_status.success() {
        return Err(anyhow::anyhow!("Runtime compilation failed"));
    }
    println!("      ✅ Runtime library compiled");

    // Assemble and link
    println!("   Assembling and linking...");
    let exe_file = output_base.to_string();

    let mut clang_cmd = Command::new("clang");
    clang_cmd
        .arg(&s_file)
        .arg(&runtime_o)  // Link runtime library
        .arg("-o")
        .arg(&exe_file);

    // Add optimization flags in release mode
    if release {
        clang_cmd.arg("-O2");
    }

    let status = clang_cmd
        .status()
        .with_context(|| "Failed to run clang")?;

    // Strip binary in release mode
    if release {
        println!("   Stripping binary...");
        let strip_status = Command::new("llvm-strip")
            .arg(&exe_file)
            .status();

        if let Ok(true) = strip_status.map(|s| s.success()) {
            println!("      ✅ Stripped debug symbols");
        } else {
            // llvm-strip might not be available, try strip
            let strip_status = Command::new("strip")
                .arg(&exe_file)
                .status();
            if strip_status.map(|s| s.success()).unwrap_or(false) {
                println!("      ✅ Stripped debug symbols");
            }
        }
    }

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
    if fs::remove_file(&runtime_o).is_ok() {
        println!("      🧹 Cleaned up: {}", runtime_o);
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
