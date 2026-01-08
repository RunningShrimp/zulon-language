// Test Closure AST → HIR Lowering

use zulon_parser::Parser;
use zulon_hir::{lower_ast_simple, HirCrate};

fn main() {
    println!("=== ZULON Closure AST → HIR Lowering Test ===\n");

    let source = r#"
fn main() {
    // Test 1: Simple closure with one parameter
    let square = |x| x * x;

    // Test 2: Closure with multiple parameters
    let add = |x, y| x + y;

    // Test 3: Closure with type annotations
    let typed_add = |x: i32, y: i32| -> i32 { x + y };

    // Test 4: Closure with block body
    let complex = |x| {
        let y = x * 2;
        y + 10
    };

    // Test 5: Nested closures
    let outer = |x| {
        let inner = |y| x + y;
        inner(10)
    };

    // Test 6: Immediate invocation
    let result = (|a, b| a + b)(10, 20);
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
            eprintln!("❌ Lowering error: {}", e);
            return;
        }
    };

    println!("\n=== Step 3: Inspecting HIR ===");

    // Step 3: Inspect the lowered HIR
    for (i, item) in hir.items.iter().enumerate() {
        match item {
            zulon_hir::HirItem::Function(func) => {
                println!("\nFunction {}: {}", i + 1, func.name);
                println!("  Parameters: {}", func.params.len());

                // Print function body
                println_closure_expressions(&func.body, 2);
            }
            _ => {}
        }
    }

    println!("\n=== Test Complete ===");
    println!("✅ All closures lowered successfully!");
}

fn println_closure_expressions(block: &zulon_hir::HirBlock, indent: usize) {
    let indent_str = " ".repeat(indent);

    // Print statements
    for stmt in &block.statements {
        match stmt {
            zulon_hir::HirStatement::Local(local) => {
                println!("{}let {} = {:?}", indent_str, local.name, local.init);
            }
            zulon_hir::HirStatement::Expression(expr) => {
                print_closure_expression(expr, &indent_str);
            }
            zulon_hir::HirStatement::Semi(expr) => {
                print!("{};", indent_str);
                print_closure_expression(expr, &indent_str);
                println!();
            }
            _ => {}
        }
    }

    // Print trailing expression
    if let Some(expr) = &block.trailing_expr {
        print!("{}(trailing) ", indent_str);
        print_closure_expression(expr, &indent_str);
        println!();
    }
}

fn print_closure_expression(expr: &zulon_hir::HirExpression, indent: &str) {
    match expr {
        zulon_hir::HirExpression::Closure { params, return_ty, .. } => {
            println!("|closure| params={:?} return_ty={:?}", params, return_ty);
        }
        zulon_hir::HirExpression::Call { func, args, .. } => {
            print!("(");
            print_closure_expression(func, indent);
            println!(")({:?})", args);
        }
        zulon_hir::HirExpression::BinaryOp { op, left, right, .. } => {
            print!("(");
            print_closure_expression(left, indent);
            print!(" {:?} ", op);
            print_closure_expression(right, indent);
            print!(")");
        }
        zulon_hir::HirExpression::Variable(name, ..) => {
            print!("{}", name);
        }
        zulon_hir::HirExpression::Literal(lit, ..) => {
            print!("{:?}", lit);
        }
        _ => {
            print!("(...)");
        }
    }
}
