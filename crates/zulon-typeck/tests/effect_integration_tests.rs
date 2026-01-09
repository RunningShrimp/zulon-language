// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the Effect System
//!
//! These tests verify the complete end-to-end functionality of
//! the effect system, from parsing through type checking.

use zulon_parser::Parser;
use zulon_typeck::TypeChecker;

/// Helper function to parse and type check a source string
fn check_source(source: &str) -> Result<(), String> {
    let mut parser = Parser::from_source(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut checker = TypeChecker::new();
    checker.check(&ast).map_err(|e| format!("Type error: {:?}", e))
}

// ========== Basic Effect Tests ==========

#[test]
fn test_pure_function_no_effects() {
    let source = r#"
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Effect Propagation Tests ==========

#[test]
fn test_effect_propagation_simple() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn helper() -> i32 | IO {
            read()
        }

        fn caller() -> i32 | IO {
            helper()
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_effect_propagation_deep_nesting() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn level3() -> i32 | IO {
            read()
        }

        fn level2() -> i32 | IO {
            level3()
        }

        fn level1() -> i32 | IO {
            level2()
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_effect_propagation_multiple_calls() {
    let source = r#"
        effect IO {
            fn read1() -> i32
            fn read2() -> i32
        }

        fn helper1() -> i32 | IO {
            read1()
        }

        fn helper2() -> i32 | IO {
            read2()
        }

        fn caller() -> i32 | IO {
            let x = helper1();
            let y = helper2();
            x + y
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Purity Tests ==========

#[test]
fn test_pure_function_calls_pure() {
    let source = r#"
        fn helper(x: i32) -> i32 {
            x + 1
        }

        fn caller(x: i32) -> i32 {
            helper(x)
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Effect Accumulation Tests ==========

#[test]
fn test_effects_accumulate() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn helper() -> i32 | IO {
            read()
        }

        fn caller() -> i32 | IO {
            helper();
            helper();
            42
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_effects_in_if_expression() {
    let source = r#"
        effect IO {
            fn read1() -> i32
            fn read2() -> i32
        }

        fn helper1() -> i32 | IO {
            read1()
        }

        fn helper2() -> i32 | IO {
            read2()
        }

        fn test(x: bool) -> i32 | IO {
            if x {
                helper1()
            } else {
                helper2()
            }
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Arithmetic with Effects ==========

#[test]
fn test_arithmetic_with_effect_calls() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn helper() -> i32 | IO {
            read()
        }

        fn caller() -> i32 | IO {
            helper() + 20
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_arithmetic_chain_with_effects() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn helper() -> i32 | IO {
            read()
        }

        fn caller() -> i32 | IO {
            helper() + helper() * 2
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Complex Scenarios ==========

#[test]
fn test_function_with_multiple_parameters() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn process(x: i32, y: i32, z: i32) -> i32 | IO {
            read() + x + y + z
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_function_returning_call_result() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn helper() -> i32 | IO {
            read()
        }

        fn caller() -> i32 | IO {
            helper()
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Edge Cases ==========

#[test]
fn test_empty_function_body() {
    let source = r#"
        fn empty() -> i32 {
            42
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_function_with_unit_return() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn helper() -> i32 | IO {
            read()
        }

        fn caller() {
            helper();
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_multiple_functions_interacting() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn reader() -> i32 | IO {
            read()
        }

        fn processor(x: i32) -> i32 {
            x + 1
        }

        fn caller() -> i32 | IO {
            let x = reader();
            processor(x)
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Effect Declaration Scenarios ==========

#[test]
fn test_function_declares_no_effects_but_calls_pure() {
    let source = r#"
        fn helper(x: i32) -> i32 {
            x + 1
        }

        fn caller(x: i32) -> i32 {
            helper(x)
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_function_with_error_type_and_effects() {
    let source = r#"
        effect IO {
            fn read() -> i32
        }

        fn parse_file() -> i32 | ParseError | IO {
            read()
        }
    "#;

    assert!(check_source(source).is_ok());
}

// ========== Real-World Scenarios ==========

#[test]
fn test_file_processing_scenario() {
    let source = r#"
        effect IO {
            fn read_file() -> i32
            fn write_file(data: i32)
        }

        fn process_file() -> i32 {
            let data = read_file();
            let result = data * 2;
            write_file(result);
            result
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_data_pipeline_scenario() {
    let source = r#"
        effect IO {
            fn fetch_data() -> i32
        }

        fn stage1() -> i32 | IO {
            fetch_data()
        }

        fn stage2(x: i32) -> i32 {
            x + 10
        }

        fn stage3(x: i32) -> i32 {
            x * 2
        }

        fn pipeline() -> i32 | IO {
            let data = stage1();
            let processed = stage2(data);
            stage3(processed)
        }
    "#;

    assert!(check_source(source).is_ok());
}

#[test]
fn test_conditional_io_scenario() {
    let source = r#"
        effect IO {
            fn read_opt1() -> i32
            fn read_opt2() -> i32
        }

        fn read_option1() -> i32 | IO {
            read_opt1()
        }

        fn read_option2() -> i32 | IO {
            read_opt2()
        }

        fn conditional_read(use_opt1: bool) -> i32 | IO {
            if use_opt1 {
                read_option1()
            } else {
                read_option2()
            }
        }
    "#;

    assert!(check_source(source).is_ok());
}
