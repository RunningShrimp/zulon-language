// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! String Utilities Demonstration
//!
//! This example demonstrates string utility functions:
//! - zulon_strlen: Get string length
//! - zulon_strcmp: Compare two strings

use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{
    LirBlock, LirConstant, LirExternal, LirFunction, LirInstruction, LirTerminator, LirTy,
};

fn main() {
    println!("🚀 ZULON String Utilities Demo\n");

    // Create external function declarations
    let externals = vec![
        // Output functions
        LirExternal {
            name: "zulon_println".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
            return_type: LirTy::Unit,
        },
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
        // String utility functions
        LirExternal {
            name: "zulon_strlen".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
            return_type: LirTy::ISize, // size_t
        },
        LirExternal {
            name: "zulon_strcmp".to_string(),
            param_types: vec![LirTy::Ptr(Box::new(LirTy::I8)), LirTy::Ptr(Box::new(LirTy::I8))],
            return_type: LirTy::I32,
        },
    ];

    // Create function that demonstrates string utilities
    let demo_function = create_demo_function();

    // Create build configuration
    let config = BuildConfig {
        output: "string_utils_demo".into(),
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
            println!("💡 Run it with: ./string_utils_demo");
            println!();
            println!("   The program will:");
            println!("   1. Calculate the length of 'Hello, World!'");
            println!("   2. Compare 'apple' and 'banana'");
            println!("   3. Compare 'apple' and 'apple'");
            println!();
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

/// Create a function that demonstrates string utilities
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

    let block = LirBlock {
        id: 0,
        phi_nodes: HashMap::new(),
        instructions: vec![
            // ===== Test strlen =====
            // Load string "Hello, World!"
            LirInstruction::Const {
                dest: 0,
                value: LirConstant::String("Hello, World!".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            // Get its length
            LirInstruction::CallExternal {
                dest: Some(1),
                func_name: "zulon_strlen".to_string(),
                args: vec![0],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::ISize,
            },
            // Print "Length of 'Hello, World!': "
            LirInstruction::Const {
                dest: 2,
                value: LirConstant::String("Length of 'Hello, World!': ".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println".to_string(),
                args: vec![2],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            // Print the length (as i64)
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println_i64".to_string(),
                args: vec![1],
                arg_types: vec![LirTy::I64],
                return_type: LirTy::Unit,
            },

            // ===== Test strcmp (different strings) =====
            // Load string "apple"
            LirInstruction::Const {
                dest: 3,
                value: LirConstant::String("apple".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            // Load string "banana"
            LirInstruction::Const {
                dest: 4,
                value: LirConstant::String("banana".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            // Compare them
            LirInstruction::CallExternal {
                dest: Some(5),
                func_name: "zulon_strcmp".to_string(),
                args: vec![3, 4],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8)), LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::I32,
            },
            // Print result
            LirInstruction::Const {
                dest: 6,
                value: LirConstant::String("strcmp('apple', 'banana'): ".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println".to_string(),
                args: vec![6],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println_i32".to_string(),
                args: vec![5],
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },

            // ===== Test strcmp (same strings) =====
            // Load "test" twice
            LirInstruction::Const {
                dest: 7,
                value: LirConstant::String("test".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::Const {
                dest: 8,
                value: LirConstant::String("test".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            // Compare them
            LirInstruction::CallExternal {
                dest: Some(9),
                func_name: "zulon_strcmp".to_string(),
                args: vec![7, 8],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8)), LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::I32,
            },
            // Print result
            LirInstruction::Const {
                dest: 10,
                value: LirConstant::String("strcmp('test', 'test'): ".to_string()),
                ty: LirTy::Ptr(Box::new(LirTy::I8)),
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println".to_string(),
                args: vec![10],
                arg_types: vec![LirTy::Ptr(Box::new(LirTy::I8))],
                return_type: LirTy::Unit,
            },
            LirInstruction::CallExternal {
                dest: None,
                func_name: "zulon_println_i32".to_string(),
                args: vec![9],
                arg_types: vec![LirTy::I32],
                return_type: LirTy::Unit,
            },

            // Return success
            LirInstruction::Const {
                dest: 11,
                value: LirConstant::Integer(0),
                ty: LirTy::I32,
            },
        ],
        terminator: Some(LirTerminator::Return(Some(11))),
    };

    func.blocks.insert(0, block);
    func
}
