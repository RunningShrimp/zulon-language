// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Effect System for ZULON
//!
//! This module implements the effect system for tracking and validating
//! side effects in ZULON programs.

use std::collections::HashSet;
use std::fmt;

/// A single effect that a function may have
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// I/O effect (reading/writing to external world)
    IO,

    /// Memory allocation effect
    Alloc,

    /// Mutation effect (modifying specific variable)
    Mut(String),

    /// Async effect (async/await)
    Async,

    /// Throws effect (can throw specific error type)
    Throws(String),

    /// Custom user-defined effect
    Custom(String),

    /// Combination of multiple effects
    All(Vec<Effect>),
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::IO => write!(f, "IO"),
            Effect::Alloc => write!(f, "Alloc"),
            Effect::Mut(name) => write!(f, "Mut({})", name),
            Effect::Async => write!(f, "Async"),
            Effect::Throws(ty) => write!(f, "Throws({})", ty),
            Effect::Custom(name) => write!(f, "{}", name),
            Effect::All(effects) => {
                write!(f, "[")?;
                for (i, effect) in effects.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", effect)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// A set of effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSet {
    effects: HashSet<Effect>,
}

impl EffectSet {
    /// Create a new empty effect set (pure function)
    pub fn new() -> Self {
        EffectSet {
            effects: HashSet::new(),
        }
    }

    /// Create a pure effect set
    pub fn pure() -> Self {
        Self::new()
    }

    /// Insert an effect into the set
    pub fn insert(&mut self, effect: Effect) {
        match effect {
            Effect::All(effects) => {
                for e in effects {
                    self.effects.insert(e);
                }
            }
            _ => {
                self.effects.insert(effect);
            }
        }
    }

    /// Check if the set contains a specific effect
    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    /// Check if the set is empty (pure function)
    pub fn is_pure(&self) -> bool {
        self.effects.is_empty()
    }

    /// Get the number of effects in the set
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Union two effect sets
    pub fn union(&self, other: &EffectSet) -> EffectSet {
        let mut result = EffectSet::new();
        for effect in &self.effects {
            result.effects.insert(effect.clone());
        }
        for effect in &other.effects {
            result.effects.insert(effect.clone());
        }
        result
    }

    /// Check if self is a subset of other
    pub fn is_subset(&self, other: &EffectSet) -> bool {
        self.effects.is_subset(&other.effects)
    }

    /// Get the difference (self - other)
    pub fn difference(&self, other: &EffectSet) -> EffectSet {
        let mut result = EffectSet::new();
        for effect in &self.effects {
            if !other.effects.contains(effect) {
                result.effects.insert(effect.clone());
            }
        }
        result
    }

    /// Convert to vector
    pub fn to_vec(&self) -> Vec<Effect> {
        self.effects.iter().cloned().collect()
    }

    /// Create an IO effect set
    pub fn io() -> Self {
        let mut set = EffectSet::new();
        set.insert(Effect::IO);
        set
    }

    /// Create an allocation effect set
    pub fn alloc() -> Self {
        let mut set = EffectSet::new();
        set.insert(Effect::Alloc);
        set
    }

    /// Create an async effect set
    pub fn async_effect() -> Self {
        let mut set = EffectSet::new();
        set.insert(Effect::Async);
        set
    }

    /// Parse effect from string
    pub fn from_str(s: &str) -> Option<Effect> {
        match s {
            "IO" => Some(Effect::IO),
            "Alloc" => Some(Effect::Alloc),
            "Async" => Some(Effect::Async),
            _ if s.starts_with("Mut(") => {
                let name = s.strip_prefix("Mut(")?.strip_suffix(")")?;
                Some(Effect::Mut(name.to_string()))
            }
            _ if s.starts_with("Throws(") => {
                let ty = s.strip_prefix("Throws(")?.strip_suffix(")")?;
                Some(Effect::Throws(ty.to_string()))
            }
            _ => Some(Effect::Custom(s.to_string())),
        }
    }
}

impl Default for EffectSet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pure() {
            write!(f, "Pure")
        } else {
            let effects: Vec<_> = self.to_vec();
            write!(f, "[")?;
            for (i, effect) in effects.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", effect)?;
            }
            write!(f, "]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_creation() {
        let io = Effect::IO;
        assert_eq!(io.to_string(), "IO");

        let alloc = Effect::Alloc;
        assert_eq!(alloc.to_string(), "Alloc");

        let mut_effect = Effect::Mut("x".to_string());
        assert_eq!(mut_effect.to_string(), "Mut(x)");
    }

    #[test]
    fn test_effect_set_pure() {
        let pure = EffectSet::pure();
        assert!(pure.is_pure());
        assert_eq!(pure.len(), 0);
        assert_eq!(pure.to_string(), "Pure");
    }

    #[test]
    fn test_effect_set_insert() {
        let mut set = EffectSet::new();
        set.insert(Effect::IO);
        assert!(!set.is_pure());
        assert_eq!(set.len(), 1);
        assert!(set.contains(&Effect::IO));
    }

    #[test]
    fn test_effect_set_union() {
        let mut io = EffectSet::new();
        io.insert(Effect::IO);

        let mut alloc = EffectSet::new();
        alloc.insert(Effect::Alloc);

        let combined = io.union(&alloc);
        assert_eq!(combined.len(), 2);
        assert!(combined.contains(&Effect::IO));
        assert!(combined.contains(&Effect::Alloc));
    }

    #[test]
    fn test_effect_set_subset() {
        let mut io = EffectSet::new();
        io.insert(Effect::IO);

        let mut io_alloc = EffectSet::new();
        io_alloc.insert(Effect::IO);
        io_alloc.insert(Effect::Alloc);

        assert!(io.is_subset(&io_alloc));
        assert!(!io_alloc.is_subset(&io));
    }

    #[test]
    fn test_effect_set_difference() {
        let mut io_alloc = EffectSet::new();
        io_alloc.insert(Effect::IO);
        io_alloc.insert(Effect::Alloc);

        let mut io = EffectSet::new();
        io.insert(Effect::IO);

        let diff = io_alloc.difference(&io);
        assert_eq!(diff.len(), 1);
        assert!(diff.contains(&Effect::Alloc));
    }

    #[test]
    fn test_effect_from_str() {
        assert_eq!(EffectSet::from_str("IO"), Some(Effect::IO));
        assert_eq!(EffectSet::from_str("Alloc"), Some(Effect::Alloc));
        assert_eq!(EffectSet::from_str("Async"), Some(Effect::Async));
        assert_eq!(EffectSet::from_str("Mut(x)"), Some(Effect::Mut("x".to_string())));
        assert_eq!(
            EffectSet::from_str("Throws(Error)"),
            Some(Effect::Throws("Error".to_string()))
        );
    }

    #[test]
    fn test_effect_set_helpers() {
        let io = EffectSet::io();
        assert!(io.contains(&Effect::IO));
        assert_eq!(io.len(), 1);

        let alloc = EffectSet::alloc();
        assert!(alloc.contains(&Effect::Alloc));
        assert_eq!(alloc.len(), 1);

        let async_effect = EffectSet::async_effect();
        assert!(async_effect.contains(&Effect::Async));
        assert_eq!(async_effect.len(), 1);
    }
}
