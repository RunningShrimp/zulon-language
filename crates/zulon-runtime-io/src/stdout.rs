// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Standard output functions

use std::io::Write;
use std::sync::Mutex;

use crate::IoResult;

// Lazy-initialized stdout lock
lazy_static::lazy_static! {
    static ref STDOUT: Mutex<std::io::Stdout> = Mutex::new(std::io::stdout());
}

/// Print to stdout without newline
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::print;
///
/// print("Hello, ");
/// print("world!");
/// // Output: Hello, world!
/// ```
pub fn print(s: &str) -> IoResult<()> {
    let mut stdout = STDOUT.lock().unwrap();
    stdout.write_all(s.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

/// Print to stdout with newline
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::println;
///
/// println("Hello, world!");
/// // Output: Hello, world!
/// ```
pub fn println(s: &str) -> IoResult<()> {
    let mut stdout = STDOUT.lock().unwrap();
    stdout.write_all(s.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

/// Print to stdout with formatting (simplified)
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::eprint;
///
/// eprint("Error: {}", "something went wrong");
/// ```
#[allow(dead_code)]
pub fn eprint(s: &str) -> IoResult<()> {
    let mut stderr = std::io::stderr().lock();
    stderr.write_all(s.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

/// Print to stderr with newline
///
/// # Example
///
/// ```rust
/// use zulon_runtime_io::eprintln;
///
/// eprintln("Error: something went wrong");
/// ```
#[allow(dead_code)]
pub fn eprintln(s: &str) -> IoResult<()> {
    let mut stderr = std::io::stderr().lock();
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

    #[test]
    fn test_multiple_prints() {
        print("Hello, ").unwrap();
        print("world!").unwrap();
        println("").unwrap();
    }
}
