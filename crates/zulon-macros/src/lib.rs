// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Macro system for ZULON
//!
//! Provides compile-time macro expansion capabilities.

use std::collections::HashMap;

/// Simple identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
}

impl Identifier {
    pub fn new(name: &str) -> Self {
        Identifier {
            name: name.to_string(),
        }
    }
}

/// Macro definition
#[derive(Debug, Clone)]
pub struct Macro {
    pub name: Identifier,
    pub rules: Vec<MacroRule>,
}

/// Macro expansion rule
#[derive(Debug, Clone)]
pub struct MacroRule {
    pub matcher: MacroMatcher,
    pub expander: MacroExpander,
}

/// Pattern matcher for macros
#[derive(Debug, Clone)]
pub struct MacroMatcher {
    pub patterns: Vec<PatternFragment>,
}

/// Macro expansion template
#[derive(Debug, Clone)]
pub struct MacroExpander {
    pub template: Vec<TemplateFragment>,
}

/// Fragment in pattern matching
#[derive(Debug, Clone)]
pub enum PatternFragment {
    /// Literal token
    Literal(String),
    /// Variable binding $name
    Var(String),
    /// Repetition $(...)* or $(...)+
    Repetition {
        inner: Vec<PatternFragment>,
        separator: Option<String>,
    },
}

/// Fragment in expansion template
#[derive(Debug, Clone)]
pub enum TemplateFragment {
    /// Literal text
    Literal(String),
    /// Variable reference $name
    Var(String),
    /// Repetition expansion
    Repetition {
        inner: Vec<TemplateFragment>,
        separator: Option<String>,
        var: String,
    },
}

/// Macro expansion engine
pub struct MacroExpanderEngine {
    macros: HashMap<String, Macro>,
}

impl MacroExpanderEngine {
    pub fn new() -> Self {
        MacroExpanderEngine {
            macros: HashMap::new(),
        }
    }

    /// Register a macro
    pub fn register_macro(&mut self, macro_def: Macro) {
        let name = macro_def.name.name.clone();
        self.macros.insert(name, macro_def);
    }

    /// Expand a macro invocation
    pub fn expand(&self, name: &str, input: &str) -> Result<String, String> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| format!("Macro '{}' not found", name))?;

        // Try each rule until one matches
        for rule in &macro_def.rules {
            if let Some(binding) = self.try_match(&rule.matcher, input) {
                return Ok(self.expand_template(&rule.expander, &binding));
            }
        }

        Err(format!("No matching rule for macro '{}'", name))
    }

    /// Try to match input against a pattern
    fn try_match(&self, matcher: &MacroMatcher, input: &str) -> Option<HashMap<String, String>> {
        let mut binding = HashMap::new();
        let mut pos = 0;

        for (i, frag) in matcher.patterns.iter().enumerate() {
            match frag {
                PatternFragment::Literal(lit) => {
                    if input.len() >= pos + lit.len() && &input[pos..pos + lit.len()] == lit {
                        pos += lit.len();
                    } else {
                        return None;
                    }
                }
                PatternFragment::Var(name) => {
                    // Find the next literal to determine where this var ends
                    let end_pos = if i + 1 < matcher.patterns.len() {
                        if let PatternFragment::Literal(next_lit) = &matcher.patterns[i + 1] {
                            if let Some(found_pos) = input[pos..].find(next_lit) {
                                pos + found_pos
                            } else {
                                input.len()
                            }
                        } else {
                            input.len()
                        }
                    } else {
                        input.len()
                    };

                    binding.insert(name.clone(), input[pos..end_pos].to_string());
                    pos = end_pos;
                }
                PatternFragment::Repetition { .. } => {
                    return None;
                }
            }
        }

        Some(binding)
    }

    /// Expand a template with variable bindings
    fn expand_template(&self, expander: &MacroExpander, binding: &HashMap<String, String>) -> String {
        let mut result = String::new();

        for frag in &expander.template {
            match frag {
                TemplateFragment::Literal(lit) => {
                    result.push_str(lit);
                }
                TemplateFragment::Var(name) => {
                    if let Some(value) = binding.get(name) {
                        result.push_str(value);
                    }
                }
                TemplateFragment::Repetition { .. } => {
                    // TODO
                }
            }
        }

        result
    }
}

