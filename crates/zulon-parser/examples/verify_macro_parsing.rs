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
    
    println!("=== Macro Parsing Verification ===\n");
    println!("✅ Parsed successfully!");
    println!("Number of items: {}\n", ast.items.len());
    
    // Find the function
    if let Some(item) = ast.items.first() {
        if let zulon_parser::ast::ItemKind::Function(func) = &item.kind {
            println!("Function name: {}", func.name.name);
            
            // Check for attributes
            if !func.attributes.is_empty() {
                println!("Attributes: {} found", func.attributes.len());
                for attr in &func.attributes {
                    println!("  - #[{}]", attr.name.name);
                }
            }
            
            // Check body for macro invocation
            if !func.body.statements.is_empty() {
                let stmt = &func.body.statements[0];
                if let zulon_parser::ast::StatementKind::Expr(expr) = &stmt.kind {
                    match &expr.kind {
                        ExpressionKind::MacroInvocation { macro_name, args, delimiter } => {
                            println!("\n🎉 MACRO INVOCATION DETECTED!");
                            println!("  Macro name: {}", macro_name.name);
                            println!("  Number of arguments: {}", args.len());
                            println!("  Delimiter: {:?}", delimiter);
                            println!("\n✅ SUCCESS: Macro invocation parsing works correctly!");
                        }
                        _ => {
                            println!("\nExpression kind: {:?}", expr.kind);
                            println!("❌ Expected MacroInvocation");
                        }
                    }
                }
            }
        }
    }
}
