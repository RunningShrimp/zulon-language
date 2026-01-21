// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Buffered I/O utilities

use std::io::{self, BufRead, BufReader as StdBufReader, BufWriter as StdBufWriter, Read, Write};

use crate::fs::error::IoError;
use crate::result::Outcome;

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// Buffered reader that wraps any type implementing Read
#[derive(Debug)]
pub struct BufReader<R> {
    inner: StdBufReader<R>,
}

impl<R: Read> BufReader<R> {
    /// Creates a new BufReader with default buffer size
    pub fn new(inner: R) -> BufReader<R> {
        BufReader {
            inner: StdBufReader::new(inner),
        }
    }

    /// Creates a new BufReader with specified capacity
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        BufReader {
            inner: StdBufReader::with_capacity(capacity, inner),
        }
    }

    /// Gets a reference to the underlying reader
    pub fn get_ref(&self) -> &StdBufReader<R> {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader
    pub fn get_mut(&mut self) -> &mut StdBufReader<R> {
        &mut self.inner
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.inner.read(buf).map_err(|e| IoError::Io(e.kind()))
    }

    fn read_all(&mut self) -> IoResult<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];

        loop {
            match self.read(&mut chunk) {
                crate::Outcome::Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..n]);
                }
                crate::Outcome::Err(e) => return crate::Outcome::Err(IoError::Io(e.kind())),
            }
        }

        crate::Outcome::Ok(buffer)
    }
}

/// Buffered writer that wraps any type implementing Write
#[derive(Debug)]
pub struct BufWriter<W> {
    inner: StdBufWriter<W>,
}

impl<W: Write> BufWriter<W> {
    /// Creates a new BufWriter with default buffer size
    pub fn new(inner: W) -> BufWriter<W> {
        BufWriter {
            inner: StdBufWriter::new(inner),
        }
    }

    /// Creates a new BufWriter with specified capacity
    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W> {
        BufWriter {
            inner: StdBufWriter::with_capacity(capacity, inner),
        }
    }

    /// Gets a reference to the underlying writer
    pub fn get_ref(&self) -> &StdBufWriter<W> {
        &self.inner
    }

    /// Gets a mutable reference to the underlying writer
    pub fn get_mut(&mut self) -> &mut StdBufWriter<W> {
        &mut self.inner
    }

    /// Flushes the buffer
    pub fn flush(&mut self) -> IoResult<()> {
        self.inner.flush().map_err(|e| IoError::Io(e.kind()))
    }

    /// Consumes this BufWriter, returning the underlying writer
    pub fn into_inner(mut self) -> IoResult<W> {
        self.flush()?;
        Ok(self.inner.into_inner())
    }
}

impl<W: Write> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.inner.write(buf).map_err(|e| IoError::Io(e.kind()))
    }

    fn flush(&mut self) -> IoResult<()> {
        self.inner.flush().map_err(|e| IoError::Io(e.kind()))
    }

    fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        self.inner.write_all(buf).map_err(|e| IoError::Io(e.kind()))
    }
}
