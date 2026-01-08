// Debug AST for loop
use zulon_parser::Parser;

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

    println!("=== AST ===\n");
    for item in &ast.items {
        if let zulon_parser::ast::ItemKind::Function(func) = &item.kind {
            println!("Function: {}\n", func.name.name);
            print_ast_block(&func.body, 0);
        }
    }

    Ok(())
}

fn print_ast_block(block: &zulon_parser::ast::Block, indent: usize) {
    let indent_str = "  ".repeat(indent);

    println!("{}Block ({} statements):\n", indent_str, block.statements.len());

    for (idx, stmt) in block.statements.iter().enumerate() {
        println!("{}  [{}]: {:?}\n", indent_str, idx, stmt.kind);
    }

    if let Some(expr) = &block.trailing_expr {
        println!("{}  Trailing: {:?}\n", indent_str, expr.kind);
    }
}
