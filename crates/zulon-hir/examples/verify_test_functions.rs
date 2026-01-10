// Verify HIR test function detection
use zulon_parser::Parser;
use zulon_hir::lower_ast_simple;

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

fn normal_function() -> i32 {
    42
}
"#;

    println!("🔍 Parsing test source...");
    let mut parser = Parser::from_source(source);

    match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsed {} items\n", ast.items.len());

            // Lower to HIR
            println!("📝 Lowering to HIR...");
            match lower_ast_simple(&ast) {
                Ok(hir) => {
                    println!("✅ HIR created with {} items\n", hir.items.len());

                    // Find test functions
                    let tests = zulon_hir::hir::HirFunction::filter_tests(&hir.items);

                    println!("🧪 Found {} test functions:\n", tests.len());

                    for test in &tests {
                        println!("  - {} (ignored: {})",
                            test.name,
                            test.is_ignored_test()
                        );
                    }

                    // Verify results
                    println!("\n✅ Verification complete!");
                    println!("   - test_addition detected: {}",
                        tests.iter().any(|t| t.name == "test_addition"));
                    println!("   - test_slow detected and ignored: {}",
                        tests.iter().any(|t| t.name == "test_slow" && t.is_ignored_test()));
                    println!("   - normal_function NOT in tests: {}",
                        !tests.iter().any(|t| t.name == "normal_function"));
                }
                Err(e) => {
                    eprintln!("❌ HIR lowering error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
