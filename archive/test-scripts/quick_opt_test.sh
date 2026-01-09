#!/bin/bash
# Quick optimization test - modify and rebuild

echo "=== Quick Optimization Level Test ==="
echo ""

# Create test directory
mkdir -p opt_test
cd opt_test

# Create a simple test program
cat > test.rs << 'RUST'
use std::collections::HashMap;
use zulon_build::{BuildConfig, BuildPipeline};
use zulon_lir::{LirBlock, LirConstant, LirFunction, LirInstruction, LirTerminator, LirTy};

fn main() {
    println!("Testing optimization levels...");
    
    for opt_level in 0..=3 {
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

        let block = LirBlock {
            id: 0,
            phi_nodes: HashMap::new(),
            instructions: vec![],
            terminator: Some(LirTerminator::Return(Some(0))),
        };

        func.blocks.insert(0, block);

        let config = BuildConfig {
            output: format!("test_opt{}", opt_level),
            keep_intermediates: true,
            opt_level: opt_level,
            target: None,
        };

        let mut pipeline = BuildPipeline::new(config);
        pipeline.add_functions(vec![func]);

        print!("  -O{}: ", opt_level);
        match pipeline.build() {
            Ok(path) => {
                // Get file size
                let size = std::fs::metadata(&path).unwrap().len();
                println!("✅ {} bytes", size);
                
                // Benchmark runtime (5 runs)
                let mut total = 0u128;
                for _ in 0..5 {
                    let start = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    
                    std::process::Command::new(&path)
                        .output()
                        .expect("Failed to execute");
                    
                    let end = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    
                    total += (end - start) / 1_000_000; // Convert to ms
                }
                let avg = total / 5;
                println!("      Avg runtime: {}ms", avg);
            }
            Err(e) => {
                println!("❌ {}", e);
            }
        }
    }
}
RUST

# Compile and run the test
echo "Building test program..."
cargo build --release --example test 2>&1 | grep -E "(Compiling|Finished)" || true

echo ""
echo "Running optimization benchmark..."
echo ""
cargo run --release --example test 2>&1

cd ..
echo ""
echo "✅ Test complete! Check binary sizes in opt_test/"
