// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build script for zulon-std-core
//!
//! Propagates the runtime C library link to dependents

fn main() {
    // The zulon-runtime-core crate uses `links = "zulon_entry"` which makes
    // its link directives available to us via DEP_ZULON_ENTRY_ variables
    //
    // We need to forward the link library to our dependents

    if let Ok(lib_path) = std::env::var("DEP_ZULON_ENTRY_ZULON_RUNTIME_CORE_LIB_PATH") {
        // Forward the library path
        println!("cargo:rustc-link-search=native={}", lib_path);
    }

    // Link the C runtime library
    println!("cargo:rustc-link-lib=static=zulon_entry");
}
