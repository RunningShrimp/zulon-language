// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Closure capture analysis
//!
//! This module implements capture analysis for closures, detecting which variables
//! from outer scopes are used in closure bodies and determining the capture mode.
//!
//! # Capture Modes
//!
//! - **ImmutableRef** (`&x`): Read-only access to the variable
//! - **MutableRef** (`&mut x`): The closure modifies the variable
//! - **ByValue** (`x`): The closure takes ownership of the variable
//!
//! # Algorithm
//!
//! 1. Walk the closure body expression tree
//! 2. For each variable reference, check if it's:
//!    - A closure parameter (not captured)
//!    - A local variable in the closure (not captured)
//!    - A variable from outer scope (captured)
//! 3. Determine capture mode based on usage pattern

use std::collections::{HashMap, HashSet};

use super::{HirCapture, HirCaptureMode, HirExpression, HirTy, HirStatement};
use zulon_parser::Span;

/// Environment trait for variable scope information
///
/// This allows capture analysis to work with different type checker implementations.
pub trait Environment {
    /// Check if a variable exists in the outer scope
    fn contains(&self, name: &str) -> bool;

    /// Get the type of a variable
    fn get_type(&self, name: &str) -> Option<HirTy>;
}

/// Capture analysis result
#[derive(Debug, Clone)]
pub struct CaptureAnalysis {
    /// All captures found in the closure
    pub captures: Vec<HirCapture>,
    /// Variables that are captured by immutable reference
    pub immutable_refs: HashSet<String>,
    /// Variables that are captured by mutable reference
    pub mutable_refs: HashSet<String>,
    /// Variables that are captured by value
    pub by_value: HashSet<String>,
}

/// Capture analyzer for closures
pub struct CaptureAnalyzer<'a, E: Environment> {
    /// Environment (for variable scope information)
    env: &'a E,
    /// Closure parameter names (to exclude from captures)
    closure_params: HashSet<String>,
    /// Variables that are captured (name -> (type, mode, span))
    captures: HashMap<String, (HirTy, HirCaptureMode, Span)>,
    /// Variables defined in the closure body (local variables)
    local_vars: HashSet<String>,
}

impl<'a, E: Environment> CaptureAnalyzer<'a, E> {
    /// Create a new capture analyzer
    pub fn new(env: &'a E, closure_params: Vec<String>) -> Self {
        CaptureAnalyzer {
            env,
            closure_params: closure_params.into_iter().collect(),
            captures: HashMap::new(),
            local_vars: HashSet::new(),
        }
    }

    /// Analyze a closure expression for captures
    pub fn analyze(&mut self, closure: &HirExpression) -> CaptureAnalysis {
        // Walk the closure body to find captures
        self.walk_expression(closure);

        // Build the result
        let mut immutable_refs = HashSet::new();
        let mut mutable_refs = HashSet::new();
        let mut by_value = HashSet::new();

        let captures: Vec<HirCapture> = self
            .captures
            .iter()
            .map(|(name, (ty, mode, span))| {
                // Track by mode
                match mode {
                    HirCaptureMode::ImmutableRef => {
                        immutable_refs.insert(name.clone());
                    }
                    HirCaptureMode::MutableRef => {
                        mutable_refs.insert(name.clone());
                    }
                    HirCaptureMode::ByValue => {
                        by_value.insert(name.clone());
                    }
                }

                HirCapture {
                    name: name.clone(),
                    mode: *mode,
                    ty: ty.clone(),
                    span: span.clone(),
                }
            })
            .collect();

        CaptureAnalysis {
            captures,
            immutable_refs,
            mutable_refs,
            by_value,
        }
    }

