// Async Runtime Demo
//
// This example demonstrates how to use the ZULON async runtime
// from Rust code. This is for testing purposes - in production,
// ZULON code would use the effect system directly.

use std::time::Duration;
use zulon_async_runtime::{AsyncOperation, RuntimeBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ZULON Async Runtime Demo");
    println!("========================\n");

    // Create the runtime
    let mut runtime = RuntimeBuilder::new().build()?;

    println!("✅ Runtime created successfully\n");

    // Test 1: Sleep operation
    println!("Test 1: Sleep Operation");
    println!("---------------------");

    let start = std::time::Instant::now();
    let result = runtime.event_loop_mut().submit(AsyncOperation::Sleep {
        duration_ms: 100,
    });
    let elapsed = start.elapsed();

    match result {
        Ok(_) => println!("✅ Sleep completed in {:?}", elapsed),
        Err(e) => println!("❌ Sleep failed: {}", e),
    }

    // Test 2: Unsupported operation (should fail gracefully)
    println!("\nTest 2: Unsupported Operation");
    println!("-----------------------------");

    let result = runtime.event_loop_mut().submit(AsyncOperation::FileRead {
        path: "/tmp/test.txt".to_string(),
    });

    match result {
        Ok(_) => println!("❌ Should have failed!"),
        Err(e) => println!("✅ Correctly rejected unsupported operation: {}", e),
    }

    // Test 3: Platform info
    println!("\nTest 3: Platform Information");
    println!("--------------------------");

    println!("Target OS: {}", std::env::consts::OS);
    println!("Target Arch: {}", std::env::consts::ARCH);
    println!("Event Loop: {}", runtime.event_loop().is_empty());

    println!("\n✅ All tests completed!");

    Ok(())
}
