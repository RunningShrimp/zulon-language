// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::fs::error::IoResult;
use crate::fs::metadata::Metadata;
use crate::fs::path::Path;
use crate::io::traits::{Read, Seek, Write};
use std::fs as std_fs;
use std::io as std_io;

/// Options and flags which can be used to configure how a file is opened
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    /// Creates a blank new set of options
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    /// Sets the option for read access
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for the append mode
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option for creating a new file if it doesn't exist
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option for creating a new file or failing if it already exists
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Opens a file at the path with the specified options
    pub fn open<P: AsRef<Path>>(&self, path: P) -> IoResult<File> {
        let mut opts = std_fs::OpenOptions::new();
        opts.read(self.read)
            .write(self.write)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create)
            .create_new(self.create_new);

        match std_fs::File::open(path.as_ref().as_str()) {
            Ok(f) => crate::Outcome::Ok(File { inner: f }),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        OpenOptions::new()
    }
}

/// A reference to an open file on the filesystem
#[derive(Debug)]
pub struct File {
    inner: std_fs::File,
}

impl File {
    /// Attempts to open a file in read-only mode
    pub fn open<P: AsRef<Path>>(path: P) -> IoResult<File> {
        OpenOptions::new().read(true).open(path)
    }

    /// Opens a file in write-only mode
    pub fn create<P: AsRef<Path>>(path: P) -> IoResult<File> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    }

    /// Opens a file for appending
    pub fn append<P: AsRef<Path>>(path: P) -> IoResult<File> {
        OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)
    }

    /// Returns the metadata for the file
    pub fn metadata(&self) -> IoResult<Metadata> {
        match self.inner.metadata() {
            Ok(meta) => crate::Outcome::Ok(Metadata::from_std(meta)),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }

    /// Attempts to sync all OS-internal metadata to disk
    pub fn sync_all(&self) -> IoResult<()> {
        match self.inner.sync_all() {
            Ok(_) => crate::Outcome::Ok(()),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }

    /// Attempts to sync file data to disk
    pub fn sync_data(&self) -> IoResult<()> {
        match self.inner.sync_data() {
            Ok(_) => crate::Outcome::Ok(()),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }

    /// Truncates or extends the underlying file
    pub fn set_len(&self, size: u64) -> IoResult<()> {
        match self.inner.set_len(size) {
            Ok(_) => crate::Outcome::Ok(()),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }

    /// Queries metadata about the underlying file without consuming the file
    pub fn metadata_only<P: AsRef<Path>>(path: P) -> IoResult<Metadata> {
        match std_fs::metadata(path.as_ref().as_str()) {
            Ok(meta) => crate::Outcome::Ok(Metadata::from_std(meta)),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self.inner.try_clone() {
            Ok(mut handle) => match std_io::Read::read(&mut handle, buf) {
                Ok(n) => crate::Outcome::Ok(n),
                Err(e) => crate::Outcome::Err(e.into()),
            },
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        match std_io::Write::write(&mut self.inner, buf) {
            Ok(n) => crate::Outcome::Ok(n),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match std_io::Write::flush(&mut self.inner) {
            Ok(_) => crate::Outcome::Ok(()),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }
}

impl Seek for File {
    fn seek(&mut self, pos: crate::io::traits::SeekFrom) -> IoResult<u64> {
        let pos = match pos {
            crate::io::traits::SeekFrom::Start(offset) => std_io::SeekFrom::Start(offset),
            crate::io::traits::SeekFrom::End(offset) => std_io::SeekFrom::End(offset),
            crate::io::traits::SeekFrom::Current(offset) => std_io::SeekFrom::Current(offset),
        };

        match std_io::Seek::seek(&mut self.inner, pos) {
            Ok(n) => crate::Outcome::Ok(n),
            Err(e) => crate::Outcome::Err(e.into()),
        }
    }
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> IoResult<()> {
    match std_fs::remove_file(path.as_ref().as_str()) {
        Ok(_) => crate::Outcome::Ok(()),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn remove_dir<P: AsRef<Path>>(path: P) -> IoResult<()> {
    match std_fs::remove_dir(path.as_ref().as_str()) {
        Ok(_) => crate::Outcome::Ok(()),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> IoResult<()> {
    match std_fs::remove_dir_all(path.as_ref().as_str()) {
        Ok(_) => crate::Outcome::Ok(()),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn create_dir<P: AsRef<Path>>(path: P) -> IoResult<()> {
    match std_fs::create_dir(path.as_ref().as_str()) {
        Ok(_) => crate::Outcome::Ok(()),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> IoResult<()> {
    match std_fs::create_dir_all(path.as_ref().as_str()) {
        Ok(_) => crate::Outcome::Ok(()),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> IoResult<()> {
    match std_fs::rename(from.as_ref().as_str(), to.as_ref().as_str()) {
        Ok(_) => crate::Outcome::Ok(()),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> IoResult<u64> {
    match std_fs::copy(from.as_ref().as_str(), to.as_ref().as_str()) {
        Ok(n) => crate::Outcome::Ok(n),
        Err(e) => crate::Outcome::Err(e.into()),
    }
}
