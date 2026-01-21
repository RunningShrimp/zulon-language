// Complete compilation pipeline test
// Lexer → Parser → HIR → MIR → LIR

use zulon_hir::lower_ast_simple;
use zulon_lir::lower::LirLoweringContext;
use zulon_mir::lower_hir;
use zulon_parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZULON Complete Compilation Pipeline Test ===\n");

    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn compute(x: i32) -> i32 {
    if x > 10 {
        add(x, 5)
    } else {
        add(x, 10)
    }
}

fn main() {
    let result = compute(15);
}
"#;

    println!("Source code:");
    println!("{}", source);
    println!("\n{}", "=".repeat(60));

    // Step 1: Parsing (AST)
    println!("Step 1: Lexer + Parser → AST");
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;

    println!("  ✅ AST generated: {} items", ast.items.len());
    for (i, item) in ast.items.iter().enumerate() {
        if let zulon_parser::ast::ItemKind::Function(f) = &item.kind {
            println!("     [{}] fn {}", i, f.name.name);
        }
    }

    // Step 2: HIR Lowering
    println!("\nStep 2: AST → HIR (High-level IR)");
    let hir = lower_ast_simple(&ast)?;

    println!("  ✅ HIR generated: {} items", hir.items.len());
    for (i, item) in hir.items.iter().enumerate() {
        if let zulon_hir::HirItem::Function(f) = item {
            println!("     [{}] fn {} -> {}", i, f.name, f.return_type);
        }
    }

    // Step 3: MIR Lowering
    println!("\nStep 3: HIR → MIR (Mid-level IR)");
    let mir = lower_hir(&hir)?;

    println!("  ✅ MIR generated: {} functions", mir.functions.len());
    for (i, func) in mir.functions.iter().enumerate() {
        println!(
            "     [{}] fn {}({}) -> {}",
            i,
            func.name,
            func.params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.ty.display_name()))
                .collect::<Vec<_>>()
                .join(", "),
            func.return_type.display_name()
        );
        println!("         {} basic blocks", func.blocks.len());
    }

    // Step 4: LIR Lowering
    println!("\nStep 4: MIR → LIR (Low-level IR)");
    let mut ctx = LirLoweringContext::new();
    let lir = ctx.lower_body(&mir)?;

    println!("  ✅ LIR generated: {} functions", lir.functions.len());
    for (i, func) in lir.functions.iter().enumerate() {
        println!(
            "     [{}] fn {} -> {}",
            i,
            func.name,
            func.return_type.display_name()
        );
        println!("         {} basic blocks (SSA form)", func.blocks.len());
    }

    println!("\n{}", "=".repeat(60));
    println!("\n🎉 COMPLETE PIPELINE WORKING!");
    println!("\nCompilation Stages:");
    println!("  1. ✅ Lexer + Parser (Source → AST)");
    println!("  2. ✅ HIR Lowering (AST → Typed HIR)");
    println!("  3. ✅ MIR Lowering (HIR → Basic Blocks)");
    println!("  4. ✅ LIR Lowering (MIR → Optimized LIR)");
    println!("\nNext Steps:");
    println!("  5. Code Generation (LIR → LLVM IR)");
    println!("  6. Object Generation (LLVM IR → Machine Code)");
    println!("  7. Linking (Object Files → Executable)");

    Ok(())
}
