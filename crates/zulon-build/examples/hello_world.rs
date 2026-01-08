// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hello World Example
//!
//! This example demonstrates string literal support by printing
//! the classic "Hello, World!" message.

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON Hello World Example\n");

    // Create external function declaration for string printing
    let print_external = LirExternal {
        name: "zulon_print".to_string(),
        param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))], // i8* for C strings
        return_type: LirTy::Unit,
    };

    // Create function that prints "Hello, World!"
    let hello_function = create_hello_world_function();

    // Create build configuration
    // Note: opt_level defaults to 2 (-O2) for production performance
    let config = BuildConfig {
        output: "hello_world".into(),
        keep_intermediates: true,
        ..Default::default()  // Use default opt_level: 2
    };

    println!("📦 Building executable...");

    // Create build pipeline
    let mut pipeline = BuildPipeline::new(config);
    pipeline.add_externals(vec![print_external]);
    pipeline.add_functions(vec![hello_function]);

    // Build!
    match pipeline.build() {
        Ok(exe_path) => {
            println!("✅ Build successful!");
            println!("   Executable: {}", exe_path.display());
            println!();
            println!("💡 Run it with: ./hello_world");
            println!("   Expected output: Hello, World!");
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

/// Create a function that prints "Hello, World!"
fn create_hello_world_function() -> LirFunction {
    let mut func = LirFunction {
        name: "zulon_main".to_string(),
        params: vec![],
        param_types: vec![],
        return_type: LirTy::I32,
        blocks: HashMap::new(),
        entry_block: 0,
        next_id: 1,
        next_vreg: 0,
        external_funcs: vec!["zulon_print".to_string()],
    };

    // Create a block that prints the message
    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // Load string constant "Hello, World!"
            LirInstruction::Const {
                dest: 0,
                value: LirConstant::String("Hello, World!".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            // Print the string
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_print".to_string(),
                args: vec![0],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
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
