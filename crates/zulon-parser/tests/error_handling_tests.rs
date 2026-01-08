// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for error handling features (throw, ?, | separator)

use zulon_parser::Parser;

/// Test helper to parse source code
fn parse(source: &str) -> zulon_parser::ast::Ast {
    let mut parser = Parser::from_source(source);
    parser.parse().expect("Parsing failed")
}

// ============================================================================
// Throw Statement Tests
// ============================================================================

#[test]
fn test_throw_simple() {
    let source = r#"
        fn f() -> i32 | MyError {
            throw MyError::Invalid;
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert_eq!(func.name.name, "f");
        assert!(func.error_type.is_some());
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_throw_with_value() {
    let source = r#"
        fn f() -> i32 | Error {
            throw Error::Message("something went wrong");
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_throw_complex_expression() {
    let source = r#"
        fn f() -> i32 | Error {
            throw create_error();
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Question Mark Operator Tests
// ============================================================================

#[test]
fn test_question_mark_simple() {
    let source = r#"
        fn f() -> i32 | Error {
            let x = risky_function()?;
            Outcome::Ok(x)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_question_mark_chain() {
    let source = r#"
        fn f() -> i32 | Error {
            let x = func1()?.func2()?;
            Outcome::Ok(x)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_question_mark_in_expression() {
    let source = r#"
        fn f() -> i32 | Error {
            let x = risky_function()? + 1;
            Outcome::Ok(x)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_question_mark_with_match() {
    let source = r#"
        fn f() -> i32 | Error {
            let x = match risky_function()? {
                _ => 42,
            };
            Outcome::Ok(x)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_question_mark_nested() {
    let source = r#"
        fn f() -> i32 | Error {
            let x = func1(func2()?);
            Outcome::Ok(x)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Pipe Separator Tests
// ============================================================================

#[test]
fn test_pipe_separator_simple() {
    let source = r#"
        fn f() -> i32 | MyError {
            Outcome::Ok(42)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert!(func.error_type.is_some());
        assert!(func.effects.is_empty());
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_pipe_separator_with_effects() {
    let source = r#"
        fn f() -> i32 | IoError | IoEffect {
            Outcome::Ok(42)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert!(func.error_type.is_some());
        assert_eq!(func.effects.len(), 1);
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_pipe_separator_multiple_effects() {
    let source = r#"
        fn f() -> Response | HttpError | IoEffect + DatabaseEffect + LogEffect {
            Outcome::Ok(Response::ok())
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert!(func.error_type.is_some());
        assert_eq!(func.effects.len(), 3);
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_pipe_separator_complex_types() {
    let source = r#"
        fn f() -> MyResult | CustomError {
            Outcome::Ok(42)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_pipe_separator_in_generic_function() {
    let source = r#"
        fn f<T>() -> T | Error {
            Outcome::Ok(42)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert!(func.error_type.is_some());
    } else {
        panic!("Expected function");
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_error_handling_complete() {
    let source = r#"
        fn divide(a: i32, b: i32) -> i32 | DivideError {
            if b == 0 {
                throw DivideError::Zero;
            }
            Outcome::Ok(a / b)
        }

        fn calculate() -> i32 | DivideError {
            let x = divide(10, 2)?;
            Outcome::Ok(x * 2)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);

    // Check first function
    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert_eq!(func.name.name, "divide");
        assert!(func.error_type.is_some());
    } else {
        panic!("Expected function");
    }

    // Check second function
    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[1].kind {
        assert_eq!(func.name.name, "calculate");
        assert!(func.error_type.is_some());
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_complex_error_handling() {
    let source = r#"
        fn process() -> Result | IoError | Io + Database {
            let data = read_file()?;
            let result = save_to_database(data)?;
            Outcome::Ok(result)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[0].kind {
        assert_eq!(func.name.name, "process");
        assert!(func.error_type.is_some());
        assert_eq!(func.effects.len(), 2);
    } else {
        panic!("Expected function");
    }
}
