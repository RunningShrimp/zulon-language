// Test Closure Capture Analysis
//
// This example demonstrates and tests the closure capture analysis functionality.

use zulon_hir::{
    analyze_captures, HirBlock, HirExpression, HirStatement, HirTy, SimpleEnvironment,
};
use zulon_parser::{Span, Position};

fn main() {
    println!("=== ZULON Closure Capture Analysis Test ===\n");

    // Test 1: Simple closure capturing an outer variable
    println!("Test 1: Closure capturing outer variable");
    test_simple_capture();

    // Test 2: Closure with no captures
    println!("\nTest 2: Closure with no captures");
    test_no_captures();

    // Test 3: Closure capturing multiple variables
    println!("\nTest 3: Closure capturing multiple variables");
    test_multiple_captures();

    // Test 4: Nested closures
    println!("\nTest 4: Nested closures");
    test_nested_captures();

    // Test 5: Closure with local variable (not captured)
    println!("\nTest 5: Closure with local variable");
    test_local_variable_not_captured();

    println!("\n=== All Capture Analysis Tests Complete ===");
}

fn make_span(line: usize, col: usize) -> Span {
    Span {
        start: Position { line, column: col },
        end: Position { line, column: col + 1 },
    }
}

/// Test 1: Simple closure that captures an outer variable
fn test_simple_capture() {
    // Create environment with outer variable
    let mut env = SimpleEnvironment::new();
    env.add("x".to_string(), HirTy::I32);

    // Create closure that references x: |y| x + y
    let closure_body = HirExpression::BinaryOp {
        op: zulon_hir::HirBinOp::Add,
        left: Box::new(HirExpression::Variable("x".to_string(), 0, HirTy::I32, make_span(1, 5))),
        right: Box::new(HirExpression::Variable("y".to_string(), 1, HirTy::I32, make_span(1, 9))),
        ty: HirTy::I32,
        span: make_span(1, 7),
    };

    // Analyze captures
    let analysis = analyze_captures(&env, &closure_body, vec!["y".to_string()]);

    println!("  Captures: {} variable(s)", analysis.captures.len());
    for capture in &analysis.captures {
        println!("    - {} (mode: {:?}, type: {:?})", capture.name, capture.mode, capture.ty);
    }

    assert_eq!(analysis.captures.len(), 1);
    assert_eq!(analysis.captures[0].name, "x");
    assert_eq!(analysis.captures[0].mode, zulon_hir::HirCaptureMode::ImmutableRef);

    println!("  ✅ Test passed: Correctly captured 'x' by immutable reference");
}

/// Test 2: Closure with no captures
fn test_no_captures() {
    // Empty environment
    let env = SimpleEnvironment::new();

    // Closure that only uses its parameter: |x| x * 2
    let closure_body = HirExpression::BinaryOp {
        op: zulon_hir::HirBinOp::Mul,
        left: Box::new(HirExpression::Variable("x".to_string(), 0, HirTy::I32, make_span(1, 5))),
        right: Box::new(HirExpression::Literal(
            zulon_hir::HirLiteral::Integer(2),
            1,
            HirTy::I32,
            make_span(1, 9),
        )),
        ty: HirTy::I32,
        span: make_span(1, 7),
    };

    // Analyze captures
    let analysis = analyze_captures(&env, &closure_body, vec!["x".to_string()]);

    println!("  Captures: {} variable(s)", analysis.captures.len());

    assert_eq!(analysis.captures.len(), 0);

    println!("  ✅ Test passed: No variables captured (only uses parameter)");
}

