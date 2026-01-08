use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test multiple mutable variables in the same loop
    let source = r#"
fn main() -> i32 {
    let mut sum = 0;
    let mut count = 0;
    let mut product = 1;

    let mut i = 0;
    while i < 5 {
        sum = sum + i;
        count = count + 1;
        product = product * 2;
        i = i + 1
    };

    sum + count + product
}
"#;

    println!("=== Testing Multiple Mutable Variables ===\n");
    println!("Expected: sum=0+1+2+3+4=10, count=5, product=2^5=32");
    println!("Total: 10 + 5 + 32 = 47\n");

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;
    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;

    println!("✅ All IR lowerings successful");

    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = CodeGenerator::new(&mut buffer);

    for func in &lir.functions {
        codegen.generate_function(func)?;
    }

    let llvm_ir = String::from_utf8(buffer.into_inner())?;
    std::fs::write("multi_vars_loop.ll", &llvm_ir)?;
    println!("✅ Generated LLVM IR");

    println!("\n=== Compiling and Running ===");
    let status = std::process::Command::new("llc")
        .arg("multi_vars_loop.ll")
        .arg("-o")
        .arg("multi_vars_loop.s")
        .status()?;

    if !status.success() {
        return Err("llc failed".into());
    }

    let status = std::process::Command::new("clang")
        .arg("multi_vars_loop.s")
        .arg("-o")
        .arg("multi_vars_loop")
        .status()?;

    if !status.success() {
        return Err("clang failed".into());
    }

    let output = std::process::Command::new("./multi_vars_loop")
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);
    println!("Program exited with code: {}", exit_code);

    if exit_code == 47 {
        println!("✅ SUCCESS! Multiple mutable variables work correctly!");
    } else {
        println!("❌ Failed! Expected 47, got {}", exit_code);
    }

    Ok(())
}
