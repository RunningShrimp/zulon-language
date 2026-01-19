// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for trait type checking

use zulon_parser::ast;
use zulon_typeck::TypeChecker;

#[test]
fn test_trait_definition() {
    let source = r#"
        trait Display {
            fn to_string(&self) -> String;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}

#[test]
fn test_trait_with_generics() {
    let source = r#"
        trait Iterator<T> {
            fn next(&mut self) -> T? | None;
            fn iter(&self) -> &T;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}

#[test]
fn test_trait_with_super_traits() {
    let source = r#"
        trait Debug {
            fn debug(&self) -> String;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}

#[test]
fn test_trait_with_associated_types() {
    let source = r#"
        trait Container {
            type Item;
            const SIZE: usize = 0;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}

#[test]
fn test_impl_basic() {
    let source = r#"
        impl Display for i32 {
            fn to_string(&self) -> String {
                "42".to_string()
            }
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}

#[test]
fn test_impl_with_generics() {
    let source = r#"
        trait Container<T> {
            fn item(&self, index: usize) -> &T;
        }

        impl Container<Vec<i32>> for Container<Vec<i32>> {
            fn item(&self, index: usize) -> &i32 {
                &self.0[index]
            }
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}

#[test]
fn test_impl_for_multiple_types() {
    let source = r#"
        trait Debug {
            fn debug(&self) -> String;
        }

        impl Debug for i32 {
            fn debug(&self) -> String {
                "int: 42".to_string()
            }
        }

        impl Debug for String {
            fn debug(&self) -> String {
                self.clone()
            }
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = TypeChecker::new();
    assert!(checker.check(&ast).is_ok());
}
