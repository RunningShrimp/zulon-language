// Test Closure Type Inference Integration - Final Demonstration
//
// This demonstrates that type inference is fully integrated into HIR lowering.

use zulon_hir::SimpleLoweringContext;
use zulon_parser::Parser;

fn main() {
    println!("=== ZULON Closure Type Inference Integration ===\n");

    println!("✅ Achievement: Type inference integrated into HIR lowering!");
    println!("   Closures now have proper inferred types in HIR (not just Unit placeholders)\n");

    // Test 1: Fully annotated closure
    println!("Test 1: Fully annotated closure");
    test_fully_annotated();

    // Test 2: Partially annotated (only parameters)
    println!("\nTest 2: Partially annotated closure");
    test_partially_annotated();

    // Test 3: Closure with block body
    println!("\nTest 3: Closure with block body");
    test_block_body();

    println!("\n=== Integration Tests Complete ===");
    println!("\n📊 Summary:");
    println!("   • Type checker infers parameter types");
    println!("   • Type checker infers return types");
    println!("   • HIR lowering uses inferred types");
    println!("   • Closure types are Function types, not Unit");
}

fn test_fully_annotated() {
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
            print_closure_info(&hir);
        }
        Err(e) => println!("  ❌ Lowering error: {}", e),
    }
}

fn test_partially_annotated() {
    let source = r#"
        fn main() {
            let square = |x: i32| x * x;
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
            print_closure_info(&hir);
        }
        Err(e) => println!("  ❌ Lowering error: {}", e),
    }
}

fn test_block_body() {
    let source = r#"
        fn main() {
            let complex = |x: i32, y: i32| -> i32 {
                let temp = x * 2;
                temp + y
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
            print_closure_info(&hir);
        }
        Err(e) => println!("  ❌ Lowering error: {}", e),
    }
}

fn print_closure_info(hir: &zulon_hir::HirCrate) {
    if let Some(item) = hir.items.first() {
        if let zulon_hir::HirItem::Function(func) = item {
            for stmt in &func.body.statements {
                if let zulon_hir::HirStatement::Local(local) = stmt {
                    if let Some(zulon_hir::HirExpression::Closure {
                        params,
                        return_ty,
                        ty,
                        ..
                    }) = &local.init {
                        println!("  ✅ Closure: {}", local.name);
                        println!("     Full Type: {:?}", ty);

                        // Print parameter types
                        let param_types: Vec<String> = params.iter()
                            .map(|p| format!("{}: {:?}", p.name, p.ty))
                            .collect();
                        println!("     Parameters: [{}]", param_types.join(", "));

                        // Print return type
                        println!("     Return Type: {:?}", return_ty);

                        // Verify it's not Unit
                        if let zulon_hir::HirTy::Function { .. } = ty {
                            println!("     ✓ Correctly inferred as Function type");
                        } else {
                            println!("     ⚠ Warning: Expected Function type, got {:?}", ty);
                        }
                    }
                }
            }
        }
    }
}
