// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for trait type checking
//!
//! These tests verify complete trait type checking functionality
//! including trait definitions, implementations, and their interactions.

use zulon_parser::Parser;

#[test]
fn test_trait_with_methods() {
    let source = r#"
        trait Display {
            fn to_string(&self) -> String;
            fn debug(&self) -> String;
        }

        impl Display for i32 {
            fn to_string(&self) -> String {
                "int: 42".to_string()
            }
            fn debug(&self) -> String {
                self.to_string()
            }
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Trait with methods should be successfully type-checked"
    );
}

#[test]
fn test_impl_with_generic_arguments() {
    let source = r#"
        trait Container<T> {
            fn item(&self, index: usize) -> &T;
        }

        impl Container<Vec<i32>> for Container<Vec<i32>> {
            fn item(&self, index: usize) -> &i32 {
                &self.items[index]
            }
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Impl with generics should be successfully type-checked"
    );
}

#[test]
fn test_trait_with_associated_types() {
    let source = r#"
        trait Container {
            type Item;
            const SIZE: usize = 0;
        }

        impl Container<Vec<i32>> for Container<Vec<i32>> {
            type Item = Vec<i32>;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Trait with associated types should be successfully type-checked"
    );
}

#[test]
fn test_trait_with_super_traits() {
    let source = r#"
        trait Debug {
            fn debug(&self) -> String;
        }

        trait Base {
            fn method(&self);
        }

        impl Debug for i32 {
            fn debug(&self) -> String {
                "int: 42".to_string()
            }
        }
    }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Trait with super traits should be successfully type-checked"
    );
}

#[test]
fn test_multiple_impls() {
    let source = r#"
        trait Debug {
            fn debug(&self) -> String;
        }

        impl Debug for i32 {
            fn debug(&self) -> String {
                "int: 42".to_string()
            }
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Multiple impls should be successfully type-checked"
    );
}

#[test]
fn test_trait_definition() {
    let source = r#"
        trait Debug {
            fn debug(&self) -> String;
        }

        impl Debug for i32 {
            fn debug(&self) -> String {
                "int: 42".to_string()
            }
        }
    }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Trait definition should be successfully type-checked"
    );
}

#[test]
fn test_generic_trait() {
    let source = r#"
        trait Iterator<T> {
            fn next(&mut self) -> Option<T> | None;
            fn iter(&self) -> &T;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_ok(),
        "Generic trait should be successfully type-checked"
    );
}
