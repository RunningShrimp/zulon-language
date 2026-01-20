// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for const, static, module, use, and extern_crate type checking
//!
//! These tests verify that top-level declarations are properly type-checked.

use zulon_parser::Parser;

#[test]
fn test_const_definition() {
    let source = r#"
        const MAX_SIZE: usize = 100;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Const definition should succeed");
}

#[test]
fn test_const_with_expression() {
    let source = r#"
        const MAX_SIZE: usize = 100 + 50;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Const with expression should succeed");
}

#[test]
fn test_const_type_mismatch() {
    let source = r#"
        const INVALID: i32 = "string";
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_err(), "Const type mismatch should fail");
}

#[test]
fn test_static_definition() {
    let source = r#"
        static COUNTER: i32 = 0;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Static definition should succeed");
}

#[test]
fn test_mutable_static() {
    let source = r#"
        static mut COUNTER: i32 = 0;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Mutable static should succeed");
}

#[test]
fn test_static_type_mismatch() {
    let source = r#"
        static INVALID: i32 = "string";
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_err(), "Static type mismatch should fail");
}

#[test]
fn test_module_inline() {
    let source = r#"
        mod inline_module {
            fn helper() -> i32 {
                42
            }
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Inline module should succeed");
}

#[test]
fn test_use_simple() {
    let source = r#"
        use std::vec;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Simple use should succeed");
}

#[test]
fn test_use_with_alias() {
    let source = r#"
        use std::vec as VecExt;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Use with alias should succeed");
}

#[test]
fn test_use_glob() {
    let source = r#"
        use std::vec::*;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Glob use should succeed");
}

#[test]
fn test_use_list() {
    let source = r#"
        use std::{vec, vecdeque};
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "List use should succeed");
}

#[test]
fn test_extern_crate() {
    let source = r#"
        extern crate serde;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Extern crate should succeed");
}

#[test]
fn test_extern_crate_with_rename() {
    let source = r#"
        extern crate serde as json;
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Extern crate with rename should succeed");
}
