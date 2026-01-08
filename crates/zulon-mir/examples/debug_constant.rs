// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Debug test for constant value propagation through IR levels

use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple test with a constant
    let source = r#"
fn test_const() -> i32 {
    42
}
"#;

    println!("=== Testing Constant Value Propagation ===\n");
    println!("Source code:");
    println!("{}", source);
    println!();

    // Parse to AST
    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse()?;

    println!("\n--- Step 1: AST ---");
    println!("Top-level items: {}", ast.items.len());
    for (i, item) in ast.items.iter().enumerate() {
        println!("  [{}] {:?}", i, item);
    }

    // Lower to HIR
    let hir = lower_ast_simple(&ast)?;

    println!("\n--- Step 2: HIR ---");
    println!("Items: {}", hir.items.len());
    for (i, item) in hir.items.iter().enumerate() {
        println!("  [{}] {:?}", i, item);
        if let zulon_hir::HirItem::Function(func) = item {
            println!("    Function name: {}", func.name);
            println!("    Params: {:?}", func.params);
            println!("    Return type: {:?}", func.return_type);
            let block = &func.body;
            println!("    Body statements: {}", block.statements.len());
            for (j, stmt) in block.statements.iter().enumerate() {
                println!("      [{}] {:?}", j, stmt);
            }
            if let Some(trailing) = &block.trailing_expr {
                println!("    Trailing: {:?}", trailing);
            }
        }
    }

    // Lower to MIR
    let mir = lower_hir(&hir)?;

    println!("\n--- Step 3: MIR ---");
    println!("Functions: {}", mir.functions.len());
    for func in &mir.functions {
        println!("  fn {}:", func.name);
        println!("    params: {:?}", func.params);
        println!("    blocks: {}", func.blocks.len());
        for (bid, block) in &func.blocks {
            println!("    block {}:", bid);
            for (i, inst) in block.instructions.iter().enumerate() {
                println!("      [{}] {:?}", i, inst);
            }
            if let Some(terminator) = &block.terminator {
                println!("      terminator: {:?}", terminator);
            }
        }
    }

    // Try a more complex example with comparison
    let source2 = r#"
fn compare(x: i32) -> i32 {
    if x > 10 {
        5
    } else {
        10
    }
}
"#;

    println!("\n\n=== Testing Comparison with Constants ===\n");
    println!("Source code:");
    println!("{}", source2);
    println!();

    let mut parser2 = zulon_parser::Parser::from_source(source2);
    let ast2 = parser2.parse()?;
    let hir2 = lower_ast_simple(&ast2)?;
    let mir2 = lower_hir(&hir2)?;

    println!("\n--- MIR for comparison function ---");
    for func in &mir2.functions {
        if func.name == "compare" {
            println!("  fn {}:", func.name);
            for (bid, block) in &func.blocks {
                println!("    block {}:", bid);
                for (i, inst) in block.instructions.iter().enumerate() {
                    println!("      [{}] {:?}", i, inst);
                }
            }
        }
    }

    Ok(())
}
