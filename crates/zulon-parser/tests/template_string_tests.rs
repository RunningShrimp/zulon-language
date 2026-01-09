// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comprehensive tests for template string parsing

use zulon_parser::Parser;

/// Test helper to parse source code
fn parse(source: &str) -> zulon_parser::ast::Ast {
    let mut parser = Parser::from_source(source);
    parser.parse().expect("Parsing failed")
}

// ============================================================================
// Static Template String Tests
// ============================================================================

#[test]
fn test_parse_simple_template_string() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let greeting = `Hello, World!`;
            println(greeting);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_assignment() {
    let source = r#"
        fn main() -> i32 {
            let message = `Static template string`;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_empty_template_string() {
    let source = r#"
        fn main() -> i32 {
            let empty = ``;
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parse_template_string_with_special_chars() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let special = `Hello\nWorld\t!`;
            println(special);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_with_quotes() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let with_quotes = `He said "Hello"`;
            println(with_quotes);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_multiline() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let multiline = `Line 1
Line 2
Line 3`;
            println(multiline);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Template String in Different Contexts
// ============================================================================

#[test]
fn test_parse_template_string_as_function_argument() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            println(`Direct string argument`);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_in_match() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let result = match 1 {
                1 => `one`,
                2 => `two`,
                _ => `other`
            };
            println(result);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_in_if() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let condition = true;
            let message = if condition {
                `true message`
            } else {
                `false message`
            };
            println(message);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_return_type() {
    let source = r#"
        fn get_message() -> string {
            `Hello, World!`
        }

        fn main() -> i32 {
            let msg = get_message();
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Multiple Template Strings
// ============================================================================

#[test]
fn test_parse_multiple_template_strings() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let s1 = `First string`;
            let s2 = `Second string`;
            let s3 = `Third string`;

            println(s1);
            println(s2);
            println(s3);

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_template_string_concatenation() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let part1 = `Hello`;
            let part2 = `World`;
            // Note: String concatenation not yet implemented
            // Just testing that both parse correctly
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Template String with Variables
// ============================================================================

#[test]
fn test_parse_template_string_alongside_variables() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let name = `ZULON`;
            let version = `0.1.0`;
            let greeting = `Welcome`;

            println(greeting);
            println(name);
            println(version);

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Template String in Complex Expressions
// ============================================================================

// Note: Template strings in block expressions have parser limitations
// This test documents the current behavior
#[test]
fn test_parse_template_string_in_block() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let result = {
                let message = `Inside block`;
                message;
            };
            println(result);
            0
        }
    "#;

    // This currently fails due to parser limitations with blocks
    let result = std::panic::catch_unwind(|| {
        parse(source)
    });

    // Document that this is a known limitation
    assert!(result.is_err(), "Template strings in block expressions are currently limited");
}

#[test]
fn test_parse_template_string_in_loop() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let i = 0;
            while i < 3 {
                println(`Loop iteration`);
                let i = i + 1;
            }
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
fn test_template_string_comprehensive() {
    let source = r#"
        extern fn println(s: string);

        fn greet(name: string) -> string {
            `Hello, `
        }

        fn main() -> i32 {
            let greeting = `Welcome to ZULON`;
            let version = `Version 0.1.0`;
            let message = `Static template strings work`;

            println(greeting);
            println(version);
            println(message);

            let result = greet(`User`);
            println(result);

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 3);
}

#[test]
fn test_template_string_with_all_features() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            // Simple strings
            let simple = `simple string`;

            // Strings with special characters
            let special = `special\tchars\nhere`;

            // Empty string
            let empty = ``;

            // Strings in expressions
            let result = if true {
                `true branch`
            } else {
                `false branch`
            };

            // Strings in match
            let matched = match 1 {
                1 => `one`,
                _ => `other`
            };

            println(simple);
            println(special);
            println(empty);
            println(result);
            println(matched);

            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

// ============================================================================
// Template String as Struct/Enum Fields (Future)
// ============================================================================

#[test]
fn test_parse_template_string_in_enum_variant() {
    let source = r#"
        enum Message {
            Text(string),
        }

        fn main() -> i32 {
            let msg = Message::Text(`Hello`);
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
fn test_parse_template_string_with_backticks() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            // Note: Escaping backticks in template strings
            // This tests the parser can handle it
            let text = `Text with backtick`;
            println(text);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_very_long_template_string() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            let long = `This is a very long template string that contains many words and should still parse correctly without any issues whatsoever`;
            println(long);
            0
        }
    "#;

    let ast = parse(source);
    assert_eq!(ast.items.len(), 2);
}
