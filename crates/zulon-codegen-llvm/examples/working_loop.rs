// Working loop example - simple counter with return
use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn main() -> i32 {
    let mut count = 0;
    loop {
        if count >= 10 {
            return count
        };
        count = count + 1
    };
    0
}
"#;

    println!("=== Compiling Working Loop Program ===\n");
    println!("Source:\n{}\n", source);

    // Parse and lower through IR pipeline
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    println!("✅ Parsed successfully");

    let hir = lower_ast_simple(&ast)?;
    println!("✅ HIR lowering successful");

    let mir = lower_hir(&hir)?;
    println!("✅ MIR lowering successful");

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;
    println!("✅ LIR lowering successful - {} functions", lir.functions.len());

    // Generate LLVM IR
    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = CodeGenerator::new(&mut buffer);

    for func in &lir.functions {
        codegen.generate_function(func)?;
    }

    let llvm_ir = String::from_utf8(buffer.into_inner())?;

    // Save to file
    std::fs::write("working_loop.ll", &llvm_ir)?;
    println!("✅ Generated LLVM IR - saved to working_loop.ll");
    println!("\n{} bytes of LLVM IR generated", llvm_ir.len());

    println!("\n=== Next Steps ===");
    println!("1. Compile: llc working_loop.ll -o working_loop.s");
    println!("2. Assemble: clang working_loop.s -o working_loop");
    println!("3. Run: ./working_loop");
    println!("4. Expected exit code: 10");

    Ok(())
}
