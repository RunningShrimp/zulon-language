// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! String type for ZULON standard library
//!
//! `String` is a growable, UTF-8 encoded string. Internally, it wraps a `Vec<u8>`
//! and ensures that all contents are valid UTF-8.

use crate::vec::Vec;
use crate::traits::{Clone, PartialEq, Eq, PartialOrd, Ord, Hash};

/// A growable, UTF-8 encoded string.
///
/// `String` is a growable string buffer that stores valid UTF-8 encoded text.
/// It is implemented as a wrapper around `Vec<u8>` with UTF-8 validation.
///
/// # Examples
///
/// ```rust
/// let mut s = String::new();
/// s.push('H');
/// s.push('i');
/// assert_eq!(s.as_str(), "Hi");
///
/// let s2 = String::from("hello");
/// assert_eq!(s2.len(), 5);
/// ```

pub struct String {
    vec: Vec<u8>,
}

impl String {
    /// Creates a new empty `String`.
    ///
    /// The string will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::new();
    /// assert_eq!(s.len(), 0);
    /// ```
    pub fn new() -> Self {
        String { vec: Vec::new() }
    }

    /// Creates a new `String` from a `&str`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello");
    /// assert_eq!(s.len(), 5);
    /// assert_eq!(s.as_str(), "hello");
    /// ```
    pub fn from(s: &str) -> Self {
        let mut string = String::new();
        string.vec.extend(s.as_bytes());
        string
    }

    /// Converts the string into a `&str` slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello");
    /// assert_eq!(s.as_str(), "hello");
    /// ```
    pub fn as_str(&self) -> &str {
        // SAFETY: We maintain the invariant that vec always contains valid UTF-8
        unsafe { std::str::from_utf8_unchecked(self.vec.as_slice()) }
    }

    /// Returns the length of the string in bytes.
    ///
    /// Note that this is **not** the number of characters (code points).
    /// For that, use `.chars().count()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello");
    /// assert_eq!(s.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if the string has a length of zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::new();
    /// assert!(s.is_empty());
    ///
    /// let s = String::from("hello");
    /// assert!(!s.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns the capacity of the string in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello");
    /// assert!(s.capacity() >= 5);
    /// ```
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Appends a given character to the end of this `String`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut s = String::from("abc");
    /// s.push('d');
    /// s.push('e');
    /// assert_eq!(s.as_str(), "abcde");
    /// ```
    pub fn push(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        let len = ch.encode_utf8(&mut buf).len();
        self.vec.extend(&buf[0..len]);
    }

    /// Appends a given string slice onto the end of this `String`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut s = String::from("foo");
    /// s.push_str("bar");
    /// assert_eq!(s.as_str(), "foobar");
    /// ```
    pub fn push_str(&mut self, s: &str) {
        self.vec.extend(s.as_bytes());
    }

    /// Removes the last character from the string buffer and returns it.
    ///
    /// Returns `None` if this `String` is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut s = String::from("foo");
    /// assert_eq!(s.pop(), Some('o'));
    /// assert_eq!(s.as_str(), "fo");
    /// ```
    pub fn pop(&mut self) -> Option<char> {
        if self.is_empty() {
            return None;
        }

        // Find the last UTF-8 character boundary
        let len = self.len();
        let mut char_len = 1;
        for i in (0..len).rev() {
            let byte = self.vec.as_slice()[i];
            if byte < 0x80 || byte >= 0xC0 {
                // Found character boundary
                char_len = len - i;
                break;
            }
        }

        // Extract the character
        let char_bytes = self.vec.as_slice()[len - char_len..len].to_vec();
        let ch = std::str::from_utf8(&char_bytes).ok()?.chars().next()?;

        // Remove from string
        unsafe {
            self.vec.set_len(len - char_len);
        }

        Some(ch)
    }

