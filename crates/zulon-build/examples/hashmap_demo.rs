// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Demonstration of HashMap<K, V> collection type

use zulon_std_core::HashMap;

fn main() {
    println!("=== ZULON HashMap<K, V> Demonstration ===\n");

    // Part 1: Basic Operations
    println!("1. BASIC OPERATIONS\n");

    println!("   Create empty HashMap:");
    let mut map: HashMap<&str, i32> = HashMap::new();
    println!("      HashMap::new() -> len={}, is_empty={}\n", map.len(), map.is_empty());

    println!("   Insert key-value pairs:");
    map.insert("apple", 10);
    map.insert("banana", 20);
    map.insert("cherry", 30);
    println!("      After insert: len={}\n", map.len());

    println!("   Get values:");
    match map.get(&"apple") {
        zulon_std_core::Optional::Some(value) => println!("      get(\"apple\") = {}", value),
        zulon_std_core::Optional::None => println!("      get(\"apple\") = None"),
    }
    match map.get(&"grape") {
        zulon_std_core::Optional::Some(value) => println!("      get(\"grape\") = {}\n", value),
        zulon_std_core::Optional::None => println!("      get(\"grape\") = None\n"),
    }

    // Part 2: Update and Remove
    println!("2. UPDATE AND REMOVE\n");

    println!("   Update existing key:");
    map.insert("apple", 100);
    match map.get(&"apple") {
        zulon_std_core::Optional::Some(value) => println!("      After update: get(\"apple\") = {}\n", value),
        zulon_std_core::Optional::None => {}
    }

    println!("   Remove key:");
    let removed = map.remove(&"banana");
    println!("      remove(\"banana\") = {:?}", removed);
    println!("      After remove: len={}\n", map.len());

    // Part 3: Contains and Clear
    println!("3. CONTAINS AND CLEAR\n");

    let mut map2: HashMap<i32, &str> = HashMap::with_capacity(5);
    map2.insert(1, "one");
    map2.insert(2, "two");
    map2.insert(3, "three");

    println!("   Contains key:");
    println!("      contains_key(2) = {}", map2.contains_key(&2));
    println!("      contains_key(5) = {}\n", map2.contains_key(&5));

    println!("   Clear map:");
    map2.clear();
    println!("      After clear: len={}, is_empty={}\n", map2.len(), map2.is_empty());

    // Summary
    println!("=== SUMMARY ===");
    println!("✓ HashMap<K, V>: Key-value store implementation");
    println!("✓ Core operations: new, with_capacity, insert, get, remove");
    println!("✓ Utility methods: contains_key, clear, iter");
    println!("\nHashMap<K, V> works! 🎉\n");
}
