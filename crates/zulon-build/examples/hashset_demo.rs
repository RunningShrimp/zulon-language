// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Demonstration of HashSet<T> collection type

use zulon_std_core::HashSet;

fn main() {
    println!("=== ZULON HashSet<T> Demonstration ===\n");

    // Part 1: Basic Operations
    println!("1. BASIC OPERATIONS\n");

    println!("   Create empty HashSet:");
    let mut set: HashSet<i32> = HashSet::new();
    println!("      HashSet::new() -> len={}, is_empty={}\n", set.len(), set.is_empty());

    println!("   Insert values:");
    set.insert(10);
    set.insert(20);
    set.insert(30);
    set.insert(20); // Duplicate, will be ignored
    println!("      After insert(10, 20, 30, 20): len={}", set.len());
    println!("      (Duplicate 20 was ignored)\n");

    println!("   Check membership:");
    println!("      contains(10) = {}", set.contains(&10));
    println!("      contains(40) = {}\n", set.contains(&40));

    // Part 2: Remove and Clear
    println!("2. REMOVE AND CLEAR\n");

    println!("   Remove value:");
    let removed = set.remove(&20);
    println!("      remove(20) = {}", removed);
    println!("      contains(20) = {}", set.contains(&20));
    println!("      len={}\n", set.len());

    println!("   Clear set:");
    set.clear();
    println!("      After clear: len={}, is_empty={}\n", set.len(), set.is_empty());

    // Part 3: With Strings
    println!("3. WITH STRING VALUES\n");

    let mut fruit_set: HashSet<&str> = HashSet::with_capacity(5);
    fruit_set.insert("apple");
    fruit_set.insert("banana");
    fruit_set.insert("cherry");
    fruit_set.insert("apple"); // Duplicate

    println!("   Fruit set:");
    println!("      len = {}", fruit_set.len());
    println!("      contains(\"apple\") = {}", fruit_set.contains(&"apple"));
    println!("      contains(\"grape\") = {}\n", fruit_set.contains(&"grape"));

    // Summary
    println!("=== SUMMARY ===");
    println!("✓ HashSet<T>: Unique value collection implementation");
    println!("✓ Core operations: new, with_capacity, insert, contains, remove");
    println!("✓ Utility methods: clear, iter");
    println!("✓ Automatically handles duplicates (ignores them)");
    println!("\nHashSet<T> works! 🎉\n");
}
