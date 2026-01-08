use zulon_parser::Parser;

fn main() {
    let source = std::fs::read_to_string("test_attribute_parsing.zl")
        .expect("Failed to read file");

    let mut parser = Parser::from_source(&source);

    match parser.parse() {
        Ok(ast) => {
            println!("✅ Successfully parsed test file!");
            println!("Found {} items", ast.items.len());

            for item in &ast.items {
                if let ast::ItemKind::Function(func) = &item.kind {
                    if !func.attributes.is_empty() {
                        println!("  Function '{}' has {} attributes:",
                            func.name.name, func.attributes.len());
                        for attr in &func.attributes {
                            print!("    #[{}", attr.name.name);
                            if !attr.args.is_empty() {
                                print!("(");
                                for (i, arg) in attr.args.iter().enumerate() {
                                    if i > 0 { print!(", "); }
                                    match arg {
                                        ast::AttributeArg::Ident(ident) => print!("{}", ident.name),
                                        ast::AttributeArg::KeyValue { key, value } => {
                                            print!("{} = \"{}\"", key.name, value);
                                        }
                                        ast::AttributeArg::String(s) => print!("\"{}\"", s),
                                    }
                                }
                                print!(")");
                            }
                            println!("]");
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
