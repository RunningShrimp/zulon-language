// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Demonstration of VecDeque<T> collection type

use zulon_std_core::VecDeque;

fn main() {
    println!("=== ZULON VecDeque<T> Demonstration ===\n");

    // Part 1: Basic Operations
    println!("1. BASIC OPERATIONS\n");

    println!("   Create empty VecDeque:");
    let mut deque: VecDeque<i32> = VecDeque::new();
    println!("      VecDeque::new() -> len={}, is_empty={}\n", deque.len(), deque.is_empty());

    println!("   Push to back:");
    deque.push_back(10);
    deque.push_back(20);
    deque.push_back(30);
    println!("      push_back(10, 20, 30): len={}\n", deque.len());

    println!("   Push to front:");
    deque.push_front(5);
    println!("      push_front(5): len={}", deque.len());
    match deque.front() {
        zulon_std_core::Optional::Some(val) => println!("      front() = {}", val),
        zulon_std_core::Optional::None => {},
    }
    match deque.back() {
        zulon_std_core::Optional::Some(val) => println!("      back() = {}\n", val),
        zulon_std_core::Optional::None => {},
    }

    // Part 2: Pop Operations
    println!("2. POP OPERATIONS\n");

    println!("   Pop from back:");
    let popped_back = deque.pop_back();
    println!("      pop_back() = {:?}", popped_back);
    println!("      len={}\n", deque.len());

    println!("   Pop from front:");
    let popped_front = deque.pop_front();
    println!("      pop_front() = {:?}", popped_front);
    match deque.front() {
        zulon_std_core::Optional::Some(val) => println!("      front() now = {}\n", val),
        zulon_std_core::Optional::None => {},
    }

    // Part 3: Access and Clear
    println!("3. ACCESS AND CLEAR\n");

    let mut deque2: VecDeque<i32> = VecDeque::new();
    deque2.push_back(100);
    deque2.push_back(200);
    deque2.push_back(300);

    println!("   Get by index:");
    match deque2.get(1) {
        zulon_std_core::Optional::Some(val) => println!("      get(1) = {}", val),
        zulon_std_core::Optional::None => {},
    }

    println!("\n   Clear deque:");
    deque2.clear();
    println!("      After clear: len={}, is_empty={}\n", deque2.len(), deque2.is_empty());

    // Summary
    println!("=== SUMMARY ===");
    println!("✓ VecDeque<T>: Double-ended queue implementation");
    println!("✓ Front operations: push_front(), pop_front(), front()");
    println!("✓ Back operations: push_back(), pop_back(), back()");
    println!("✓ Random access: get(index)");
    println!("✓ Note: MVP uses Vec internally (O(n) for front ops)");
    println!("✓ TODO: Implement ring buffer for O(1) front operations");
    println!("\nVecDeque<T> works! 🎉\n");
}
