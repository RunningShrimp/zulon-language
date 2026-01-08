// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! File operations

use std::fs::File as StdFile;
use std::io::{Read, Write};
use std::path::Path;

use crate::{IoError, IoResult};

/// File handle for reading and writing
///
/// # Example
///
/// ```rust,no_run
/// use zulon_runtime_io::File;
///
/// // Write to a file
/// let mut file = File::create("test.txt").unwrap();
/// file.write_all(b"Hello, world!").unwrap();
///
/// // Read from a file
/// let mut file = File::open("test.txt").unwrap();
/// let mut contents = Vec::new();
/// file.read_to_end(&mut contents).unwrap();
/// assert_eq!(contents, b"Hello, world!");
/// ```
pub struct File {
    inner: StdFile,
}

impl File {
    /// Open a file for reading
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zulon_runtime_io::File;
    ///
    /// let file = File::open("test.txt").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        StdFile::open(path)
            .map(|inner| File { inner })
            .map_err(IoError::from)
    }

    /// Create a new file for writing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zulon_runtime_io::File;
    ///
    /// let file = File::create("test.txt").unwrap();
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        StdFile::create(path)
            .map(|inner| File { inner })
            .map_err(IoError::from)
    }

    /// Open a file for appending
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zulon_runtime_io::File;
    ///
    /// let file = File::append("test.txt").unwrap();
    /// ```
    pub fn append<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        StdFile::options()
            .append(true)
            .open(path)
            .map(|inner| File { inner })
            .map_err(IoError::from)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_file_create_and_write() {
        let file = File::create("/tmp/test_zulon_create.txt").unwrap();
        drop(file); // Just verify it creates successfully
    }

    #[test]
    fn test_file_write_and_read() {
        // Write
        let mut file = File::create("/tmp/test_zulon_rw.txt").unwrap();
        file.write_all(b"Hello, world!").unwrap();
        file.flush().unwrap();

        // Read
        let mut file = File::open("/tmp/test_zulon_rw.txt").unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"Hello, world!");
    }

    #[test]
    fn test_file_append() {
        // Create file
        {
            let mut file = File::create("/tmp/test_zulon_append.txt").unwrap();
            file.write_all(b"Hello").unwrap();
        }

        // Append
        {
            let mut file = File::append("/tmp/test_zulon_append.txt").unwrap();
            file.write_all(b", world!").unwrap();
        }

        // Verify
        let mut file = File::open("/tmp/test_zulon_append.txt").unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"Hello, world!");
    }
}
