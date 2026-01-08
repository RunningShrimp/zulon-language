use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn main() -> i32 {
    let result = compute(15);
    result
}
"#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;

    for func in &lir.functions {
        if func.name == "main" {
            println!("=== LIR Function: {} ===", func.name);
            for (block_id, block) in &func.blocks {
                println!("\nBlock {}:", block_id);
                for (i, inst) in block.instructions.iter().enumerate() {
                    println!("  [{}] {:?}", i, inst);
                }
                if let Some(terminator) = &block.terminator {
                    println!("  Terminator: {:?}", terminator);
                }
            }
        }
    }

    Ok(())
}
