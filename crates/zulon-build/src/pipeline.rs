// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build pipeline for compiling ZULON programs

use crate::error::{BuildError, Result};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use zulon_codegen_llvm::CodeGenerator;
use zulon_lir::{LirExternal, LirFunction};

// Import runtime crate to trigger build
extern crate zulon_runtime_core;

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Output file path
    pub output: PathBuf,
    /// Whether to keep intermediate files
    pub keep_intermediates: bool,
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target: Option<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output: PathBuf::from("a.out"),
            keep_intermediates: false,
            // Default to -O2 for production-ready performance
            // Use -O0 for faster compilation during development
            opt_level: 2,
            target: None,
        }
    }
}

/// Build pipeline
pub struct BuildPipeline {
    config: BuildConfig,
    lir_functions: Vec<LirFunction>,
    lir_externals: Vec<LirExternal>,
}

impl BuildPipeline {
    /// Create a new build pipeline
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            lir_functions: Vec::new(),
            lir_externals: Vec::new(),
        }
    }

    /// Add LIR functions to compile
    pub fn add_functions(&mut self, functions: Vec<LirFunction>) {
        self.lir_functions.extend(functions);
    }

    /// Add external function declarations
    pub fn add_externals(&mut self, externals: Vec<LirExternal>) {
        self.lir_externals.extend(externals);
    }

    /// Run the build pipeline
    pub fn build(&mut self) -> Result<PathBuf> {
        // Step 1: Generate LLVM IR to .ll file
        let ll_file = self.generate_llvm_ir()?;

        // Step 2: Validate LLVM IR with llvm-as (optional but recommended)
        self.validate_llvm_ir(&ll_file)?;

        // Step 3: Compile to object file with llc
        let o_file = self.compile_to_object(&ll_file)?;

        // Step 4: Link to executable
        let exe_file = self.link_executable(&o_file)?;

        // Clean up intermediates if requested
        if !self.config.keep_intermediates {
            self.cleanup_intermediates(&ll_file, &o_file)?;
        }

        Ok(exe_file)
    }

    /// Step 1: Generate LLVM IR
    fn generate_llvm_ir(&mut self) -> Result<PathBuf> {
        let ll_file = self.config.output.with_extension("ll");

        // Generate LLVM IR to buffer
        let mut buffer = Vec::new();
        {
            let mut codegen = CodeGenerator::new(&mut buffer);
            codegen.generate_module_with_externals(&self.lir_functions, &self.lir_externals)
                .map_err(|e| BuildError::CodeGeneration(e.to_string()))?;
        }

        // Write to file
        let mut file = File::create(&ll_file)
            .map_err(|e| BuildError::Io(format!("Failed to create {}: {}", ll_file.display(), e)))?;

        file.write_all(&buffer)
            .map_err(|e| BuildError::Io(format!("Failed to write {}: {}", ll_file.display(), e)))?;

        Ok(ll_file)
    }

    /// Step 2: Validate LLVM IR with llvm-as
    fn validate_llvm_ir(&self, ll_file: &Path) -> Result<()> {
        let bc_file = ll_file.with_extension("bc");

        let status = Command::new("llvm-as")
            .arg(ll_file)
            .arg("-o")
            .arg(&bc_file)
            .output()
            .map_err(|e| BuildError::ToolNotFound(format!("llvm-as: {}", e)))?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            return Err(BuildError::LlvmAsFailed(stderr.to_string()));
        }

        // Remove the .bc file (it's just for validation)
        std::fs::remove_file(&bc_file)
            .map_err(|e| BuildError::Io(format!("Failed to remove {}: {}", bc_file.display(), e)))?;

        Ok(())
    }

    /// Step 3: Compile to object file with llc
    fn compile_to_object(&self, ll_file: &Path) -> Result<PathBuf> {
        let o_file = self.config.output.with_extension("o");

        let mut cmd = Command::new("llc");
        cmd.arg(ll_file)
            .arg("-filetype=obj")
            .arg("-o")
            .arg(&o_file);

        // Add optimization level
        cmd.arg(format!("-O{}", self.config.opt_level));

        // Add target triple if specified
        if let Some(ref target) = self.config.target {
            cmd.arg("-mtriple").arg(target);
        }

        let output = cmd
            .output()
            .map_err(|e| BuildError::ToolNotFound(format!("llc: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::LlcFailed(stderr.to_string()));
        }

        Ok(o_file)
    }

    /// Step 4: Link to executable
    fn link_executable(&self, o_file: &Path) -> Result<PathBuf> {
        let exe_file = &self.config.output;

        // Try different linkers
        let linkers = ["ld", "ld64", "lld", "gold"];

        let mut last_error = None;

        for linker in &linkers {
            let result = self.try_linker(linker, o_file, exe_file);

            // If linker not found, try next one
            if let Err(BuildError::ToolNotFound(_)) = &result {
                continue;
            }

            // If linking succeeded, return exe file
            if result.is_ok() {
                return Ok(exe_file.clone());
            }

            // Store error and try next linker
            last_error = Some(result.err().unwrap());
        }

        // All linkers failed
        Err(last_error.unwrap_or_else(|| BuildError::LinkerFailed(
            "No linker found".to_string(),
        )))
    }

    /// Try linking with a specific linker
    fn try_linker(&self, linker: &str, o_file: &Path, exe_file: &Path) -> Result<()> {
        let mut cmd = Command::new(linker);
        cmd.arg(o_file)
            .arg("-o")
            .arg(exe_file);

        // Add platform-specific linker flags first
        #[cfg(target_os = "macos")]
        {
            // Link against System library on macOS
            cmd.arg("-lSystem");
            cmd.arg("-syslibroot");
            cmd.arg("/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk");
        }

        #[cfg(target_os = "linux")]
        {
            cmd.args(["-dynamic-linker", "/lib64/ld-linux-x86-64.so.2"]);
        }

        // Add ZULON runtime library - dynamic discovery
        if let Some(runtime_lib) = self.find_runtime_library() {
            cmd.arg(&runtime_lib);
        } else {
            eprintln!("Warning: ZULON runtime library not found, linking may fail");
        }

        // Add system libraries on Linux
        #[cfg(target_os = "linux")]
        {
            cmd.arg("-lc").arg("-lm");
        }

        let output = cmd
            .output()
            .map_err(|e| BuildError::ToolNotFound(format!("{}: {}", linker, e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::LinkerFailed(format!(
                "{}: {}",
                linker, stderr
            )));
        }

        Ok(())
    }

    /// Find the ZULON runtime library
    fn find_runtime_library(&self) -> Option<String> {
        use std::path::PathBuf;

        // Get the current executable's directory to find the target directory
        if let Ok(exe_path) = std::env::current_exe() {
            // Path is like: target/debug/examples/print_call
            // We need to find: target/debug/build/zulon-runtime-core-*/out/libzulon_entry.a
            if let Some(target_pos) = exe_path.to_string_lossy().find("/target/") {
                let base_path = &exe_path.to_string_lossy()[..target_pos];
                let target_path = format!("{}/target", base_path);

                // Search for the library in debug/build
                let build_dir = PathBuf::from(&target_path).join("debug/build");
                if let Ok(entries) = std::fs::read_dir(&build_dir) {
                    for entry in entries.flatten() {
                        let lib_path = entry.path().join("out/libzulon_entry.a");
                        if lib_path.exists() {
                            return Some(lib_path.to_string_lossy().to_string());
                        }
                    }
                }

                // Also try release/build
                let build_dir = PathBuf::from(&target_path).join("release/build");
                if let Ok(entries) = std::fs::read_dir(&build_dir) {
                    for entry in entries.flatten() {
                        let lib_path = entry.path().join("out/libzulon_entry.a");
                        if lib_path.exists() {
                            return Some(lib_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Clean up intermediate files
    fn cleanup_intermediates(&self, ll_file: &Path, o_file: &Path) -> Result<()> {
        // Remove .ll file
        if ll_file.exists() {
            std::fs::remove_file(ll_file)
                .map_err(|e| BuildError::Io(format!("Failed to remove {}: {}", ll_file.display(), e)))?;
        }

        // Remove .o file
        if o_file.exists() {
            std::fs::remove_file(o_file)
                .map_err(|e| BuildError::Io(format!("Failed to remove {}: {}", o_file.display(), e)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use zulon_lir::LirTy;

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert_eq!(config.output, PathBuf::from("a.out"));
        assert_eq!(config.keep_intermediates, false);
        assert_eq!(config.opt_level, 2); // Default to -O2 for production
        assert!(config.target.is_none());
    }

    #[test]
    fn test_pipeline_creation() {
        let config = BuildConfig::default();
        let pipeline = BuildPipeline::new(config);
        assert_eq!(pipeline.lir_functions.len(), 0);
    }

    #[test]
    fn test_add_functions() {
        let config = BuildConfig::default();
        let mut pipeline = BuildPipeline::new(config);

        let func = create_simple_function();
        pipeline.add_functions(vec![func]);

        assert_eq!(pipeline.lir_functions.len(), 1);
    }

    fn create_simple_function() -> LirFunction {
        LirFunction {
            name: "test".to_string(),
            params: vec![],
            param_types: vec![],
            return_type: LirTy::I32,
            blocks: HashMap::new(),
            entry_block: 0,
            next_id: 1,
            next_vreg: 0,
            external_funcs: vec![],
        }
    }
}
