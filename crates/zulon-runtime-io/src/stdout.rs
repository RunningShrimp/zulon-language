// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Standard output functions
//!
//! Provides print and println functionality using std::io::stdout.

use std::io::{self, Write};

// Type alias for compatibility
type IoResult<T> = Result<T, crate::error::IoError>;

/// Print to stdout without newline
///
/// # Example
///
/// ```
/// use zulon_runtime_io::print;
///
/// print("Hello, ");
/// print("world!");
/// // Output: Hello, world!
/// ```
pub fn print(s: &str) -> IoResult<()> {
    std::io::stdout().write_all(s.as_bytes())?;
    std::io::stdout().flush()?;
    Ok(())
}

/// Print to stdout with newline
///
/// # Example
///
/// ```
/// use zulon_runtime_io::println;
///
/// println("Hello, world!");
/// // Output: Hello, world!\n
/// ```
pub fn println(s: &str) -> IoResult<()> {
    std::io::stdout().write_all(s.as_bytes())?;
    std::io::stdout().write_all(b"\n")?;
    std::io::stdout().flush()?;
    Ok(())
}

/// Print to stderr without newline
///
/// # Example
///
/// ```
/// use zulon_runtime_io::eprint;
///
/// eprint("Error: {}", "something went wrong");
/// // Output to stderr: Error: something went wrong
/// ```
#[allow(dead_code)]
pub fn eprint(s: &str) -> IoResult<()> {
    let mut stderr = std::io::stderr();
    stderr.write_all(s.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

/// Print to stderr with newline
///
/// # Example
///
/// ```
/// use zulon_runtime_io::eprintln;
///
/// eprintln("Error: something went wrong");
/// // Output to stderr: Error: something went wrong\n
/// ```
#[allow(dead_code)]
pub fn eprintln(s: &str) -> IoResult<()> {
    let mut stderr = std::io::stderr();
    stderr.write_all(s.as_bytes())?;
    stderr.write_all(b"\n")?;
    stderr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print() {
        let result = print("test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_println() {
        let result = println("test");
        assert!(result.is_ok());
    }
}
