// Complete compilation pipeline: ZULON Source → LLVM IR
// Demonstrates the full journey from source to executable-ready LLVM IR

use std::io::Cursor;
use zulon_codegen_llvm::CodeGenerator;
use zulon_hir::lower_ast_simple;
use zulon_lir::lower::LirLoweringContext;
use zulon_mir::lower_hir;
use zulon_parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZULON Complete Pipeline: Source → LLVM IR ===\n");

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

fn main() -> i32 {
    let result = compute(15);
    result
}
"#;

    println!("Source Code:");
    println!("{}", source);
    println!("\n{}", "=".repeat(70));

    // Step 1: Parsing (Source → AST)
    println!("\n[Step 1] Lexer + Parser → AST");
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    println!("  ✅ Parsed {} items", ast.items.len());
    for (i, item) in ast.items.iter().enumerate() {
        if let zulon_parser::ast::ItemKind::Function(f) = &item.kind {
            println!("     [{}] fn {}", i, f.name.name);
        }
    }

    // Step 2: HIR Lowering (AST → Typed HIR)
    println!("\n[Step 2] AST → HIR (Typed IR)");
    let hir = lower_ast_simple(&ast)?;
    println!("  ✅ HIR: {} items (type-explicit)", hir.items.len());

    // Step 3: MIR Lowering (HIR → Basic Blocks)
    println!("\n[Step 3] HIR → MIR (Control Flow Explicit)");
    let mir = lower_hir(&hir)?;
    println!("  ✅ MIR: {} functions", mir.functions.len());
    for func in &mir.functions {
        println!(
            "     - fn {} with {} basic blocks",
            func.name,
            func.blocks.len()
        );
    }

    // Step 4: LIR Lowering (MIR → SSA Form)
    println!("\n[Step 4] MIR → LIR (SSA Form)");
    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;
    println!("  ✅ LIR: {} functions (SSA)", lir.functions.len());

    // Step 5: LLVM IR Generation
    println!("\n[Step 5] LIR → LLVM IR (Executable)");
    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = CodeGenerator::new(&mut buffer);

    // Generate module with target triple
    codegen.generate_module(&lir.functions)?;

    let llvm_ir = String::from_utf8(buffer.into_inner())?;

    println!("  ✅ LLVM IR generated ({} bytes)", llvm_ir.len());

    // Display generated LLVM IR
    println!("\n{}", "=".repeat(70));
    println!("\n🎉 Generated LLVM IR:\n");
    println!("{}", llvm_ir);

    println!("\n{}", "=".repeat(70));
    println!("\n✅ COMPLETE PIPELINE SUCCESSFUL!");
    println!("\nCompilation Flow:");
    println!("  1. ✅ Source Code (ZULON)");
    println!("  2. ✅ AST (Parser)");
    println!("  3. ✅ HIR (Typed)");
    println!("  4. ✅ MIR (Basic Blocks)");
    println!("  5. ✅ LIR (SSA)");
    println!("  6. ✅ LLVM IR (Executable)");
    println!("\nNext Steps:");
    println!("  - Save LLVM IR to file: main.ll");
    println!("  - Compile with: llc-18 main.ll -o main.o");
    println!("  - Link with: clang main.o -o main");
    println!("  - Run: ./main");

    Ok(())
}
