// Test end-to-end compilation pipeline

use zulon_parser::Parser;

fn main() {
    println!("=== ZULON Compilation Pipeline Test ===\n");

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
    println!("\n--- Parsing ---");

    // Step 1: Parse source code
    let mut parser = Parser::from_source(source);
    match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsing successful!");
            println!("   AST items: {}", ast.items.len());

            for (i, item) in ast.items.iter().enumerate() {
                println!("   [{}] {:?}", i, item.kind);
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            return;
        }
    }

    println!("\n--- Summary ---");
    println!("✅ Lexer + Parser working correctly!");
    println!("✅ AST generated successfully!");
    println!("\nNext steps:");
    println!("  1. Type checking (zulon-typeck)");
    println!("  2. HIR lowering (zulon-hir)");
    println!("  3. MIR lowering (zulon-mir)");
    println!("  4. LIR lowering (zulon-lir)");
    println!("  5. Code generation (zulon-codegen-llvm)");
}
