// Test mutable variable parsing
use zulon_parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test 1: Simple mutable variable
    println!("Test 1: let mut x = 0");
    let source1 = r#"
fn main() -> i32 {
    let mut x = 0;
    x
}
"#;

    match test_parse(source1) {
        Ok(_) => println!("  ✅ PASS\n"),
        Err(e) => println!("  ❌ FAIL - {}\n", e),
    }

    // Test 2: Mutable variable with reassignment
    println!("Test 2: x = x + 1");
    let source2 = r#"
fn main() -> i32 {
    let mut x = 0;
    x = x + 1;
    x
}
"#;

    match test_parse(source2) {
        Ok(_) => println!("  ✅ PASS\n"),
        Err(e) => println!("  ❌ FAIL - {}\n", e),
    }

    // Test 3: While loop with mutation
    println!("Test 3: while x < 10");
    let source3 = r#"
fn main() -> i32 {
    let mut x = 0;
    while x < 10 {
        x = x + 1
    };
    x
}
"#;

    match test_parse(source3) {
        Ok(_) => println!("  ✅ PASS\n"),
        Err(e) => println!("  ❌ FAIL - {}\n", e),
    }

    Ok(())
}

fn test_parse(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_source(source);
    let ast = parser.parse()?;
    println!("  Parsed {} items", ast.items.len());
    Ok(())
}
