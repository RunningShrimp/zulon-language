// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build script for zulon-runtime-core
//!
//! Compiles the C runtime entry point

fn main() {
    // Compile the C entry point
    cc::Build::new()
        .file("c/zulon_entry.c")
        .compile("zulon_entry");

    // Tell cargo where to find the compiled library
    println!("cargo:rerun-if-changed=c/zulon_entry.c");

    // Get OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Emit link search path for this crate
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Link the compiled library for this crate
    println!("cargo:rustc-link-lib=static=zulon_entry");

    // Store the library path for dependent crates
    // We use a DEP_ZULON_RUNTIME_CORE_ prefix that Cargo will propagate
    println!("cargo:ZULON_RUNTIME_CORE_LIB_PATH={}", out_dir);
}
