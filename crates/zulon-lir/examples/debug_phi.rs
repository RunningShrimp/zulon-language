use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;
use zulon_codegen_llvm::CodeGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
"#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    // Get compute function
    for func in &mir.functions {
        if func.name == "compute" {
            println!("=== MIR Function: {} ===", func.name);
            println!("\nBlock predecessors:");
            println!("  Block 0 (entry)");
            println!("  Block 1 (then) <- from block 0");
            println!("  Block 2 (else) <- from block 0");
            println!("  Block 3 (join) <- from block 1 and block 2");

            println!("\nBlock return values (should be):");
            println!("  Block 1: Temp(5) from Call");
            println!("  Block 2: Temp(8) from Call");
            println!("  Block 3: should have Phi with both Temp(5) and Temp(8)");
        }
    }

    Ok(())
}
