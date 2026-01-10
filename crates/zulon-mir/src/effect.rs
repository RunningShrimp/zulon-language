// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Effect System
//!
//! ZULON's effect system tracks and controls side effects in functions.
//! Key concepts:
//!
//! - **Pure Functions**: No side effects (no IO, no state modification)
//! - **IO Effects**: Input/output operations
//! - **State Effects**: Modification of external state
//! - **Effect Polymorphism**: Functions can be generic over effects

use crate::error::{MirError, Result};
use crate::mir::*;
use std::collections::{HashMap, HashSet};

/// Effect kind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Input/output effect
    Io,

    /// Memory allocation effect
    Alloc,

    /// State modification effect
    State,

    /// Panic/divergence effect
    Panic,

    /// Non-termination effect
    NonTermination,
}

/// Effect set (combination of effects)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSet {
    effects: HashSet<Effect>,
}

impl EffectSet {
    /// Create an empty effect set (pure function)
    pub fn pure() -> Self {
        EffectSet {
            effects: HashSet::new(),
        }
    }

    /// Create an effect set with specific effects
    pub fn new(effects: Vec<Effect>) -> Self {
        EffectSet {
            effects: effects.into_iter().collect(),
        }
    }

    /// Check if the function is pure
    pub fn is_pure(&self) -> bool {
        self.effects.is_empty()
    }

    /// Check if the function has IO effects
    pub fn has_io(&self) -> bool {
        self.effects.contains(&Effect::Io)
    }

    /// Check if the function can panic
    pub fn can_panic(&self) -> bool {
        self.effects.contains(&Effect::Panic)
    }

    /// Add an effect to the set
    pub fn add(&mut self, effect: Effect) {
        self.effects.insert(effect);
    }

    /// Union two effect sets
    pub fn union(&self, other: &EffectSet) -> EffectSet {
        let mut result = self.effects.clone();
        result.extend(other.effects.clone());
        EffectSet { effects: result }
    }
}

/// Effect checker
pub struct EffectChecker {
    /// Current function being checked
    current_effects: EffectSet,

    /// Function effect signatures
    function_effects: HashMap<String, EffectSet>,
}

impl EffectChecker {
    /// Create a new effect checker
    pub fn new() -> Self {
        EffectChecker {
            current_effects: EffectSet::pure(),
            function_effects: HashMap::new(),
        }
    }

    /// Register a function's effect signature
    pub fn register_function(&mut self, name: String, effects: EffectSet) {
        self.function_effects.insert(name, effects);
    }

    /// Check a function for effects
    pub fn check_function(&mut self, func: &MirFunction) -> Result<EffectSet> {
        // Reset current effects
        self.current_effects = EffectSet::pure();

        // Check all basic blocks
        for block in func.blocks.values() {
            self.check_block(block)?;
        }

        Ok(self.current_effects.clone())
    }

    /// Check a basic block for effects
    fn check_block(&mut self, block: &MirBasicBlock) -> Result<()> {
        // Check all instructions
        for inst in &block.instructions {
            self.check_instruction(inst)?;
        }

        // Check terminator
        if let Some(terminator) = &block.terminator {
            self.check_terminator(terminator)?;
        }

        Ok(())
    }

    /// Check an instruction for effects
    fn check_instruction(&mut self, inst: &MirInstruction) -> Result<()> {
        match inst {
            MirInstruction::Call { func, args: _, dest: _, return_type: _ } => {
                // Check the callee's effects
                let func_name = match func {
                    MirPlace::Local(name) => name.clone(),
                    MirPlace::Temp(_) | MirPlace::Param(_) => {
                        // Indirect call - assume IO effect
                        self.current_effects.add(Effect::Io);
                        return Ok(());
                    }
                    _ => {
                        // Complex expression - assume IO effect
                        self.current_effects.add(Effect::Io);
                        return Ok(());
                    }
                };

                // Look up function effects
                if let Some(effects) = self.function_effects.get(&func_name) {
                    // Union the callee's effects with current effects
                    self.current_effects = self.current_effects.union(effects);
                } else {
                    // Unknown function - assume IO effect
                    self.current_effects.add(Effect::Io);
                }
            }

            MirInstruction::Borrow { dest: _, src: _, mutable: _, ty: _ } => {
                // Borrowing doesn't have effects (in this simplified model)
            }

            MirInstruction::Drop { place: _, ty } => {
                // Drop may have effects if the type has a destructor
                if ty.needs_drop() {
                    // Destructor could have arbitrary effects
                    self.current_effects.add(Effect::State);
                }
            }

            _ => {
                // Most instructions are pure
            }
        }

        Ok(())
    }

    /// Check a terminator for effects
    fn check_terminator(&mut self, terminator: &MirTerminator) -> Result<()> {
        match terminator {
            MirTerminator::Unreachable => {
                // Unreachable code implies panic/non-termination
                self.current_effects.add(Effect::Panic);
                self.current_effects.add(Effect::NonTermination);
            }

            MirTerminator::Return(_) => {
                // Return is pure
            }

            MirTerminator::Throw(_) => {
                // Throw is pure (error value is already computed)
            }

            MirTerminator::Goto { target: _ } => {
                // Goto is pure
            }

            MirTerminator::If { condition: _, then_block: _, else_block: _ } => {
                // If is pure (effects are in the blocks)
            }

            MirTerminator::Switch { scrutinee: _, targets: _, default: _ } => {
                // Switch is pure (effects are in the blocks)
            }

            MirTerminator::EffectCall { .. } => {
                // Effect calls will be transformed to regular calls later
                // For now, treat them as having no effect
            }
        }

        Ok(())
    }

    /// Check if a function's actual effects match its declared effects
    pub fn check_effect_compatibility(
        &self,
        func_name: &str,
        declared: &EffectSet,
        actual: &EffectSet,
    ) -> Result<()> {
        // Check if actual effects are a subset of declared effects
        for effect in &actual.effects {
            if !declared.effects.contains(effect) {
                return Err(MirError::TypeError(format!(
                    "Function '{}' has undeclared effect: {:?}",
                    func_name, effect
                )));
            }
        }

        Ok(())
    }
}

impl Default for EffectChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Public API for effect checking
pub fn check_effects(func: &MirFunction) -> Result<EffectSet> {
    let mut checker = EffectChecker::new();
    checker.check_function(func)
}
