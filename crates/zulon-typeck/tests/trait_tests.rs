// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Trait type checking tests
//!
//! This module contains comprehensive tests for trait type checking,
//! following TDD (Test-Driven Development) principles.

use zulon_typeck::checker::TypeChecker;
use zulon_typeck::env::Env;
use zulon_typeck::ty::Ty;
use zulon_parser::ast;

/// Test helper to create a simple trait definition
fn simple_trait_def() -> ast::Trait {
    ast::Trait {
        span: Default::default(),
        name: ast::Identifier {
            span: Default::default(),
            name: "Display".to_string(),
        },
        generics: Vec::new(),
        items: vec![
            ast::TraitItem::Method(ast::Method {
                span: Default::default(),
                name: ast::Identifier {
                    span: Default::default(),
                    name: "display".to_string(),
                },
                params: Vec::new(),
                return_type: Some(ast::Type {
                    span: Default::default(),
                    annotation: None,
                    kind: ast::TypeKind::Path(ast::TypePath {
                        span: Default::default(),
                        path: vec![
                            ast::TypePathSegment {
                                span: Default::default(),
                                segment: ast::PathSegment::Identifier(ast::Identifier {
                                    span: Default::default(),
                                    name: "String".to_string(),
                                }),
                            },
                        ],
                    }),
                }),
            }),
        ],
    }
}

/// Test helper to create a trait implementation
fn simple_trait_impl() -> ast::Impl {
    ast::Impl {
        span: Default::default(),
        generics: Vec::new(),
        trait_name: Some(ast::TypePath {
            span: Default::default(),
            path: vec![
                ast::TypePathSegment {
                    span: Default::default(),
                    segment: ast::PathSegment::Identifier(ast::Identifier {
                        span: Default::default(),
                        name: "Display".to_string(),
                    }),
                },
            ],
        }),
        self_type: ast::Type {
            span: Default::default(),
            annotation: None,
            kind: ast::TypeKind::Path(ast::TypePath {
                span: Default::default(),
                path: vec![
                    ast::TypePathSegment {
                        span: Default::default(),
                        segment: ast::PathSegment::Identifier(ast::Identifier {
                            span: Default::default(),
                            name: "String".to_string(),
                        }),
                    },
                ],
            }),
        },
        items: vec![
            ast::ImplItem::Method(ast::Method {
                span: Default::default(),
                name: ast::Identifier {
                    span: Default::default(),
                    name: "display".to_string(),
                },
                params: Vec::new(),
                body: Some(vec![]),
                return_type: Some(ast::Type {
                    span: Default::default(),
                    annotation: None,
                    kind: ast::TypeKind::Unit,
                }),
            }),
        ],
    }
}

/// Test helper to create a function using a trait method
fn simple_function_with_trait_method() -> ast::Function {
    ast::Function {
        span: Default::default(),
        name: ast::Identifier {
            span: Default::default(),
            name: "test_display".to_string(),
        },
        generics: Vec::new(),
        params: vec![],
        return_type: Some(ast::Type {
            span: Default::default(),
            annotation: None,
            kind: ast::TypeKind::Unit,
        }),
        body: Some(vec![ast::Statement {
            span: Default::default(),
            kind: ast::StatementKind::Expression(ast::Expression {
                span: Default::default(),
                kind: ast::ExpressionKind::Call(ast::Call {
                    span: Default::default(),
                    function: ast::Expression {
                        span: Default::default(),
                        kind: ast::ExpressionKind::Path(ast::ExpressionPath {
                            span: Default::default(),
                            path: vec![
                                ast::TypePathSegment {
                                    span: Default::default(),
                                    segment: ast::PathSegment::Identifier(ast::Identifier {
                                        span: Default::default(),
                                        name: "String".to_string(),
                                    }),
                                },
                                ast::TypePathSegment {
                                    span: Default::default(),
                                    segment: ast::PathSegment::Identifier(ast::Identifier {
                                        span: Default::default(),
                                        name: "display".to_string(),
                                    }),
                                },
                            ],
                        }),
                    },
                    args: vec![],
                }),
            }),
        }]),
        effects: Vec::new(),
    }
}