    /// Walk an expression tree to find variable captures
    fn walk_expression(&mut self, expr: &HirExpression) {
        match expr {
            // Variable reference - check if it should be captured
            HirExpression::Variable(name, _id, _ty, span) => {
                self.handle_variable_reference(name, span);
            }

            // Binary operation - check for assignment, then walk both sides
            HirExpression::BinaryOp { op, left, right, .. } => {
                // Check if this is an assignment to a variable
                if *op == super::HirBinOp::Assign {
                    if let HirExpression::Variable(name, _id, _ty, span) = &**left {
                        // This is a variable being modified
                        if self.should_capture(name) {
                            self.record_capture(
                                name.clone(),
                                HirCaptureMode::MutableRef,
                                span.clone(),
                            );
                        }
                    }
                }

                // Walk both sides
                self.walk_expression(left);
                self.walk_expression(right);
            }

            // Unary operation - walk operand
            HirExpression::UnaryOp { operand, .. } => {
                self.walk_expression(operand);
            }

            // Function call - walk function and arguments
            HirExpression::Call { func, args, .. } => {
                self.walk_expression(func);
                for arg in args {
                    self.walk_expression(arg);
                }
            }

            // Block - walk statements and trailing expression
            HirExpression::Block(block) => {
                for stmt in &block.statements {
                    self.walk_statement(stmt);
                }

                if let Some(trailing) = &block.trailing_expr {
                    self.walk_expression(trailing);
                }
            }

            // If expression - walk condition, then, and else blocks
            HirExpression::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.walk_expression(condition);
                // Walk then_block by creating a temporary Block expression
                self.walk_expression(&HirExpression::Block(then_block.clone()));
                if let Some(else_block) = else_block {
                    self.walk_expression(&HirExpression::Block(else_block.clone()));
                }
            }

            // Loop expressions - walk body
            HirExpression::Loop { body, .. } => {
                self.walk_expression(&HirExpression::Block(body.clone()));
            }

            HirExpression::While { condition, body, .. } => {
                self.walk_expression(condition);
                self.walk_expression(&HirExpression::Block(body.clone()));
            }

            HirExpression::For {
                pattern,
                iter,
                body,
                ..
            } => {
                // For loops introduce a new binding
                if let super::HirPattern::Binding(name, _ty, _span) = pattern {
                    self.local_vars.insert(name.clone());
                }

                self.walk_expression(iter);
                self.walk_expression(&HirExpression::Block(body.clone()));
            }

            // Return expression - walk the value
            HirExpression::Return(value, _span) => {
                if let Some(v) = value {
                    self.walk_expression(v);
                }
            }

            // Break expression - walk the value
            HirExpression::Break(value, _span) => {
                if let Some(v) = value {
                    self.walk_expression(v);
                }
            }

            // Closure - nested closure, walk its body
            HirExpression::Closure { body, .. } => {
                self.walk_expression(body);
            }

            // Literals - no captures
            HirExpression::Literal(_, _, _, _) => {}

            // Continue - no captures
            HirExpression::Continue(_) => {}

            // Other expression types - ignore for now
            _ => {}
        }
    }

    /// Walk a statement to find variable captures
    fn walk_statement(&mut self, stmt: &HirStatement) {
        match stmt {
            HirStatement::Local(local) => {
                // This is a local variable definition in the closure
                self.local_vars.insert(local.name.clone());

                // Walk the initializer
                if let Some(init) = &local.init {
                    self.walk_expression(init);
                }
            }

            HirStatement::Expression(expr) => {
                self.walk_expression(expr);
            }

            HirStatement::Semi(expr) => {
                self.walk_expression(expr);
            }

            _ => {}
        }
    }

    /// Handle a variable reference - determine if it should be captured
    fn handle_variable_reference(&mut self, name: &str, span: &Span) {
        // Don't capture if it's a closure parameter
        if self.closure_params.contains(name) {
            return;
        }

        // Don't capture if it's a local variable in the closure body
        if self.local_vars.contains(name) {
            return;
        }

        // Check if it exists in outer scope
        if self.is_outer_variable(name) {
            self.record_capture(name.to_string(), HirCaptureMode::ImmutableRef, span.clone());
        }
    }

    /// Check if a variable should be captured
    fn should_capture(&self, name: &str) -> bool {
        // Don't capture closure parameters
        if self.closure_params.contains(name) {
            return false;
        }

        // Don't capture local variables
        if self.local_vars.contains(name) {
            return false;
        }

        // Capture if it's from outer scope
        self.is_outer_variable(name)
    }

    /// Check if a variable is from outer scope (exists in environment)
    fn is_outer_variable(&self, name: &str) -> bool {
        // Check if the variable exists in the environment
        // This indicates it's from an outer scope
        self.env.contains(name)
    }

    /// Record a capture for a variable
    fn record_capture(&mut self, name: String, mode: HirCaptureMode, span: Span) {
        // Get the variable's type from the environment
        let ty = self.env.get_type(&name).unwrap_or(HirTy::Unit);

        // Check if we've already captured this variable
        if let Some((_, existing_mode, _)) = self.captures.get(&name) {
            // Upgrade capture mode if necessary
            let new_mode = self.merge_capture_modes(*existing_mode, mode);
            self.captures.insert(name, (ty, new_mode, span));
            return;
        }

        // Record new capture
        self.captures.insert(name, (ty, mode, span));
    }

    /// Merge two capture modes, returning the more restrictive one
    fn merge_capture_modes(&self, existing: HirCaptureMode, new: HirCaptureMode) -> HirCaptureMode {
        match (existing, new) {
            // ImmutableRef + anything → the more restrictive mode
            (HirCaptureMode::ImmutableRef, new) => new,

            // MutableRef + MutableRef → MutableRef
            (HirCaptureMode::MutableRef, HirCaptureMode::MutableRef) => {
                HirCaptureMode::MutableRef
            }

            // MutableRef + ByValue → ByValue (ownership transfer)
            (HirCaptureMode::MutableRef, HirCaptureMode::ByValue) => HirCaptureMode::ByValue,

            // ByValue + anything → ByValue
            (HirCaptureMode::ByValue, _) => HirCaptureMode::ByValue,

            // Default to more restrictive
            (_, _) => new,
        }
    }
}

/// Convenience function to analyze captures in a closure
pub fn analyze_captures<E: Environment>(
    env: &E,
    closure: &HirExpression,
    closure_params: Vec<String>,
) -> CaptureAnalysis {
    let mut analyzer = CaptureAnalyzer::new(env, closure_params);
    analyzer.analyze(closure)
}

/// Simple environment implementation for testing
#[derive(Debug, Clone, Default)]
pub struct SimpleEnvironment {
    variables: HashMap<String, HirTy>,
}

impl SimpleEnvironment {
    /// Create a new empty environment
    pub fn new() -> Self {
        SimpleEnvironment {
            variables: HashMap::new(),
        }
    }

    /// Add a variable to the environment
    pub fn add(&mut self, name: String, ty: HirTy) {
        self.variables.insert(name, ty);
    }
}

impl Environment for SimpleEnvironment {
    fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    fn get_type(&self, name: &str) -> Option<HirTy> {
        self.variables.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_analyzer_creation() {
        let env = SimpleEnvironment::new();
        let params = vec!["x".to_string(), "y".to_string()];
        let _analyzer = CaptureAnalyzer::new(&env, params);
    }

    #[test]
    fn test_simple_environment() {
        let mut env = SimpleEnvironment::new();
        env.add("outer_var".to_string(), HirTy::I32);

        assert!(env.contains("outer_var"));
        assert!(!env.contains("nonexistent"));

        assert_eq!(env.get_type("outer_var"), Some(HirTy::I32));
        assert_eq!(env.get_type("nonexistent"), None);
    }
}
