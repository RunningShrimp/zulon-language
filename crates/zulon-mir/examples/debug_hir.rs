use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;

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

    println!("=== HIR Debug ===\n");

    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    let hir = lower_ast_simple(&ast)?;

    for item in &hir.items {
        if let zulon_hir::HirItem::Function(func) = item {
            println!("Function: {}", func.name);
            println!("{:#?}", func.body);
        }
    }

    Ok(())
}