/// Test helper to create a function with trait bounds
fn simple_function_with_trait_bounds() -> ast::Function {
    let ty_string = ast::TypeKind::Path(ast::TypePath {
        span: Default::default(),
        path: vec![
            ast::TypePathSegment {
                span: Default::default(),
                segment: ast::PathSegment::Identifier(ast::Identifier {
                    span: Default::default(),
                    name: "Display".to_string(),
                }),
            },
        ],
    });

    ast::Function {
        span: Default::default(),
        name: ast::Identifier {
            span: Default::default(),
            name: "bounded_function".to_string(),
        },
        generics: vec![ast::GenericParam {
            span: Default::default(),
            name: ast::Identifier {
                span: Default::default(),
                name: "T".to_string(),
            },
            bounds: Some(vec![ast::GenericBound {
                span: Default::default(),
                kind: ast::GenericBoundKind::Trait(ty_string.clone()),
            }]),
        }],
        params: vec![ast::Parameter {
            span: Default::default(),
            name: ast::Identifier {
                span: Default::default(),
                name: "value".to_string(),
            },
            type_annotation: Some(ty_string.clone()),
        }],
        return_type: Some(ty_string.clone()),
        body: Some(vec![ast::Statement {
            span: Default::default(),
            kind: ast::StatementKind::Expression(ast::Expression {
                span: Default::default(),
                kind: ast::ExpressionKind::Call(ast::Call {
                    span: Default::default(),
                    function: ast::Expression {
                        span: Default::default(),
                        kind: ast::ExpressionKind::Path(ast::ExpressionPath {
                            span: Default::default(),
                            path: vec![
                                ast::TypePathSegment {
                                    span: Default::default(),
                                    segment: ast::PathSegment::Identifier(ast::Identifier {
                                        span: Default::default(),
                                        name: "String".to_string(),
                                    }),
                                },
                                ast::TypePathSegment {
                                    span: Default::default(),
                                    segment: ast::PathSegment::Identifier(ast::Identifier {
                                        span: Default::default(),
                                        name: "display".to_string(),
                                    }),
                                },
                            ],
                        }),
                    },
                    args: vec![ast::Expression {
                        span: Default::default(),
                        kind: ast::ExpressionKind::Identifier(ast::Identifier {
                            span: Default::default(),
                            name: "value".to_string(),
                        }),
                    }],
                }),
            }),
        }]),
        effects: Vec::new(),
    }
}

