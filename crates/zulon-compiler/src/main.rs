// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ZULON compiler CLI

use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use zulon_compiler::{Compiler, CompilerConfig};

/// ZULON programming language compiler
#[derive(Parser, Debug)]
#[command(name = "zulonc")]
#[command(author = "ZULON Language Team")]
#[command(version = "0.1.0")]
#[command(about = "Compile ZULON source files to executables", long_about = None)]
struct Args {
    /// Input file to compile
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file path
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Optimization level (0-3, default: 2)
    #[arg(short = 'O', long, value_name = "LEVEL")]
    opt_level: Option<u8>,

    /// Keep intermediate files
    #[arg(long)]
    keep_intermediates: bool,

    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    #[arg(long, value_name = "TRIPLE")]
    target: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file not found: {}", args.input.display());
    }

    // Validate optimization level
    let opt_level = args.opt_level.unwrap_or(2);
    if opt_level > 3 {
        anyhow::bail!("Optimization level must be 0-3, got: {}", opt_level);
    }

    // Create compiler config
    let config = CompilerConfig {
        opt_level,
        output: args.output,
        keep_intermediates: args.keep_intermediates,
        target: args.target,
    };

    // Run compiler
    let compiler = Compiler::new(config);
    compiler.compile_file(&args.input)?;

    Ok(())
}
