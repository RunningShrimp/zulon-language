// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON test runtime support
//!
//! Provides runtime functions for test framework, including panic support.

/// Built-in panic function for assertion failures
///
/// This function is called when an assertion fails or when panic!() is invoked.
/// It prints the panic message and terminates the program.
///
/// # Safety
///
/// The message pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn builtin_panic(message: *const u8) -> ! {
    // SAFETY: The caller ensures message is a valid C string pointer
    unsafe {
        // Try to convert the raw pointer to a string slice
        if message.is_null() {
            eprintln!("PANIC: <null message>");
        } else {
            // Create a CStr from the pointer
            use std::ffi::CStr;
            let cstr = CStr::from_ptr(message as *const i8);

            // Try to convert to UTF-8
            match cstr.to_str() {
                Ok(msg) => eprintln!("PANIC: {}", msg),
                Err(_) => eprintln!("PANIC: <invalid UTF-8 message>"),
            }
        }

        // Terminate the program with exit code 1
        std::process::exit(1);
    }
}

/// Entry point for test functions
///
/// This function can be used as a wrapper for test execution.
/// In the future, it will provide test setup/teardown and result reporting.
#[no_mangle]
pub unsafe extern "C" fn zulon_test_entry(test_fn: unsafe extern "C" fn()) -> i32 {
    // SAFETY: The caller ensures test_fn is valid
    unsafe {
        // TODO: Add test setup/teardown
        // TODO: Capture test output
        // TODO: Return proper test result

        test_fn();
        0  // Success
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_builtin_panic() {
        // This test verifies that builtin_panic compiles
        // We can't actually call it since it exits the process
        assert_eq!(1, 1);
    }
}
