use std::io::Cursor;
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn main() -> i32 {
    let mut count = 0;
    loop {
        if count >= 10 {
            return count
        };
        count = count + 1
    };
    0
}
"#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    println!("MIR Functions:");
    for func in &mir.functions {
        println!("\nFunction: {}", func.name);
        println!("  Blocks: {}", func.blocks.len());
        
        for (block_id, block) in &func.blocks {
            println!("\n  Block {}:", block_id);
            println!("    Instructions: {}", block.instructions.len());
            println!("    Terminator: {:?}", block.terminator);
        }
    }

    Ok(())
}
