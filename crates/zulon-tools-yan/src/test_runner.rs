// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test runner for ZULON
//!
//! Discovers and runs tests marked with #[test]

#![allow(dead_code)]

use std::path::Path;

/// Test result
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// Test passed successfully
    Passed,
    /// Test failed with the given message
    Failed(String),
    /// Test was ignored
    Ignored,
}

/// Test metadata
#[derive(Debug, Clone)]
pub struct Test {
    /// Test name
    pub name: String,
    /// Module path
    pub module: String,
    /// Source file
    pub file: String,
    /// Line number
    pub line: usize,
}

/// Test runner
pub struct TestRunner {
    tests: Vec<Test>,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        TestRunner {
            tests: Vec::new(),
        }
    }

    /// Load tests from JSON metadata file
    pub fn load_from_json(&mut self, json_path: &Path) -> Result<usize, String> {
        use std::fs;

        let json_content = fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read JSON: {}", e))?;

        // Parse JSON - our format is array of test objects
        let parsed: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let count_before = self.tests.len();

        if let Some(arr) = parsed.as_array() {
            for test_obj in arr {
                if let Some(name) = test_obj.get("name").and_then(|v| v.as_str()) {
                    let ignored = test_obj.get("ignored")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    self.tests.push(Test {
                        name: name.to_string(),
                        module: String::new(),
                        file: json_path.to_string_lossy().to_string(),
                        line: 0,
                    });

                    if ignored {
                        // Skip ignored tests
                        self.tests.pop();
                    }
                }
            }
        }

        Ok(self.tests.len() - count_before)
    }

    /// Discover tests in a file (simplified version)
    pub fn discover_tests(&mut self, file: &Path) -> Result<usize, String> {
        // For MVP, we'll do a simple text-based search for #[test]
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let mut count = 0;
        let file_name = file.to_string_lossy().to_string();

        // Simple heuristic: look for "fn test_" or "#[test]" followed by "fn"
        for (line_num, line) in content.lines().enumerate() {
            if line.contains("#[test]") || line.contains("fn test_") {
                // Extract function name
                if let Some(start) = line.find("fn ") {
                    let rest = &line[start + 3..];
                    if let Some(end) = rest.find('(') {
                        let name = rest[..end].trim().to_string();
                        if !name.is_empty() {
                            self.tests.push(Test {
                                name,
                                module: "".to_string(),
                                file: file_name.clone(),
                                line: line_num + 1,
                            });
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Discover tests in multiple files
    pub fn discover_tests_in_files(&mut self, files: &[&Path]) -> Result<usize, String> {
        let mut total = 0;
        for file in files {
            total += self.discover_tests(file)?;
        }
        Ok(total)
    }

    /// Run all tests
    pub fn run(&self) -> TestSummary {
        let mut summary = TestSummary::new();

        if self.tests.is_empty() {
            println!("running 0 tests");
            println!();
            println!("test result: OK. 0 passed; 0 failed; 0 ignored");
            return summary;
        }

        println!("running {} tests", self.tests.len());
        println!();

        for test in &self.tests {
            print!("test {} ... ", test.name);

            // TODO: Actually run the test
            // For now, just mark as passed
            let result = self.run_single_test(test);

            match result {
                TestResult::Passed => {
                    println!("ok");
                    summary.passed += 1;
                }
                TestResult::Failed(msg) => {
                    println!("FAILED");
                    println!("  {}", msg);
                    summary.failed += 1;
                }
                TestResult::Ignored => {
                    println!("ignored");
                    summary.ignored += 1;
                }
            }
        }

        println!();
        println!("test result: {}. {} passed; {} failed; {} ignored",
            if summary.failed > 0 { "FAILED" } else { "OK" },
            summary.passed,
            summary.failed,
            summary.ignored
        );

        summary
    }

    /// Run a single test
    fn run_single_test(&self, test: &Test) -> TestResult {
        use std::process::Command;
        use std::path::Path;

        // The JSON file is like: "test_with_asserts.test.json"
        // The test source is: "test_with_asserts.zl"
        // The test main is: "test_with_asserts.test_main.zl"
        // The executable will be: "test_with_asserts"

        // Derive paths from JSON path
        let json_path = Path::new(&test.file);
        let base_name = json_path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".test.json"))
            .unwrap_or("test");

        let parent_dir = json_path
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let test_source = parent_dir.join(format!("{}.zl", base_name));
        let test_main = parent_dir.join(format!("{}.test_main.zl", base_name));
        let exe_path = parent_dir.join(base_name);

        // Auto-compile test if executable doesn't exist
        if !exe_path.exists() {
            if !test_source.exists() {
                return TestResult::Failed(format!(
                    "Test source not found: {}",
                    test_source.display()
                ));
            }

            if !test_main.exists() {
                return TestResult::Failed(format!(
                    "Test main not found: {}. Did you run the compiler?",
                    test_main.display()
                ));
            }

            // Concatenate source files and compile as single unit
            // This is simpler than merging LLVM IR and avoids scope issues
            println!("    📦 Merging test sources...");
            let test_source_content = match std::fs::read_to_string(&test_source) {
                Ok(s) => s,
                Err(e) => return TestResult::Failed(format!("Failed to read test source: {}", e)),
            };
            let test_main_content = match std::fs::read_to_string(&test_main) {
                Ok(s) => s,
                Err(e) => return TestResult::Failed(format!("Failed to read test main: {}", e)),
            };

            // Remove the extern fn printf from test_main (it's already in test source)
            // Also remove comments and empty lines before main
            let test_main_lines: Vec<&str> = test_main_content.lines()
                .skip_while(|line| {
                    let trimmed = line.trim();
                    trimmed.is_empty()
                        || trimmed.starts_with("//")
                        || trimmed.starts_with("extern fn")
                })
                .collect();

            let merged_source = format!("{}\n\n// Merged test main\n{}",
                test_source_content,
                test_main_lines.join("\n")
            );

            // Use a hidden directory to avoid test discovery picking up our files
            let temp_dir = parent_dir.join(".zulon_test_cache");
            let _ = std::fs::create_dir_all(&temp_dir);
            let merged_source_file = temp_dir.join(format!("{}.merged.zl", base_name));
            if let Err(e) = std::fs::write(&merged_source_file, merged_source) {
                return TestResult::Failed(format!("Failed to write merged source: {}", e));
            }

            // Compile merged source to LLVM IR
            println!("    📦 Compiling merged source to LLVM IR...");
            let merged_ll = temp_dir.join(format!("zulon_test_{}.ll", base_name));
            let result1 = Command::new("cargo")
                .args(["run", "-p", "zulon-compiler", "--"])
                .arg("-o").arg(&merged_ll)
                .arg(&merged_source_file)
                .output();

            let success1 = result1.as_ref()
                .map(|r| r.status.success())
                .unwrap_or(false);
            if !success1 {
                let stderr = result1.as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .or_else(|| result1.as_ref().ok().map(|r| {
                        String::from_utf8_lossy(&r.stderr).to_string()
                    }))
                    .unwrap_or_default();
                let stdout = result1.as_ref()
                    .ok()
                    .map(|r| String::from_utf8_lossy(&r.stdout).to_string())
                    .unwrap_or_default();
                // DON'T clean up merged source - keep it for debugging
                // let _ = std::fs::remove_file(&merged_source_file);
                return TestResult::Failed(format!(
                    "Failed to compile merged source:\nSTDOUT:\n{}\nSTDERR:\n{}",
                    stdout, stderr
                ));
            }

            // Compile LLVM IR to assembly
            println!("    📦 Compiling to assembly...");
            let merged_s = temp_dir.join(format!("zulon_test_{}.s", base_name));
            let result2 = Command::new("llc")
                .arg(&merged_ll)
                .arg("-o").arg(&merged_s)
                .output();

            let success2 = result2.as_ref()
                .map(|r| r.status.success())
                .unwrap_or(false);
            if !success2 {
                let stderr = result2.as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .or_else(|| result2.as_ref().ok().map(|r| {
                        String::from_utf8_lossy(&r.stderr).to_string()
                    }))
                    .unwrap_or_else(|| "Unknown error".to_string());
                let _ = std::fs::remove_file(&merged_source_file);
                let _ = std::fs::remove_file(&merged_ll);
                return TestResult::Failed(format!("Failed to compile LLVM IR to assembly: {}", stderr));
            }

            // Assemble and link with runtime
            println!("    📦 Linking executable...");
            let result3 = Command::new("clang")
                .arg(&merged_s)
                .arg("-o").arg(&exe_path)
                .arg("-O2")
                .output();

            let success3 = result3.as_ref()
                .map(|r| r.status.success())
                .unwrap_or(false);
            if !success3 {
                let stderr = result3.as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .or_else(|| result3.as_ref().ok().map(|r| {
                        String::from_utf8_lossy(&r.stderr).to_string()
                    }))
                    .unwrap_or_default();
                // Clean up intermediate files
                let _ = std::fs::remove_file(&merged_source_file);
                let _ = std::fs::remove_file(&merged_ll);
                let _ = std::fs::remove_file(&merged_s);
                return TestResult::Failed(format!("Failed to link executable: {}", stderr));
            }

            // Clean up intermediate files (keep for now for debugging)
            // let _ = std::fs::remove_file(&merged_source_file);
            // let _ = std::fs::remove_file(&merged_ll);
            // let _ = std::fs::remove_file(&merged_s);
            // let _ = std::fs::remove_dir(&temp_dir);

            println!("    ✅ Build complete: {}", exe_path.display());
        }

        // Convert to absolute path to ensure Command can find it
        let exe_path = exe_path.canonicalize().unwrap_or_else(|_| exe_path.clone());

        // Execute the test
        let output = Command::new(&exe_path)
            .output();

        match output {
            Ok(result) => {
                let exit_code = result.status.code().unwrap_or(-1);

                if exit_code == 0 {
                    TestResult::Passed
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    let stdout = String::from_utf8_lossy(&result.stdout);

                    let msg = if !stderr.is_empty() {
                        stderr.to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        format!("Exit code: {}", exit_code)
                    };

                    TestResult::Failed(msg)
                }
            }
            Err(e) => {
                TestResult::Failed(format!("Failed to execute test: {}", e))
            }
        }
    }

    /// Get number of discovered tests
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Test summary
#[derive(Debug, Clone)]
pub struct TestSummary {
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of ignored tests
    pub ignored: usize,
}

impl TestSummary {
    /// Create a new test summary
    pub fn new() -> Self {
        TestSummary {
            passed: 0,
            failed: 0,
            ignored: 0,
        }
    }

    /// Get total number of tests
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.ignored
    }

    /// Check if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new();
        assert_eq!(runner.tests.len(), 0);
    }

    #[test]
    fn test_summary_creation() {
        let summary = TestSummary::new();
        assert_eq!(summary.total(), 0);
        assert!(summary.is_success());
    }

    #[test]
    fn test_summary_with_results() {
        let mut summary = TestSummary::new();
        summary.passed = 5;
        summary.failed = 1;
        summary.ignored = 1;

        assert_eq!(summary.total(), 7);
        assert!(!summary.is_success());
    }

    #[test]
    fn test_empty_runner() {
        let runner = TestRunner::new();
        assert_eq!(runner.test_count(), 0);

        let summary = runner.run();
        assert_eq!(summary.total(), 0);
        assert!(summary.is_success());
    }
}
