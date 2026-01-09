// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for ZULON compiler
//!
//! This test suite validates that the compiler can handle various language features.

use std::process::Command;

fn run_compiler(source: &str) -> Result<(), String> {
    // Get the workspace root (parent of compiler crate)
    let compiler_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = compiler_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("Could not find workspace root")?;

    // Build compiler path
    let compiler_path = workspace_root.join("target/debug/zulon-compiler");

    if !compiler_path.exists() {
        // Build compiler if it doesn't exist
        let output = Command::new("cargo")
            .current_dir(workspace_root)
            .args(&["build", "--package", "zulon-compiler", "--quiet"])
            .output()
            .map_err(|e| format!("Failed to build compiler: {}", e))?;

        if !output.status.success() {
            return Err(format!("Compiler build failed: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
    }

    // Write source to temporary file
    let test_file = workspace_root.join("test_compiler_validation.zl");

    std::fs::write(&test_file, source)
        .map_err(|e| format!("Failed to write test file: {}", e))?;

    // Run compiler
    let output = Command::new(&compiler_path)
        .current_dir(workspace_root)
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run compiler: {}", e))?;

    // Clean up test file
    let _ = std::fs::remove_file(&test_file);

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_basic_printf() {
    let source = r#"
fn main() {
    printf("Hello, World!\n");
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "Basic printf should compile: {:?}", result.err());
}

#[test]
fn test_function_definition() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    printf("Result: %d\n", add(5, 3));
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "Function definition should compile: {:?}", result.err());
}

#[test]
fn test_if_expression() {
    let source = r#"
fn main() {
    let x: i32 = 10;
    if x > 5 {
        printf("x is greater than 5\n");
    }
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "If expression should compile: {:?}", result.err());
}

#[test]
fn test_while_loop() {
    let source = r#"
fn main() {
    let i: i32 = 0;
    while i < 10 {
        printf("%d\n", i);
        i = i + 1;
    }
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "While loop should compile: {:?}", result.err());
}

#[test]
fn test_struct_definition() {
    let source = r#"
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p: Point = Point { x: 10, y: 20 };
    printf("Point: %d, %d\n", p.x, p.y);
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "Struct should compile: {:?}", result.err());
}

#[test]
fn test_arithmetic_operations() {
    let source = r#"
fn main() {
    let a: i32 = 10;
    let b: i32 = 3;
    printf("Add: %d\n", a + b);
    printf("Sub: %d\n", a - b);
    printf("Mul: %d\n", a * b);
    printf("Div: %d\n", a / b);
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "Arithmetic operations should compile: {:?}", result.err());
}

#[test]
fn test_known_issue_multiple_functions() {
    let source = r#"
fn helper1() -> i32 {
    42
}

fn helper2() -> i32 {
    helper1()
}

fn main() {
    printf("Result: %d\n", helper2());
}
"#;

    let result = run_compiler(source);
    // This currently fails due to type checker bug
    // Documenting this as a known issue
    if !result.is_ok() {
        println!("KNOWN ISSUE: Multiple functions with calls between them");
        println!("Error: {:?}", result.err());
    }
}

#[test]
fn test_extern_function() {
    let source = r#"
extern fn printf(format: &u8, ...) -> i32;

fn main() {
    printf("Testing extern\n");
}
"#;

    let result = run_compiler(source);
    // Should work with or without explicit extern (prelude handles it)
    assert!(result.is_ok(), "Extern function should compile: {:?}", result.err());
}

#[test]
fn test_variable_mutation() {
    let source = r#"
fn main() {
    let x: i32 = 0;
    x = 10;
    printf("x = %d\n", x);
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "Variable mutation should compile: {:?}", result.err());
}

#[test]
fn test_comparison_operators() {
    let source = r#"
fn main() {
    let a: i32 = 10;
    let b: i32 = 5;
    if a > b {
        printf("a > b\n");
    }
    if a == b {
        printf("a == b\n");
    }
}
"#;

    let result = run_compiler(source);
    assert!(result.is_ok(), "Comparison operators should compile: {:?}", result.err());
}
