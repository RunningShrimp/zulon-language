#!/bin/bash
set -e

# 编译嵌套循环测试
echo "=== Testing Nested Loop ==="

# 创建测试程序
cat > /tmp/test_nested.zl << 'ZLEOF'
fn main() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 5 {
        let mut j = 0;
        while j < 3 {
            sum = sum + 1;
            j = j + 1
        };
        i = i + 1
    };
    sum
}
ZLEOF

# 使用已有的while_loop_example修改源码来测试
cd crates/zulon-codegen-llvm/examples

# 修改while_loop_example.rs来测试嵌套循环
cat > while_loop_example.rs << 'RSEOF'
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
    while i < 5 {
        let mut j = 0;
        while j < 3 {
            sum = sum + 1;
            j = j + 1
        };
        i = i + 1
    };
    sum
}
"#;

    println!("=== Compiling Nested Loop Program ===\n");
    println!("Expected result: 5 * 3 = 15\n");

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
    std::fs::write("while_loop.ll", &llvm_ir)?;
    println!("✅ Generated LLVM IR");

    println!("\n=== Compiling and Running ===");
    let status = std::process::Command::new("llc")
        .arg("while_loop.ll")
        .arg("-o")
        .arg("while_loop.s")
        .status()?;
    
    if !status.success() {
        return Err("llc failed".into());
    }

    let status = std::process::Command::new("clang")
        .arg("while_loop.s")
        .arg("-o")
        .arg("while_loop")
        .status()?;

    if !status.success() {
        return Err("clang failed".into());
    }

    let output = std::process::Command::new("./while_loop")
        .output()?;
    
    let exit_code = output.status.code().unwrap_or(-1);
    println!("Program exited with code: {}", exit_code);

    if exit_code == 15 {
        println!("✅ SUCCESS! Nested loops work correctly!");
    } else {
        println!("❌ Failed! Expected 15, got {}", exit_code);
    }

    Ok(())
}
RSEOF

# 编译并运行
cd /Users/didi/Desktop/zulon-language
cargo run --package zulon-codegen-llvm --example while_loop_example 2>&1 | tail -15
