// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comprehensive tests for match expression parsing

use zulon_parser::Parser;

/// Test helper to parse source code
fn parse(source: &str) -> zulon_parser::ast::Ast {
    let mut parser = Parser::from_source(source);
    parser.parse().expect("Parsing failed")
}

// ============================================================================
// Basic Match Expression Tests
// ============================================================================

#[test]
fn test_parse_match_simple() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let result = match x {
                1 => 10,
                2 => 20,
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_two_arms() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let result = match x {
                1 => 10,
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_single_wildcard() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let result = match x {
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Match with Block Arms
// ============================================================================

#[test]
fn test_parse_match_with_block_arms() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let x = 2;
            let result = match x {
                1 => {
                    println("Got one");
                    1
                },
                2 => {
                    println("Got two");
                    2
                },
                _ => {
                    println("Got something else");
                    0
                }
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_match_mixed_arms() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let result = match x {
                1 => 10,
                2 => {
                    let y = 20;
                    y
                },
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Match with Different Patterns
// ============================================================================

#[test]
fn test_parse_match_integer_literals() {
    let source = r#"
        fn main() -> i32 {
            let x = 42;
            let result = match x {
                0 => 0,
                1 => 1,
                2 => 2,
                42 => 42,
                _ => -1
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_boolean_literals() {
    let source = r#"
        fn main() -> i32 {
            let flag = true;
            let result = match flag {
                true => 1,
                false => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_wildcard() {
    let source = r#"
        fn main() -> i32 {
            let x = 100;
            let result = match x {
                1 => 10,
                2 => 20,
                _ => 999
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Match in Different Contexts
// ============================================================================

#[test]
fn test_parse_match_in_let_statement() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let y = match x {
                1 => 10,
                _ => 0
            };
            y
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_in_function_call() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let x = 1;
            print_int(match x {
                1 => 10,
                _ => 0
            });
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_match_in_if_condition() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            if match x {
                1 => true,
                _ => false
            } {
                10
            } else {
                0
            }
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_in_while_loop() {
    let source = r#"
        fn main() -> i32 {
            let x = 0;
            while match x {
                0 => true,
                _ => false
            } {
                let x = 1;
            }
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Nested Match Tests
// ============================================================================

#[test]
fn test_parse_nested_match() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let y = 2;
            let result = match x {
                1 => match y {
                    1 => 10,
                    2 => 20,
                    _ => 0
                },
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_in_match_arm() {
    let source = r#"
        fn main() -> i32 {
            let x = 1;
            let y = 2;
            let result = match x {
                1 => match y {
                    1 => 100,
                    _ => 0
                },
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Match with Tuple Field Access
// ============================================================================

#[test]
fn test_parse_match_with_tuple_field() {
    let source = r#"
        fn main() -> i32 {
            let tuple = (1, 2);
            let result = match tuple.0 {
                1 => 10,
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

// ============================================================================
// Match Return Types
// ============================================================================

#[test]
fn test_parse_match_returning_integer() {
    let source = r#"
        fn get_value(x: i32) -> i32 {
            match x {
                1 => 10,
                _ => 0
            }
        }

        fn main() -> i32 {
            get_value(1)
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_match_returning_string() {
    let source = r#"
        fn get_message(x: i32) -> string {
            match x {
                1 => `one`,
                2 => `two`,
                _ => `other`
            }
        }

        fn main() -> i32 {
            let msg = get_message(1);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Complex Match Tests
// ============================================================================

#[test]
fn test_parse_match_with_expressions() {
    let source = r#"
        fn main() -> i32 {
            let x = 5;
            let y = 10;
            let result = match x {
                1 => y + 1,
                2 => y * 2,
                _ => y - 1
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_with_function_calls() {
    let source = r#"
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let x = 1;
            let result = match x {
                1 => {
                    print_int(10);
                    10
                },
                _ => {
                    print_int(0);
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
// Integration Tests
// ============================================================================

#[test]
fn test_match_comprehensive() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);

        fn classify_number(x: i32) -> i32 {
            match x {
                0 => {
                    println("Number is zero");
                    0
                },
                1 => {
                    println("Number is one");
                    1
                },
                2 => {
                    println("Number is two");
                    2
                },
                _ => {
                    println("Number is something else");
                    -1
                }
            }
        }

        fn main() -> i32 {
            let result1 = classify_number(0);
            let result2 = classify_number(1);
            let result3 = classify_number(100);

            print_int(result1);
            print_int(result2);
            print_int(result3);

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 4);  // println, print_int, classify_number, main
}

#[test]
fn test_match_in_control_flow() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let x = 2;

            if match x {
                1 => true,
                _ => false
            } {
                println("x is 1")
            } else {
                println("x is not 1")
            }

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_parse_match_many_arms() {
    let source = r#"
        fn main() -> i32 {
            let x = 5;
            let result = match x {
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                5 => 5,
                _ => 0
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_match_with_complex_blocks() {
    let source = r#"
        extern fn println(s: string);
        extern fn print_int(n: i32);

        fn main() -> i32 {
            let x = 1;
            let result = match x {
                1 => {
                    let a = 10;
                    let b = 20;
                    println("In block");
                    print_int(a);
                    print_int(b);
                    a + b
                },
                _ => {
                    println("In default");
                    0
                }
            };
            result
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 3);  // println, print_int, main
}
