// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

use zulon_std_core::{Optional, Outcome};

fn main() {
    println!("=== ZULON Standard Library Core Demo ===\n");

    // Part 1: Optional
    println!("1. OPTIONAL<T> - Optional Values\n");

    let some = Optional::Some(42);
    let none = Optional::None::<i32>;
    println!("   Create: Some(42) = {:?}, None = {:?}\n", some, none);
    println!("   Check: is_some() = {}, is_none() = {}\n", some.is_some(), none.is_none());
    println!("   Unwrap: Some(42).unwrap() = {}", Optional::Some(42).unwrap());
    println!("   Unwrap: None.unwrap_or(100) = {}\n", none.unwrap_or(100));
    println!("   Map: Some(42).map(|x| x * 2) = {:?}\n", some.map(|x| x * 2));

    // Part 2: Outcome  
    println!("2. OUTCOME<T, E> - Error Handling\n");

    let ok: Outcome<i32, &str> = Outcome::Ok(42);
    let err = Outcome::Err::<i32, &str>("error");
    println!("   Create: Ok(42) = {:?}, Err(\"error\") = {:?}\n", ok, err);
    println!("   Check: is_ok() = {}, is_err() = {}\n", ok.is_ok(), err.is_err());
    println!("   Unwrap: Ok(42).unwrap() = {}", Outcome::<i32, &str>::Ok(42).unwrap());
    println!("   Unwrap: Err(\"error\").unwrap_or(100) = {}\n", err.unwrap_or(100));
    println!("   Map: Ok(42).map(|x| x * 2) = {:?}\n", ok.map(|x| x * 2));

    // Part 3: Real-World Example
    println!("3. REAL-WORLD EXAMPLE: Safe Division\n");

    fn safe_divide(a: i32, b: i32) -> Outcome<i32, String> {
        if b == 0 {
            Outcome::Err(format!("Cannot divide {} by {}", a, b))
        } else {
            Outcome::Ok(a / b)
        }
    }

    match safe_divide(100, 5) {
        Outcome::Ok(result) => println!("   100 / 5 = {}", result),
        Outcome::Err(e) => println!("   Error: {}", e),
    }

    match safe_divide(100, 0) {
        Outcome::Ok(result) => println!("   100 / 0 = {}", result),
        Outcome::Err(e) => println!("   Error: {}", e),
    }

    println!("\n=== SUMMARY ===");
    println!("✓ Optional<T>: Type-safe optional values");
    println!("✓ Outcome<T, E>: Error handling");
    println!("\nZULON Standard Library Core works! 🎉\n");
}
