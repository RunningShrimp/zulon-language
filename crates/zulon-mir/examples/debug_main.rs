use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

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

    for func in &mir.functions {
        if func.name == "main" {
            println!("Function: {}", func.name);
            for (block_id, block) in &func.blocks {
                println!("\nBlock {}:", block_id);
                for (i, inst) in block.instructions.iter().enumerate() {
                    println!("  [{}] {:?}", i, inst);
                }
                if let Some(term) = &block.terminator {
                    println!("  Terminator: {:?}", term);
                }
            }
        }
    }

    Ok(())
}
