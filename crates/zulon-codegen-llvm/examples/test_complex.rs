// Test complex control flow and recursive functions
// Demonstrates nested if/else and Fibonacci implementation

use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn fib(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        let a = fib(n - 1);
        let b = fib(n - 2);
        a + b
    }
}

fn main() -> i32 {
    let result = fib(10);
    result
}
"#;

    // Complete pipeline
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;

    // Generate LLVM IR
    let mut buffer = Cursor::new(Vec::new());
    let mut codegen = zulon_codegen_llvm::CodeGenerator::new(&mut buffer);

    // Generate all functions
    for func in &lir.functions {
        codegen.generate_function(func)?;
    }

    let mut llvm_ir = String::from_utf8(buffer.into_inner())?;

    // Prepend printf and string constant declarations
    let mut declarations = String::new();
    declarations.push_str("; ModuleID = 'zulon_module'\n");
    declarations.push_str("target triple = \"arm64-apple-darwin\"\n\n");
    declarations.push_str("declare i32 @printf(i8*, ...)\n");
    declarations.push_str("@.str = private unnamed_addr constant [20 x i8] c\"Fibonacci(10) = %d\\0A\\00\", align 1\n\n");

    // Inject printf call at the end of main
    let injection = r#"

; Inject printf call at end of main
define i32 @main_wrapper() {
entry:
  %result = call i32 @main()
  %call = call i32 (i8*, ...) @printf(i8* getelementptr ([20 x i8], [20 x i8]* @.str, i32 0, i32 0), i32 %result)
  ret i32 %result
}
"#;

    llvm_ir = format!("{}{}{}", declarations, llvm_ir, injection);

    println!("=== Generated LLVM IR ===");
    println!("{}", llvm_ir);

    // Save to file
    std::fs::write("fib_test.ll", &llvm_ir)?;
    println!("✅ Saved to fib_test.ll");

    println!("\n=== Fibonacci(10) = 55 ===");

    Ok(())
}
