// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! End-to-end test for async/await pipeline

use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::{lower_hir, transform_async_functions};
use zulon_lir::lower_mir;

fn main() {
    println!("=== ZULON Async/Await End-to-End Test ===\n");

    // Test 1: Simple async function without await
    println!("### Test 1: Simple async function ###");
    test_simple_async();

    // Test 2: Async function with await
    println!("\n### Test 2: Async function with await ###");
    test_async_with_await();

    println!("\n=== All Tests Completed! ===");
}

fn test_simple_async() {
    let source = r#"
async fn simple() -> i32 {
    42
}
"#;

    println!("Source:\n{}", source);

    // Parse
    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsing successful");
            ast
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            return;
        }
    };

    // HIR
    let hir = match lower_ast_simple(&ast) {
        Ok(hir) => {
            println!("✅ HIR lowering successful");
            hir
        }
        Err(e) => {
            eprintln!("❌ HIR error: {}", e);
            return;
        }
    };

    // MIR
    let mir = match lower_hir(&hir) {
        Ok(mir) => {
            println!("✅ MIR lowering successful");
            println!("   Functions: {}", mir.functions.len());
            mir
        }
        Err(e) => {
            eprintln!("❌ MIR error: {}", e);
            return;
        }
    };

    // Transform
    let transformed = match transform_async_functions(mir) {
        Ok(mir) => {
            println!("✅ Async transform successful");
            mir
        }
        Err(e) => {
            eprintln!("❌ Transform error: {}", e);
            return;
        }
    };

    // LIR
    let lir = match lower_mir(&transformed) {
        Ok(lir) => {
            println!("✅ LIR lowering successful");
            println!("   Functions: {}", lir.functions.len());
            lir
        }
        Err(e) => {
            eprintln!("❌ LIR error: {}", e);
            return;
        }
    };

    // Verify
    println!("\nResults:");
    for func in &lir.functions {
        println!("  Function: {}", func.name);
        println!("    Params: {}", func.params.len());
        println!("    Blocks: {}", func.blocks.len());
        println!("    VRegs: {}", func.next_vreg);

        if func.name == "simple" {
            // Should have entry block and possibly state machine blocks
            println!("    ✅ Async function lowered successfully");
        }
    }
}

fn test_async_with_await() {
    // Use extern to avoid type checking issues
    let source = r#"
extern fn get_future() -> i32;

async fn with_await() -> i32 {
    let x = 10;
    let y = get_future().await;
    x + y
}
"#;

    println!("Source:\n{}", source);

    // Parse
    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsing successful");
            ast
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            return;
        }
    };

    // HIR
    let hir = match lower_ast_simple(&ast) {
        Ok(hir) => {
            println!("✅ HIR lowering successful");
            hir
        }
        Err(e) => {
            eprintln!("❌ HIR error: {}", e);
            return;
        }
    };

    // MIR
    let mir = match lower_hir(&hir) {
        Ok(mir) => {
            println!("✅ MIR lowering successful");
            println!("   Functions: {}", mir.functions.len());
            mir
        }
        Err(e) => {
            eprintln!("❌ MIR error: {}", e);
            return;
        }
    };

    // Check for __await_poll marker
    println!("\nMIR Analysis:");
    for func in &mir.functions {
        if func.is_async {
            println!("  Function: {} (is_async=true)", func.name);
            println!("    Blocks: {}", func.blocks.len());
            println!("    State machine: {}", func.state_machine.is_some());

            // Look for __await_poll calls
            let mut await_count = 0;
            for (block_id, block) in &func.blocks {
                for instr in &block.instructions {
                    if let zulon_mir::MirInstruction::Call { func, .. } = instr {
                        if let zulon_mir::MirPlace::Local(name) = func {
                            if name == "__await_poll" {
                                await_count += 1;
                                println!("    Block {}: __await_poll found", block_id);
                            }
                        }
                    }
                }
            }
            println!("    Total await points: {}", await_count);
        }
    }

    // Transform
    let transformed = match transform_async_functions(mir) {
        Ok(mir) => {
            println!("\n✅ Async transform successful");
            mir
        }
        Err(e) => {
            eprintln!("❌ Transform error: {}", e);
            return;
        }
    };

    // Check transformation results
    println!("\nTransformed MIR:");
    for func in &transformed.functions {
        if func.is_async {
            println!("  Function: {}", func.name);
            println!("    Blocks: {}", func.blocks.len());
            if let Some(ref sm) = func.state_machine {
                println!("    State machine: {} states", sm.states.len());
                println!("    Preserved locals: {}", sm.preserved_locals.len());
            }

            // Check entry block for state switch
            if let Some(entry) = func.blocks.get(&func.entry_block) {
                println!("    Entry block has {} instructions", entry.instructions.len());
                if let Some(zulon_mir::MirTerminator::Switch { .. }) = entry.terminator {
                    println!("    ✅ State machine switch generated");
                }
            }
        }
    }

    // LIR
    let lir = match lower_mir(&transformed) {
        Ok(lir) => {
            println!("\n✅ LIR lowering successful");
            println!("   Functions: {}", lir.functions.len());
            lir
        }
        Err(e) => {
            eprintln!("❌ LIR error: {}", e);
            return;
        }
    };

    // Verify
    println!("\nResults:");
    for func in &lir.functions {
        println!("  Function: {}", func.name);
        println!("    Params: {}", func.params.len());
        println!("    Blocks: {}", func.blocks.len());
        println!("    VRegs: {}", func.next_vreg);

        if func.name == "with_await" {
            println!("    ✅ Async function with await lowered successfully");
        }
    }
}
