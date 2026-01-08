use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
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

    for func in &mir.functions {
        if func.name == "compute" {
            println!("MIR function: {}", func.name);
            println!("Params: {:?}", func.params);
            for (bid, block) in &func.blocks {
                println!("Block {}:", bid);
                for inst in &block.instructions {
                    println!("  {:?}", inst);
                }
                if let Some(term) = &block.terminator {
                    println!("  Term: {:?}", term);
                }
            }
        }
    }

    Ok(())
}
