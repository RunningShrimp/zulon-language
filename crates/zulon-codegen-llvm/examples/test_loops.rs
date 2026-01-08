// Test if loops are supported in the compiler pipeline
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Loop Support ===\n");

    // Test 1: Simple loop
    println!("Test 1: Infinite loop");
    let source1 = r#"
fn main() -> i32 {
    loop {
        return 42
    }
}
"#;

    match test_compile(source1, "Infinite loop") {
        Ok(_) => println!("  ✅ PASS"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    // Test 2: While loop
    println!("\nTest 2: While loop");
    let source2 = r#"
fn main() -> i32 {
    let mut x = 0;
    while x < 10 {
        x = x + 1
    };
    x
}
"#;

    match test_compile(source2, "While loop") {
        Ok(_) => println!("  ✅ PASS"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    // Test 3: For loop with range
    println!("\nTest 3: For loop");
    let source3 = r#"
fn main() -> i32 {
    let mut sum = 0;
    for i in 0..10 {
        sum = sum + i
    };
    sum
}
"#;

    match test_compile(source3, "For loop") {
        Ok(_) => println!("  ✅ PASS"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    // Test 4: Break statement
    println!("\nTest 4: Break statement");
    let source4 = r#"
fn main() -> i32 {
    let mut x = 0;
    loop {
        if x > 10 {
            break
        };
        x = x + 1
    };
    x
}
"#;

    match test_compile(source4, "Break statement") {
        Ok(_) => println!("  ✅ PASS"),
        Err(e) => println!("  ❌ FAIL - {}", e),
    }

    println!("\n=== Summary ===");
    println!("Loop support in compiler:");
    println!("  Parser: ✅ (implemented)");
    println!("  HIR:    ?");
    println!("  MIR:    ?");
    println!("  LIR:    ?");
    println!("  LLVM:   ?");

    Ok(())
}

fn test_compile(source: &str, _name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;

    println!("  Parsed successfully");

    let hir = lower_ast_simple(&ast)?;
    println!("  HIR lowering successful");

    let mir = lower_hir(&hir)?;
    println!("  MIR lowering successful");

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;
    println!("  LIR lowering successful - {} functions", lir.functions.len());

    Ok(())
}