    /// Truncates this `String`, removing all contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut s = String::from("foo");
    /// s.clear();
    /// assert!(s.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.vec.clear()
    }

    /// Splits the string into two at the given byte index.
    ///
    /// # Panics
    ///
    /// Panics if `at` is not on a UTF-8 code point boundary, or if it is beyond the end of the string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut hello = String::from("Hello, World!");
    /// let world = hello.split_off(7);
    /// assert_eq!(hello.as_str(), "Hello, ");
    /// assert_eq!(world.as_str(), "World!");
    /// ```
    pub fn split_off(&mut self, at: usize) -> String {
        assert!(at <= self.len(), "split_off at {} exceeds length {}", at, self.len());
        assert!(self.is_char_boundary(at), "split_off at {} is not a character boundary", at);

        // Create the other string from bytes at 'at' onwards
        let mut other_vec = Vec::new();
        for i in at..self.len() {
            other_vec.push(self.vec.as_slice()[i]);
        }

        // Truncate self to 'at'
        unsafe {
            self.vec.set_len(at);
        }

        String { vec: other_vec }
    }

    /// Checks that the given position is on a UTF-8 character boundary.
    fn is_char_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.len() {
            return true;
        }

        let byte = self.vec.as_slice()[index];
        byte < 0x80 || byte >= 0xC0
    }

    /// Removes the specified range from the string, and returns it as an iterator.
    pub fn remove_range(&mut self, range: std::ops::Range<usize>) {
        assert!(range.start <= range.end, "invalid range");
        assert!(range.end <= self.len(), "range end out of bounds");
        assert!(self.is_char_boundary(range.start), "range start not on character boundary");
        assert!(self.is_char_boundary(range.end), "range end not on character boundary");

        let len = range.end - range.start;
        let remaining = self.len() - range.end;

        // Shift remaining bytes left
        for i in 0..remaining {
            let byte = self.vec.as_slice()[range.end + i];
            self.vec.as_mut_slice()[range.start + i] = byte;
        }

        unsafe {
            self.vec.set_len(self.len() - len);
        }
    }

    /// Returns an iterator over the `char`s of this `String`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("Hello");
    /// let mut chars = s.chars();
    /// assert_eq!(chars.next(), Some('H'));
    /// assert_eq!(chars.next(), Some('e'));
    /// ```
    pub fn chars(&self) -> Chars<'_> {
        Chars {
            string: self,
            index: 0,
        }
    }

    /// Truncates this `String` to the specified length.
    ///
    /// # Panics
    ///
    /// Panics if `new_len` is greater than the string's current length, or if it is not on a character boundary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut s = String::from("Hello, World!");
    /// s.truncate(5);
    /// assert_eq!(s.as_str(), "Hello");
    /// ```
    pub fn truncate(&mut self, new_len: usize) {
        assert!(new_len <= self.len(), "truncate length {} exceeds length {}", new_len, self.len());
        assert!(self.is_char_boundary(new_len), "truncate length {} is not on character boundary", new_len);

        unsafe {
            self.vec.set_len(new_len);
        }
    }

    /// Reserves capacity for at least `additional` more bytes to be inserted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut s = String::new();
    /// s.reserve(10);
    /// assert!(s.capacity() >= 10);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional)
    }

    /// Returns a byte slice of this `String`'s contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello");
    /// assert_eq!(s.as_bytes(), &[104, 101, 108, 108, 111]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.vec.as_slice()
    }

    /// Returns a String with leading and trailing whitespace removed.
    pub fn trim(&self) -> String {
        let trimmed = self.as_str().trim();
        let mut result = String::new();
        result.vec.extend(trimmed.as_bytes());
        result
    }

    /// Returns a String with leading whitespace removed.
    pub fn trim_start(&self) -> String {
        let trimmed = self.as_str().trim_start();
        let mut result = String::new();
        result.vec.extend(trimmed.as_bytes());
        result
    }

    /// Returns a String with trailing whitespace removed.
    pub fn trim_end(&self) -> String {
        let trimmed = self.as_str().trim_end();
        let mut result = String::new();
        result.vec.extend(trimmed.as_bytes());
        result
    }

    /// Splits the string by a delimiter and returns a vector of substrings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello-world-foo");
    /// let parts: Vec<String> = s.split('-').map(|s| s.to_string()).collect();
    /// assert_eq!(parts.len(), 3);
    /// ```
    pub fn split<'a>(&'a self, delimiter: char) -> Split<'a> {
        Split {
            string: self,
            delimiter,
            index: 0,
        }
    }

    /// Splits the string on whitespace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello world foo");
    /// let parts: Vec<String> = s.split_whitespace().map(|s| s.to_string()).collect();
    /// assert_eq!(parts.len(), 3);
    /// ```
    pub fn split_whitespace(&self) -> SplitWhitespace<'_> {
        SplitWhitespace {
            string: self,
            index: 0,
        }
    }

    /// Replaces all occurrences of a pattern with another string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello world");
    /// let replaced = s.replace("world", "rust");
    /// assert_eq!(replaced.as_str(), "hello rust");
    /// ```
    pub fn replace(&self, from: &str, to: &str) -> String {
        let replaced = self.as_str().replace(from, to);
        let mut result = String::new();
        result.vec.extend(replaced.as_bytes());
        result
    }

    /// Replaces first N occurrences of a pattern with another string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello hello hello");
    /// let replaced = s.replacen("hello", "hi", 2);
    /// assert_eq!(replaced.as_str(), "hi hi hello");
    /// ```
    pub fn replacen(&self, from: &str, to: &str, count: usize) -> String {
        let replaced = self.as_str().replacen(from, to, count);
        let mut result = String::new();
        result.vec.extend(replaced.as_bytes());
        result
    }

    /// Returns a lowercase version of this string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("HELLO");
    /// assert_eq!(s.to_lowercase().as_str(), "hello");
    /// ```
    pub fn to_lowercase(&self) -> String {
        let lower = self.as_str().to_lowercase();
        let mut result = String::new();
        result.vec.extend(lower.as_bytes());
        result
    }

    /// Returns an uppercase version of this string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello");
    /// assert_eq!(s.to_uppercase().as_str(), "HELLO");
    /// ```
    pub fn to_uppercase(&self) -> String {
        let upper = self.as_str().to_uppercase();
        let mut result = String::new();
        result.vec.extend(upper.as_bytes());
        result
    }

    /// Checks if the string contains a pattern.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello world");
    /// assert!(s.contains("world"));
    /// assert!(!s.contains("rust"));
    /// ```
    pub fn contains(&self, pattern: &str) -> bool {
        self.as_str().contains(pattern)
    }

    /// Checks if the string starts with a pattern.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello world");
    /// assert!(s.starts_with("hello"));
    /// assert!(!s.starts_with("world"));
    /// ```
    pub fn starts_with(&self, pattern: &str) -> bool {
        self.as_str().starts_with(pattern)
    }

    /// Checks if the string ends with a pattern.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let s = String::from("hello world");
    /// assert!(s.ends_with("world"));
    /// assert!(!s.ends_with("hello"));
    /// ```
    pub fn ends_with(&self, pattern: &str) -> bool {
        self.as_str().ends_with(pattern)
    }

    /// Returns the substring between the specified byte indices.
    ///
    /// # Panics
    ///
    /// Panics if the indices are not on character boundaries or out of bounds.
    pub fn substring(&self, start: usize, end: usize) -> String {
        assert!(start <= end, "start > end");
        assert!(end <= self.len(), "end out of bounds");
        assert!(self.is_char_boundary(start), "start not on character boundary");
        assert!(self.is_char_boundary(end), "end not on character boundary");

        String::from(&self.as_str()[start..end])
    }
}

