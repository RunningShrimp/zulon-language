// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::fs::error::IoResult;
use std::fs as std_fs;
use std::path::Path as StdPath;

/// Metadata information about a file or directory
#[derive(Debug, Clone)]
pub struct Metadata {
    file_type: FileType,
    len: u64,
    modified: Option<std::time::SystemTime>,
    accessed: Option<std::time::SystemTime>,
    created: Option<std::time::SystemTime>,
    readonly: bool,
}

impl Metadata {
    pub(crate) fn from_std(meta: std_fs::Metadata) -> Self {
        let file_type = if meta.is_file() {
            FileType::File
        } else if meta.is_dir() {
            FileType::Dir
        } else if meta.is_symlink() {
            FileType::Symlink
        } else {
            FileType::Unknown
        };

        Metadata {
            file_type,
            len: meta.len(),
            modified: meta.modified().ok(),
            accessed: meta.accessed().ok(),
            created: meta.created().ok(),
            readonly: meta.permissions().readonly(),
        }
    }

    /// Returns the file type
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Returns the size in bytes
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns true if the size is 0
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the modified time if available
    pub fn modified(&self) -> Option<std::time::SystemTime> {
        self.modified
    }

    /// Returns the accessed time if available
    pub fn accessed(&self) -> Option<std::time::SystemTime> {
        self.accessed
    }

    /// Returns the created time if available
    pub fn created(&self) -> Option<std::time::SystemTime> {
        self.created
    }

    /// Returns true if the file is readonly
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }
}

/// Possible file types
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

pub fn metadata(path: &std::path::Path) -> IoResult<Metadata> {
    match std_fs::metadata(path) {
        Ok(meta) => crate::Outcome::Ok(Metadata::from_std(meta)),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn try_metadata(path: &std::path::Path) -> IoResult<Metadata> {
    match std_fs::metadata(path) {
        Ok(meta) => crate::Outcome::Ok(Metadata::from_std(meta)),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}
