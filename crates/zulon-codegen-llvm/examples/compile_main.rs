// Generate LLVM IR and save to file for compilation

use std::io::Cursor;
use std::fs::write;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZULON Compiler: Source → LLVM IR File ===\n");

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

    // Complete pipeline
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    println!("✅ Parsed {} functions", ast.items.len());

    let hir = lower_ast_simple(&ast)?;
    println!("✅ Lowered to HIR");

    let mir = lower_hir(&hir)?;
    println!("✅ Lowered to MIR ({} functions)", mir.functions.len());

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;
    println!("✅ Lowered to LIR (SSA form)");

    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = CodeGenerator::new(&mut buffer);

    for func in &lir.functions {
        codegen.generate_function(func)?;
    }

    let llvm_ir = String::from_utf8(buffer.into_inner())?;
    println!("✅ Generated LLVM IR ({} bytes)", llvm_ir.len());

    // Save to file
    write("output.ll", &llvm_ir)?;
    println!("✅ Saved to output.ll");

    println!("\n=== Generated LLVM IR ===");
    println!("{}", llvm_ir);

    println!("\n=== Next Steps ===");
    println!("1. Compile to assembly:");
    println!("   llc output.ll -o output.s");
    println!("\n2. Assemble to object:");
    println!("   clang -c output.s -o output.o");
    println!("\n3. Link to executable:");
    println!("   clang output.o -o zulon_program");
    println!("\n4. Run:");
    println!("   ./zulon_program");
    println!("\nExpected result: 20 (15 > 10, so compute(15) = add(15, 5) = 20)");

    Ok(())
}
