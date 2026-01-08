// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! println Functions Demonstration
//!
//! This example demonstrates all println functions that automatically
//! add newlines after printing values.

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON println Functions Demo\n");

    // Create external function declarations for all println functions
    let externals = vec![
        LirExternal {
            name: "zulon_println_i32".to_string(),
            param_types: vec![LirTy::I32],
            return_type: LirTy::Unit,
        },
        LirExternal {
            name: "zulon_println_i64".to_string(),
            param_types: vec![LirTy::I64],
            return_type: LirTy::Unit,
        },
        LirExternal {
            name: "zulon_println_f64".to_string(),
            param_types: vec![LirTy::F64],
            return_type: LirTy::Unit,
        },
        LirExternal {
            name: "zulon_println".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))], // i8* for C strings
            return_type: LirTy::Unit,
        },
    ];

    // Create function that demonstrates all println functions
    let demo_function = create_demo_function();

    // Create build configuration
    let config = BuildConfig {
        output: "println_demo".into(),
        keep_intermediates: true,
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
            println!("💡 Run it with: ./println_demo");
            println!("   Expected output:");
            println!("      Line 1: Integer 32");
            println!("      Line 2: Integer 64");
            println!("      Line 3: Float 64");
            println!("      Line 4: Hello with println!");
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

/// Create a function that demonstrates all println functions
fn create_demo_function() -> LirFunction {
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

    // Create a block that demonstrates all println functions
    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // Println i32: 42
            LirInstruction::Const {
                dest: 0,
                value: LirConstant::Integer(42),
                ty: LirTy::I32,
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println_i32".to_string(),
                args: vec![0],
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },
            // Println i64: -123456789012
            LirInstruction::Const {
                dest: 1,
                value: LirConstant::Integer(123456789012),
                ty: LirTy::I64,
            },
            LirInstruction::UnaryOp {
                dest: 2,
                op: zulon_lir::LirUnaryOp::Neg,
                operand: 1,
                ty: LirTy::I64,
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println_i64".to_string(),
                args: vec![2],
                arg_types: vec![LirTy::I64],
                return_type: LirTy::Unit,
            },
            // Println f64: 3.14159
            LirInstruction::Const {
                dest: 3,
                value: LirConstant::Float(3.14159),
                ty: LirTy::F64,
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println_f64".to_string(),
                args: vec![3],
                arg_types: vec![LirTy::F64],
                return_type: LirTy::Unit,
            },
            // Println string: "Hello with println!"
            LirInstruction::Const {
                dest: 4,
                value: LirConstant::String("Hello with println!".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println".to_string(),
                args: vec![4],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            // Load return value 0
            LirInstruction::Const {
                dest: 5,
                value: LirConstant::Integer(0),
                ty: LirTy::I32,
            },
        ],
        terminator: Some(LirTerminator::Return(Some(5))),
    };

    func.blocks.insert(0, block);
    func
}
