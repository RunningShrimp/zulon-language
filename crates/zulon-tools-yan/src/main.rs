// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! YAN - ZULON Language Package Manager
//!
//! The yan tool provides a command-line interface for managing ZULON projects,
//! including building, running, and creating new projects.

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::Path;

mod build;
mod test_runner;

#[derive(Parser)]
#[command(name = "yan")]
#[command(about = "ZULON Language Package Manager", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a ZULON project
    Build {
        /// Build in release mode (optimized)
        #[arg(short, long)]
        release: bool,

        /// Package to build
        #[arg(short, long)]
        package: Option<String>,

        /// Number of parallel jobs
        #[arg(short, long, default_value = "4")]
        jobs: usize,

        /// Build an example
        #[arg(long)]
        example: Option<String>,
    },

    /// Run a ZULON project or example
    Run {
        /// Binary to run
        #[arg(short, long)]
        bin: Option<String>,

        /// Example to run
        #[arg(long)]
        example: Option<String>,

        /// Arguments to pass to the binary
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// Run in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Create a new ZULON project
    New {
        /// Project name
        name: String,

        /// Path to create the project
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Clean build artifacts
    Clean {
        /// Clean all artifacts
        #[arg(short, long)]
        all: bool,

        /// Package to clean
        #[arg(short, long)]
        package: Option<String>,
    },

    /// Run tests
    Test {
        /// Test filter (run only tests matching pattern)
        #[arg(short, long)]
        filter: Option<String>,

        /// Show test output
        #[arg(short, long)]
        verbose: bool,

        /// Compile in release mode
        #[arg(short, long)]
        release: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { release, package, jobs, example } => {
            build::check_project_dir()?;

            if let Some(ex) = example {
                build::build_example(&ex, release)?;
            } else {
                build::build_project(release, package.as_deref(), jobs)?;
            }
            Ok(())
        }

        Commands::Run { bin, example, args, release } => {
            println!("🚀 Running ZULON project...");

            if let Some(ex) = example {
                println!("   Example: {}", ex);
                run_example(&ex, &args, release)?;
            } else if let Some(b) = bin {
                println!("   Binary: {}", b);
                run_binary(&b, &args, release)?;
            } else {
                run_binary(&get_default_binary()?, &args, release)?;
            }

            Ok(())
        }

        Commands::New { name, path } => {
            create_project(&name, path.as_deref())?;
            Ok(())
        }

        Commands::Clean { all, package } => {
            clean_project(all, package.as_deref())?;
            Ok(())
        }

        Commands::Test { filter, verbose, release } => {
            run_tests(filter, verbose, release)?;
            Ok(())
        }
    }
}

/// Run a built binary
fn run_binary(bin: &str, args: &[String], release: bool) -> Result<()> {
    let profile = if release { "release" } else { "debug" };

    let binary_path = format!("target/{}/{}", profile, bin);

    println!("   Running: {}", binary_path);
    if !args.is_empty() {
        println!("   Args: {:?}", args);
    }
    println!();

    if !std::path::Path::new(&binary_path).exists() {
        return Err(anyhow::anyhow!(
            "Binary not found: {}. Run `yan build` first.", binary_path
        ));
    }

    let status = std::process::Command::new(&binary_path)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run binary: {}", binary_path))?;

    if status.success() {
        println!("✅ Run complete!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Run failed with exit code: {:?}", status.code()))
    }
}

/// Run an example
fn run_example(example: &str, args: &[String], release: bool) -> Result<()> {
    let profile = if release { "release" } else { "debug" };

    let example_path = format!("target/{}/examples/{}", profile, example);

    println!("   Running: {}", example_path);
    if !args.is_empty() {
        println!("   Args: {:?}", args);
    }
    println!();

    if !std::path::Path::new(&example_path).exists() {
        return Err(anyhow::anyhow!(
            "Example not found: {}. Run `yan build --example {}` first.",
            example_path, example
        ));
    }

    let status = std::process::Command::new(&example_path)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run example: {}", example_path))?;

    if status.success() {
        println!("✅ Run complete!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Run failed with exit code: {:?}", status.code()))
    }
}

/// Get the default binary name from Cargo.toml
fn get_default_binary() -> Result<String> {
    use std::fs::read_to_string;

    let cargo_toml = read_to_string("Cargo.toml")
        .with_context(|| "Failed to read Cargo.toml".to_string())?;

    // Simple parsing to find package name
    for line in cargo_toml.lines() {
        if line.trim().starts_with("name = ") {
            let name = line.split("=\"").nth(1)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse package name"))?;
            let name = name.split("\"").next()
                .ok_or_else(|| anyhow::anyhow!("Failed to parse package name"))?;
            return Ok(name.replace("-", "_"));
        }
    }

    // Fallback: use directory name
    let current_dir = std::env::current_dir()
        .with_context(|| "Failed to get current directory".to_string())?;
    let dir_name = current_dir.file_name()
        .ok_or_else(|| anyhow::anyhow!("Failed to get directory name"))?;
    let name = dir_name.to_string_lossy().replace("-", "_");
    Ok(name)
}

