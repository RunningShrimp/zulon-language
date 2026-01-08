// Debug HIR for loop
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;

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

    println!("=== HIR ===\n");
    for item in &hir.items {
        if let zulon_hir::HirItem::Function(func) = item {
            println!("Function: {}\n", func.name);
            print_hir_block(&func.body, 0);
        }
    }

    Ok(())
}

fn print_hir_block(block: &zulon_hir::HirBlock, indent: usize) {
    let indent_str = "  ".repeat(indent);

    for stmt in &block.statements {
        match stmt {
            zulon_hir::HirStatement::Local(local) => {
                println!("{}Local: {}\n", indent_str, local.name);
                if let Some(init) = &local.init {
                    println!("{}  Init: {:?}\n", indent_str, init);
                }
            }
            zulon_hir::HirStatement::Expression(expr) => {
                println!("{}Expr: {:?}\n", indent_str, expr);
            }
            zulon_hir::HirStatement::Semi(expr) => {
                println!("{}Semi: {:?}\n", indent_str, expr);
            }
            zulon_hir::HirStatement::Item(_) => {
                println!("{}Item\n", indent_str);
            }
        }
    }

    if let Some(expr) = &block.trailing_expr {
        println!("{}Trailing: {:?}\n", indent_str, expr);
    }
}
