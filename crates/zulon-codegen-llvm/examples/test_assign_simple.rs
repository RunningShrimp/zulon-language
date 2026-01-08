// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
// Simple assignment test
fn main() -> i32 {
    let mut x = 5;
    x = 10;
    x
}
"#;

    println!("=== Compiling Simple Assignment Test ===\n");
    println!("Source:\n{}\n", source);

    // Parse
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    println!("✅ Parsed successfully\n");

    // HIR
    let hir = lower_ast_simple(&ast)?;
    println!("✅ HIR lowering successful\n");
    println!("HIR items: {}\n", hir.items.len());

    for item in &hir.items {
        if let zulon_hir::HirItem::Function(func) = item {
            println!("Function: {}\n", func.name);
            println!("  Params: {}\n", func.params.len());
            println!("  Body:\n");
            debug_print_block(&func.body, 2);
        }
    }

    // MIR
    let mir = lower_hir(&hir)?;
    println!("✅ MIR lowering successful\n");
    println!("MIR functions: {}\n", mir.functions.len());

    for func in &mir.functions {
        println!("MIR Function: {}\n", func.name);
        for (block_id, block) in &func.blocks {
            println!("  Block {}:\n", block_id);
            for instr in &block.instructions {
                println!("    {:?}\n", instr);
            }
            if let Some(term) = &block.terminator {
                println!("    Terminator: {:?}\n", term);
            }
        }
    }

    // LIR
    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;
    println!("✅ LIR lowering successful - {} functions\n\n", lir.functions.len());

    // LLVM Codegen
    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = CodeGenerator::new(&mut buffer);

    for func in &lir.functions {
        codegen.generate_function(func)?;
    }

    let llvm_ir = String::from_utf8(buffer.into_inner())?;

    std::fs::write("test_assign_simple.ll", &llvm_ir)?;
    println!("✅ Generated LLVM IR - saved to test_assign_simple.ll\n");
    println!("{} bytes of LLVM IR generated\n\n", llvm_ir.len());

    println!("=== Next Steps ===");
    println!("1. Compile: llc test_assign_simple.ll -o test_assign_simple.s");
    println!("2. Assemble: clang test_assign_simple.s -o test_assign_simple");
    println!("3. Run: ./test_assign_simple");
    println!("4. Expected exit code: 10");

    Ok(())
}

fn debug_print_block(block: &zulon_hir::HirBlock, indent: usize) {
    let indent_str = " ".repeat(indent);

    for stmt in &block.statements {
        match stmt {
            zulon_hir::HirStatement::Local(local) => {
                println!("{}Local: {} (type: {:?})\n", indent_str, local.name, local.ty);
                if let Some(init) = &local.init {
                    println!("{}  Init: {:?}\n", indent_str, init);
                }
            }
            _ => {
                println!("{}Other statement: {:?}\n", indent_str, stmt);
            }
        }
    }

    if let Some(expr) = &block.trailing_expr {
        println!("{}Expr: {:?}\n", indent_str, expr);
    }
}
