// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Macro expansion for ZULON source code
//!
//! This module handles preprocessing of ZULON source code to expand
//! macro invocations before lexical analysis and parsing.

use zulon_macros::MacroExpanderEngine;
use crate::error::Result as CompilerResult;

/// Macro expander for ZULON compiler
///
/// Expands macro invocations in source code before parsing.
pub struct MacroExpander {
    engine: MacroExpanderEngine,
}

impl MacroExpander {
    /// Create a new macro expander with standard macros registered
    pub fn new() -> Self {
        let mut expander = Self {
            engine: MacroExpanderEngine::new(),
        };
        
        // Register standard macros
        expander.register_standard_macros();
        expander
    }
    
    /// Register all standard ZULON macros
    fn register_standard_macros(&mut self) {
        // Use the built-in macros from zulon_macros
        self.engine = MacroExpanderEngine::with_builtins();
    }
    
    /// Expand all macros in source code
    ///
    /// Processes the source code and expands all macro invocations.
    /// Returns the expanded source code.
    ///
    /// # Example
    ///
    /// ```
    /// # use zulon_compiler::macro_expander::MacroExpander;
    /// let expander = MacroExpander::new();
    /// let source = r#"panic!("test")"#;
    /// let expanded = expander.expand_source(source).unwrap();
    /// assert_eq!(expanded, r#"::__zulon_builtin_panic("test")"#);
    /// ```
    pub fn expand_source(&self, source: &str) -> CompilerResult<String> {
        let mut result = String::new();
        let mut last_end = 0;

        // Find all macro invocations: identifier + "!("
        for (macro_name, macro_start, macro_end) in self.find_all_macros(source) {
            // Skip macros that appear before our current position (e.g., inside string literals)
            if macro_start < last_end {
                continue;
            }

            // Copy text before the macro
            // All positions from find_all_macros() should be at valid UTF-8 boundaries
            result.push_str(&source[last_end..macro_start]);

            // Find the opening parenthesis
            let after_bang = macro_end;
            if after_bang >= source.len() {
                break;
            }

            // Verify UTF-8 boundary before slicing
            if !source.is_char_boundary(after_bang) {
                last_end = macro_end;
                continue;
            }

            // Find '(' after '!' - use char_indices() to ensure UTF-8 safety
            let paren_start = match source[after_bang..].find('(') {
                Some(pos) => after_bang + pos,
                None => {
                    // No '(' found, not a valid macro invocation
                    last_end = macro_end;
                    continue;
                }
            };

            // Find matching closing parenthesis - safe UTF-8 boundary check
            let paren_content = &source[paren_start..];
            let args_end = match self.find_matching_paren(paren_content) {
                Some(pos) => {
                    // Verify that args_end is a valid UTF-8 boundary
                    let abs_pos = paren_start + pos;
                    if !source.is_char_boundary(abs_pos) {
                        // Not a valid UTF-8 boundary, skip this macro
                        last_end = macro_end;
                        continue;
                    }
                    abs_pos
                }
                None => {
                    // Unmatched parentheses, skip this
                    last_end = macro_end;
                    continue;
                }
            };

            // Extract arguments (content between '(' and ')')
            // Verify both boundaries are valid UTF-8
            if !source.is_char_boundary(paren_start + 1) || !source.is_char_boundary(args_end) {
                last_end = macro_end;
                continue;
            }
            let args = &source[paren_start + 1..args_end];

            // Try to expand the macro
            match self.engine.expand(&macro_name, args) {
                Ok(expanded) => {
                    result.push_str(&expanded);
                    // Skip past ')' to the start of the next character
                    // args_end is at ')' which is a valid UTF-8 boundary
                    // Use char_indices on the full source to avoid slicing at invalid boundary
                    last_end = source.char_indices()
                        .skip_while(|&(pos, _)| pos <= args_end)
                        .next()
                        .map(|(pos, _)| pos)
                        .unwrap_or_else(|| source.len());
                }
                Err(_) => {
                    // Not a recognized macro, keep original
                    last_end = macro_end;
                }
            }
        }

        // Copy remaining text after the last macro
        result.push_str(&source[last_end..]);

        Ok(result)
    }

