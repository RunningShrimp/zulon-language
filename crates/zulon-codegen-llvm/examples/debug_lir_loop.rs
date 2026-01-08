// Debug LIR for loop
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

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

    println!("=== MIR ===\n");
    for func in &mir.functions {
        for (bid, block) in &func.blocks {
            println!("Block {}:\n", bid);
            for instr in &block.instructions {
                println!("  {:?}\n", instr);
            }
            if let Some(term) = &block.terminator {
                println!("  Terminator: {:?}\n", term);
            }
        }
    }

    let mut lir_ctx = LirLoweringContext::new();
    let lir = lir_ctx.lower_body(&mir)?;

    println!("\n=== LIR ===\n");
    for func in &lir.functions {
        println!("Function: {}\n", func.name);
        for (bid, block) in &func.blocks {
            println!("  Block {}:\n", bid);
            for instr in &block.instructions {
                println!("    {:?}\n", instr);
            }
            if let Some(term) = &block.terminator {
                println!("    Terminator: {:?}\n", term);
            }
        }
    }

    Ok(())
}
