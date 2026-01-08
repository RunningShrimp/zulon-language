// Test Closure Type Inference Integration with HIR Lowering
//
// This example demonstrates the end-to-end integration of:
// 1. Parsing
// 2. Type inference for closures
// 3. AST → HIR lowering with inferred types

use zulon_hir::SimpleLoweringContext;
use zulon_parser::Parser;

fn main() {
    println!("=== ZULON Closure HIR Integration Test ===\n");

    // Test 1: Closure with explicit type annotations
    println!("Test 1: Closure with explicit type annotations");
    test_explicit_typed_closure();

    // Test 2: Closure with return type annotation
    println!("\nTest 2: Closure with return type annotation");
    test_closure_with_return_type();

    // Test 3: Closure capturing outer variable
    println!("\nTest 3: Closure capturing outer variable");
    test_closure_with_capture();

    println!("\n=== All HIR Integration Tests Complete ===");
}

fn test_explicit_typed_closure() {
    let source = r#"
        fn main() {
            let add = |x: i32, y: i32| -> i32 { x + y };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut lowering_ctx = SimpleLoweringContext::new();
    match lowering_ctx.lower_ast(&ast) {
        Ok(hir) => {
            println!("  ✅ HIR lowering successful!");

            // Examine the closure in HIR
            if let Some(item) = hir.items.first() {
                if let zulon_hir::HirItem::Function(func) = item {
                    // Walk through the function body to find the closure
                    examine_closure_in_block(&func.body);
                }
            }
        }
        Err(e) => println!("  ❌ Lowering error: {}", e),
    }
}

fn test_closure_with_return_type() {
    let source = r#"
        fn main() {
            let square = |x: i32| -> i32 {
                let result = x * x;
                result
            };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut lowering_ctx = SimpleLoweringContext::new();
    match lowering_ctx.lower_ast(&ast) {
        Ok(hir) => {
            println!("  ✅ HIR lowering successful!");
            if let Some(item) = hir.items.first() {
                if let zulon_hir::HirItem::Function(func) = item {
                    examine_closure_in_block(&func.body);
                }
            }
        }
        Err(e) => println!("  ❌ Lowering error: {}", e),
    }
}

fn test_closure_with_capture() {
    let source = r#"
        fn main() {
            let x = 10;
            let add_x = |y: i32| x + y;
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut lowering_ctx = SimpleLoweringContext::new();
    match lowering_ctx.lower_ast(&ast) {
        Ok(hir) => {
            println!("  ✅ HIR lowering successful!");
            if let Some(item) = hir.items.first() {
                if let zulon_hir::HirItem::Function(func) = item {
                    examine_closure_in_block(&func.body);
                }
            }
        }
        Err(e) => println!("  ❌ Lowering error: {}", e),
    }
}

/// Recursively examine a block to find closures and print their types
fn examine_closure_in_block(block: &zulon_hir::HirBlock) {
    for stmt in &block.statements {
        if let zulon_hir::HirStatement::Local(local) = stmt {
            if let zulon_hir::HirExpression::Closure {
                params,
                return_ty,
                ty,
                captures,
                ..
            } = &local.init.as_ref().unwrap() {
                println!("    Closure: {}", local.name);
                println!("      Type: {:?}", ty);
                println!("      Parameters: {}", params.len());
                for param in params {
                    println!("        - {}: {:?}", param.name, param.ty);
                }
                println!("      Return type: {:?}", return_ty);
                println!("      Captures: {} variable(s)", captures.len());
                for capture in captures {
                    println!("        - {} ({:?})", capture.name, capture.mode);
                }
            }
        }
    }
}
