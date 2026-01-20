// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for trait type checking
//!
//! These tests verify the complete trait type checking
//! functionality in realistic scenarios.

use zulon_parser::Parser;

#[test]
fn test_full_trait_definition_cycle() {
    // Test complete trait definition flow
    let source = r#"
        trait Display {
            fn to_string(&self) -> String;
        }

        trait Debug {
            fn debug(&self) -> String;
        }

        impl Display for i32 {
            fn to_string(&self) -> String {
                "42".to_string()
            }
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Trait definition should succeed");
}

#[test]
fn test_trait_with_generic_constraints() {
    // Test trait with bounds and generics
    let source = r#"
        trait Comparable<T>: Ord {
            fn compare(&self, other: &T) -> i32;
        }

        impl<T: Ord> for i32 {
            fn compare(&self, other: &T) -> i32 {
                self - other
            }
        }
    "#;

    let mut parser = Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Trait with generics should succeed");
}

#[test]
fn test_trait_impl_with_method_mismatch() {
    // Test that impl method signature mismatch is detected
    let source = r#"
        trait Debug {
            fn debug(&self) -> String;
        }

        impl Debug for i32 {
            fn debug(&self) -> String {
                "int: 42".to_string()
            }
        }

        impl Debug for i32 {
            fn debug(&self, _extra: i32) -> String {
                "int: 42".to_string()
            }
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(
        result.is_err(),
        "Impl with wrong method signature should fail"
    );
}

#[test]
fn test_multiple_traits() {
    // Test multiple trait definitions and their interactions
    let source = r#"
        trait Display {
            fn to_string(&self) -> String;
        }

        trait Iterator<T> {
            fn next(&mut self) -> T? | None;
        }

        impl Display for i32 {
            fn to_string(&self) -> String {
                "42".to_string()
            }
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Multiple traits should succeed");
}

#[test]
fn test_trait_associated_types() {
    // Test trait with associated types
    let source = r#"
        trait Container {
            type Item;
        }

        impl Container for Vec<i32> {
            type Item = Vec<i32>;
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Trait with associated types should succeed");
}

#[test]
fn test_trait_hierarchy() {
    // Test trait inheritance
    let source = r#"
        trait Base {
            fn method(&self);
        }

        trait Derived: Base {
            fn derived_method(&self);
            fn base_method(&self);
        }
    "#;

    let mut parser = zulon_parser::Parser::from_source(source);
    let ast = parser.parse().unwrap();

    let mut checker = zulon_typeck::TypeChecker::new();
    let result = checker.check(&ast);

    assert!(result.is_ok(), "Trait hierarchy should succeed");
}
