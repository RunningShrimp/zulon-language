// Test closure parsing
use zulon_parser::Parser;

#[test]
fn test_simple_closure() {
    let source = r#"
        fn main() {
            let add = |x, y| x + y;
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    assert_eq!(ast.items.len(), 1);

    // Check that we can parse closures without errors
    println!("Parsed simple closure successfully");
}

#[test]
fn test_closure_with_types() {
    let source = r#"
        fn main() {
            let add = |x: i32, y: i32| -> i32 { x + y };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    assert_eq!(ast.items.len(), 1);
    println!("Parsed typed closure successfully");
}

#[test]
fn test_closure_with_block_body() {
    let source = r#"
        fn main() {
            let complex = |x| {
                let y = x * 2;
                y + 10
            };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    assert_eq!(ast.items.len(), 1);
    println!("Parsed block closure successfully");
}

#[test]
fn test_empty_closure_limitation() {
    // Empty closures || expr are NOT supported due to || ambiguity
    // Users must use: fn() instead or |_dummy| as workaround
    let source = r#"
        fn main() {
            // This fails - || is parsed as logical OR
            let get_value = || 42;
        }
    "#;

    let mut parser = Parser::from_source(source);
    let result = parser.parse();

    // Expected to fail - this is a known limitation
    assert!(result.is_err());
    println!("Confirmed empty closure limitation - users should use fn() or |_dummy|");
}

#[test]
fn test_immediate_closure_invocation() {
    let source = r#"
        fn main() {
            let result = (|a, b| a + b)(10, 20);
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    assert_eq!(ast.items.len(), 1);
    println!("Parsed immediate closure invocation successfully");
}

#[test]
fn test_nested_closures() {
    let source = r#"
        fn main() {
            let outer = |x| {
                let inner = |y| x + y;
                inner(10)
            };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    assert_eq!(ast.items.len(), 1);
    println!("Parsed nested closures successfully");
}