impl Default for MacroExpanderEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in macros
impl MacroExpanderEngine {
    /// Create with all built-in macros registered
    pub fn with_builtins() -> Self {
        let mut engine = Self::new();
        engine.register_assert_macros();
        engine
    }

    /// Register all built-in macros
    fn register_assert_macros(&mut self) {
        // panic!($message)
        self.macros.insert("panic".to_string(), Macro {
            name: Identifier::new("panic"),
            rules: vec![
                // Simple form: panic!("message") - match just the message content
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("message".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "::__zulon_builtin_panic(".to_string()
                            ),
                            TemplateFragment::Var("message".to_string()),
                            TemplateFragment::Literal(")".to_string()),
                        ],
                    },
                },
                // Formatted form: panic!("format: {}", arg1, arg2)
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("format_string".to_string()),
                            PatternFragment::Literal(", ".to_string()),
                            PatternFragment::Var("args".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "::__zulon_builtin_panic_formatted(".to_string()
                            ),
                            TemplateFragment::Var("format_string".to_string()),
                            TemplateFragment::Literal(", ".to_string()),
                            TemplateFragment::Var("args".to_string()),
                            TemplateFragment::Literal(")".to_string()),
                        ],
                    },
                },
            ],
        });

        // stringify!($expr) - converts expression to string
        self.macros.insert("stringify".to_string(), Macro {
            name: Identifier::new("stringify"),
            rules: vec![
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("expr".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal("\"".to_string()),
                            TemplateFragment::Var("expr".to_string()),
                            TemplateFragment::Literal("\"".to_string()),
                        ],
                    },
                },
            ],
        });

        // assert!($condition)
        self.macros.insert("assert".to_string(), Macro {
            name: Identifier::new("assert"),
            rules: vec![
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("condition".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "if (".to_string()
                            ),
                            TemplateFragment::Var("condition".to_string()),
                            TemplateFragment::Literal(
                                ") { } else { return 1; }".to_string()
                            ),
                        ],
                    },
                },
            ],
        });

        // assert_eq!($left, $right)
        // Expands to an if statement that checks the condition
        // NOTE: Early return from if blocks is a known limitation in current MIR lowering
        // The macro compiles successfully but the return doesn't actually exit the function early
        // This will be fixed when proper early return support is implemented
        self.macros.insert("assert_eq".to_string(), Macro {
            name: Identifier::new("assert_eq"),
            rules: vec![
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("left".to_string()),
                            PatternFragment::Literal(", ".to_string()),
                            PatternFragment::Var("right".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "if (".to_string()
                            ),
                            TemplateFragment::Var("left".to_string()),
                            TemplateFragment::Literal(
                                " != ".to_string()
                            ),
                            TemplateFragment::Var("right".to_string()),
                            TemplateFragment::Literal(
                                ") { return 1; }".to_string()
                            ),
                        ],
                    },
                },
            ],
        });

        // assert_ne!($left, $right)
        self.macros.insert("assert_ne".to_string(), Macro {
            name: Identifier::new("assert_ne"),
            rules: vec![
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("left".to_string()),
                            PatternFragment::Literal(", ".to_string()),
                            PatternFragment::Var("right".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "if (".to_string()
                            ),
                            TemplateFragment::Var("left".to_string()),
                            TemplateFragment::Literal(
                                " == ".to_string()
                            ),
                            TemplateFragment::Var("right".to_string()),
                            TemplateFragment::Literal(
                                ") {\n    \
                                    ::__zulon_builtin_panic(\"assertion failed: \", stringify!(".to_string()
                            ),
                            TemplateFragment::Var("left".to_string()),
                            TemplateFragment::Literal(
                                "), \" == \", stringify!(".to_string()
                            ),
                            TemplateFragment::Var("right".to_string()),
                            TemplateFragment::Literal(
                                "));\n\
                                }".to_string()
                            ),
                        ],
                    },
                },
            ],
        });

        // println!($format_string, $args...)
        // Expands to a call to external printf function
        // Note: Requires "extern fn printf(s: &u8, ...) -> i32;" at module level
        // Simple form: println!("Hello")
        self.macros.insert("println".to_string(), Macro {
            name: Identifier::new("println"),
            rules: vec![
                // Simple form with just format string
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("format_string".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "printf(".to_string()
                            ),
                            TemplateFragment::Var("format_string".to_string()),
                            TemplateFragment::Literal(");\n".to_string()),
                        ],
                    },
                },
                // Form with arguments: println!("Value: {}", x)
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("format_string".to_string()),
                            PatternFragment::Literal(", ".to_string()),
                            PatternFragment::Var("args".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal(
                                "printf(".to_string()
                            ),
                            TemplateFragment::Var("format_string".to_string()),
                            TemplateFragment::Literal(", ".to_string()),
                            TemplateFragment::Var("args".to_string()),
                            TemplateFragment::Literal(");\n".to_string()),
                        ],
                    },
                },
            ],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expander_creation() {
        let engine = MacroExpanderEngine::new();
        assert_eq!(engine.macros.len(), 0);
    }

    #[test]
    fn test_builtin_macros() {
        let engine = MacroExpanderEngine::with_builtins();
        assert!(engine.macros.contains_key("assert"));
        assert!(engine.macros.contains_key("assert_eq"));
        assert!(engine.macros.contains_key("assert_ne"));
        assert!(engine.macros.contains_key("panic"));
        assert!(engine.macros.contains_key("stringify"));
        assert!(engine.macros.contains_key("println"));
    }

    #[test]
    fn test_panic_macro() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("panic", "\"test message\"");
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("panic! expansion: {}", expanded);
        assert!(expanded.contains("::__zulon_builtin_panic"));
    }

    #[test]
    fn test_stringify_macro() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("stringify", "x + y");
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("stringify! expansion: {}", expanded);
        assert!(expanded.contains("\"x + y\""));
    }

    #[test]
    fn test_assert_macro() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("assert", "x > 0");
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("assert! expansion: {}", expanded);
        // The macro should expand to some form of conditional
        assert!(expanded.contains("if") || expanded.contains("match"));
        // Should contain the condition
        assert!(expanded.contains("x > 0"));
    }

    #[test]
    fn test_assert_eq_macro() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("assert_eq", "a, b");
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("assert_eq! expansion: {}", expanded);
        // The macro should expand to some form of conditional
        assert!(expanded.contains("if") || expanded.contains("match"));
        // Should check for inequality
        assert!(expanded.contains("!=") || expanded.contains("ne"));
        // Should contain the operands
        assert!(expanded.contains("a") && expanded.contains("b"));
    }

    #[test]
    fn test_assert_ne_macro() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("assert_ne", "a, b");
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("assert_ne! expansion: {}", expanded);
        assert!(expanded.contains("if"));
        assert!(expanded.contains("=="));
    }

    #[test]
    fn test_simple_expansion() {
        let mut engine = MacroExpanderEngine::new();

        engine.macros.insert("test".to_string(), Macro {
            name: Identifier::new("test"),
            rules: vec![
                MacroRule {
                    matcher: MacroMatcher {
                        patterns: vec![
                            PatternFragment::Var("x".to_string()),
                        ],
                    },
                    expander: MacroExpander {
                        template: vec![
                            TemplateFragment::Literal("result: ".to_string()),
                            TemplateFragment::Var("x".to_string()),
                        ],
                    },
                },
            ],
        });

        // Test with proper input that matches pattern "!(...)"
        let result = engine.expand("test", "foo)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "result: foo)");
    }

    #[test]
    fn test_println_macro() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("println", r#""Hello, World!""#);
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("println! expansion: {}", expanded);
        assert!(expanded.contains("printf("));
        assert!(expanded.contains("Hello, World!"));
    }

    #[test]
    fn test_println_with_args() {
        let engine = MacroExpanderEngine::with_builtins();
        let result = engine.expand("println", r#""Value: {}", x"#);
        assert!(result.is_ok());
        let expanded = result.unwrap();
        println!("println! with args expansion: {}", expanded);
        assert!(expanded.contains("printf("));
        assert!(expanded.contains("Value: {}"));
        assert!(expanded.contains("x"));
    }
}
