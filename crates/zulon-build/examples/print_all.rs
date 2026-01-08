// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Comprehensive Print Example
//!
//! This example demonstrates calling various runtime print functions
//! to output different types of values.

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON Comprehensive Print Example\n");

    // Create external function declarations
    let externals = vec![
        LirExternal {
            name: "zulon_print_i32".to_string(),
            param_types: vec![LirTy::I32],
            return_type: LirTy::Unit,
        },
        LirExternal {
            name: "zulon_print_i64".to_string(),
            param_types: vec![LirTy::I64],
            return_type: LirTy::Unit,
        },
        LirExternal {
            name: "zulon_print_f64".to_string(),
            param_types: vec![LirTy::F64],
            return_type: LirTy::Unit,
        },
    ];

    // Create function that demonstrates all print functions
    let demo_function = create_comprehensive_print_function();

    // Create build configuration
    let config = BuildConfig {
        output: "print_all_example".into(),
        keep_intermediates: true,
        target: None,
        ..Default::default()  // Uses opt_level: 2 (-O2)
    };

    println!("📦 Building executable...");

    // Create build pipeline
    let mut pipeline = BuildPipeline::new(config);
    pipeline.add_externals(externals);
    pipeline.add_functions(vec![demo_function]);

    // Build!
    match pipeline.build() {
        Ok(exe_path) => {
            println!("✅ Build successful!");
            println!("   Executable: {}", exe_path.display());
            println!();
            println!("💡 Run it with: ./print_all_example");
            println!("   Expected output:");
            println!("      Integer 32: 42");
            println!("      Integer 64: -123456789012");
            println!("      Float 64: 3.14159");
        }
        Err(e) => {
            eprintln!("❌ Build failed: {}", e);
            eprintln!();
            eprintln!("⚠️  Note: This example requires LLVM tools:");
            eprintln!("   - llvm-as (LLVM assembler)");
            eprintln!("   - llc (LLVM compiler)");
            eprintln!("   - ld or lld (linker)");
        }
    }
}

/// Create a function that demonstrates all print functions
fn create_comprehensive_print_function() -> LirFunction {
    let mut func = LirFunction {
        name: "zulon_main".to_string(),
        params: vec![],
        param_types: vec![],
        return_type: LirTy::I32,
        blocks: HashMap::new(),
        entry_block: 0,
        next_id: 1,
        next_vreg: 0,
        external_funcs: Vec::new(),
    };

    // Create a block that prints various values
    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // Load i32 constant 42
            LirInstruction::Const {
                dest: 0,
                value: LirConstant::Integer(42),
                ty: LirTy::I32,
            },
            // Print i32: 42
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print_i32".to_string(),
                args: vec![0],
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },
            // Load i64 constant -123456789012
            LirInstruction::Const {
                dest: 1,
                value: LirConstant::Integer(123456789012),
                ty: LirTy::I64,
            },
            // Negate to make it negative (since we don't have negative constants yet)
            LirInstruction::UnaryOp {
                dest: 2,
                op: zulon_lir::LirUnaryOp::Neg,
                operand: 1,
                ty: LirTy::I64,
            },
            // Print i64: -123456789012
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print_i64".to_string(),
                args: vec![2],
                arg_types: vec![LirTy::I64],
                return_type: LirTy::Unit,
            },
            // Load f64 constant 3.14159
            LirInstruction::Const {
                dest: 3,
                value: LirConstant::Float(3.14159),
                ty: LirTy::F64,
            },
            // Print f64: 3.14159
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print_f64".to_string(),
                args: vec![3],
                arg_types: vec![LirTy::F64],
                return_type: LirTy::Unit,
            },
            // Load return value 0
            LirInstruction::Const {
                dest: 4,
                value: LirConstant::Integer(0),
                ty: LirTy::I32,
            },
        ],
        terminator: Some(LirTerminator::Return(Some(4))),
    };

    func.blocks.insert(0, block);
    func
}
