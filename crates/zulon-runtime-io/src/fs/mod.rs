// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # File System Module
//!
//! Minimal file system operations using standard library only.
//! This eliminates circular dependency on zulon-std-core.

use std::fs;
use std::io::{self, BufRead, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Simple File type alias
pub type File = std::fs::File;

/// Simple Directory entry type alias
pub type DirEntry = std::fs::DirEntry;

/// Simple Metadata type alias
pub type Metadata = std::fs::Metadata;

/// Error type alias
pub type Error = std::io::Error;

/// Creates a new directory
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    std::fs::create_dir(path)
}

/// Checks if a path exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    std::path::Path::new(path).exists()
}

/// Removes a file
pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    std::fs::remove_file(path)
}

/// Removes a directory
pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    std::fs::remove_dir(path)
}

/// Reads all bytes from a file
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let mut buffer = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Reads entire file contents into a string
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let mut buffer = String::with_capacity(metadata.len() as usize);
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Writes data to a file
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_ref())?;
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_create_and_write() {
        let test_file = "/tmp/test_zulon_fs_minimal.txt";
        let content = b"Minimal FS test";

        // Write
        write(test_file, content).unwrap();

        // Read
        let read_content = read_to_string(test_file).unwrap();
        assert_eq!(read_content, String::from_utf8(content.to_vec()).unwrap());

        // Cleanup
        remove_file(test_file).unwrap();
        assert!(!exists(test_file));
    }
}
