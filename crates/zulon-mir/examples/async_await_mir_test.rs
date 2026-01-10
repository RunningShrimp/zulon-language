// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test async/await MIR lowering

use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

fn main() {
    println!("=== ZULON Async/Await MIR Lowering Test ===\n");

    let source = r#"
// Basic async function
async fn hello() {
    println("Hello, async world!");
}

// Async function with return type
async fn fetch_data() -> i32 {
    42
}

// Regular function for comparison
fn regular_func() -> i32 {
    100
}

// Async function with await
async fn example() {
    let result = fetch_data().await;
    println(result);
}
"#;

    println!("Source code:");
    println!("{}", source);
    println!("\n=== Step 1: Parsing ===");

    // Parse source code
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

    // Lower AST to HIR
    let hir = match lower_ast_simple(&ast) {
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

    println!("\n=== Step 3: Verify HIR Async Functions ===");
    let mut async_count = 0;
    for (i, item) in hir.items.iter().enumerate() {
        match item {
            zulon_hir::hir::HirItem::Function(func) => {
                println!("  [{}] {} (is_async={})", i, func.name, func.is_async);
                if func.is_async {
                    async_count += 1;
                }
            }
            _ => {}
        }
    }
    println!("Total async functions: {}", async_count);

    println!("\n=== Step 4: Lowering to MIR ===");

    // Lower HIR to MIR
    let mir = match lower_hir(&hir) {
        Ok(mir) => {
            println!("✅ MIR lowering successful!");
            println!("   MIR functions: {}", mir.functions.len());
            mir
        }
        Err(e) => {
            eprintln!("❌ MIR lowering error: {}", e);
            return;
        }
    };

    println!("\n=== Step 5: Verify MIR Async Functions ===");
    let mut mir_async_count = 0;
    for (i, func) in mir.functions.iter().enumerate() {
        println!("  [{}] {} (is_async={}, has_state_machine={})",
            i, func.name, func.is_async, func.state_machine.is_some());

        if func.is_async {
            mir_async_count += 1;

            // Verify state machine exists
            if let Some(sm) = &func.state_machine {
                println!("      └─ State machine: {} states, output type: {:?}",
                    sm.states.len(), sm.output_type);
            }
        }
    }
    println!("Total async MIR functions: {}", mir_async_count);

    println!("\n=== Step 6: Summary ===");
    println!("✅ HIR async functions: {}", async_count);
    println!("✅ MIR async functions: {}", mir_async_count);

    if async_count == mir_async_count && async_count > 0 {
        println!("✅ All async functions successfully lowered to MIR!");
    } else {
        println!("❌ Mismatch between HIR and MIR async counts!");
        return;
    }

    println!("\n=== Test Completed Successfully! ===");
}
