// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for async/await syntax parsing

use zulon_parser::{Lexer, Parser};

#[test]
fn test_async_function_parsing() {
    let source = r#"
        async fn hello() {
            println("Hello, async world!");
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");
    assert!(!tokens.is_empty(), "Should have tokens");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");

    // Check that the first item is a function with is_async = true
    let func = &ast.items[0];
    match &func.kind {
        zulon_parser::ItemKind::Function(f) => {
            assert!(f.is_async, "Function should be async");
            assert_eq!(f.name.name, "hello");
        }
        _ => panic!("Expected function item"),
    }
}

#[test]
fn test_regular_function_parsing() {
    let source = r#"
        fn hello() {
            println("Hello, world!");
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");

    // Check that the function is NOT async
    let func = &ast.items[0];
    match &func.kind {
        zulon_parser::ItemKind::Function(f) => {
            assert!(!f.is_async, "Regular function should not be async");
            assert_eq!(f.name.name, "hello");
        }
        _ => panic!("Expected function item"),
    }
}

#[test]
fn test_async_function_with_return_type() {
    let source = r#"
        async fn fetch_data() -> i32 {
            42
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");

    let func = &ast.items[0];
    match &func.kind {
        zulon_parser::ItemKind::Function(f) => {
            assert!(f.is_async, "Function should be async");
            assert_eq!(f.name.name, "fetch_data");
            assert!(f.return_type.is_some(), "Should have return type");
        }
        _ => panic!("Expected function item"),
    }
}

#[test]
fn test_async_function_with_params() {
    let source = r#"
        async fn add(x: i32, y: i32) -> i32 {
            x + y
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    let func = &ast.items[0];
    match &func.kind {
        zulon_parser::ItemKind::Function(f) => {
            assert!(f.is_async, "Function should be async");
            assert_eq!(f.name.name, "add");
            assert_eq!(f.params.len(), 2, "Should have 2 parameters");
        }
        _ => panic!("Expected function item"),
    }
}

#[test]
fn test_await_expression() {
    let source = r#"
        fn test() {
            let result = future.await;
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");
}

#[test]
fn test_await_expression_with_method_call() {
    let source = r#"
        fn test() {
            let result = some_future().await;
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");
}

#[test]
fn test_chained_await() {
    let source = r#"
        fn test() {
            let result = future1.await.future2.await;
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");
}

#[test]
fn test_async_function_with_await() {
    let source = r#"
        async fn example() {
            let result = fetch_data().await;
            println(result);
        }
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    assert!(errors.is_empty(), "Lexer should have no errors");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let ast = result.unwrap();

    assert!(!ast.items.is_empty(), "Should have at least one item");

    let func = &ast.items[0];
    match &func.kind {
        zulon_parser::ItemKind::Function(f) => {
            assert!(f.is_async, "Function should be async");
            assert_eq!(f.name.name, "example");
        }
        _ => panic!("Expected function item"),
    }
}
