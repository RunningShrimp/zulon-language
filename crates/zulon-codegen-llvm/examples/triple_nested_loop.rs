use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn main() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 3 {
        let mut j = 0;
        while j < 2 {
            let mut k = 0;
            while k < 2 {
                sum = sum + 1;
                k = k + 1
            };
            j = j + 1
        };
        i = i + 1
    };
    sum
}
"#;

    println!("=== Compiling 3-Level Nested Loop ===\n");
    println!("Expected result: 3 * 2 * 2 = 12\n");

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
    std::fs::write("triple_nested_loop.ll", &llvm_ir)?;
    println!("✅ Generated LLVM IR");

    println!("\n=== Compiling and Running ===");
    let status = std::process::Command::new("llc")
        .arg("triple_nested_loop.ll")
        .arg("-o")
        .arg("triple_nested_loop.s")
        .status()?;

    if !status.success() {
        return Err("llc failed".into());
    }

    let status = std::process::Command::new("clang")
        .arg("triple_nested_loop.s")
        .arg("-o")
        .arg("triple_nested_loop")
        .status()?;

    if !status.success() {
        return Err("clang failed".into());
    }

    let output = std::process::Command::new("./triple_nested_loop")
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);
    println!("Program exited with code: {}", exit_code);

    if exit_code == 12 {
        println!("✅ SUCCESS! 3-level nested loops work correctly!");
    } else {
        println!("❌ Failed! Expected 12, got {}", exit_code);
    }

    Ok(())
}
