// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test discovery for HIR
//!
//! This module provides functionality to discover test functions
//! (functions marked with #[test]) in HIR crates.

use crate::hir::*;
use zulon_parser::ast::Attribute;

/// A discovered test function
#[derive(Debug, Clone)]
pub struct DiscoveredTest {
    /// Test function name
    pub name: String,
    /// Module path (e.g., "my_module::tests")
    pub module_path: String,
    /// Whether the test should be ignored
    pub ignored: bool,
    /// Whether the test is expected to panic
    pub should_panic: bool,
    /// Expected panic message (if specified)
    pub expected_panic_message: Option<String>,
}

/// Discover all test functions in an HIR crate
pub fn discover_tests(hir_crate: &HirCrate) -> Vec<DiscoveredTest> {
    let mut tests = Vec::new();
    discover_tests_in_items(&hir_crate.items, "", &mut tests);
    tests
}

/// Discover tests in a list of items
fn discover_tests_in_items(items: &[HirItem], module_path: &str, tests: &mut Vec<DiscoveredTest>) {
    for item in items {
        if let HirItem::Function(func) = item {
            if is_test_function(func) {
                let test = DiscoveredTest {
                    name: func.name.clone(),
                    module_path: module_path.to_string(),
                    ignored: has_ignore_attribute(&func.attributes),
                    should_panic: has_should_panic_attribute(&func.attributes),
                    expected_panic_message: get_expected_panic_message(&func.attributes),
                };
                tests.push(test);
            }
        }
    }
}

/// Check if a function has the #[test] attribute
fn is_test_function(func: &HirFunction) -> bool {
    func.attributes.iter().any(|attr| attr.name.name == "test")
}

/// Check if a function has the #[ignore] attribute
fn has_ignore_attribute(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attr| attr.name.name == "ignore")
}

/// Check if a function has the #[should_panic] attribute
fn has_should_panic_attribute(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attr| attr.name.name == "should_panic")
}

/// Get the expected panic message from #[should_panic(expected = "...")]
fn get_expected_panic_message(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .find(|attr| attr.name.name == "should_panic")
        .and_then(|attr| {
            attr.args.iter().find_map(|arg| {
                if let zulon_parser::ast::AttributeArg::KeyValue { key, value } = arg {
                    if key.name == "expected" {
                        Some(value.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_parser::ast::{Attribute, AttributeArg, Identifier};
    use crate::{HirCrate, HirItem, HirFunction, HirBlock};
    use zulon_parser::ast::Span;

    #[test]
    fn test_discover_simple_test() {
        let func = HirFunction {
            id: 0,
            name: "test_addition".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: HirTy::Unit,
            error_type: None,
            effects: Vec::new(),
            attributes: vec![
                Attribute {
                    name: Identifier::new(Span::default(), "test"),
                    args: Vec::new(),
                }
            ],
            body: HirBlock {
                id: 0,
                statements: Vec::new(),
                trailing_expr: None,
                ty: HirTy::Unit,
                span: Span::default(),
            },
            span: Span::default(),
        };

        let hir_crate = HirCrate {
            items: vec![HirItem::Function(func)],
            span: Span::default(),
        };

        let tests = discover_tests(&hir_crate);
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "test_addition");
        assert!(!tests[0].ignored);
        assert!(!tests[0].should_panic);
    }

    #[test]
    fn test_discover_ignored_test() {
        let func = HirFunction {
            id: 0,
            name: "test_slow_operation".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: HirTy::Unit,
            error_type: None,
            effects: Vec::new(),
            attributes: vec![
                Attribute {
                    name: Identifier::new(Span::default(), "test"),
                    args: Vec::new(),
                },
                Attribute {
                    name: Identifier::new(Span::default(), "ignore"),
                    args: Vec::new(),
                },
            ],
            body: HirBlock {
                id: 0,
                statements: Vec::new(),
                trailing_expr: None,
                ty: HirTy::Unit,
                span: Span::default(),
            },
            span: Span::default(),
        };

        let hir_crate = HirCrate {
            items: vec![HirItem::Function(func)],
            span: Span::default(),
        };

        let tests = discover_tests(&hir_crate);
        assert_eq!(tests.len(), 1);
        assert!(tests[0].ignored);
    }

    #[test]
    fn test_discover_should_panic_test() {
        let func = HirFunction {
            id: 0,
            name: "test_panic".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: HirTy::Unit,
            error_type: None,
            effects: Vec::new(),
            attributes: vec![
                Attribute {
                    name: Identifier::new(Span::default(), "test"),
                    args: Vec::new(),
                },
                Attribute {
                    name: Identifier::new(Span::default(), "should_panic"),
                    args: vec![
                        AttributeArg::KeyValue {
                            key: Identifier::new(Span::default(), "expected"),
                            value: "index out of bounds".to_string(),
                        }
                    ],
                },
            ],
            body: HirBlock {
                id: 0,
                statements: Vec::new(),
                trailing_expr: None,
                ty: HirTy::Unit,
                span: Span::default(),
            },
            span: Span::default(),
        };

        let hir_crate = HirCrate {
            items: vec![HirItem::Function(func)],
            span: Span::default(),
        };

        let tests = discover_tests(&hir_crate);
        assert_eq!(tests.len(), 1);
        assert!(tests[0].should_panic);
        assert_eq!(tests[0].expected_panic_message, Some("index out of bounds".to_string()));
    }
}
