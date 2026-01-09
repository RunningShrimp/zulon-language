// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Effect inference for ZULON
//!
//! This module implements automatic effect inference for functions,
//! allowing functions to inherit effects from the code they call.

use crate::effect::{Effect, EffectSet};
use crate::ty::Ty;

/// Effect inference engine
pub struct EffectInference {
    /// Known effects for functions
    #[allow(dead_code)]
    known_effects: Vec<(String, EffectSet)>,
}

impl EffectInference {
    /// Create a new effect inference engine
    pub fn new() -> Self {
        EffectInference {
            known_effects: Vec::new(),
        }
    }

    /// Infer the effects of a function from its body
    pub fn infer_function_effects(&self, _function_body: &Ty) -> EffectSet {
        // TODO: Implement full effect inference from AST
        // For now, return empty effect set (pure function)
        EffectSet::new()
    }

    /// Propagate effects from a function call to the caller
    pub fn propagate_call_effects(
        &self,
        caller_effects: &mut EffectSet,
        callee_name: &str,
        callee_effects: &EffectSet,
    ) {
        // Record the callee's effects
        self.record_known_effects(callee_name.to_string(), callee_effects.clone());

        // Propagate effects to caller
        for effect in callee_effects.to_vec() {
            caller_effects.insert(effect);
        }
    }

    /// Record known effects for a function
    pub fn record_known_effects(&self, _name: String, _effects: EffectSet) {
        // TODO: Store known effects for lookup
        // For now, this is a no-op
    }

    /// Check if a function's declared effects match its inferred effects
    pub fn check_effect_declaration(
        &self,
        _declared: &EffectSet,
        _inferred: &EffectSet,
    ) -> bool {
        // TODO: Implement effect declaration checking
        // For now, always return true (allow any declaration)
        true
    }
}

impl Default for EffectInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_creation() {
        let inference = EffectInference::new();
        assert_eq!(inference.known_effects.len(), 0);
    }

    #[test]
    fn test_propagate_call_effects() {
        let inference = EffectInference::new();

        let mut caller_effects = EffectSet::new();
        let mut callee_effects = EffectSet::new();
        callee_effects.insert(Effect::IO);

        inference.propagate_call_effects(
            &mut caller_effects,
            "println",
            &callee_effects,
        );

        // Caller should now have IO effect
        assert!(caller_effects.contains(&Effect::IO));
    }

    #[test]
    fn test_propagate_multiple_effects() {
        let inference = EffectInference::new();

        let mut caller_effects = EffectSet::new();
        let mut callee_effects = EffectSet::new();
        callee_effects.insert(Effect::IO);
        callee_effects.insert(Effect::Alloc);

        inference.propagate_call_effects(
            &mut caller_effects,
            "complex_function",
            &callee_effects,
        );

        // Caller should have both effects
        assert!(caller_effects.contains(&Effect::IO));
        assert!(caller_effects.contains(&Effect::Alloc));
        assert_eq!(caller_effects.len(), 2);
    }

    #[test]
    fn test_infer_function_effects() {
        let inference = EffectInference::new();

        // TODO: When we have AST, we can infer from body
        let inferred = inference.infer_function_effects(&Ty::Unit);
        assert!(inferred.is_pure());
    }

    #[test]
    fn test_check_effect_declaration() {
        let inference = EffectInference::new();

        let mut declared = EffectSet::new();
        declared.insert(Effect::IO);

        let mut inferred = EffectSet::new();
        inferred.insert(Effect::IO);

        // Should match
        assert!(inference.check_effect_declaration(&declared, &inferred));

        // Add extra effect to inferred
        inferred.insert(Effect::Alloc);

        // TODO: Should fail when checking is implemented
        // For now, always returns true
    }
}
