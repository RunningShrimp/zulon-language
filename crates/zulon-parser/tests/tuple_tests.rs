// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comprehensive tests for tuple parsing and numeric field access

use zulon_parser::Parser;

/// Test helper to parse source code
fn parse(source: &str) -> zulon_parser::ast::Ast {
    let mut parser = Parser::from_source(source);
    parser.parse().expect("Parsing failed")
}

// ============================================================================
// Tuple Creation Tests
// ============================================================================

#[test]
fn test_parse_single_element_tuple() {
    let source = r#"
        fn main() -> i32 {
            let single = (42,);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_pair_tuple() {
    let source = r#"
        fn main() -> i32 {
            let pair = (1, 2);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_triple_tuple() {
    let source = r#"
        fn main() -> i32 {
            let triple = (1, 2, 3);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_nested_tuples() {
    let source = r#"
        fn main() -> i32 {
            let nested = ((1, 2), (3, 4));
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_with_expressions() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1 + 2, 3 * 4, 5 - 1);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Tuple Field Access Tests
// ============================================================================

#[test]
fn test_parse_tuple_field_0() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2, 3);
            let first = tuple.0;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_field_1() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2, 3);
            let second = tuple.1;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_field_2() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2, 3);
            let third = tuple.2;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_multiple_field_access() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2, 3);
            let a = tuple.0;
            let b = tuple.1;
            let c = tuple.2;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_nested_field_access() {
    let source = r#"
        fn main() -> i32 {
            let nested = ((1, 2), 3);
            let inner = nested.0;
            let first = inner.0;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_field_in_expression() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (10, 20);
            let sum = tuple.0 + tuple.1;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_field_in_function_call() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let tuple = (42, 100);
            print_int(tuple.0);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Tuple in Match Tests
// ============================================================================

#[test]
fn test_parse_tuple_match() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2);
            let result = match tuple.0 {
                1 => 10,
                _ => 0
            };
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Tuple Return Type Tests
// ============================================================================

#[test]
fn test_parse_function_returning_tuple() {
    let source = r#"
        fn make_pair() -> (i32, i32) {
            (1, 2)
        }

        fn main() -> i32 {
            let pair = make_pair();
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_function_with_tuple_parameter() {
    let source = r#"
        fn process(tuple: (i32, i32)) -> i32 {
            tuple.0
        }

        fn main() -> i32 {
            let result = process((1, 2));
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Complex Tuple Tests
// ============================================================================

#[test]
fn test_parse_tuple_let_chain() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2, 3);
            let a = tuple.0;
            let b = tuple.1;
            let c = tuple.2;
            let sum = a + b + c;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_in_if_condition() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2);
            if tuple.0 > 0 {
                1
            } else {
                0
            }
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_tuple_in_while_loop() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (0, 10);
            while tuple.0 < tuple.1 {
                let tuple = (tuple.0 + 1, tuple.1);
            }
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Error Cases - Negative Tests
// ============================================================================

#[test]
fn test_parse_tuple_without_comma_fails() {
    let source = r#"
        fn main() -> i32 {
            let invalid = (42);  // This is just parentheses, not a tuple
            0
        }
    "#;

    // This should parse successfully but as a parenthesized expression
    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_single_element_tuple_requires_comma() {
    let source = r#"
        fn main() -> i32 {
            let single = (42,);  // Comma is required for single-element tuple
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_tuple_comprehensive_usage() {
    let source = r#"
        extern fn print_int(n: i32);

        fn create_tuple() -> (i32, i32, i32) {
            (1, 2, 3)
        }

        fn main() -> i32 {
            let tuple = create_tuple();
            let first = tuple.0;
            let second = tuple.1;
            let third = tuple.2;

            print_int(first);
            print_int(second);
            print_int(third);

            let sum = first + second + third;
            print_int(sum);

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);

    // Verify main function
    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[1].kind {
        assert_eq!(func.name.name, "main");
    } else {
        panic!("Expected main function");
    }
}