/// Test 3: Closure capturing multiple variables
fn test_multiple_captures() {
    // Create environment with multiple outer variables
    let mut env = SimpleEnvironment::new();
    env.add("a".to_string(), HirTy::I32);
    env.add("b".to_string(), HirTy::I32);
    env.add("c".to_string(), HirTy::I32);

    // Closure that uses a, b, and its parameter x: |x| a + b + x
    let closure_body = HirExpression::BinaryOp {
        op: zulon_hir::HirBinOp::Add,
        left: Box::new(HirExpression::BinaryOp {
            op: zulon_hir::HirBinOp::Add,
            left: Box::new(HirExpression::Variable("a".to_string(), 0, HirTy::I32, make_span(1, 5))),
            right: Box::new(HirExpression::Variable("b".to_string(), 1, HirTy::I32, make_span(1, 9))),
            ty: HirTy::I32,
            span: make_span(1, 7),
        }),
        right: Box::new(HirExpression::Variable("x".to_string(), 2, HirTy::I32, make_span(1, 13))),
        ty: HirTy::I32,
        span: make_span(1, 11),
    };

    // Analyze captures
    let analysis = analyze_captures(&env, &closure_body, vec!["x".to_string()]);

    println!("  Captures: {} variable(s)", analysis.captures.len());
    for capture in &analysis.captures {
        println!("    - {} (mode: {:?})", capture.name, capture.mode);
    }

    assert_eq!(analysis.captures.len(), 2);

    let capture_names: Vec<_> = analysis.captures.iter().map(|c| c.name.as_str()).collect();
    assert!(capture_names.contains(&"a"));
    assert!(capture_names.contains(&"b"));

    println!("  ✅ Test passed: Correctly captured 'a' and 'b'");
}

/// Test 4: Nested closures
fn test_nested_captures() {
    // Outer environment
    let mut env = SimpleEnvironment::new();
    env.add("outer".to_string(), HirTy::I32);

    // Inner closure: |y| outer + y
    let inner_closure_body = HirExpression::BinaryOp {
        op: zulon_hir::HirBinOp::Add,
        left: Box::new(HirExpression::Variable(
            "outer".to_string(),
            0,
            HirTy::I32,
            make_span(2, 5),
        )),
        right: Box::new(HirExpression::Variable("y".to_string(), 1, HirTy::I32, make_span(2, 13))),
        ty: HirTy::I32,
        span: make_span(2, 9),
    };

    // Inner closure captures 'outer' from outer scope
    let inner_analysis = analyze_captures(&env, &inner_closure_body, vec!["y".to_string()]);

    println!("  Inner closure captures: {} variable(s)", inner_analysis.captures.len());
    for capture in &inner_analysis.captures {
        println!("    - {}", capture.name);
    }

    assert_eq!(inner_analysis.captures.len(), 1);
    assert_eq!(inner_analysis.captures[0].name, "outer");

    println!("  ✅ Test passed: Nested closure correctly captures 'outer'");
}

/// Test 5: Closure with local variable (should not be captured)
fn test_local_variable_not_captured() {
    // Environment with outer variable
    let mut env = SimpleEnvironment::new();
    env.add("outer_var".to_string(), HirTy::I32);

    // Closure body with local variable: |x| { let y = x * 2; y + 10 }
    let closure_block = HirBlock {
        id: 0,
        statements: vec![HirStatement::Local(zulon_hir::HirLocal {
            id: 1,
            name: "y".to_string(),
            ty: HirTy::I32,
            init: Some(HirExpression::BinaryOp {
                op: zulon_hir::HirBinOp::Mul,
                left: Box::new(HirExpression::Variable("x".to_string(), 0, HirTy::I32, make_span(1, 12))),
                right: Box::new(HirExpression::Literal(
                    zulon_hir::HirLiteral::Integer(2),
                    2,
                    HirTy::I32,
                    make_span(1, 16),
                )),
                ty: HirTy::I32,
                span: make_span(1, 14),
            }),
            span: make_span(1, 8),
        })],
        trailing_expr: Some(HirExpression::BinaryOp {
            op: zulon_hir::HirBinOp::Add,
            left: Box::new(HirExpression::Variable("y".to_string(), 3, HirTy::I32, make_span(1, 21))),
            right: Box::new(HirExpression::Literal(
                zulon_hir::HirLiteral::Integer(10),
                4,
                HirTy::I32,
                make_span(1, 25),
            )),
            ty: HirTy::I32,
            span: make_span(1, 23),
        }),
        ty: HirTy::I32,
        span: make_span(1, 0),
    };

    let closure_body = HirExpression::Block(Box::new(closure_block));

    // Analyze captures
    let analysis = analyze_captures(&env, &closure_body, vec!["x".to_string()]);

    println!("  Captures: {} variable(s)", analysis.captures.len());
    for capture in &analysis.captures {
        println!("    - {}", capture.name);
    }

    // Should have 0 captures (y is local, x is parameter, outer_var is not used)
    assert_eq!(analysis.captures.len(), 0);

    println!("  ✅ Test passed: Local variable 'y' not captured, only parameter 'x' used");
}