/// Iterator for string splitting by delimiter.
pub struct Split<'a> {
    string: &'a String,
    delimiter: char,
    index: usize,
}

impl<'a> Iterator for Split<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        // Handle empty string case
        if self.string.is_empty() {
            if self.index == 0 {
                self.index = 1;
                return Some("");
            }
            return None;
        }

        if self.index > self.string.len() {
            return None;
        }

        let slice = &self.string.as_str()[self.index..];

        match slice.find(self.delimiter) {
            Some(pos) => {
                let result = &slice[..pos];
                self.index += pos + 1;
                Some(result)
            }
            None => {
                self.index = self.string.len() + 1;
                Some(slice)
            }
        }
    }
}

/// Iterator for splitting string on whitespace.
pub struct SplitWhitespace<'a> {
    string: &'a String,
    index: usize,
}

impl<'a> Iterator for SplitWhitespace<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let slice = &self.string.as_str()[self.index..];

        // Skip leading whitespace
        let start = match slice.find(|c: char| !c.is_whitespace()) {
            Some(pos) => pos,
            None => return None,
        };

        let slice = &slice[start..];

        // Find end of word
        let end = match slice.find(|c: char| c.is_whitespace()) {
            Some(pos) => pos,
            None => slice.len(),
        };

        self.index += start + end;
        Some(&slice[..end])
    }
}

