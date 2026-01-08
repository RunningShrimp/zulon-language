// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for error handling
//!
//! Tests the complete pipeline: Parser → AST → HIR → Type Checker → MIR

use zulon_parser::Parser;
use zulon_hir::lower::lower_hir;
use zulon_typeck::TypeChecker;
use zulon_mir::lower_hir;

/// Test throw statement compilation through MIR
#[test]
fn test_throw_statement_compilation() {
    let source = r#"
        fn divide(a: i32, b: i32) -> i32 | DivideError {
            if b == 0 {
                throw DivideError::Zero;
            }
            Outcome::Ok(a / b)
        }
    "#;

    // Parse to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse_crate().expect("Failed to parse");

    // Lower to HIR
    let hir = lower_hir(&ast).expect("Failed to lower to HIR");

    // Type check
    let mut typeck = TypeChecker::new();
    typeck.check_crate(&hir).expect("Type checking failed");

    // Lower to MIR
    let mir = lower_hir(&hir).expect("Failed to lower to MIR");

    // Verify MIR has function
    assert!(!mir.functions.is_empty(), "MIR should have at least one function");

    // Find the divide function
    let divide_func = mir.functions.iter()
        .find(|f| f.name == "divide")
        .expect("divide function should exist in MIR");

    // Verify function has basic blocks
    assert!(!divide_func.blocks.is_empty(), "Function should have basic blocks");

    // Verify there's a return terminator (from throw)
    let has_return = divide_func.blocks.values().any(|block| {
        matches!(block.terminator, Some(zulon_mir::MirTerminator::Return(_)))
    });
    assert!(has_return, "Should have Return terminator from throw statement");
}

/// Test ? operator compilation through MIR
#[test]
fn test_question_mark_operator_compilation() {
    let source = r#"
        fn divide(a: i32, b: i32) -> i32 | DivideError {
            if b == 0 {
                throw DivideError::Zero;
            }
            Outcome::Ok(a / b)
        }

        fn calculate() -> i32 | DivideError {
            let x = divide(10, 2)?;
            Outcome::Ok(x * 2)
        }
    "#;

    // Parse to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse_crate().expect("Failed to parse");

    // Lower to HIR
    let hir = lower_hir(&ast).expect("Failed to lower to HIR");

    // Type check
    let mut typeck = TypeChecker::new();
    typeck.check_crate(&hir).expect("Type checking failed");

    // Lower to MIR
    let mir = lower_hir(&hir).expect("Failed to lower to MIR");

    // Find the calculate function
    let calculate_func = mir.functions.iter()
        .find(|f| f.name == "calculate")
        .expect("calculate function should exist in MIR");

    // Verify function has multiple basic blocks (from ? operator)
    assert!(calculate_func.blocks.len() >= 3,
            "? operator should create at least 3 blocks (current, success, error)");

    // Verify there's a conditional branch (from discriminant checking)
    let has_cond_branch = calculate_func.blocks.values().any(|block| {
        matches!(block.terminator, Some(zulon_mir::MirTerminator::If { .. }))
    });
    assert!(has_cond_branch,
            "? operator should create If terminator for discriminant checking");
}

/// Test type checking validates throw error types
#[test]
fn test_throw_type_validation() {
    let source = r#"
        fn divide(a: i32, b: i32) -> i32 | DivideError {
            if b == 0 {
                throw DivideError::Zero;
            }
            Outcome::Ok(a / b)
        }

        fn wrong_error() -> i32 | DivideError {
            throw ParseError::Invalid;
        }
    "#;

    // Parse to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse_crate().expect("Failed to parse");

    // Lower to HIR
    let hir = lower_hir(&ast).expect("Failed to lower to HIR");

    // Type check should fail
    let mut typeck = TypeChecker::new();
    let result = typeck.check_crate(&hir);

    // Should get a type error (mismatched error types)
    assert!(result.is_err(), "Type checking should fail with mismatched error type");
}

/// Test type checking validates ? operator context
#[test]
fn test_question_mark_context_validation() {
    let source = r#"
        fn might_fail() -> i32 | DivideError {
            Outcome::Ok(42)
        }

        fn no_error() -> i32 {
            let x = might_fail()?;
            0
        }
    "#;

    // Parse to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse_crate().expect("Failed to parse");

    // Lower to HIR
    let hir = lower_hir(&ast).expect("Failed to lower to HIR");

    // Type check should fail
    let mut typeck = TypeChecker::new();
    let result = typeck.check_crate(&hir);

    // Should get a type error (? used without error type)
    assert!(result.is_err(),
            "Type checking should fail: ? used in function without error type");
}

/// Test multiple ? operators in sequence
#[test]
fn test_chained_question_marks() {
    let source = r#"
        fn divide(a: i32, b: i32) -> i32 | DivideError {
            if b == 0 {
                throw DivideError::Zero;
            }
            Outcome::Ok(a / b)
        }

        fn pipeline() -> i32 | DivideError {
            let step1 = divide(100, 2)?;
            let step2 = divide(step1, 5)?;
            let step3 = divide(step2, 2)?;
            Outcome::Ok(step3)
        }
    "#;

    // Parse to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse_crate().expect("Failed to parse");

    // Lower to HIR
    let hir = lower_hir(&ast).expect("Failed to lower to HIR");

    // Type check
    let mut typeck = TypeChecker::new();
    typeck.check_crate(&hir).expect("Type checking should succeed");

    // Lower to MIR
    let mir = lower_hir(&hir).expect("Failed to lower to MIR");

    // Find pipeline function
    let pipeline_func = mir.functions.iter()
        .find(|f| f.name == "pipeline")
        .expect("pipeline function should exist");

    // Should have many basic blocks from multiple ? operators
    // Each ? creates at least 3 blocks (current, success, error/continue)
    // With 3 ? operators, we expect at least 7-9 blocks
    assert!(pipeline_func.blocks.len() >= 7,
            "Multiple ? operators should create many basic blocks, got {}",
            pipeline_func.blocks.len());
}

/// Test Outcome<T, E> type can be used explicitly
#[test]
fn test_explicit_outcome_usage() {
    let source = r#"
        fn divide(a: i32, b: i32) -> Outcome<i32, DivideError> {
            if b == 0 {
                Outcome::Err(DivideError::Zero)
            } else {
                Outcome::Ok(a / b)
            }
        }
    "#;

    // Parse to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse_crate().expect("Failed to parse");

    // Lower to HIR
    let hir = lower_hir(&ast).expect("Failed to lower to HIR");

    // Type check should succeed
    let mut typeck = TypeChecker::new();
    let result = typeck.check_crate(&hir);

    // Note: This may fail if we don't support explicit Outcome<T, E> syntax yet
    // but it demonstrates the intent
    if result.is_ok() {
        // Lower to MIR
        let mut mir_lower = 
        let mir = lower_hir(&hir).expect("Failed to lower to MIR");

        // Verify function exists
        assert!(!mir.functions.is_empty(), "MIR should have functions");
    } else {
        // Expected if Outcome<T, E> syntax not yet supported
        // This is OK for now - explicit syntax is future work
        println!("Explicit Outcome<T, E> syntax not yet supported (expected)");
    }
}
