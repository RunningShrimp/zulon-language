// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! MIR lowering demonstration
//!
//! This example shows how HIR is lowered to MIR.

use zulon_mir::{lower_hir, MirBody};
use zulon_hir::lower_ast_simple;
use zulon_parser::{Lexer, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    let x = a + b;
    let y = x * 2;
    y
}

fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
"#;

    println!("=== ZULON MIR Lowering Demonstration ===\n");
    println!("Source code:");
    println!("{}", source);
    println!();

    // Step 1: Lexing
    println!("Step 1: Lexing...");
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.lex_all();

    if !lex_errors.is_empty() {
        println!("  Lexer errors:");
        for err in &lex_errors {
            println!("    {}", err);
        }
    }

    println!("  Generated {} tokens", tokens.len());
    println!();

    // Step 2: Parsing
    println!("Step 2: Parsing...");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    println!("  Parsed {} items", ast.items.len());
    println!();

    // Step 3: Lower to HIR
    println!("Step 3: Lowering AST to HIR...");
    let hir = lower_ast_simple(&ast)?;

    println!("  Generated HIR with {} items", hir.items.len());
    println!();

    // Step 4: Lower HIR to MIR
    println!("Step 4: Lowering HIR to MIR...");
    let mir: MirBody = lower_hir(&hir)?;

    println!("  Generated MIR with {} functions", mir.functions.len());
    println!();

    // Step 5: Inspect MIR
    println!("=== MIR Output ===\n");
    for func in &mir.functions {
        inspect_mir_function(func, 0);
    }

    Ok(())
}

fn inspect_mir_function(func: &zulon_mir::MirFunction, indent: usize) {
    let indent_str = "  ".repeat(indent);

    println!("{}function {}(", indent_str, func.name);
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}: {}", param.name, param.ty.display_name());
    }
    println!(") -> {}", func.return_type.display_name());
    println!("{{");

    // Print basic blocks
    for (block_id, block) in &func.blocks {
        println!("{}block{}:", indent_str, block_id);

        // Print instructions
        for inst in &block.instructions {
            println!("{}  {}", indent_str, format_instruction(inst, &func));
        }

        // Print terminator
        if let Some(terminator) = &block.terminator {
            println!("{}  -> {}", indent_str, format_terminator(terminator));
        }
    }

    println!("{}}}", indent_str);
    println!();
}

fn format_instruction(inst: &zulon_mir::MirInstruction, _func: &zulon_mir::MirFunction) -> String {
    match inst {
        zulon_mir::MirInstruction::Const { dest, value, ty } => {
            format!("_{} = const {} ({:?})", dest, ty.display_name(), value)
        }
        zulon_mir::MirInstruction::Copy { dest, src } => {
            format!("_{} = copy {}", dest, format_place(src))
        }
        zulon_mir::MirInstruction::Move { dest, src } => {
            format!("_{} = move {}", dest, format_place(src))
        }
        zulon_mir::MirInstruction::BinaryOp { dest, op, left, right, ty } => {
            format!("_{} = {} _{} _{} ({})",
                dest, format_bin_op(*op), left, right, ty.display_name())
        }
        zulon_mir::MirInstruction::UnaryOp { dest, op, operand, ty } => {
            format!("_{} = {} _{} ({})",
                dest, format_unary_op(*op), operand, ty.display_name())
        }
        zulon_mir::MirInstruction::Call { dest, func, args, return_type } => {
            let dest_str = if let Some(d) = dest {
                format!("_{} = ", d)
            } else {
                String::new()
            };
            format!("{}call {}({}) -> {}",
                dest_str,
                format_place(func),
                args.iter().map(|a| format_place(a)).collect::<Vec<_>>().join(", "),
                return_type.display_name())
        }
        zulon_mir::MirInstruction::Load { dest, src, ty } => {
            format!("_{} = load {} {}", dest, ty.display_name(), format_place(src))
        }
        zulon_mir::MirInstruction::Store { dest, src, ty } => {
            format!("store {} _{} -> {}", ty.display_name(), src, format_place(dest))
        }
        zulon_mir::MirInstruction::Borrow { dest, src, mutable, ty } => {
            let mut_str = if *mutable { "mut " } else { "" };
            format!("_{} = borrow&{}{} {}",
                dest, mut_str, ty.display_name(), format_place(src))
        }
        zulon_mir::MirInstruction::Drop { place, ty } => {
            format!("drop {} ({})", ty.display_name(), format_place(place))
        }
    }
}

