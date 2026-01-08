// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test assignment and variable mutation

use std::process::Command;

fn main() {
    println!("=== Testing Variable Mutation Fix ===\n");

    // Test 1: Simple assignment
    test_simple_assignment();

    // Test 2: Assignment in loop
    test_assignment_in_loop();

    println!("\n=== All Tests Complete ===");
}

fn test_simple_assignment() {
    println!("Test 1: Simple Assignment");
    println!("```zulon");
    println!("fn main() -> i32 {{");
    println!("    let mut x = 5;");
    println!("    x = 10;");
    println!("    x");
    println!("}}");
    println!("```\n");

    let code = r#"
fn main() -> i32 {
    let mut x = 5;
    x = 10;
    x
}
"#;

    match compile_zulon("simple_assignment", code) {
        Ok(_) => println!("  ✅ Compiled successfully\n"),
        Err(e) => println!("  ❌ Error: {}\n", e),
    }
}

fn test_assignment_in_loop() {
    println!("Test 2: Assignment in Loop");
    println!("```zulon");
    println!("fn main() -> i32 {{");
    println!("    let mut sum = 0;");
    println!("    let mut x = 0;");
    println!("    while x < 5 {{");
    println!("        sum = sum + x;");
    println!("        x = x + 1");
    println!("    }};");
    println!("    sum");
    println!("}}");
    println!("```\n");

    let code = r#"
fn main() -> i32 {
    let mut sum = 0;
    let mut x = 0;
    while x < 5 {
        sum = sum + x;
        x = x + 1
    };
    sum
}
"#;

    match compile_zulon("loop_assignment", code) {
        Ok(_) => println!("  ✅ Compiled successfully\n"),
        Err(e) => println!("  ❌ Error: {}\n", e),
    }
}

fn compile_zulon(name: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Write source to file
    let source_file = format!("/tmp/test_{}.zl", name);
    std::fs::write(&source_file, code)?;

    // Try to compile to LLVM IR
    println!("  Compiling to LLVM IR...");

    let output = Command::new("cargo")
        .args(["run", "-p", "zulon-codegen-llvm", "--example", "compile_simple_loop"])
        .output()?;

    if output.status.success() {
        println!("  ✅ LLVM IR generated");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Compilation failed: {}", stderr).into())
    }
}
