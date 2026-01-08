use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("test_nested_loop.zl")?;

    println!("=== Compiling Nested Loop Program ===\n");

    // Parse and lower through IR pipeline
    let mut parser = Parser::from_source(&source);
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
    std::fs::write("test_nested_loop.ll", &llvm_ir)?;
    println!("✅ Generated LLVM IR - saved to test_nested_loop.ll");
    println!("\n=== Compiling and Running ===");

    // Compile with LLVM
    std::process::Command::new("llc")
        .arg("test_nested_loop.ll")
        .arg("-o")
        .arg("test_nested_loop.s")
        .status()?;

    std::process::Command::new("clang")
        .arg("test_nested_loop.s")
        .arg("-o")
        .arg("test_nested_loop")
        .status()?;

    let output = std::process::Command::new("./test_nested_loop")
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);
    println!("Program exited with code: {}", exit_code);

    if exit_code == 15 {
        println!("✅ SUCCESS! Expected: 5 * 3 = 15");
    } else {
        println!("❌ Failed! Expected 15, got {}", exit_code);
    }

    Ok(())
}
