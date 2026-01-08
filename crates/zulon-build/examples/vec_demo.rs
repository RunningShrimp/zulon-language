// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Demonstration of Vec<T> collection type

use zulon_std_core::Vec;

fn main() {
    println!("=== ZULON Vec<T> Demonstration ===\n");

    // Part 1: Basic Operations
    println!("1. BASIC OPERATIONS\n");

    println!("   Create empty Vec:");
    let mut vec: Vec<i32> = Vec::new();
    println!("      Vec::new() -> capacity={}, len={}\n", vec.capacity(), vec.len());

    println!("   Push elements:");
    vec.push(10);
    vec.push(20);
    vec.push(30);
    println!("      After push(10, 20, 30): len={}\n", vec.len());

    println!("   Pop element:");
    let popped = vec.pop();
    println!("      pop() = {:?}\n", popped);

    println!("   Reserve capacity:");
    vec.reserve(10);
    println!("      reserve(10) -> capacity={}\n", vec.capacity());

    // Part 2: Vec with Capacity
    println!("2. WITH CAPACITY\n");

    let mut vec2: Vec<i32> = Vec::with_capacity(5);
    println!("   Vec::with_capacity(5): capacity={}\n", vec2.capacity());

    vec2.push(1);
    vec2.push(2);
    vec2.push(3);
    println!("   After pushing 1, 2, 3: len={}, capacity={}\n", vec2.len(), vec2.capacity());

    // Part 3: Clear and Truncate
    println!("3. CLEAR AND TRUNCATE\n");

    let mut vec3: Vec<i32> = Vec::with_capacity(10);
    vec3.push(1);
    vec3.push(2);
    vec3.push(3);
    vec3.push(4);
    vec3.push(5);
    println!("   Initial: len={}", vec3.len());

    vec3.truncate(3);
    println!("   truncate(3): len={}", vec3.len());

    vec3.clear();
    println!("   clear(): len={}, is_empty={}\n", vec3.len(), vec3.is_empty());

    // Summary
    println!("=== SUMMARY ===");
    println!("✓ Vec<T>: Dynamic array implementation");
    println!("✓ Core operations: new, with_capacity, push, pop");
    println!("✓ Capacity management: reserve, clear, truncate");
    println!("\nVec<T> works! 🎉\n");
}
