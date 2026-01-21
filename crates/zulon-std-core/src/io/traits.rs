// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! I/O traits for file operations

use crate::fs::error::{IoError, IoResult};
use crate::traits::PartialEq;

/// Read trait for reading from a source
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>;

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
                crate::Outcome::Err(e) => return crate::Outcome::Err(e),
            }
        }

        crate::Outcome::Ok(buffer)
    }
}

/// Write trait for writing to a destination
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize>;

    fn flush(&mut self) -> IoResult<()>;

    fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        let mut remaining = buf;

        while !remaining.is_empty() {
            match self.write(remaining) {
                crate::Outcome::Ok(n) => {
                    if n == 0 {
                        return crate::Outcome::Err(IoError::WriteZero);
                    }
                    remaining = &remaining[n..];
                }
                crate::Outcome::Err(e) => return crate::Outcome::Err(e),
            }
        }

        crate::Outcome::Ok(())
    }
}

/// Seek trait for seeking within a stream
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64>;
}

/// Enumeration of possible methods to seek within an I/O object
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SeekFrom {
    /// Seek from the start
    Start(u64),
    /// Seek from the end
    End(i64),
    /// Seek from the current position
    Current(i64),
}