/// Iterator over the characters of a `String`.
pub struct Chars<'a> {
    string: &'a String,
    index: usize,
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.index >= self.string.len() {
            return None;
        }

        let slice = &self.string.as_str()[self.index..];
        let ch = slice.chars().next()?;
        self.index += ch.len_utf8();
        Some(ch)
    }
}

// Default implementation
impl Default for String {
    fn default() -> Self {
        String::new()
    }
}

// Clone implementation
impl Clone for String {
    fn clone(&self) -> Self {
        String {
            vec: self.vec.clone(),
        }
    }
}

// PartialEq implementation
impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.vec.eq(&other.vec)
    }
}

// Eq implementation
impl Eq for String {}

// PartialOrd implementation
impl PartialOrd for String {
    fn partial_cmp(&self, other: &Self) -> Option<crate::traits::Ordering> {
        match self.as_str().partial_cmp(other.as_str()) {
            Some(std::cmp::Ordering::Less) => Some(crate::traits::Ordering::Less),
            Some(std::cmp::Ordering::Equal) => Some(crate::traits::Ordering::Equal),
            Some(std::cmp::Ordering::Greater) => Some(crate::traits::Ordering::Greater),
            None => None,
        }
    }

    fn lt(&self, other: &Self) -> bool {
        self.as_str() < other.as_str()
    }

    fn le(&self, other: &Self) -> bool {
        self.as_str() <= other.as_str()
    }

    fn gt(&self, other: &Self) -> bool {
        self.as_str() > other.as_str()
    }

    fn ge(&self, other: &Self) -> bool {
        self.as_str() >= other.as_str()
    }
}

// Ord implementation
impl Ord for String {
    fn cmp(&self, other: &Self) -> crate::traits::Ordering {
        match self.as_str().cmp(other.as_str()) {
            std::cmp::Ordering::Less => crate::traits::Ordering::Less,
            std::cmp::Ordering::Equal => crate::traits::Ordering::Equal,
            std::cmp::Ordering::Greater => crate::traits::Ordering::Greater,
        }
    }
}

// Hash implementation
impl Hash for String {
    fn hash(&self) -> u64 {
        self.as_str().hash()
    }
}

// Display implementation
impl std::fmt::Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Debug implementation
impl std::fmt::Debug for String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}

// Convert from &str to String
impl From<&str> for String {
    fn from(s: &str) -> Self {
        String::from(s)
    }
}

