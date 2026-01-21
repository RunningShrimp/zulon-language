// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::fs::error::IoResult;
use std::path::Path as StdPath;
use std::path::PathBuf as StdPathBuf;

/// Path abstraction for cross-platform file system paths
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    inner: StdPathBuf,
}

impl Path {
    /// Creates a new Path from a string
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Path {
            inner: StdPathBuf::from(s.as_ref()),
        }
    }

    /// Creates a new Path from a string slice
    pub fn from_str(s: &str) -> Self {
        Path::new(s)
    }

    /// Returns the path as a string slice
    pub fn as_str(&self) -> &str {
        self.inner.to_str().unwrap_or("")
    }

    /// Returns the path as a bytes slice
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_os_str().as_encoded_bytes()
    }

    /// Returns true if the path is absolute
    pub fn is_absolute(&self) -> bool {
        self.inner.is_absolute()
    }

    /// Returns true if the path is relative
    pub fn is_relative(&self) -> bool {
        self.inner.is_relative()
    }

    /// Returns the parent directory if available
    pub fn parent(&self) -> Option<Path> {
        self.inner.parent().map(|p| Path {
            inner: p.to_path_buf(),
        })
    }

    /// Returns the file name if available
    pub fn file_name(&self) -> Option<&str> {
        self.inner.file_name().and_then(|n| n.to_str())
    }

    /// Returns the file stem (name without extension)
    pub fn file_stem(&self) -> Option<&str> {
        self.inner.file_stem().and_then(|n| n.to_str())
    }

    /// Returns the file extension
    pub fn extension(&self) -> Option<&str> {
        self.inner.extension().and_then(|n| n.to_str())
    }

    /// Joins another path to this path
    pub fn join<P: AsRef<Path>>(&self, path: P) -> Path {
        let result = self.inner.join(path.as_ref().as_str());
        Path { inner: result }
    }

    /// Returns an iterator over the components of the path
    pub fn components(&self) -> Components {
        Components {
            inner: self.inner.components(),
        }
    }

    /// Checks if the path exists
    pub fn exists(&self) -> bool {
        self.inner.exists()
    }

    /// Checks if the path points to a file
    pub fn is_file(&self) -> bool {
        self.inner.is_file()
    }

    /// Checks if the path points to a directory
    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }

    /// Canonicalizes the path (resolves symlinks and removes . and ..)
    pub fn canonicalize(&self) -> IoResult<Path> {
        match self.inner.canonicalize() {
            Ok(p) => crate::Outcome::Ok(Path { inner: p }),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }
}

impl AsRef<StdPath> for Path {
    fn as_ref(&self) -> &StdPath {
        self.inner.as_path()
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}

/// Iterator over path components
#[derive(Debug, Clone)]
pub struct Components<'a> {
    inner: std::path::Components<'a>,
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|c| match c {
            std::path::Component::Prefix(p) => {
                Component::Prefix(p.as_os_str().to_str().unwrap_or(""))
            }
            std::path::Component::RootDir => Component::RootDir,
            std::path::Component::CurDir => Component::CurDir,
            std::path::Component::ParentDir => Component::ParentDir,
            std::path::Component::Normal(p) => Component::Normal(p.to_str().unwrap_or("")),
        })
    }
}

/// A single component of a path
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component<'a> {
    /// A Windows prefix (e.g., "C:")
    Prefix(&'a str),
    /// The root directory
    RootDir,
    /// Current directory component
    CurDir,
    /// Parent directory component
    ParentDir,
    /// Normal path component
    Normal(&'a str),
}

impl<'a> Component<'a> {
    /// Returns true if this component is the root directory
    pub fn is_root(&self) -> bool {
        matches!(self, Component::RootDir)
    }

    /// Returns true if this component is current directory
    pub fn is_current(&self) -> bool {
        matches!(self, Component::CurDir)
    }

    /// Returns true if this component is parent directory
    pub fn is_parent(&self) -> bool {
        matches!(self, Component::ParentDir)
    }

    /// Returns true if this component is a normal component
    pub fn is_normal(&self) -> bool {
        matches!(self, Component::Normal(_))
    }
}

/// Owned path buffer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathBuf {
    inner: StdPathBuf,
}

impl PathBuf {
    /// Creates a new empty PathBuf
    pub fn new() -> Self {
        PathBuf {
            inner: StdPathBuf::new(),
        }
    }

    /// Creates a new PathBuf from a string
    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        PathBuf {
            inner: StdPathBuf::from(s.as_ref()),
        }
    }

    /// Returns the underlying Path
    pub fn as_path(&self) -> &Path {
        #[allow(clippy::transmute_ptr_to_ref)]
        unsafe {
            std::mem::transmute(&self.inner)
        }
    }

    /// Pushes a path component onto the end
    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self.inner.push(path.as_ref().as_str());
    }

    /// Pops the last path component
    pub fn pop(&mut self) -> bool {
        self.inner.pop()
    }

    /// Clears the path
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns the capacity in bytes
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserves capacity for at least additional bytes
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }
}

impl Default for PathBuf {
    fn default() -> Self {
        PathBuf::new()
    }
}

impl<S: AsRef<str>> From<S> for PathBuf {
    fn from(s: S) -> Self {
        PathBuf::from_str(s)
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl std::fmt::Display for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}

/// Joins multiple path components into a single path
pub fn join<P: AsRef<Path>>(base: &Path, path: P) -> Path {
    let result = base.inner.join(path.as_ref().as_str());
    Path { inner: result }
}

/// Normalizes a path by resolving . and .. segments
pub fn normalize(path: &Path) -> Path {
    let normalized = path.inner.components().collect::<StdPathBuf>();
    Path { inner: normalized }
}

/// Canonicalizes a path, resolving all symlinks
pub fn canonicalize(path: &Path) -> IoResult<Path> {
    match path.inner.canonicalize() {
        Ok(p) => crate::Outcome::Ok(Path { inner: p }),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}
