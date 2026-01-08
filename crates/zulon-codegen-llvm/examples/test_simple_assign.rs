// Simple test of assignment
use zulon_parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test 1: Multiple let statements
    println!("Test 1: Multiple let statements");
    let source1 = r#"
fn main() -> i32 {
    let x = 0;
    let y = 5;
    x + y
}
"#;
    test_parse(source1);

    // Test 2: Assignment
    println!("\nTest 2: Assignment");
    let source2 = std::fs::read_to_string("test_assign_debug.zl")?;
    test_parse(&source2);

    Ok(())
}

fn test_parse(source: &str) -> bool {
    let mut parser = Parser::from_source(source);
    match parser.parse() {
        Ok(ast) => {
            println!("  ✅ Parsed {} items", ast.items.len());
            true
        }
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            false
        }
    }
}
