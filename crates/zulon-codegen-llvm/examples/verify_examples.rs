// Verify which ZULON example programs can be compiled with current compiler
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

fn test_compile(source: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing: {}", name);

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;

    println!("  ✅ PASS - {} functions compiled", lir.functions.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZULON Compiler Verification Test ===\n");

    // Test 1: Minimal program
    let test1 = r#"
fn main() -> i32 {
    42
}
"#;
    test_compile(test1, "Minimal (return constant)")?;

    // Test 2: Function calls
    let test2 = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() -> i32 {
    let result = add(10, 32);
    result
}
"#;
    test_compile(test2, "Function calls with let binding")?;

    // Test 3: If/else
    let test3 = r#"
fn max(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}

fn main() -> i32 {
    let m = max(10, 20);
    m
}
"#;
    test_compile(test3, "If/else expression")?;

    // Test 4: Nested if/else
    let test4 = r#"
fn categorize(n: i32) -> i32 {
    if n < 0 {
        -1
    } else {
        if n > 100 {
            1
        } else {
            0
        }
    }
}

fn main() -> i32 {
    let x = categorize(50);
    x
}
"#;
    test_compile(test4, "Nested if/else")?;

    // Test 5: Recursion
    let test5 = r#"
fn fib(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        let a = fib(n - 1);
        let b = fib(n - 2);
        a + b
    }
}

fn main() -> i32 {
    let result = fib(10);
    result
}
"#;
    test_compile(test5, "Recursive function (Fibonacci)")?;

    // Test 6: Arithmetic operations
    let test6 = r#"
fn compute(a: i32, b: i32) -> i32 {
    let sum = a + b;
    let diff = a - b;
    let prod = a * b;
    sum + diff + prod
}

fn main() -> i32 {
    compute(5, 3)
}
"#;
    test_compile(test6, "Multiple arithmetic operations")?;

    // Test 7: Complex expressions
    let test7 = r#"
fn complex_calc(x: i32) -> i32 {
    if x > 10 {
        let y = x * 2;
        let z = y + 5;
        z
    } else {
        let w = x + 10;
        w
    }
}

fn main() -> i32 {
    complex_calc(15)
}
"#;
    test_compile(test7, "Complex expression with let bindings")?;

    println!("\n=== Summary ===");
    println!("✅ All 7 test cases passed!");
    println!("\nSupported features:");
    println!("  ✅ Functions with multiple parameters");
    println!("  ✅ If/else expressions");
    println!("  ✅ Nested conditionals");
    println!("  ✅ Recursive functions");
    println!("  ✅ Let bindings");
    println!("  ✅ Integer arithmetic (+, -, *)");
    println!("  ✅ Comparison operators (>, <, <=, >=)");
    println!("\nCurrent limitations:");
    println!("  ❌ Structs and impl blocks");
    println!("  ❌ Loops (for, while, loop)");
    println!("  ❌ Pattern matching");
    println!("  ❌ References and borrowing");
    println!("  ❌ Error handling types");
    println!("  ❌ Macros (println!, etc.)");

    Ok(())
}