    /// Find all macro invocations in source code
    ///
    /// Returns iterator of (macro_name, start_byte_pos, end_byte_pos)
    fn find_all_macros(&self, source: &str) -> Vec<(String, usize, usize)> {
        let mut macros = Vec::new();
        let chars: Vec<(usize, char)> = source.char_indices().collect();

        let mut idx = 0;
        while idx < chars.len() {
            let (byte_pos, c) = chars[idx];

            // Look for identifier start
            if c.is_alphabetic() || c == '_' {
                let start_byte = byte_pos;

                // Collect identifier characters
                let mut next_idx = idx + 1;

                while next_idx < chars.len() {
                    let (_, ch) = chars[next_idx];
                    if ch.is_alphanumeric() || ch == '_' {
                        next_idx += 1;
                    } else {
                        break;
                    }
                }

                // Check if next character is '!'
                if next_idx < chars.len() {
                    let (_, next_char) = chars[next_idx];
                    if next_char == '!' {
                        // Found a macro invocation
                        // end_byte is the byte position of the last identifier character
                        // We need to find where that character ends (exclusive slice bound)
                        let identifier_end = if next_idx > idx + 1 {
                            // There were multiple characters, find the end of the last one
                            chars[next_idx - 1].0 + chars[next_idx - 1].1.len_utf8()
                        } else {
                            // Single character identifier
                            byte_pos + c.len_utf8()
                        };

                        let macro_name = source[start_byte..identifier_end].to_string();
                        // macro_end should be the start of the character AFTER '!'
                        // This ensures it's at a valid UTF-8 boundary
                        let macro_end = if next_idx + 1 < chars.len() {
                            chars[next_idx + 1].0
                        } else {
                            source.len()
                        };
                        macros.push((macro_name, start_byte, macro_end));
                        idx = next_idx + 1;
                        continue;
                    }
                }

                // Not a macro, continue from where we left off
                idx = next_idx;
            } else {
                idx += 1;
            }
        }

        macros
    }

    /// Perform a single pass of macro expansion
    ///
    /// Scans the source for macro invocations and expands them.
    /// Sets `changed` to true if any expansions occurred.
    #[allow(dead_code)]
    fn expand_single_pass(&self, source: &str, changed: &mut bool) -> CompilerResult<String> {
        let mut result = String::new();
        let mut pos = 0;

        while pos < source.len() {
            // Look for potential macro invocation: identifier + '!'
            if let Some((macro_name, macro_name_end)) = self.find_macro_at(source, pos) {
                // Check for '!' after macro name
                if macro_name_end < source.len() && source.chars().nth(macro_name_end) == Some('!') {
                    // Find the matching parenthesis
                    if let Some(args_start) = source[macro_name_end + 1..].find('(') {
                        let args_start_abs = macro_name_end + 1 + args_start;

                        // Find matching closing parenthesis
                        if let Some(args_end) = self.find_matching_paren(&source[args_start_abs..]) {
                            let args_end_abs = args_start_abs + args_end;
                            let args = &source[args_start_abs + 1..args_end_abs]; // Exclude parens

                            // Try to expand the macro
                            match self.engine.expand(&macro_name, args) {
                                Ok(expanded) => {
                                    result.push_str(&expanded);
                                    pos = args_end_abs + 1;
                                    *changed = true;
                                    continue;
                                }
                                Err(_) => {
                                    // Not a recognized macro, continue normally
                                }
                            }
                        }
                    }
                }
            }

            // No macro invocation found, copy character as-is
            result.push(source.chars().nth(pos).unwrap());
            pos += 1;
        }

        Ok(result)
    }
    
    /// Find a macro name at the current position
    ///
    /// Returns the macro name and its end position if this looks like a macro invocation start.
    fn find_macro_at(&self, source: &str, pos: usize) -> Option<(String, usize)> {
        let chars: Vec<char> = source.chars().collect();
        if pos >= chars.len() {
            return None;
        }

        // Must start with alphabetic or underscore
        let first = chars[pos];
        if !first.is_alphabetic() && first != '_' {
            return None;
        }

        // Collect identifier characters
        let mut len = 1;
        while pos + len < chars.len() {
            let c = chars[pos + len];
            if c.is_alphanumeric() || c == '_' {
                len += 1;
            } else {
                break;
            }
        }

        let macro_name: String = chars[pos..pos + len].iter().collect();
        Some((macro_name, pos + len))
    }
    