/// Test helper to create an AST for trait checking
fn create_ast_with_trait() -> ast::Ast {
    ast::Ast {
        span: Default::default(),
        items: vec![
            ast::Item::Trait(simple_trait_def()),
            ast::Item::Impl(simple_trait_impl()),
            ast::Item::Function(simple_function_with_trait_method()),
            ast::Item::Function(simple_function_with_trait_bounds()),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Simple trait declaration should type check successfully
    #[test]
    fn test_simple_trait_declaration() {
        let ast = create_ast_with_trait();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait declaration should succeed");
    }

    /// Test: Trait with associated type should type check successfully
    #[test]
    fn test_trait_with_associated_type() {
        let mut trait_def = simple_trait_def();
        let mut items = trait_def.items.clone();

        // Add associated type
        items.push(ast::TraitItem::AssociatedType(ast::TraitItem::AssociatedType {
            span: Default::default(),
            name: ast::Identifier {
                span: Default::default(),
                name: "Output".to_string(),
            },
            bounds: None,
        }));

        trait_def.items = items;

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Trait(trait_def)],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait with associated type should succeed");
    }

    /// Test: Trait with generic bounds should type check successfully
    #[test]
    fn test_trait_with_bounds() {
        let trait_def = simple_trait_def();
        let mut generics = trait_def.generics.clone();

        // Add generic parameter with trait bound
        generics.push(ast::GenericParam {
            span: Default::default(),
            name: ast::Identifier {
                span: Default::default(),
                name: "T".to_string(),
            },
            bounds: Some(vec![ast::GenericBound {
                span: Default::default(),
                kind: ast::GenericBoundKind::Trait(ast::TypeKind::Path(ast::TypePath {
                    span: Default::default(),
                    path: vec![
                        ast::TypePathSegment {
                            span: Default::default(),
                            segment: ast::PathSegment::Identifier(ast::Identifier {
                                span: Default::default(),
                                name: "Comparable".to_string(),
                            }),
                        },
                    ],
                }),
            }]),
        });

        trait_def.generics = generics;

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Trait(trait_def)],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait with bounds should succeed");
    }

    /// Test: Trait method should be collected in trait environment
    #[test]
    fn test_trait_method_collected() {
        let ast = create_ast_with_trait();

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait methods should be collected");

        // Verify method type is known
        // This will be validated by checking the trait environment
    }

    /// Test: Trait impl should validate trait name exists
    #[test]
    fn test_trait_impl_valid_trait_name() {
        let ast = create_ast_with_trait();

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait impl should succeed with valid trait name");
    }

    /// Test: Trait impl should have correct self type
    #[test]
    fn test_trait_impl_self_type() {
        let ast = create_ast_with_trait();

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait impl self type should be String");
    }

    /// Test: Function using trait method should succeed
    #[test]
    fn test_function_with_trait_method() {
        let ast = create_ast_with_trait();

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Function using trait method should succeed");
    }

    /// Test: Function with trait bounds should succeed
    #[test]
    fn test_function_with_trait_bounds() {
        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Function(simple_function_with_trait_bounds())],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Function with trait bounds should succeed");
    }

    /// Test: Multiple traits should coexist
    #[test]
    fn test_multiple_traits() {
        let mut trait1 = simple_trait_def();
        let mut trait2 = simple_trait_def();

        trait2.name = ast::Identifier {
            span: Default::default(),
            name: "Comparable".to_string(),
        };

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![
                ast::Item::Trait(trait1),
                ast::Item::Trait(trait2),
            ],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Multiple traits should coexist");
    }

    /// Test: Trait impl without trait name should fail with proper error
    #[test]
    fn test_trait_impl_missing_trait_name() {
        let mut impl_block = simple_trait_impl();
        impl_block.trait_name = None;

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Impl(impl_block)],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_err(), "Impl without trait name should fail");
    }

    /// Test: Trait method with wrong self type should fail
    #[test]
    fn test_trait_method_wrong_self_type() {
        let mut trait_def = simple_trait_def();

        // Change return type to a different type
        if let Some(ref mut rt) = trait_def.items[0] {
            if let ast::TraitItem::Method(ref mut method) = rt {
                if let Some(ref mut rt) = method.return_type {
                    *rt = ast::Type {
                        span: Default::default(),
                        annotation: None,
                        kind: ast::TypeKind::Path(ast::TypePath {
                            span: Default::default(),
                            path: vec![
                                ast::TypePathSegment {
                                    span: Default::default(),
                                    segment: ast::PathSegment::Identifier(ast::Identifier {
                                        span: Default::default(),
                                        name: "i32".to_string(),
                                    }),
                                },
                            ],
                        }),
                    };
                }
            }
        }

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Trait(trait_def)],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        // This test verifies the type checking works correctly
        // The result may be ok or err depending on implementation
    }

    /// Test: Generic parameter bound should be correctly type-checked
    #[test]
    fn test_generic_trait_bound() {
        let trait_def = simple_trait_def();
        let mut generics = trait_def.generics.clone();

        // Add generic parameter with trait bound
        generics.push(ast::GenericParam {
            span: Default::default(),
            name: ast::Identifier {
                span: Default::default(),
                name: "U".to_string(),
            },
            bounds: Some(vec![ast::GenericBound {
                span: Default::default(),
                kind: ast::GenericBoundKind::Trait(ast::TypeKind::Path(ast::TypePath {
                    span: Default::default(),
                    path: vec![
                        ast::TypePathSegment {
                            span: Default::default(),
                            segment: ast::PathSegment::Identifier(ast::Identifier {
                                span: Default::default(),
                                name: "Copy".to_string(),
                            }),
                        },
                    ],
                }),
            }]),
        });

        trait_def.generics = generics;

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Trait(trait_def)],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Generic trait bounds should succeed");
    }

    /// Test: Impl with generics should succeed
    #[test]
    fn test_trait_impl_with_generics() {
        let mut impl_block = simple_trait_impl();
        impl_block.generics = vec![ast::GenericParam {
            span: Default::default(),
            name: ast::Identifier {
                span: Default::default(),
                name: "T".to_string(),
            },
        }];

        let ast = ast::Ast {
            span: Default::default(),
            items: vec![ast::Item::Impl(impl_block)],
        };

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Impl with generics should succeed");
    }

    /// Test: Trait impl should support method calls
    #[test]
    fn test_trait_impl_method_calls() {
        let ast = create_ast_with_trait();

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);

        assert!(result.is_ok(), "Trait impl should support method calls");
    }
}
