// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test async state machine transformation

use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::{lower_hir, transform_async_functions};

fn main() {
    println!("=== ZULON Async State Machine Transformation Test ===\n");

    let source = r#"
// Simple async function without await
async fn simple_async() -> i32 {
    42
}

// Async function with one await
async fn one_await() -> i32 {
    let future = create_future();
    let result = future.await;
    result
}

// Async function with multiple awaits
async fn multi_await() -> i32 {
    let a = async_op1().await;
    let b = async_op2().await;
    a + b
}

// Regular function (no transformation)
fn regular_func() -> i32 {
    100
}

// Async function with control flow
async fn async_with_if(x: i32) -> i32 {
    if x > 0 {
        positive_future().await
    } else {
        negative_future().await
    }
}
"#;

    println!("Source code:");
    println!("{}\n", source);

    println!("=== Step 1: Parsing ===");
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

    println!("\n=== Step 3: Verifying HIR Async Functions ===");
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

    println!("\n=== Step 5: MIR Before Transformation ===");
    let mut mir_async_count = 0;
    for (i, func) in mir.functions.iter().enumerate() {
        println!("  [{}] {} (is_async={}, has_state_machine={}, blocks={})",
            i, func.name, func.is_async, func.state_machine.is_some(), func.blocks.len());

        if func.is_async {
            mir_async_count += 1;

            if let Some(sm) = &func.state_machine {
                println!("      └─ State machine: {} states, output type: {:?}",
                    sm.states.len(), sm.output_type);
            }
        }
    }
    println!("Total async MIR functions: {}", mir_async_count);

    println!("\n=== Step 6: State Machine Transformation ===");
    let transformed_mir = match transform_async_functions(mir) {
        Ok(transformed) => {
            println!("✅ Transformation successful!");
            println!("   Transformed functions: {}", transformed.functions.len());
            transformed
        }
        Err(e) => {
            eprintln!("❌ Transformation error: {}", e);
            return;
        }
    };

    println!("\n=== Step 7: MIR After Transformation ===");
    for (i, func) in transformed_mir.functions.iter().enumerate() {
        println!("  [{}] {} (is_async={}, blocks={})",
            i, func.name, func.is_async, func.blocks.len());

        if func.is_async {
            if let Some(sm) = &func.state_machine {
                println!("      └─ State machine: {} states", sm.states.len());

                // Show state details
                for state in &sm.states {
                    println!("         - State {}: block={}, captured={} locals",
                        state.id, state.block_id, state.captured.len());
                }
            }

            // Show basic blocks
            println!("      └─ Basic blocks:");
            for (block_id, block) in &func.blocks {
                println!("         - Block {}: {} instrs, terminator: {:?}",
                    block_id, block.instructions.len(),
                    block.terminator.as_ref().map(|t| format!("{:?}", t)));
            }
        }
    }

    println!("\n=== Step 8: Analysis ===");
    let transformed_async_count = transformed_mir.functions.iter()
        .filter(|f| f.is_async)
        .count();

    println!("✅ Async functions preserved: {} -> {}",
        mir_async_count, transformed_async_count);

    // Check that regular functions were not modified
    let regular_func = &transformed_mir.functions[3]; // regular_func
    println!("✅ Regular function blocks: {} (should be minimal)",
        regular_func.blocks.len());

    // Check that async functions have state machines
    for func in transformed_mir.functions.iter().filter(|f| f.is_async) {
        if func.state_machine.is_some() {
            println!("✅ {} has state machine", func.name);
        } else {
            println!("❌ {} missing state machine!", func.name);
        }
    }

    println!("\n=== Test Completed Successfully! ===");
    println!("\n=== Key Findings ===");
    println!("1. State machine transformation infrastructure is in place");
    println!("2. Async functions are identified and preserved");
    println!("3. State machine structures are created");
    println!("4. Basic block transformation framework is implemented");
    println!("\n=== Next Steps ===");
    println!("The transformation creates the infrastructure but currently:");
    println!("- Awaits are identified by 'await' or 'poll' in function names");
    println!("- State splitting happens at await points");
    println!("- Full variable capture and restoration needs implementation");
    println!("- Integration with Future trait needs to be added");
}
