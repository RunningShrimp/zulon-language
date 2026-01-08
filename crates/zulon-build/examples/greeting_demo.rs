// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Interactive Greeting Demo
//!
//! This example demonstrates simple character-by-character input
//! to create an interactive greeting program.

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON Interactive Greeting Demo\n");

    // Create external function declarations
    let externals = vec![
        // Output functions
        LirExternal {
            name: "zulon_print".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
            return_type: LirTy::Unit,
        },
        LirExternal {
            name: "zulon_putchar".to_string(),
            param_types: vec![LirTy::I32],
            return_type: LirTy::Unit,
        },
        // Input function
        LirExternal {
            name: "zulon_getchar".to_string(),
            param_types: vec![],
            return_type: LirTy::I32,
        },
    ];

    // Create function that asks for name and greets
    let greeting_function = create_greeting_function();

    // Create build configuration
    let config = BuildConfig {
        output: "greeting_demo".into(),
        keep_intermediates: true,
        target: None,
        ..Default::default()  // Uses opt_level: 2 (-O2)
    };

    println!("📦 Building executable...");

    // Create build pipeline
    let mut pipeline = BuildPipeline::new(config);
    pipeline.add_externals(externals);
    pipeline.add_functions(vec![greeting_function]);

    // Build!
    match pipeline.build() {
        Ok(exe_path) => {
            println!("✅ Build successful!");
            println!("   Executable: {}", exe_path.display());
            println!();
            println!("💡 Run it with: ./greeting_demo");
            println!();
            println!("   The program will:");
            println!("   1. Ask you to type your initial");
            println!("   2. Greet you with that initial");
            println!();
            println!("   Example: Type 'J' and press Enter");
            println!("   Output: Hello, J!");
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

/// Create a function that asks for initial and greets user
fn create_greeting_function() -> LirFunction {
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

    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // Print prompt
            LirInstruction::Const {
                dest: 0,
                value: LirConstant::String("Type your initial: ".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print".to_string(),
                args: vec![0],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            // Read character
            LirInstruction::CallExternal {
                dest: Some(1),
                func_name: "zulon_getchar".to_string(),
                args: vec![],
                arg_types: vec![],
                return_type: LirTy::I32,
            },
            // Print greeting start
            LirInstruction::Const {
                dest: 2,
                value: LirConstant::String("Hello, ".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print".to_string(),
                args: vec![2],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            // Print the character
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_putchar".to_string(),
                args: vec![1],
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },
            // Print exclamation and newline
            LirInstruction::Const {
                dest: 3,
                value: LirConstant::String("!\n".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print".to_string(),
                args: vec![3],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            // Return success
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
