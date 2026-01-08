// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Print Call Example
//!
//! This example demonstrates calling external runtime functions
//! to print values from a ZULON program.

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON External Function Call Example\n");

    // Create external function declaration
    let print_i32_external = LirExternal {
        name: "zulon_print_i32".to_string(),
        param_types: vec![LirTy::I32],
        return_type: LirTy::Unit,
    };

    // Create function that calls the external print function
    let print_function = create_print_call_function();

    // Create build configuration
    let config = BuildConfig {
        output: "print_call_example".into(),
        keep_intermediates: true,
        target: None,
        ..Default::default()  // Uses opt_level: 2 (-O2)
    };

    println!("📦 Building executable...");

    // Create build pipeline
    let mut pipeline = BuildPipeline::new(config);
    pipeline.add_externals(vec![print_i32_external]);
    pipeline.add_functions(vec![print_function]);

    // Build!
    match pipeline.build() {
        Ok(exe_path) => {
            println!("✅ Build successful!");
            println!("   Executable: {}", exe_path.display());
            println!();
            println!("💡 Run it with: ./print_call_example");
            println!("   Expected output: 42");
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

/// Create a function that calls the runtime print function
fn create_print_call_function() -> LirFunction {
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

    // Create basic block that loads 42, calls print, and returns 0
    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // Load constant 42
            LirInstruction::Const {
                dest: 0,
                value: LirConstant::Integer(42),
                ty: LirTy::I32,
            },
            // Call external function zulon_print_i32(42)
            LirInstruction::CallExternal {
                dest: None, // No return value
                func_name: "zulon_print_i32".to_string(),
                args: vec![0], // Pass v0 (which holds 42)
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },
        ],
        terminator: Some(LirTerminator::Return(Some(0))), // Return 0
    };

    func.blocks.insert(0, block);
    func
}
