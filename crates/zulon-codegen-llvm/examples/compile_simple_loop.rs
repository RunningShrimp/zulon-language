// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test simple loop compilation

use std::io::Cursor;
use std::fs;
use std::process::Command;

use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Simple Loop Compilation\n");
    println!("{}", "=".repeat(60));

    // Parse
    let source = fs::read_to_string("test_simple_loop.zl")?;
    let mut parser = Parser::from_source(&source);
    let ast = parser.parse()
        .map_err(|e| format!("Parse error: {}", e))?;

    println!("✅ 1. Parse successful");

    // HIR lowering
    let hir = lower_ast_simple(&ast)
        .map_err(|e| format!("HIR lowering error: {:?}", e))?;

    println!("✅ 2. HIR lowering successful");

    // MIR lowering
    let mir = lower_hir(&hir)
        .map_err(|e| format!("MIR lowering error: {:?}", e))?;

    println!("✅ 3. MIR lowering successful");

    // LIR lowering
    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)
        .map_err(|e| format!("LIR lowering error: {:?}", e))?;

    println!("✅ 4. LIR lowering successful ({} functions)", lir.functions.len());

    // LLVM code generation
    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = CodeGenerator::new(&mut buffer);

    for func in &lir.functions {
        codegen.generate_function(func)
            .map_err(|e| format!("LLVM codegen error for function {}: {:?}", func.name, e))?;
    }

    let llvm_ir = String::from_utf8(buffer.into_inner())?;

    println!("✅ 5. LLVM IR generation successful");

    // Write LLVM IR to file
    fs::write("test_simple_loop.ll", &llvm_ir)?;
    println!("✅ 6. Wrote LLVM IR to test_simple_loop.ll");

    // Compile with LLVM
    let output = Command::new("llc")
        .arg("-filetype=obj")
        .arg("-o")
        .arg("test_simple_loop.o")
        .arg("test_simple_loop.ll")
        .output()
        .map_err(|e| format!("Failed to run llc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llc compilation failed:\n{}", stderr).into());
    }

    println!("✅ 7. Compiled to test_simple_loop.o");

    // Link executable
    let output = Command::new("clang")
        .arg("-o")
        .arg("test_simple_loop")
        .arg("test_simple_loop.o")
        .arg("-lSystem")
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Linking failed:\n{}", stderr).into());
    }

    println!("✅ 8. Linked to test_simple_loop executable");

    // Run the executable
    let output = Command::new("./test_simple_loop")
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Execution failed:\n{}", stderr).into());
    }

    let exit_code = output.status.code().unwrap_or(0);
    println!("✅ 9. Execution successful (exit code: {})", exit_code);

    println!("\n{}", "=".repeat(60));
    if exit_code == 42 {
        println!("🎉 SUCCESS! Loop compiled and executed correctly!");
    } else {
        println!("⚠️  Exit code was {} (expected 42)", exit_code);
    }

    Ok(())
}
