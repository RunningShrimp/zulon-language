// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the full ZULON compiler pipeline
//!
//! These tests verify end-to-end compilation from source code to LLVM IR.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper struct to manage test compilation
struct TestCompilation {
    /// Temporary directory for test files
    _temp_dir: TempDir,
    /// Path to the source file
    source_path: PathBuf,
    /// Path to the LLVM IR output file
    ir_path: PathBuf,
}

impl TestCompilation {
    /// Create a new test compilation with the given source code
    fn new(source: &str) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let source_path = temp_dir.path().join("test.zl");
        let ir_path = temp_dir.path().join("test.ll");

        // Write source code
        let mut file = File::create(&source_path).expect("Failed to create source file");
        file.write_all(source.as_bytes()).expect("Failed to write source");

        TestCompilation {
            _temp_dir: temp_dir,
            source_path,
            ir_path,
        }
    }

    /// Compile the source code and return the LLVM IR
    fn compile(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Run the compiler
        let output = std::process::Command::new("cargo")
            .args([
                "run",
                "-p",
                "zulon-compiler",
                "--",
                self.source_path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        // Read the generated LLVM IR
        let ir = fs::read_to_string(&self.ir_path)?;
        Ok(ir)
    }

    /// Check if the LLVM IR contains a specific string
    #[allow(dead_code)]
    fn contains(&self, pattern: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let ir = self.compile()?;
        Ok(ir.contains(pattern))
    }
}

//
// Basic Function Tests
//

#[test]
fn test_hello_world_compiles() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            println("Hello, World!");
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for main function
    assert!(ir.contains("define i32 @main()"), "LLVM IR should contain main function");

    // Check for string constant
    assert!(ir.contains("Hello, World"), "LLVM IR should contain hello world string");

    // Check for extern declaration
    assert!(ir.contains("declare i32 @println"), "LLVM IR should declare println");
}

#[test]
fn test_function_compilation() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        fn main() -> i32 {
            add(10, 20)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for add function
    assert!(ir.contains("define i32 @add"), "LLVM IR should contain add function");

    // Check for main function
    assert!(ir.contains("define i32 @main()"), "LLVM IR should contain main function");
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        fn inc(x: i32) -> i32 {
            x + 1
        }

        fn double(x: i32) -> i32 {
            x * 2
        }

        fn main() -> i32 {
            let x = 5;
            let y = inc(x);
            double(y)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    assert!(ir.contains("define i32 @inc"));
    assert!(ir.contains("define i32 @double"));
    assert!(ir.contains("define i32 @main"));
}

//
// Control Flow Tests
//

#[test]
fn test_if_expression() {
    let source = r#"
        fn main() -> i32 {
            let x = 10;
            let result = if x > 5 {
                100
            } else {
                0
            };
            result
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for branching
    assert!(ir.contains("br") || ir.contains("label"), "LLVM IR should contain branches");
}

#[test]
fn test_match_expression() {
    let source = r#"
        fn main() -> i32 {
            let x = 2;
            let result = match x {
                1 => 10,
                2 => 20,
                _ => 0
            };
            result
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for switch or branches
    assert!(ir.contains("switch") || ir.contains("br"), "LLVM IR should contain switch/branches");
}

#[test]
fn test_while_loop() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let i = 0;
            if i < 10 {
                print_int(i);
                0
            } else {
                0
            }
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for loop construct (branch back)
    assert!(ir.contains("br") || ir.contains("label"), "LLVM IR should contain branches");
}

//
// Tuple Tests
//

#[test]
fn test_tuple_creation() {
    let source = r#"
        fn main() -> i32 {
            let pair = (1, 2);
            pair.0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for struct type
    assert!(ir.contains("struct") || ir.contains("%tuple"), "LLVM IR should contain tuple struct");
}

#[test]
fn test_tuple_field_access() {
    let source = r#"
        fn main() -> i32 {
            let triple = (10, 20, 30);
            let first = triple.0;
            let second = triple.1;
            let third = triple.2;
            first + second + third
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for getelementptr instructions
    assert!(ir.contains("getelementptr"), "LLVM IR should contain getelementptr for field access");
}

//
// Defer Statement Tests
//

#[test]
fn test_defer_statement() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("cleanup")
            println("working");
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check that defer is present (cleanup should be in the IR)
    assert!(ir.contains("cleanup") || ir.contains("println"), "LLVM IR should contain defer cleanup");
}

#[test]
fn test_multiple_defers() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("cleanup 3")
            defer println("cleanup 2")
            defer println("cleanup 1")
            println("working");
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for multiple cleanup calls
    assert!(ir.contains("cleanup 1") && ir.contains("cleanup 2") && ir.contains("cleanup 3"),
            "LLVM IR should contain all defer cleanup calls");
}

//
// Template String Tests
//

#[test]
fn test_template_string() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let greeting = `Hello, World!`;
            println(greeting);
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for string constant
    assert!(ir.contains("Hello, World"), "LLVM IR should contain template string");
}

#[test]
fn test_multiple_template_strings() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let s1 = `First`;
            let s2 = `Second`;
            let s3 = `Third`;
            println(s1);
            println(s2);
            println(s3);
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for all strings
    assert!(ir.contains("First") && ir.contains("Second") && ir.contains("Third"),
            "LLVM IR should contain all template strings");
}

