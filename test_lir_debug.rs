use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn main() -> i32 {
    let mut count = 0;
    while count < 10 {
        count = count + 1
    };
    count
}
"#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;

    println!("LIR Functions: {}", lir.functions.len());
    for func in &lir.functions {
        println!("\nFunction: {}", func.name);
        for (block_id, block) in &func.blocks {
            println!("  Block {}:", block_id);
            for inst in &block.instructions {
                println!("    {:?}", inst);
            }
        }
    }

    Ok(())
}
