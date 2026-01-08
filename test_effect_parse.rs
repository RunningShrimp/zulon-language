// Test program to verify effect parsing
// Run with: cargo run --bin test_effect_parse

use zulon_parser::Parser;

fn main() {
    let source = r#"
        effect IO {
            read_line() -> str
            print_line(line: str)
        }

        fn greet_user() -> str ! IO {
            let name = IO::read_line();
            IO::print_line(name);
            return name;
        }
    "#;

    let mut parser = Parser::from_source(source);
    match parser.parse() {
        Ok(ast) => {
            println!("✅ Successfully parsed effect declaration!");
            println!("Found {} items", ast.items.len());

            for item in &ast.items {
                match &item.kind {
                    zulon_parser::ast::ItemKind::Effect(effect) => {
                        println!("  - Effect: {}", effect.name.name);
                        println!("    Operations: {}", effect.operations.len());
                        for op in &effect.operations {
                            println!("      - {}", op.name.name);
                        }
                    }
                    zulon_parser::ast::ItemKind::Function(func) => {
                        println!("  - Function: {}", func.name.name);
                        println!("    Effects: {}", func.effects.len());
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
        }
    }
}
