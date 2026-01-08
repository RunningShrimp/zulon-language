// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! getchar Function Demonstration
//!
//! This example demonstrates reading a single character from stdin
//! using the zulon_getchar() function.

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON getchar Function Demo\n");

    // Create external function declarations
    let externals = vec![
        // Declare putchar for output
        LirExternal {
            name: "zulon_putchar".to_string(),
            param_types: vec![LirTy::I32], // char is i32 in C
            return_type: LirTy::Unit,
        },
        // Declare getchar for input
        LirExternal {
            name: "zulon_getchar".to_string(),
            param_types: vec![],
            return_type: LirTy::I32, // returns int (char or EOF)
        },
    ];

    // Create function that reads a character and echoes it back
    let echo_function = create_echo_function();

    // Create build configuration
    let config = BuildConfig {
        output: "getchar_demo".into(),
        keep_intermediates: true,
        target: None,
        ..Default::default()  // Uses opt_level: 2 (-O2)
    };

    println!("📦 Building executable...");

    // Create build pipeline
    let mut pipeline = BuildPipeline::new(config);
    pipeline.add_externals(externals);
    pipeline.add_functions(vec![echo_function]);

    // Build!
    match pipeline.build() {
        Ok(exe_path) => {
            println!("✅ Build successful!");
            println!("   Executable: {}", exe_path.display());
            println!();
            println!("💡 Run it with: ./getchar_demo");
            println!("   The program will:");
            println!("   1. Wait for you to type a character");
            println!("   2. Echo it back to you");
            println!("   3. Exit");
            println!();
            println!("   Try typing: 'A' or 'Z' or any key!");
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

/// Create a function that reads a character and echoes it
fn create_echo_function() -> LirFunction {
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

    // Create a block that reads and echoes a character
    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // Call getchar to read a character
            LirInstruction::CallExternal {
                dest: Some(0),
                func_name: "zulon_getchar".to_string(),
                args: vec![],
                arg_types: vec![],
                return_type: LirTy::I32,
            },
            // Echo the character back using putchar
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_putchar".to_string(),
                args: vec![0],
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },
            // Load return value 0
            LirInstruction::Const {
                dest: 1,
                value: LirConstant::Integer(0),
                ty: LirTy::I32,
            },
        ],
        terminator: Some(LirTerminator::Return(Some(1))),
    };

    func.blocks.insert(0, block);
    func
}