/// Create a new ZULON project
fn create_project(name: &str, path: Option<&str>) -> Result<()> {
    let project_path = path.unwrap_or(name);

    println!("📦 Creating new ZULON project: {}", name);
    println!("   Path: {}", project_path);
    println!();

    // Check if directory already exists
    if std::path::Path::new(project_path).exists() {
        return Err(anyhow::anyhow!(
            "Directory '{}' already exists", project_path
        ));
    }

    // Create project directory
    std::fs::create_dir_all(project_path)
        .with_context(|| format!("Failed to create project directory: {}", project_path))?;

    // Create src directory
    std::fs::create_dir_all(format!("{}/src", project_path))
        .with_context(|| format!("Failed to create src directory: {}/src", project_path))?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]

# ZULON dependencies
zulon-std-core = {{ path = "../zulon-std-core" }}

[[bin]]
name = "{}"
path = "src/main.zl"
"#,
        name, name.replace("-", "_")
    );

    std::fs::write(
        format!("{}/Cargo.toml", project_path),
        cargo_toml,
    ).with_context(|| format!("Failed to write Cargo.toml for {}", project_path))?;

    // Create a sample main.zl
    let main_zl = format!(
        r#"// {} - A ZULON Project

fn main() {{
    println!("Hello, ZULON!");
    println!("Welcome to your new project: {}");
}}
"#,
        name, name
    );

    std::fs::write(
        format!("{}/src/main.zl", project_path),
        main_zl,
    ).with_context(|| format!("Failed to write main.zl for {}", project_path))?;

    // Create .gitignore
    let gitignore = r#"# ZULON build artifacts
target/
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~
"#;

    std::fs::write(
        format!("{}/.gitignore", project_path),
        gitignore,
    ).with_context(|| format!("Failed to write .gitignore for {}", project_path))?;

    // Create README.md
    let readme = format!(
        r#"# {}

A ZULON programming language project.

## Getting Started

Build the project:
```bash
yan build
```

Run the project:
```bash
yan run
```

## Learning ZULON

Check out the ZULON documentation for more information about the language.
"#,
        name
    );

    std::fs::write(
        format!("{}/README.md", project_path),
        readme,
    ).with_context(|| format!("Failed to write README.md for {}", project_path))?;

    println!("✅ Project created successfully!");
    println!();
    println!("Next steps:");
    println!("  cd {}", project_path);
    println!("  yan build");
    println!("  yan run");

    Ok(())
}

/// Clean build artifacts
fn clean_project(all: bool, package: Option<&str>) -> Result<()> {
    println!("🧹 Cleaning build artifacts...");

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("clean");

    if all {
        println!("   Cleaning all artifacts");
    }

    if let Some(pkg) = package {
        cmd.arg("-p").arg(pkg);
        println!("   Package: {}", pkg);
    }

    println!();

    let status = cmd
        .status()
        .with_context(|| "Failed to execute cargo clean".to_string())?;

    if status.success() {
        println!("✅ Clean complete!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Clean failed with exit code: {:?}", status.code()))
    }
}

/// Run tests
fn run_tests(filter: Option<String>, verbose: bool, _release: bool) -> Result<()> {
    println!("🧪 Running tests...");
    if let Some(f) = &filter {
        println!("   Filter: {}", f);
    }
    println!();

    // Find all .test.json files in the current directory and subdirectories
    let mut test_files = Vec::new();
    find_test_files(Path::new("."), &mut test_files)?;

    if test_files.is_empty() {
        println!("No test metadata files found (.test.json)");
        println!("Make sure to compile your test files first:");
        println!("  cargo run --package zulon-compiler -- your_test.zl");
        return Ok(());
    }

    // Load test metadata
    let mut runner = test_runner::TestRunner::new();
    let mut total_loaded = 0;

    for test_file in &test_files {
        match runner.load_from_json(test_file) {
            Ok(count) => {
                total_loaded += count;
                if verbose {
                    println!("Loaded {} tests from {}", count, test_file.display());
                }
            }
            Err(e) => {
                eprintln!("Error loading {}: {}", test_file.display(), e);
            }
        }
    }

    if total_loaded == 0 {
        println!("No tests found");
        return Ok(());
    }

    println!("Running {} tests...\n", total_loaded);

    // Run all tests
    let summary = runner.run();

    // Exit with appropriate code
    if summary.is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some tests failed"))
    }
}

/// Find all .test.json files in a directory recursively
fn find_test_files(dir: &Path, results: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and common build directories
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }
            find_test_files(&path, results)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if name_str.ends_with(".test.json") {
                        results.push(path);
                    }
                }
            }
        }
    }

    Ok(())
}