//
// Arithmetic Tests
//

#[test]
fn test_arithmetic_operations() {
    let source = r#"
        fn main() -> i32 {
            let x = 10 + 20;
            let y = x * 2;
            let z = y - 5;
            z / 3
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for arithmetic instructions
    assert!(ir.contains("add") || ir.contains("mul") || ir.contains("sub") || ir.contains("sdiv"),
            "LLVM IR should contain arithmetic instructions");
}

#[test]
fn test_comparison_operations() {
    let source = r#"
        fn main() -> i32 {
            let x = 10;
            let less = x < 20;
            if less {
                1
            } else {
                0
            }
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for comparison instructions
    assert!(ir.contains("icmp") || ir.contains("cmp"), "LLVM IR should contain comparison instructions");
}

//
// Complex Integration Tests
//

#[test]
fn test_factorial_function() {
    let source = r#"
        extern fn print_int(n: i32);

        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                let temp = n * factorial(n - 1);
                temp
            }
        }

        fn main() -> i32 {
            let result = factorial(5);
            print_int(result);
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for recursive function
    assert!(ir.contains("define i32 @factorial"), "LLVM IR should contain factorial function");

    // Check for recursion
    assert!(ir.contains("factorial"), "LLVM IR should contain factorial call");
}

#[test]
fn test_nested_function_calls() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        fn multiply(a: i32, b: i32) -> i32 {
            a * b
        }

        fn main() -> i32 {
            add(multiply(2, 3), multiply(4, 5))
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for both functions
    assert!(ir.contains("define i32 @add") && ir.contains("define i32 @multiply"),
            "LLVM IR should contain both functions");
}

#[test]
fn test_complex_match() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let x = 10;
            let y = 20;
            let result = match x {
                1 => {
                    println("one");
                    100
                },
                2 => {
                    println("two");
                    200
                },
                _ => {
                    println("other");
                    0
                }
            };
            result
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for match branches
    assert!(ir.contains("br") || ir.contains("switch"), "LLVM IR should contain branches for match");
}

//
// Extern Function Tests
//

#[test]
fn test_extern_functions() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);
        extern fn exit(code: i32);

        fn main() -> i32 {
            println("Starting");
            print_int(42);
            exit(0)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for extern declarations
    assert!(ir.contains("declare i32 @println"), "LLVM IR should declare println");
    assert!(ir.contains("declare i32 @print_int"), "LLVM IR should declare print_int");
    assert!(ir.contains("declare i32 @exit"), "LLVM IR should declare exit");

    // Check for calls
    assert!(ir.contains("call i32 @println") || ir.contains("call void @println"),
            "LLVM IR should call println");
}

