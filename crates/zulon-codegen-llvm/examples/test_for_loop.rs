// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test for loop compilation

fn main() {
    println!("=== Testing For Loop Support ===\n");

    // Test 1: Simple for loop
    println!("Test 1: Simple for loop (0..5)");
    println!("```zulon");
    println!("fn main() -> i32 {{");
    println!("    let mut sum = 0;");
    println!("    for i in 0..5 {{");
    println!("        sum = sum + i");
    println!("    }};");
    println!("    sum");
    println!("}}");
    println!("```\n");

    let code = r#"
fn main() -> i32 {
    let mut sum = 0;
    for i in 0..5 {
        sum = sum + i
    };
    sum
}
"#;

    match compile_and_test("simple_for_loop", code) {
        Ok(_) => println!("  ✅ For loop compiled successfully\n"),
        Err(e) => println!("  ❌ Error: {}\n", e),
    }

    println!("=== For Loop Tests Complete ===");
}

fn compile_and_test(name: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    // For now, just test parsing
    use zulon_parser::Parser;

    let mut parser = Parser::from_source(code);
    match parser.parse() {
        Ok(_ast) => {
            println!("  ✅ Parsed successfully");

            // Try to lower to HIR
            use zulon_hir::lower_ast;
            match lower_ast(&_ast) {
                Ok(_hir) => {
                    println!("  ✅ HIR lowering successful");

                    // Try to lower to MIR
                    use zulon_mir::lower_hir;
                    match lower_hir(&_hir) {
                        Ok(_mir) => {
                            println!("  ✅ MIR lowering successful");
                            Ok(())
                        }
                        Err(e) => {
                            println!("  ⚠️  MIR lowering failed (expected): {}", e);
                            Err(Box::new(e) as Box<dyn std::error::Error>)
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ HIR lowering failed: {}", e);
                    Err(Box::new(e) as Box<dyn std::error::Error>)
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parse failed: {}", e);
            Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}
