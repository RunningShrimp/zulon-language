use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

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

    println!("=== Nested Loop MIR Debug ===\n");

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;
    let mir = lower_hir(&hir)?;

    for func in &mir.functions {
        println!("Function: {}", func.name);
        for (block_id, block) in &func.blocks {
            println!("  Block {}:", block_id);
            for (i, inst) in block.instructions.iter().enumerate() {
                println!("    [{}] {:?}", i, inst);
            }
            if let Some(terminator) = &block.terminator {
                println!("    Terminator: {:?}", terminator);
            }
        }
    }

    Ok(())
}
