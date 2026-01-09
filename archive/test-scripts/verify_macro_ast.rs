use std::io::Cursor;
use zulon_parser::Parser;
use zulon_parser::ast::ExpressionKind;

fn main() {
    let source = r#"
fn test() {
    assert_eq!(2 + 2, 4);
}
"#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().expect("Parse failed");
    
    println!("✅ Parsed successfully!");
    println!("Number of items: {}", ast.items.len());
    
    // Find the function
    if let Some(item) = ast.items.first() {
        if let zulon_parser::ast::ItemKind::Function(func) = &item.kind {
            println!("Function name: {}", func.name.name);
            
            // Check for attributes
            if !func.attributes.is_empty() {
                println!("Attributes: {:?}", func.attributes);
            }
            
            // Check body for macro invocation
            let stmt = &func.body.statements[0];
            if let zulon_parser::ast::StatementKind::Expr(expr) = &stmt.kind {
                match &expr.kind {
                    ExpressionKind::MacroInvocation { macro_name, args, delimiter } => {
                        println!("✅ MACRO INVOCATION FOUND!");
                        println!("  Macro name: {}", macro_name.name);
                        println!("  Arguments: {} args", args.len());
                        println!("  Delimiter: {:?}", delimiter);
                    }
                    _ => {
                        println!("Expression kind: {:?}", expr.kind);
                    }
                }
            }
        }
    }
}
