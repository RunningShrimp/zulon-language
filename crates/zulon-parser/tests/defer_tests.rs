// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comprehensive tests for defer statement parsing

use zulon_parser::Parser;

/// Test helper to parse source code
fn parse(source: &str) -> zulon_parser::ast::Ast {
    let mut parser = Parser::from_source(source);
    parser.parse().expect("Parsing failed")
}

// ============================================================================
// Basic Defer Tests
// ============================================================================

#[test]
fn test_parse_defer_single_expression() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("cleanup")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_function_call() {
    let source = r#"
        extern fn cleanup();

        fn main() -> i32 {
            defer cleanup()
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_with_string_literal() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("Cleaning up resources")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Multiple Defer Tests
// ============================================================================

#[test]
fn test_parse_multiple_defer_statements() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("cleanup 3")
            defer println("cleanup 2")
            defer println("cleanup 1")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);

    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[1].kind {
        assert_eq!(func.name.name, "main");
    } else {
        panic!("Expected main function");
    }
}

#[test]
fn test_parse_defer_interleaved_with_code() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            println("start")
            defer println("cleanup 1")
            println("middle")
            defer println("cleanup 2")
            println("end")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Defer in Different Contexts
// ============================================================================

#[test]
fn test_parse_defer_in_if_branch() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let x = 10;
            if x > 5 {
                defer println("if branch cleanup")
                println("in if branch")
            }
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_in_both_if_branches() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let x = 10;
            if x > 5 {
                defer println("then cleanup")
                1
            } else {
                defer println("else cleanup")
                0
            }
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_before_early_return() {
    let source = r#"
        extern fn println(s: string);

        fn check_value(x: i32) -> i32 {
            defer println("cleanup")
            if x < 0 {
                return 0
            }
            x
        }

        fn main() -> i32 {
            check_value(42)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 3);
}

// ============================================================================
// Defer with Variables
// ============================================================================

#[test]
fn test_parse_defer_with_variable_access() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let value = 42;
            defer print_int(value)
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 3);
}

#[test]
fn test_parse_defer_with_tuple_field_access() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let tuple = (1, 2, 3);
            defer print_int(tuple.0)
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Defer in Nested Scopes
// ============================================================================

#[test]
fn test_parse_defer_in_nested_function() {
    let source = r#"
        extern fn println(s: string);

        fn inner() -> i32 {
            defer println("inner cleanup")
            1
        }

        fn main() -> i32 {
            defer println("outer cleanup")
            inner()
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 3);
}

// Note: Defer in while loops is currently limited due to parser constraints
// The while loop body requires semicolons, but defer statements cannot have them
// This test documents the current limitation
#[test]
fn test_parse_defer_in_while_loop() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let i = 0;
            while i < 3 {
                defer println("loop cleanup")
                println("iteration")
                let i = i + 1
            }
            0
        }
    "#;

    // This test documents the current limitation
    // In practice, use defer outside of while loops or in separate functions
    let result = std::panic::catch_unwind(|| {
        parse(source)
    });

    // Expected to fail due to parser limitation
    assert!(result.is_err());
}

// ============================================================================
// Defer Expression Tests
// ============================================================================

#[test]
fn test_parse_defer_with_binary_expression() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let a = 10;
            let b = 20;
            defer print_int(a + b)
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_with_complex_expression() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let x = 5;
            let y = 10;
            defer print_int(x * 2 + y)
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_defer_comprehensive_usage() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);

        fn process_file() -> i32 {
            println("Opening file")
            defer println("Closing file")

            println("Processing file")
            defer println("Flushing buffers")

            0
        }

        fn main() -> i32 {
            println("Starting program")
            defer println("Program exit")

            let result = process_file();

            println("Program finished")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 4);

    // Verify process_file function (index 2, after extern declarations)
    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[2].kind {
        assert_eq!(func.name.name, "process_file");
    } else {
        panic!("Expected process_file function");
    }

    // Verify main function (index 3)
    if let zulon_parser::ast::ItemKind::Function(func) = &ast.items[3].kind {
        assert_eq!(func.name.name, "main");
    } else {
        panic!("Expected main function");
    }
}

#[test]
fn test_defer_lifo_order() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("cleanup 3: last registered, first to execute")
            defer println("cleanup 2: second registered")
            defer println("cleanup 1: first registered, last to execute")

            println("Main code executing")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_defer_with_match_expression() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("cleanup")

            let result = match 2 {
                1 => {
                    println("Got one")
                    1
                },
                2 => {
                    println("Got two")
                    2
                },
                _ => {
                    println("Got something else")
                    0
                }
            };

            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_parse_defer_at_end_of_function() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            println("start")
            println("middle")
            println("end")
            defer println("final cleanup")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_at_start_of_function() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("setup cleanup")
            println("first operation")
            println("second operation")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_defer_only_statement() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            defer println("only cleanup")
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}
