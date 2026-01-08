// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test loop compilation through full pipeline

use zulon_parser::Parser;
use zulon_hir::simple_lower::SimpleLoweringPass;
use zulon_mir::lower_hir_to_mir::MirLoweringPass;
use zulon_lir::lower_mir_to_lir::LirLoweringPass;
use zulon_codegen_llvm::CodeGenerator;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing ZULON Loop Compilation Pipeline\n");
    println!("{}", "=".repeat(60));

    // Test cases
    let tests = vec![
        ("test_simple_loop.zl", "Simple infinite loop"),
        ("test_while_counter.zl", "While loop with mutable counter"),
        ("test_while_break.zl", "While loop with break"),
        ("test_while_continue.zl", "While loop with continue"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (filename, description) in tests {
        println!("\n📝 Test: {} ({})", filename, description);
        println!("   {}", "-".repeat(56));

        match test_loop_compilation(filename) {
            Ok(_) => {
                println!("   ✅ PASSED");
                passed += 1;
            }
            Err(e) => {
                println!("   ❌ FAILED:");
                println!("      {}", e);
                failed += 1;
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("📊 Results: {} passed, {} failed", passed, failed);

    if failed == 0 {
        println!("✅ All tests passed!");
    } else {
        println!("⚠️  Some tests failed");
    }

    Ok(())
}

fn test_loop_compilation(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse
    let source = fs::read_to_string(filename)?;
    let mut parser = Parser::from_source(&source);
    let ast = parser.parse()
        .map_err(|e| format!("Parse error: {}", e))?;

    println!("   1. Parse ✅");

    // HIR lowering
    let mut hir_pass = SimpleLoweringPass::new();
    let hir = hir_pass.lower_ast(&ast)
        .map_err(|e| format!("HIR lowering error: {:?}", e))?;

    println!("   2. HIR lowering ✅");

    // MIR lowering
    let mut mir_pass = MirLoweringPass::new();
    let mir = mir_pass.lower_hir(&hir)
        .map_err(|e| format!("MIR lowering error: {:?}", e))?;

    println!("   3. MIR lowering ✅");

    // LIR lowering
    let mut lir_pass = LirLoweringPass::new();
    let lir = lir_pass.lower_mir(&mir)
        .map_err(|e| format!("LIR lowering error: {:?}", e))?;

    println!("   4. LIR lowering ✅");

    // LLVM code generation
    let mut codegen = CodeGenerator::new();
    let llvm_ir = codegen.generate(&lir)
        .map_err(|e| format!("LLVM codegen error: {:?}", e))?;

    println!("   5. LLVM IR generation ✅");

    // Write LLVM IR to file
    let llvm_file = filename.replace(".zl", ".ll");
    fs::write(&llvm_file, llvm_ir)?;

    // Compile with LLVM
    let obj_file = filename.replace(".zl", ".o");
    let output = Command::new("llc")
        .arg("-filetype=obj")
        .arg("-o")
        .arg(&obj_file)
        .arg(&llvm_file)
        .output()
        .map_err(|e| format!("Failed to run llc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llc compilation failed:\n{}", stderr).into());
    }

    println!("   6. LLVM compilation ✅");

    // Link executable
    let exe_file = if filename.ends_with(".zl") {
        &filename[..filename.len() - 3]
    } else {
        filename
    };

    let output = Command::new("clang")
        .arg("-o")
        .arg(exe_file)
        .arg(&obj_file)
        .arg("-lSystem")
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Linking failed:\n{}", stderr).into());
    }

    println!("   7. Linking ✅");

    // Run the executable
    let output = Command::new(format!("./{}", exe_file))
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Execution failed:\n{}", stderr).into());
    }

    let exit_code = output.status.code().unwrap_or(0);
    println!("   8. Execution ✅ (exit code: {})", exit_code);

    Ok(())
}
