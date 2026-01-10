// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Token type for identifying event sources

/// Token for identifying event sources
///
/// Tokens are returned when registering event sources with an event loop.
/// They are used to identify which event source generated an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token(usize);

impl Token {
    /// Create a new token
    ///
    /// # Arguments
    ///
    /// * `index` - Unique identifier
    #[inline]
    pub const fn new(index: usize) -> Self {
        Token(index)
    }

    /// Get the token index
    #[inline]
    pub const fn index(&self) -> usize {
        self.0
    }
}

impl From<usize> for Token {
    #[inline]
    fn from(index: usize) -> Self {
        Token(index)
    }
}

impl From<Token> for usize {
    #[inline]
    fn from(token: Token) -> Self {
        token.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_new() {
        let token = Token::new(10);
        assert_eq!(token.index(), 10);
    }

    #[test]
    fn test_token_from_usize() {
        let token: Token = 42.into();
        assert_eq!(token.index(), 42);
    }

    #[test]
    fn test_token_into_usize() {
        let token = Token::new(99);
        let value: usize = token.into();
        assert_eq!(value, 99);
    }

    #[test]
    fn test_token_copy() {
        let token1 = Token::new(5);
        let token2 = token1;
        assert_eq!(token1, token2);
        assert_eq!(token1.index(), 5);
    }
}