fn format_terminator(terminator: &zulon_mir::MirTerminator) -> String {
    match terminator {
        zulon_mir::MirTerminator::Return(place) => {
            if let Some(p) = place {
                format!("return {}", format_place(p))
            } else {
                "return".to_string()
            }
        }
        zulon_mir::MirTerminator::Goto { target } => {
            format!("goto -> block{}", target)
        }
        zulon_mir::MirTerminator::If { condition, then_block, else_block } => {
            format!("if _{} -> block{} else block{}",
                condition, then_block, else_block)
        }
        zulon_mir::MirTerminator::Switch { scrutinee, targets, default } => {
            let targets_str = targets.iter()
                .map(|(val, blk)| format!("{:?} -> block{}", val, blk))
                .collect::<Vec<_>>()
                .join(", ");
            format!("switch _{} {{ {} }} -> block{}",
                scrutinee, targets_str, default)
        }
        zulon_mir::MirTerminator::Unreachable => {
            "unreachable".to_string()
        }
    }
}

fn format_place(place: &zulon_mir::MirPlace) -> String {
    match place {
        zulon_mir::MirPlace::Local(name) => name.clone(),
        zulon_mir::MirPlace::Temp(temp) => format!("_{}", temp),
        zulon_mir::MirPlace::Param(name) => format!("'{}", name),
        zulon_mir::MirPlace::Field { base, field } => {
            format!("{}.{}", format_place(base), field)
        }
        zulon_mir::MirPlace::Index { base, index } => {
            format!("{}[_{}]", format_place(base), index)
        }
        zulon_mir::MirPlace::Deref(place) => {
            format!("*{}", format_place(place))
        }
        zulon_mir::MirPlace::Ref { place, mutable } => {
            if *mutable {
                format!("&mut {}", format_place(place))
            } else {
                format!("&{}", format_place(place))
            }
        }
    }
}

fn format_bin_op(op: zulon_mir::MirBinOp) -> &'static str {
    match op {
        zulon_mir::MirBinOp::Add => "+",
        zulon_mir::MirBinOp::Sub => "-",
        zulon_mir::MirBinOp::Mul => "*",
        zulon_mir::MirBinOp::Div => "/",
        zulon_mir::MirBinOp::Mod => "%",
        zulon_mir::MirBinOp::BitAnd => "&",
        zulon_mir::MirBinOp::BitOr => "|",
        zulon_mir::MirBinOp::BitXor => "^",
        zulon_mir::MirBinOp::LeftShift => "<<",
        zulon_mir::MirBinOp::RightShift => ">>",
        zulon_mir::MirBinOp::And => "&&",
        zulon_mir::MirBinOp::Or => "||",
        zulon_mir::MirBinOp::Eq => "==",
        zulon_mir::MirBinOp::NotEq => "!=",
        zulon_mir::MirBinOp::Less => "<",
        zulon_mir::MirBinOp::LessEq => "<=",
        zulon_mir::MirBinOp::Greater => ">",
        zulon_mir::MirBinOp::GreaterEq => ">=",
    }
}

fn format_unary_op(op: zulon_mir::MirUnaryOp) -> &'static str {
    match op {
        zulon_mir::MirUnaryOp::Neg => "-",
        zulon_mir::MirUnaryOp::Not => "!",
        zulon_mir::MirUnaryOp::Deref => "*",
        zulon_mir::MirUnaryOp::Ref => "&",
        zulon_mir::MirUnaryOp::RefMut => "&mut",
    }
}
