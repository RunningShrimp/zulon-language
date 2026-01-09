// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON test runner CLI
//!
//! Executes tests discovered by the ZULON compiler

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use zulon_tools_yan::test_runner::TestRunner;

/// ZULON test runner
#[derive(Parser, Debug)]
#[command(name = "zulontest")]
#[command(author = "ZULON Language Team")]
#[command(version = "0.1.0")]
#[command(about = "Run ZULON tests", long_about = None)]
struct Args {
    /// Test metadata JSON files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut runner = TestRunner::new();

    // Load all test metadata files
    let mut total_loaded = 0;
    for input in &args.inputs {
        match runner.load_from_json(input) {
            Ok(count) => {
                if args.verbose {
                    println!("Loaded {} tests from {}", count, input.display());
                }
                total_loaded += count;
            }
            Err(e) => {
                eprintln!("Error loading {}: {}", input.display(), e);
                std::process::exit(1);
            }
        }
    }

    if total_loaded == 0 {
        eprintln!("No tests found in input files");
        std::process::exit(1);
    }

    if args.verbose {
        println!("Running {} tests...\n", total_loaded);
    }

    // Run all tests
    let summary = runner.run();

    // Exit with appropriate code
    if summary.is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some tests failed"))
    }
}
