// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test async/await AST → HIR lowering pipeline

use zulon_parser::Parser;
use zulon_hir::{lower_ast_simple, HirCrate};

fn main() {
    println!("=== ZULON Async/Await HIR Lowering Test ===\n");

    let source = r#"
// Basic async function
async fn hello() {
    println("Hello, async world!");
}

// Async function with return type
async fn fetch_data() -> i32 {
    42
}

// Async function with parameters
async fn add(x: i32, y: i32) -> i32 {
    x + y
}

// Function that uses await
fn test() {
    let result = future.await;
}

// Complete async/await example
async fn example() {
    let result = fetch_data().await;
    println(result);
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

    // Verify async functions were parsed correctly
    println!("\n=== Step 1.5: Verify AST Structure ===");
    for (i, item) in ast.items.iter().enumerate() {
        match &item.kind {
            zulon_parser::ast::ItemKind::Function(func) => {
                println!("  [{}] Function: {} (is_async={})",
                    i, func.name.name, func.is_async);
            }
            _ => {}
        }
    }

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

    println!("\n=== Step 3: Verify HIR Structure ===");

    // Verify HIR structure
    let mut async_count = 0;
    let mut await_count = 0;

    for (i, item) in hir.items.iter().enumerate() {
        match item {
            zulon_hir::hir::HirItem::Function(func) => {
                println!("  [{}] Function: {} (is_async={}, is_unsafe={})",
                    i, func.name, func.is_async, func.is_unsafe);

                if func.is_async {
                    async_count += 1;
                }

                // Recursively count await expressions in the body
                await_count += count_awaits_in_block(&func.body);
            }
            _ => {}
        }
    }

    println!("\n=== Step 4: Summary ===");
    println!("✅ Async functions: {}", async_count);
    println!("✅ Await expressions: {}", await_count);
    println!("✅ Parser → HIR pipeline working for async/await!");

    // Verify expected counts
    assert_eq!(async_count, 4, "Expected 4 async functions (hello, fetch_data, add, example)");
    assert_eq!(await_count, 2, "Expected 2 await expressions (test and example)");

    println!("\n=== All Tests Passed! ===");
}

/// Recursively count await expressions in a block
fn count_awaits_in_block(block: &zulon_hir::hir::HirBlock) -> usize {
    let mut count = 0;

    for stmt in &block.statements {
        match stmt {
            zulon_hir::hir::HirStatement::Local(local) => {
                if let Some(init) = &local.init {
                    count += count_awaits_in_expr(init);
                }
            }
            zulon_hir::hir::HirStatement::Expression(expr) => {
                count += count_awaits_in_expr(expr);
            }
            zulon_hir::hir::HirStatement::Semi(expr) => {
                count += count_awaits_in_expr(expr);
            }
            _ => {}
        }
    }

    if let Some(trailing) = &block.trailing_expr {
        count += count_awaits_in_expr(trailing);
    }

    count
}

/// Recursively count await expressions in an expression
fn count_awaits_in_expr(expr: &zulon_hir::hir::HirExpression) -> usize {
    let mut count = 0;

    match expr {
        zulon_hir::hir::HirExpression::Await { .. } => {
            count += 1;
        }
        zulon_hir::hir::HirExpression::Call { func, args, .. } => {
            count += count_awaits_in_expr(func);
            for arg in args {
                count += count_awaits_in_expr(arg);
            }
        }
        zulon_hir::hir::HirExpression::BinaryOp { left, right, .. } => {
            count += count_awaits_in_expr(left);
            count += count_awaits_in_expr(right);
        }
        zulon_hir::hir::HirExpression::Block(block) => {
            count += count_awaits_in_block(block);
        }
        _ => {
            // Other expression types don't contain nested expressions
            // in a way that would have await (simplified)
        }
    }

    count
}
