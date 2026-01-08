use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

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

    // Print compute function details
    for func in &mir.functions {
        if func.name == "compute" {
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
