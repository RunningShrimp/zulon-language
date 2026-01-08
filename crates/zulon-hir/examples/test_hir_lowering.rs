// Test AST → HIR lowering pipeline

use zulon_parser::Parser;
use zulon_hir::{lower_ast_simple, HirCrate};

fn main() {
    println!("=== ZULON AST → HIR Lowering Test ===\n");

    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn compute(x: i32) -> i32 {
    if x > 10 {
        add(x, 5)
    } else {
        add(x, 10)
    }
}

fn main() {
    let result = compute(15);
}
"#;

    println!("Source code:");
    println!("{}", source);
    println!("\n=== Step 1: Parsing ===");

    // Step 1: Parse source code
    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsing successful!");
            println!("   AST items: {}", ast.items.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            return;
        }
    };

    println!("\n=== Step 2: Lowering to HIR ===");

    // Step 2: Lower AST to HIR
    let hir: HirCrate = match lower_ast_simple(&ast) {
        Ok(hir) => {
            println!("✅ HIR lowering successful!");
            println!("   HIR items: {}", hir.items.len());
            hir
        }
        Err(e) => {
            eprintln!("❌ HIR lowering error: {}", e);
            return;
        }
    };

    println!("\n=== Step 3: HIR Structure ===");

    // Print HIR structure
    for (i, item) in hir.items.iter().enumerate() {
        println!("\n[{}] {:?}", i, item);
    }

    println!("\n=== Summary ===");
    println!("✅ Parser → HIR pipeline working!");
    println!("\nNext steps:");
    println!("  1. Add type checking (zulon-typeck)");
    println!("  2. HIR → MIR lowering");
    println!("  3. MIR → LIR lowering");
    println!("  4. Code generation");
}