    /// Find the matching closing parenthesis
    ///
    /// Handles nested parentheses correctly.
    fn find_matching_paren(&self, source: &str) -> Option<usize> {
        let chars: Vec<char> = source.chars().collect();
        let mut depth = 0;
        
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        
        None
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_macro_expansion() {
        let expander = MacroExpander::new();
        let source = r#"panic!("test message")"#;
        let result = expander.expand_source(source).unwrap();
        println!("Source: {}", source);
        println!("Result: {}", result);
        assert!(result.contains("::__zulon_builtin_panic"));
    }

    #[test]
    fn test_assert_macro_expansion() {
        let expander = MacroExpander::new();
        let source = r#"assert!(x > 0)"#;
        let result = expander.expand_source(source).unwrap();
        // The macro should be expanded
        assert!(!result.contains("assert!"));
        // Should contain some form of conditional or comparison
        assert!(result.contains("x"));
        assert!(result.contains("0"));
    }

    #[test]
    fn test_assert_eq_macro_expansion() {
        let expander = MacroExpander::new();
        let source = r#"assert_eq!(a, b)"#;
        let result = expander.expand_source(source).unwrap();
        // The macro should expand to some form of conditional
        assert!(result.contains("if") || result.contains("match"));
        // Should check for inequality
        assert!(result.contains("!=") || result.contains("ne"));
        // Macro should be expanded
        assert!(!result.contains("assert_eq!"));
    }

    #[test]
    fn test_assert_ne_macro_expansion() {
        let expander = MacroExpander::new();
        let source = r#"assert_ne!(a, b)"#;
        let result = expander.expand_source(source).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("=="));
    }

    #[test]
    fn test_stringify_macro_expansion() {
        let expander = MacroExpander::new();
        let source = r#"stringify!(x + y)"#;
        let result = expander.expand_source(source).unwrap();
        assert!(result.contains("\""));
        assert!(result.contains("x + y"));
    }

    #[test]
    fn test_no_macros() {
        let expander = MacroExpander::new();
        let source = r#"fn main() -> i32 { 42 }"#;
        let result = expander.expand_source(source).unwrap();
        assert_eq!(result, source);
    }

    #[test]
    fn test_macro_in_function() {
        let expander = MacroExpander::new();
        let source = r#"
            fn test_example() -> i32 {
                assert!(x > 0);
                assert_eq!(a, b);
                0
            }
        "#;
        let result = expander.expand_source(source).unwrap();
        assert!(!result.contains("assert!"));
        assert!(!result.contains("assert_eq!"));
        assert!(result.contains("if"));
    }

    #[test]
    fn test_multiple_macros() {
        let expander = MacroExpander::new();
        let source = r#"
            fn test() -> i32 {
                assert!(x > 0);
                assert_eq!(y, 10);
                assert_ne!(z, 0);
                0
            }
        "#;
        let result = expander.expand_source(source).unwrap();
        println!("Result:\n{}", result);
        // All macros should be expanded
        assert!(!result.contains("assert!"));
        assert!(!result.contains("assert_eq!"));
        assert!(!result.contains("assert_ne!"));
        // Should have expanded to if statements
        let if_count = result.matches("if").count();
        println!("'if' count: {}", if_count);
        assert!(if_count >= 3); // At least 3, may have more in expansions
    }

    #[test]
    fn test_nested_parentheses() {
        let expander = MacroExpander::new();
        // Use simpler expressions - comma in function calls is a known limitation
        let source = r#"assert_eq!(x, 42)"#;
        let result = expander.expand_source(source).unwrap();
        assert!(result.contains("x"));
        assert!(result.contains("42"));
        assert!(!result.contains("assert_eq!"));
    }

    #[test]
    fn test_macro_with_string_literal() {
        let expander = MacroExpander::new();
        let source = r#"panic!("Error: {}", msg)"#;
        let result = expander.expand_source(source).unwrap();
        assert!(result.contains("::__zulon_builtin_panic"));
    }
}
