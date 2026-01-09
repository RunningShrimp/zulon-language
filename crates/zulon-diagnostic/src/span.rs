// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Source code span and location types

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

/// A file identifier (can be shared across Locs)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(Arc<PathBuf>);

impl FileId {
    /// Create a new FileId from a path
    pub fn new(path: PathBuf) -> Self {
        Self(Arc::new(path))
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// A source code location
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loc {
    /// The file ID (if available)
    pub file: Option<FileId>,
    /// The line number (1-based)
    pub line: usize,
    /// The column number (1-based, in bytes)
    pub column: usize,
    /// The byte offset from the beginning of the file
    pub offset: usize,
}

impl Loc {
    /// Create a new location
    pub fn new(file: Option<PathBuf>, line: usize, column: usize, offset: usize) -> Self {
        Self {
            file: file.map(FileId::new),
            line,
            column,
            offset,
        }
    }

    /// Create a dummy location (for testing or synthetic errors)
    pub fn dummy() -> Self {
        Self {
            file: None,
            line: 0,
            column: 0,
            offset: 0,
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:{}", file, self.line)?;
        } else {
            write!(f, "{}", self.line)?;
        }
        write!(f, ":{}", self.column)
    }
}

/// A span of source code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    /// The start of the span
    pub lo: Loc,
    /// The end of the span (inclusive)
    pub hi: Loc,
}

impl Span {
    /// Create a new span
    pub fn new(lo: Loc, hi: Loc) -> Self {
        Self { lo, hi }
    }

    /// Create a dummy span
    pub fn dummy() -> Self {
        Self {
            lo: Loc::dummy(),
            hi: Loc::dummy(),
        }
    }

    /// Create a span at a single location
    pub fn point(loc: Loc) -> Self {
        Self {
            lo: loc.clone(),
            hi: loc,
        }
    }

    /// Check if this span is a dummy
    pub fn is_dummy(&self) -> bool {
        self.lo.file.is_none() && self.hi.file.is_none()
    }

    /// Merge two spans
    pub fn merge(&self, other: Span) -> Span {
        Span {
            lo: self.lo.clone(),
            hi: other.hi,
        }
    }

    /// Get the length of this span in bytes
    pub fn len(&self) -> usize {
        if self.hi.offset >= self.lo.offset {
            self.hi.offset - self.lo.offset
        } else {
            0
        }
    }

    /// Check if this span is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lo)
    }
}
