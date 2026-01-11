// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test main function generation (ZULON source code)
//!
//! This module generates ZULON source code for a test main function
//! that calls all discovered test functions.

use crate::test_discovery::DiscoveredTest;

/// Generate ZULON source code for a test main function
///
/// This generates a simple `main()` function that:
/// 1. Declares extern printf
/// 2. Calls each test function
/// 3. Returns 0 on success
pub fn generate_test_main_source(tests: &[DiscoveredTest]) -> String {
    let mut source = String::new();
    
    // Add extern declarations
    source.push_str("// Auto-generated test main\n");
    source.push_str("extern fn printf(format: string, ...) -> i32;\n\n");
    
    // Generate main function
    source.push_str("fn main() -> i32 {\n");
    source.push_str(&format!("    printf(\"Running {} tests...\\n\");\n", tests.len()));
    
    for test in tests {
        if test.ignored {
            source.push_str(&format!("    printf(\"test {} ... IGNORED\\n\");\n", test.name));
        } else {
            source.push_str(&format!("    {}();\n", test.name));
            source.push_str(&format!("    printf(\"test {} ... ok\\n\");\n", test.name));
        }
    }
    
    source.push_str("    printf(\"\\nAll tests passed!\\n\");\n");
    source.push_str("    0\n");
    source.push_str("}\n");
    
    source
}

/// Generate a complete test file with main function
///
/// This takes the original test file content and appends a generated main function
pub fn generate_test_file(original_content: &str, tests: &[DiscoveredTest]) -> String {
    let mut result = original_content.to_string();
    
    // Append the generated test main
    result.push_str("\n// ===== AUTO-GENERATED TEST MAIN =====\n\n");
    result.push_str(&generate_test_main_source(tests));
    
    result
}
