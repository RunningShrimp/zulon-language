use zulon_parser::Parser;

fn main() {
    let source = r#"
#[test]
fn test_addition() {
    let result = 2 + 2;
}

#[test]
#[ignore]
fn test_slow() {
    let sum = 0;
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_panic() {
    let arr = [1, 2, 3];
    let val = arr[10];
}
"#;

    let mut parser = Parser::from_source(source);

    match parser.parse() {
        Ok(ast) => {
            println!("✅ Successfully parsed test file!");
            println!("Found {} items\n", ast.items.len());

            for item in &ast.items {
                if let zulon_parser::ast::ItemKind::Function(func) = &item.kind {
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
                                        zulon_parser::ast::AttributeArg::Ident(ident) => {
                                            print!("{}", ident.name)
                                        }
                                        zulon_parser::ast::AttributeArg::KeyValue { key, value } => {
                                            print!("{} = \"{}\"", key.name, value)
                                        }
                                        zulon_parser::ast::AttributeArg::String(s) => {
                                            print!("\"{}\"", s)
                                        }
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