// Convert from String to &str
impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = String::new();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_from_str() {
        let s = String::from("hello");
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_push_char() {
        let mut s = String::new();
        s.push('H');
        s.push('i');
        assert_eq!(s.as_str(), "Hi");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_push_str() {
        let mut s = String::from("foo");
        s.push_str("bar");
        assert_eq!(s.as_str(), "foobar");
    }

    #[test]
    fn test_pop() {
        let mut s = String::from("foo");
        assert_eq!(s.pop(), Some('o'));
        assert_eq!(s.pop(), Some('o'));
        assert_eq!(s.pop(), Some('f'));
        assert_eq!(s.pop(), None);
        assert!(s.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut s = String::from("hello");
        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_truncate() {
        let mut s = String::from("hello");
        s.truncate(3);
        assert_eq!(s.as_str(), "hel");
    }

    #[test]
    fn test_split_off() {
        let mut hello = String::from("Hello, World!");
        let world = hello.split_off(7);
        assert_eq!(hello.as_str(), "Hello, ");
        assert_eq!(world.as_str(), "World!");
    }

    #[test]
    fn test_chars() {
        let s = String::from("Hello");
        let mut chars = s.chars();
        assert_eq!(chars.next(), Some('H'));
        assert_eq!(chars.next(), Some('e'));
        assert_eq!(chars.next(), Some('l'));
        assert_eq!(chars.next(), Some('l'));
        assert_eq!(chars.next(), Some('o'));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn test_unicode() {
        let mut s = String::new();
        s.push('你');
        s.push('好');
        assert_eq!(s.len(), 6); // 3 bytes per character
        assert_eq!(s.chars().count(), 2);
    }

    #[test]
    fn test_clone() {
        let s1 = String::from("hello");
        let s2 = s1.clone();
        assert_eq!(s1.as_str(), s2.as_str());
    }

    #[test]
    fn test_equality() {
        let s1 = String::from("hello");
        let s2 = String::from("hello");
        let s3 = String::from("world");
        assert!(s1.eq(&s2));
        assert!(!s1.eq(&s3));
    }

    #[test]
    fn test_ordering() {
        let s1 = String::from("abc");
        let s2 = String::from("def");
        assert!(s1.lt(&s2));
        assert!(s2.gt(&s1));
    }

    #[test]
    fn test_as_bytes() {
        let s = String::from("hello");
        assert_eq!(s.as_bytes(), &[104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_reserve() {
        let mut s = String::new();
        s.reserve(10);
        assert!(s.capacity() >= 10);
    }

    #[test]
    fn test_default() {
        let s: String = Default::default();
        assert!(s.is_empty());
    }

    #[test]
    fn test_remove_range() {
        let mut s = String::from("Hello, World!");
        s.remove_range(7..12);
        assert_eq!(s.as_str(), "Hello, !");
    }

    #[test]
    fn test_capacity() {
        let s = String::from("hello");
        assert!(s.capacity() >= 5);
    }

    // New tests for advanced methods

    #[test]
    fn test_trim() {
        let s = String::from("  hello  ");
        assert_eq!(s.trim().as_str(), "hello");
    }

    #[test]
    fn test_trim_start() {
        let s = String::from("  hello  ");
        assert_eq!(s.trim_start().as_str(), "hello  ");
    }

    #[test]
    fn test_trim_end() {
        let s = String::from("  hello  ");
        assert_eq!(s.trim_end().as_str(), "  hello");
    }

    #[test]
    fn test_split() {
        let s = String::from("hello-world-foo");
        let mut iter = s.split('-');
        assert_eq!(iter.next(), Some("hello"));
        assert_eq!(iter.next(), Some("world"));
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_split_whitespace() {
        let s = String::from("hello  world   foo");
        let mut iter = s.split_whitespace();
        assert_eq!(iter.next(), Some("hello"));
        assert_eq!(iter.next(), Some("world"));
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_replace() {
        let s = String::from("hello world");
        assert_eq!(s.replace("world", "rust").as_str(), "hello rust");
    }

    #[test]
    fn test_replacen() {
        let s = String::from("hello hello hello");
        assert_eq!(s.replacen("hello", "hi", 2).as_str(), "hi hi hello");
    }

    #[test]
    fn test_to_lowercase() {
        let s = String::from("HELLO");
        assert_eq!(s.to_lowercase().as_str(), "hello");
    }

    #[test]
    fn test_to_uppercase() {
        let s = String::from("hello");
        assert_eq!(s.to_uppercase().as_str(), "HELLO");
    }

    #[test]
    fn test_contains() {
        let s = String::from("hello world");
        assert!(s.contains("world"));
        assert!(!s.contains("rust"));
    }

    #[test]
    fn test_starts_with() {
        let s = String::from("hello world");
        assert!(s.starts_with("hello"));
        assert!(!s.starts_with("world"));
    }

    #[test]
    fn test_ends_with() {
        let s = String::from("hello world");
        assert!(s.ends_with("world"));
        assert!(!s.ends_with("hello"));
    }

    #[test]
    fn test_substring() {
        let s = String::from("hello world");
        assert_eq!(s.substring(0, 5).as_str(), "hello");
        assert_eq!(s.substring(6, 11).as_str(), "world");
    }

    #[test]
    fn test_split_empty() {
        let s = String::from("");
        let mut iter = s.split('-');
        assert_eq!(iter.next(), Some(""));
        assert_eq!(iter.next(), None);
    }
}
