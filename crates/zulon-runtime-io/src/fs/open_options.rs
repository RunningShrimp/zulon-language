// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! File opening options

use std::fs::File as StdFile;
use std::path::Path;

use crate::IoError;

use super::File;

/// Options and flags which can be used to configure how a file is opened
pub struct OpenOptions {
    inner: std::fs::OpenOptions,
}

impl OpenOptions {
    /// Creates a new blank set of options ready for configuration
    pub fn new() -> Self {
        OpenOptions {
            inner: std::fs::OpenOptions::new(),
        }
    }

    /// Sets the option for read access
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.inner.read(read);
        self
    }

    /// Sets the option for write access
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.inner.write(write);
        self
    }

    /// Sets the option for append mode
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.inner.append(append);
        self
    }

    /// Sets the option for truncating a previous file
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.inner.truncate(truncate);
        self
    }

    /// Sets the option to create a new file, or fail if it already exists
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.inner.create_new(create_new);
        self
    }

    /// Sets the option for creating a new file if it doesn't exist
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.inner.create(create);
        self
    }

    /// Opens a file at `path` with the options specified by `self`
    pub fn open<P: AsRef<Path>>(&self, path: P) -> IoResult<File> {
        self.inner
            .open(path)
            .map(|inner| File { inner: Some(inner) })
            .map_err(IoError::from)
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_options_new() {
        let opts = OpenOptions::new();
        let _ = opts;
    }

    #[test]
    fn test_open_options_read() {
        let test_file = "/tmp/test_zulon_open_opts_read.txt";
        let content = b"Read test";

        StdFile::create(test_file)
            .unwrap()
            .write_all(content)
            .unwrap();

        let opts = OpenOptions::new().read(true);
        let file = opts.open(test_file);
        assert!(file.is_ok());

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_open_options_create_new() {
        let test_file = "/tmp/test_zulon_open_opts_create.txt";

        let opts = OpenOptions::new().write(true).create_new(true);
        let file = opts.open(test_file);
        assert!(file.is_ok());

        let file2 = opts.open(test_file);
        assert!(file2.is_err());

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_open_options_append() {
        let test_file = "/tmp/test_zulon_open_opts_append.txt";

        {
            let opts = OpenOptions::new().write(true).create(true).append(true);
            let mut file = opts.open(test_file).unwrap();
            file.write_all(b"Hello").unwrap();
        }

        {
            let opts = OpenOptions::new().write(true).create(true).append(true);
            let mut file = opts.open(test_file).unwrap();
            file.write_all(b", World!").unwrap();
        }

        let content = std::fs::read_to_string(test_file).unwrap();
        assert_eq!(content, "Hello, World!");

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_open_options_default() {
        let opts: OpenOptions = Default::default();
        let _ = opts;
    }
}
