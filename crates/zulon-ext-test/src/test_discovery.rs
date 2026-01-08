// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test discovery and metadata extraction
//!
//! This module handles finding tests in compiled code and extracting
//! their metadata (name, location, etc.)

/// Metadata about a test
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestMetadata {
    /// Test function name
    pub name: String,
    /// Module path (e.g., "my_module::tests")
    pub module: String,
    /// Source file path
    pub file: String,
    /// Line number in source file
    pub line: usize,
    /// Whether test should be ignored (#[ignore])
    pub ignored: bool,
}

impl TestMetadata {
    /// Create new test metadata
    pub fn new(
        name: String,
        module: String,
        file: String,
        line: usize,
        ignored: bool,
    ) -> Self {
        TestMetadata {
            name,
            module,
            file,
            line,
            ignored,
        }
    }

    /// Get the full test name (module + function)
    pub fn full_name(&self) -> String {
        format!("{}::{}", self.module, self.name)
    }

    /// Check if test should run
    pub fn should_run(&self) -> bool {
        !self.ignored
    }
}

/// Test discovery
///
/// Discovers all test functions in a compiled binary or source code
pub struct TestDiscovery {
    /// Discovered tests
    tests: Vec<TestMetadata>,
}

impl TestDiscovery {
    /// Create a new test discovery instance
    pub fn new() -> Self {
        TestDiscovery {
            tests: Vec::new(),
        }
    }

    /// Discover tests from source code
    ///
    /// This is a placeholder. The real implementation would:
    /// 1. Parse source files
    /// 2. Find functions with `#[test]` attribute
    /// 3. Extract metadata (name, location)
    /// 4. Return list of tests
    pub fn discover_from_source(&mut self, _source_root: &str) -> Result<Vec<TestMetadata>, String> {
        // TODO: Implement actual test discovery
        // Strategy:
        // 1. Walk source tree
        // 2. Parse each .zl file
        // 3. Find fn with #[test] attribute
        // 4. Extract metadata
        Ok(self.tests.clone())
    }

    /// Discover tests from compiled binary
    ///
    /// This is a placeholder. The real implementation would:
    /// 1. Read binary's symbol table
    /// 2. Find test functions (prefixed with test__)
    /// 3. Extract metadata section
    /// 4. Return list of tests
    pub fn discover_from_binary(&mut self, _binary_path: &str) -> Result<Vec<TestMetadata>, String> {
        // TODO: Implement actual binary discovery
        // Strategy:
        // 1. Use object file parsing (e.g., goblin, object)
        // 2. Read symbol table
        // 3. Find test metadata section
        // 4. Parse test metadata
        Ok(self.tests.clone())
    }

    /// Get all discovered tests
    pub fn tests(&self) -> &[TestMetadata] {
        &self.tests
    }

    /// Add a test to the discovery list
    pub fn add_test(&mut self, test: TestMetadata) {
        self.tests.push(test);
    }

    /// Count total tests
    pub fn count(&self) -> usize {
        self.tests.len()
    }

    /// Count only tests that should run
    pub fn count_runnable(&self) -> usize {
        self.tests.iter().filter(|t| t.should_run()).count()
    }

    /// Count only ignored tests
    pub fn count_ignored(&self) -> usize {
        self.tests.iter().filter(|t| t.ignored).count()
    }
}

impl Default for TestDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_metadata_creation() {
        let meta = TestMetadata::new(
            "test_foo".to_string(),
            "my_module".to_string(),
            "test.zl".to_string(),
            42,
            false,
        );

        assert_eq!(meta.name, "test_foo");
        assert_eq!(meta.full_name(), "my_module::test_foo");
        assert!(meta.should_run());
    }

    #[test]
    fn test_test_discovery_new() {
        let discovery = TestDiscovery::new();
        assert_eq!(discovery.count(), 0);
        assert_eq!(discovery.count_runnable(), 0);
        assert_eq!(discovery.count_ignored(), 0);
    }

    #[test]
    fn test_test_discovery_add() {
        let mut discovery = TestDiscovery::new();

        discovery.add_test(TestMetadata::new(
            "test1".to_string(),
            "mod".to_string(),
            "file.zl".to_string(),
            10,
            false,
        ));

        assert_eq!(discovery.count(), 1);
        assert_eq!(discovery.count_runnable(), 1);
    }

    #[test]
    fn test_test_discovery_ignored() {
        let mut discovery = TestDiscovery::new();

        discovery.add_test(TestMetadata::new(
            "test1".to_string(),
            "mod".to_string(),
            "file.zl".to_string(),
            10,
            false,
        ));

        discovery.add_test(TestMetadata::new(
            "test2".to_string(),
            "mod".to_string(),
            "file.zl".to_string(),
            20,
            true, // ignored
        ));

        assert_eq!(discovery.count(), 2);
        assert_eq!(discovery.count_runnable(), 1);
        assert_eq!(discovery.count_ignored(), 1);
    }
}
