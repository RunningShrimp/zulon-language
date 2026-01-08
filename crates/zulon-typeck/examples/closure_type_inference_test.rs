// Test Closure Type Inference
//
// This example demonstrates and tests closure type inference.

use zulon_parser::Parser;
use zulon_typeck::TypeChecker;

fn main() {
    println!("=== ZULON Closure Type Inference Test ===\n");

    // Test 1: Simple closure without type annotations
    println!("Test 1: Simple closure with type inference");
    test_simple_closure_inference();

    // Test 2: Closure with explicit types
    println!("\nTest 2: Closure with explicit type annotations");
    test_typed_closure();

    // Test 3: Closure with return type annotation
    println!("\nTest 3: Closure with return type annotation");
    test_closure_with_return_type();

    // Test 4: Closure capturing outer variable
    println!("\nTest 4: Closure capturing outer variable");
    test_closure_with_capture();

    // Test 5: Immediate closure invocation
    println!("\nTest 5: Immediate closure invocation");
    test_immediate_closure_invocation();

    println!("\n=== All Closure Type Inference Tests Complete ===");
}

fn test_simple_closure_inference() {
    let source = r#"
        fn main() {
            let add = |x, y| x + y;
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check(&ast) {
        Ok(()) => println!("  ✅ Type checking passed!"),
        Err(e) => println!("  ❌ Type error: {}", e),
    }
}

fn test_typed_closure() {
    let source = r#"
        fn main() {
            let add = |x: i32, y: i32| -> i32 { x + y };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check(&ast) {
        Ok(()) => println!("  ✅ Type checking passed!"),
        Err(e) => println!("  ❌ Type error: {}", e),
    }
}

fn test_closure_with_return_type() {
    let source = r#"
        fn main() {
            let square = |x: i32| -> i32 {
                let result = x * x;
                result
            };
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check(&ast) {
        Ok(()) => println!("  ✅ Type checking passed!"),
        Err(e) => println!("  ❌ Type error: {}", e),
    }
}

fn test_closure_with_capture() {
    let source = r#"
        fn main() {
            let x = 10;
            let add_x = |y| x + y;
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check(&ast) {
        Ok(()) => println!("  ✅ Type checking passed!"),
        Err(e) => println!("  ❌ Type error: {}", e),
    }
}

fn test_immediate_closure_invocation() {
    let source = r#"
        fn main() {
            let result = (|a: i32, b: i32| -> i32 { a + b })(10, 20);
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ❌ Parse error: {}", e);
            return;
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check(&ast) {
        Ok(()) => println!("  ✅ Type checking passed!"),
        Err(e) => println!("  ❌ Type error: {}", e),
    }
}
