// Simple HIR Lowering Demo
//
// This example demonstrates how to:
// 1. Parse ZULON source code
// 2. Lower AST to HIR
// 3. Inspect the typed HIR representation

use zulon_hir::{lower_ast_simple, HirItem};
use zulon_parser::{Lexer, Parser};

fn main() {
    // Simple ZULON program
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    let x = a + b;
    let y = x * 2;
    y
}

fn main() -> i32 {
    let result = add(10, 20);
    result
}
"#;

    println!("=== ZULON HIR Lowering Demo ===\n");
    println!("Source code:");
    println!("{}", source);
    println!();

    // Lex the source code
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.lex_all();

    if !lex_errors.is_empty() {
        eprintln!("❌ Lexing errors:");
        for err in lex_errors {
            eprintln!("   {}", err);
        }
        return;
    }

    println!("✅ Lexing successful! Got {} tokens\n", tokens.len());

    // Parse the tokens
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsing successful!");
            println!("   Found {} top-level items\n", ast.items.len());

            // Lower AST to HIR
            match lower_ast_simple(&ast) {
                Ok(hir) => {
                    println!("✅ HIR lowering successful!");
                    println!("   Generated {} HIR items\n", hir.items.len());

                    // Inspect HIR items
                    for item in &hir.items {
                        inspect_hir_item(item, 0);
                    }
                }
                Err(e) => {
                    eprintln!("❌ HIR lowering failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parsing failed: {}", e);
        }
    }
}

fn inspect_hir_item(item: &HirItem, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match item {
        HirItem::Function(func) => {
            println!("{}Function: {}", indent_str, func.name);
            println!("{}  Parameters:", indent_str);
            for param in &func.params {
                println!("{}    {}: {:?}", indent_str, param.name, param.ty);
            }
            println!("{}  Return type: {:?}", indent_str, func.return_type);
            println!("{}  Body has {} statements", indent_str, func.body.statements.len());

            // Show statements
            for stmt in &func.body.statements {
                print!("{}    - ", indent_str);
                match stmt {
                    zulon_hir::HirStatement::Local(local) => {
                        println!("let {}: {:?}", local.name, local.ty);
                    }
                    zulon_hir::HirStatement::Semi(_) => {
                        println!("(expression statement)");
                    }
                    _ => {}
                }
            }

            // Show trailing expression
            if let Some(trailing) = &func.body.trailing_expr {
                println!("{}  Trailing expression: {:?}", indent_str, trailing.ty());
            }
        }
        _ => {
            println!("{}Other item (not inspecting)", indent_str);
        }
    }
}
