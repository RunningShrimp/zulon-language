// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Directory iteration support

use std::fs as std_fs;

use crate::fs::error::IoError;
use crate::fs::error::IoResult;
use crate::fs::path::Path;
use crate::fs::path::PathBuf;
use crate::result::Outcome;

/// An entry in a directory
///
/// Returned by the `read_dir` iterator.
#[derive(Debug, Clone)]
pub struct DirEntry {
    path: Path,
    file_type: std_fs::FileType,
}

impl DirEntry {
    /// Creates a new DirEntry from a std::fs::DirEntry
    fn from_std(entry: std_fs::DirEntry) -> Self {
        let path_str = entry.path().to_string_lossy().to_string();
        let path = Path::new(&path_str);
        let file_type = entry.file_type().unwrap_or(std_fs::FileType::Unknown);

        DirEntry { path, file_type }
    }

    /// Returns the full path to the entry
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the file name for this directory entry
    pub fn file_name(&self) -> &str {
        self.path.file_name().unwrap_or("")
    }

    /// Returns the file type for the file that this entry points to
    pub fn file_type(&self) -> FileType {
        FileType::from_std(&self.file_type)
    }

    /// Returns true if this entry is a file
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// Returns true if this entry is a directory
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    /// Returns true if this entry is a symbolic link
    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }
}

/// Possible file types for a directory entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Dir,
    /// Symbolic link
    Symlink,
    /// Unknown file type
    Unknown,
}

impl FileType {
    /// Creates a FileType from std::fs::FileType
    fn from_std(ft: &std_fs::FileType) -> Self {
        if ft.is_file() {
            FileType::File
        } else if ft.is_dir() {
            FileType::Dir
        } else if ft.is_symlink() {
            FileType::Symlink
        } else {
            FileType::Unknown
        }
    }

    /// Returns true if this file type is a regular file
    pub fn is_file(&self) -> bool {
        matches!(self, FileType::File)
    }

    /// Returns true if this file type is a directory
    pub fn is_dir(&self) -> bool {
        matches!(self, FileType::Dir)
    }

    /// Returns true if this file type is a symbolic link
    pub fn is_symlink(&self) -> bool {
        matches!(self, FileType::Symlink)
    }
}

/// Iterator over directory entries
///
/// Created by calling `read_dir` function.
#[derive(Debug)]
pub struct ReadDir {
    inner: std_fs::ReadDir,
}

impl ReadDir {
    /// Creates a new ReadDir iterator from std::fs::ReadDir
    fn from_std(read_dir: std_fs::ReadDir) -> Self {
        ReadDir { inner: read_dir }
    }
}

impl Iterator for ReadDir {
    type Item = IoResult<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| match result {
            Ok(entry) => crate::Outcome::Ok(DirEntry::from_std(entry)),
            Err(e) => crate::Outcome::Err(e.into()),
        })
    }
}

/// Reads the contents of a directory
///
/// Returns an iterator over the entries in the directory.
///
/// # Example
///
/// ```
/// use zulon_std_core::{fs::read_dir, Path};
///
/// for entry in read_dir("path/to/dir").unwrap() {
///     let entry = entry.unwrap();
///     println!("{}", entry.file_name());
/// }
/// ```
///
/// # Errors
///
/// This function will return an error in the following situations:
/// - The provided path doesn't exist
/// - The provided path is not a directory
/// - The user lacks permissions to view the contents
pub fn read_dir<P: AsRef<Path>>(path: P) -> IoResult<ReadDir> {
    match std_fs::read_dir(path.as_ref().as_str()) {
        Ok(read_dir) => crate::Outcome::Ok(ReadDir::from_std(read_dir)),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{read_dir, DirEntry, FileType};
    use std::fs as std_fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_read_dir() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create some test files
        std_fs::write(test_dir.join("file1.txt"), "content1").unwrap();
        std_fs::write(test_dir.join("file2.txt"), "content2").unwrap();
        std_fs::create_dir(test_dir.join("subdir")).unwrap();

        // Read directory
        let mut entries: Vec<_> = read_dir(test_dir.to_str().unwrap()).unwrap().collect();

        // We should have at least 3 entries (file1, file2, subdir)
        assert!(entries.len() >= 3);

        // Check that we can find our files
        let file_names: Vec<_> = entries
            .iter()
            .map(|e| e.as_ref().map(|entry| entry.file_name().to_string()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(file_names.iter().any(|n| n.contains("file1")));
        assert!(file_names.iter().any(|n| n.contains("file2")));
        assert!(file_names.iter().any(|n| n.contains("subdir")));
    }

    #[test]
    fn test_dir_entry_file_types() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create different types of entries
        std_fs::write(test_dir.join("file.txt"), "content").unwrap();
        std_fs::create_dir(test_dir.join("directory")).unwrap();

        let mut entries: Vec<_> = read_dir(test_dir.to_str().unwrap()).unwrap().collect();

        // Find entries and check types
        for entry in entries.iter() {
            if let Outcome::Ok(ref entry) = entry {
                if entry.file_name() == "file.txt" {
                    assert!(entry.is_file());
                    assert!(!entry.is_dir());
                } else if entry.file_name() == "directory" {
                    assert!(entry.is_dir());
                    assert!(!entry.is_file());
                }
            }
        }
    }

    #[test]
    fn test_read_dir_nonexistent() {
        let result = read_dir("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_dir_file_instead_of_dir() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("file.txt");

        std_fs::write(&test_file, "content").unwrap();

        let result = read_dir(test_file.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        std_fs::create_dir(&test_dir).unwrap();

        let entries: Vec<_> = read_dir(test_dir.to_str().unwrap()).unwrap().collect();
        assert_eq!(entries.len(), 0);
    }
}
