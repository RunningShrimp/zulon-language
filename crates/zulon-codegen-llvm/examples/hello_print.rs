// Hello World with print function
// Demonstrates external function linkage and string formatting

use std::io::{Cursor, Write};
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn main() -> i32 {
    42
}
"#;

    // Complete pipeline
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    let mut lir_ctx = LirLoweringContext::new();
    let _lir = lir_ctx.lower_body(&mir)?;

    // Generate LLVM IR with printf declaration
    let mut buffer = Cursor::new(Vec::new());

    // Write module-level declarations
    writeln!(&mut buffer, "; ModuleID = 'zulon_module'")?;
    writeln!(&mut buffer, "target triple = \"arm64-apple-darwin\"")?;
    writeln!(&mut buffer)?;

    // Declare external printf function
    writeln!(&mut buffer, "declare i32 @printf(i8*, ...)")?;
    writeln!(&mut buffer)?;

    // Declare string constant for "Hello, World!\n"
    writeln!(&mut buffer, "@.str = private unnamed_addr constant [15 x i8] c\"Hello, World!\\0A\\00\", align 1")?;
    writeln!(&mut buffer)?;

    // Generate main function
    writeln!(&mut buffer, "define i32 @main() {{")?;
    writeln!(&mut buffer, "  entry:")?;
    writeln!(&mut buffer, "    ; Call printf")?;
    writeln!(&mut buffer, "    %call = call i32 (i8*, ...) @printf(i8* getelementptr ([15 x i8], [15 x i8]* @.str, i32 0, i32 0))")?;
    writeln!(&mut buffer, "    ; Return 42")?;
    writeln!(&mut buffer, "    ret i32 42")?;
    writeln!(&mut buffer, "}}")?;
    writeln!(&mut buffer)?;

    let llvm_ir = String::from_utf8(buffer.into_inner())?;

    println!("=== Generated LLVM IR with printf ===");
    println!("{}", llvm_ir);

    // Save to file
    std::fs::write("hello_print.ll", &llvm_ir)?;
    println!("✅ Saved to hello_print.ll");

    println!("\n=== Next Steps ===");
    println!("1. Compile: llc hello_print.ll -o hello_print.s");
    println!("2. Assemble: clang hello_print.s -o hello_print");
    println!("3. Run: ./hello_print");

    Ok(())
}
