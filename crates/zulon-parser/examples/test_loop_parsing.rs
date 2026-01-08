// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test loop parsing functionality

use zulon_parser::{Parser, ast::*};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing ZULON Loop Parsing Functionality\n");
    println!("{}", "=".repeat(60));

    // Test file
    let source = fs::read_to_string("test_loops_current.zl")?;

    let mut parser = Parser::from_source(&source);

    match parser.parse() {
        Ok(ast) => {
            println!("\n✅ Parse successful!");
            println!("   Total items: {}", ast.items.len());

            // Analyze the parsed AST
            for (i, item) in ast.items.iter().enumerate() {
                match &item.kind {
                    ItemKind::Function(func) => {
                        println!("\n📝 Function {}: {}", i + 1, func.name.name);
                        analyze_function_for_loops(func);
                    }
                    _ => {}
                }
            }

            println!("\n{}", "=".repeat(60));
            println!("✅ All tests parsed successfully!");
        }
        Err(e) => {
            println!("\n❌ Parse error:");
            println!("   {}", e);
            println!("\n{}", "=".repeat(60));
            return Err(e.into());
        }
    }

    Ok(())
}

/// Analyze a function for loop constructs
fn analyze_function_for_loops(func: &Function) {
    let mut loop_count = 0;
    let mut while_count = 0;
    let mut for_count = 0;
    let mut break_count = 0;
    let mut continue_count = 0;
    let mut mut_var_count = 0;

    analyze_block(&func.body, &mut loop_count, &mut while_count, &mut for_count,
                  &mut break_count, &mut continue_count, &mut mut_var_count);

    println!("   ├── Loops: {}", loop_count);
    println!("   ├── While: {}", while_count);
    println!("   ├── For: {}", for_count);
    println!("   ├── Break: {}", break_count);
    println!("   ├── Continue: {}", continue_count);
    println!("   └── Mutable vars: {}", mut_var_count);
}

/// Recursively analyze a block
fn analyze_block(
    block: &Block,
    loop_count: &mut usize,
    while_count: &mut usize,
    for_count: &mut usize,
    break_count: &mut usize,
    continue_count: &mut usize,
    mut_var_count: &mut usize,
) {
    for stmt in &block.statements {
        match &stmt.kind {
            StatementKind::Local(local) => {
                if local.is_mutable {
                    *mut_var_count += 1;
                }
            }
            StatementKind::Expr(expr) => {
                analyze_expression(expr, loop_count, while_count, for_count,
                                  break_count, continue_count, mut_var_count);
            }
            _ => {}
        }
    }

    if let Some(expr) = &block.trailing_expr {
        analyze_expression(expr, loop_count, while_count, for_count,
                          break_count, continue_count, mut_var_count);
    }
}

/// Recursively analyze an expression
fn analyze_expression(
    expr: &Expression,
    loop_count: &mut usize,
    while_count: &mut usize,
    for_count: &mut usize,
    break_count: &mut usize,
    continue_count: &mut usize,
    mut_var_count: &mut usize,
) {
    match &expr.kind {
        ExpressionKind::Loop(_, _) => {
            *loop_count += 1;
        }
        ExpressionKind::While(_, _, _) => {
            *while_count += 1;
        }
        ExpressionKind::For(_, _, _, _) => {
            *for_count += 1;
        }
        ExpressionKind::Break(_) => {
            *break_count += 1;
        }
        ExpressionKind::Continue(_) => {
            *continue_count += 1;
        }
        ExpressionKind::Block(block) => {
            analyze_block(block, loop_count, while_count, for_count,
                         break_count, continue_count, mut_var_count);
        }
        ExpressionKind::If(_, then_block, else_block) => {
            analyze_block(then_block, loop_count, while_count, for_count,
                         break_count, continue_count, mut_var_count);
            if let Some(else_blk) = else_block {
                analyze_block(else_blk, loop_count, while_count, for_count,
                             break_count, continue_count, mut_var_count);
            }
        }
        _ => {
            // For other expression types, we'd need to recursively traverse
            // but for now, let's just count what we find at the top level
        }
    }
}