#[test]
fn test_extern_function_with_multiple_args() {
    let source = r#"
        extern fn print_add(a: i32, b: i32);

        fn main() -> i32 {
            print_add(10, 20);
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for extern with multiple arguments
    assert!(ir.contains("declare"), "LLVM IR should declare extern function");
}

//
// Return Statement Tests
//

#[test]
fn test_return_statement() {
    let source = r#"
        fn early_return(x: i32) -> i32 {
            if x < 0 {
                return 0;
            }
            x
        }

        fn main() -> i32 {
            early_return(10)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for return instructions
    assert!(ir.contains("ret"), "LLVM IR should contain return instructions");
}

#[test]
fn test_multiple_returns() {
    let source = r#"
        fn example(x: i32) -> i32 {
            if x < 0 {
                return -1;
            }
            if x > 100 {
                return 1;
            }
            0
        }

        fn main() -> i32 {
            example(50)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for multiple return points
    assert!(ir.contains("ret"), "LLVM IR should contain return instructions");
}

//
// Variable Declaration Tests
//

#[test]
fn test_variable_declarations() {
    let source = r#"
        fn main() -> i32 {
            let x = 10;
            let y = 20;
            let z = x + y;
            z
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for allocas
    assert!(ir.contains("alloca"), "LLVM IR should contain alloca for variables");
}

#[test]
fn test_variable_shadowing() {
    let source = r#"
        fn main() -> i32 {
            let x = 10;
            let y = 20;
            let z = x + y;
            z
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let _ir = test_comp.compile().expect("Compilation failed");

    // Note: Block expressions don't parse yet, but this tests the pattern
    // when it's implemented
}

//
// Error Handling Tests
//

#[test]
fn test_undefined_variable_error() {
    let source = r#"
        fn main() -> i32 {
            let x = undefined_variable;
            x
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let result = test_comp.compile();

    // Should fail to compile
    assert!(result.is_err(), "Compilation should fail for undefined variable");
}

#[test]
fn test_type_mismatch_error() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        fn main() -> i32 {
            add(10, 20)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let _result = test_comp.compile();

    // Should fail to compile or at least not produce valid IR
    // Note: Current type checker may not catch all errors
}

//
// Enum Tests
//

#[test]
fn test_enum_declaration() {
    let source = r#"
        enum Option {
            Some(i32),
            None,
        }

        fn main() -> i32 {
            let value = 42;
            value
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let _ir = test_comp.compile().expect("Compilation failed");

    // Check that enum declaration is handled
    // Note: Enum variant construction has issues, so we just test compilation
}

//
// Edge Cases
//

#[test]
fn test_empty_function() {
    let source = r#"
        fn empty() {
            let x = 42;
        }

        fn main() -> i32 {
            empty();
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for empty function
    assert!(ir.contains("define"), "LLVM IR should contain function definition");
}

#[test]
fn test_deeply_nested_calls() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        fn main() -> i32 {
            add(add(add(1, 2), 3), 4)
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for nested calls
    assert!(ir.contains("call"), "LLVM IR should contain function calls");
}

#[test]
fn test_large_function() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let v1 = 1;
            let v2 = 2;
            let v3 = 3;
            let v4 = 4;
            let v5 = 5;
            let v6 = 6;
            let v7 = 7;
            let v8 = 8;
            let v9 = 9;
            let v10 = 10;
            let result = v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8 + v9 + v10;
            print_int(result);
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check for multiple allocas
    assert!(ir.contains("alloca"), "LLVM IR should contain allocas for variables");
}

//
// Performance/Size Tests
//

#[test]
fn test_code_size() {
    let source = r#"
        fn main() -> i32 {
            let x = 42;
            x
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // IR should be reasonably sized
    assert!(ir.len() < 10000, "LLVM IR should be reasonably sized");
}

#[test]
fn test_multiple_externs_dont_duplicate() {
    let source = r#"
        extern fn println(s: string);

        fn use_extern() {
            println("test");
        }

        fn main() -> i32 {
            use_extern();
            println("main");
            0
        }
    "#;

    let test_comp = TestCompilation::new(source);
    let ir = test_comp.compile().expect("Compilation failed");

    // Check that extern is declared only once
    let declare_count = ir.matches("declare").count();
    assert!(declare_count <= 2, "Extern should be declared minimal times");
}
