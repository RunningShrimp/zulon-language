use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;
use zulon_mir::lower_hir;
use zulon_lir::lower::LirLoweringContext;

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

    // Print LIR for compute function
    for func in &lir.functions {
        if func.name == "compute" {
            println!("=== LIR Function: {} ===", func.name);
            println!("Parameters: {:?}", func.params);
            println!("\nBlocks:");
            for (block_id, block) in &func.blocks {
                println!("\n{}:", block_id);
                if !block.phi_nodes.is_empty() {
                    println!("  Phi nodes:");
                    for (vreg, phi) in &block.phi_nodes {
                        println!("    %{} = phi {:?}", vreg, phi.sources);
                    }
                }
                println!("  Instructions:");
                for inst in &block.instructions {
                    println!("    {:?}", inst);
                }
                if let Some(terminator) = &block.terminator {
                    println!("  Terminator: {:?}", terminator);
                }
            }
        }
    }

    Ok(())
}
