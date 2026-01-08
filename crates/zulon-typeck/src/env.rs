// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type environment
//!
//! The type environment tracks variable bindings, function signatures,
//! and type definitions during type checking.

use std::collections::HashMap;
use crate::ty::{Ty, TyVarId, Effect};

/// Type environment - tracks bindings and definitions
#[derive(Debug, Clone)]
pub struct Env {
    /// Variable bindings: name -> type
    bindings: HashMap<String, Ty>,

    /// Type definitions: name -> type
    type_defs: HashMap<String, Ty>,

    /// Function signatures: name -> function type
    functions: HashMap<String, Ty>,

    /// Effect declarations: name -> effect
    effects: HashMap<String, Effect>,

    /// Parent environment (for scoping)
    parent: Option<Box<Env>>,

    /// Next type variable ID
    next_ty_var: TyVarId,
}

impl Env {
    /// Create a new empty environment
    pub fn new() -> Self {
        Env {
            bindings: HashMap::new(),
            type_defs: HashMap::new(),
            functions: HashMap::new(),
            effects: HashMap::new(),
            parent: None,
            next_ty_var: 0,
        }
    }

    /// Create a new environment with a parent
    pub fn with_parent(parent: Env) -> Self {
        Env {
            bindings: HashMap::new(),
            type_defs: HashMap::new(),
            functions: HashMap::new(),
            effects: HashMap::new(),
            parent: Some(Box::new(parent)),
            next_ty_var: 0,
        }
    }

    /// Insert a variable binding
    pub fn insert_binding(&mut self, name: String, ty: Ty) {
        self.bindings.insert(name, ty);
    }

    /// Lookup a variable binding
    pub fn lookup_binding(&self, name: &str) -> Option<Ty> {
        // Check current environment
        if let Some(ty) = self.bindings.get(name) {
            return Some(ty.clone());
        }

        // Check parent environment
        if let Some(parent) = &self.parent {
            parent.lookup_binding(name)
        } else {
            None
        }
    }

    /// Insert a type definition
    pub fn insert_type_def(&mut self, name: String, ty: Ty) {
        self.type_defs.insert(name, ty);
    }

    /// Lookup a type definition
    pub fn lookup_type_def(&self, name: &str) -> Option<Ty> {
        // Check current environment
        if let Some(ty) = self.type_defs.get(name) {
            return Some(ty.clone());
        }

        // Check parent environment
        if let Some(parent) = &self.parent {
            parent.lookup_type_def(name)
        } else {
            None
        }
    }

    /// Insert a function signature
    pub fn insert_function(&mut self, name: String, ty: Ty) {
        self.functions.insert(name, ty);
    }

    /// Lookup a function signature
    pub fn lookup_function(&self, name: &str) -> Option<Ty> {
        // Check current environment
        if let Some(ty) = self.functions.get(name) {
            return Some(ty.clone());
        }

        // Check parent environment
        if let Some(parent) = &self.parent {
            parent.lookup_function(name)
        } else {
            None
        }
    }

    /// Insert an effect declaration
    pub fn insert_effect(&mut self, name: String, effect: Effect) {
        self.effects.insert(name, effect);
    }

    /// Lookup an effect declaration
    pub fn lookup_effect(&self, name: &str) -> Option<Effect> {
        // Check current environment
        if let Some(effect) = self.effects.get(name) {
            return Some(effect.clone());
        }

        // Check parent environment
        if let Some(parent) = &self.parent {
            parent.lookup_effect(name)
        } else {
            None
        }
    }

    /// Create a fresh type variable
    pub fn fresh_ty_var(&mut self) -> Ty {
        let id = self.next_ty_var;
        self.next_ty_var += 1;
        Ty::TyVar(id)
    }

    /// Get the next type variable ID without consuming it
    pub fn peek_next_ty_var(&self) -> TyVarId {
        self.next_ty_var
    }

    /// Enter a new scope
    pub fn enter_scope(&self) -> Env {
        Env::with_parent(self.clone())
    }

    /// Exit a scope and get the parent environment
    pub fn exit_scope(self) -> Option<Env> {
        self.parent.map(|p| *p)
    }

    /// Initialize the environment with built-in types
    pub fn with_builtins() -> Self {
        let mut env = Self::new();

        // Primitive types
        env.insert_type_def("bool".to_string(), Ty::Bool);
        env.insert_type_def("i8".to_string(), Ty::I8);
        env.insert_type_def("i16".to_string(), Ty::I16);
        env.insert_type_def("i32".to_string(), Ty::I32);
        env.insert_type_def("i64".to_string(), Ty::I64);
        env.insert_type_def("i128".to_string(), Ty::I128);
        env.insert_type_def("isize".to_string(), Ty::ISize);
        env.insert_type_def("u8".to_string(), Ty::U8);
        env.insert_type_def("u16".to_string(), Ty::U16);
        env.insert_type_def("u32".to_string(), Ty::U32);
        env.insert_type_def("u64".to_string(), Ty::U64);
        env.insert_type_def("u128".to_string(), Ty::U128);
        env.insert_type_def("usize".to_string(), Ty::USize);
        env.insert_type_def("f32".to_string(), Ty::F32);
        env.insert_type_def("f64".to_string(), Ty::F64);
        env.insert_type_def("char".to_string(), Ty::Char);
        env.insert_type_def("str".to_string(), Ty::String);
        env.insert_type_def("()".to_string(), Ty::Unit);

        // Built-in functions
        // TODO: Add built-in functions like println, etc.

        env
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_lookup() {
        let mut env = Env::new();
        env.insert_binding("x".to_string(), Ty::I32);

        assert_eq!(env.lookup_binding("x"), Some(Ty::I32));
        assert_eq!(env.lookup_binding("y"), None);
    }

    #[test]
    fn test_env_scoping() {
        let mut parent = Env::new();
        parent.insert_binding("x".to_string(), Ty::I32);

        let mut child = parent.enter_scope();
        child.insert_binding("y".to_string(), Ty::Bool);

        // Child should see both bindings
        assert_eq!(child.lookup_binding("x"), Some(Ty::I32));
        assert_eq!(child.lookup_binding("y"), Some(Ty::Bool));

        // Parent should only see its own binding
        assert_eq!(parent.lookup_binding("x"), Some(Ty::I32));
        assert_eq!(parent.lookup_binding("y"), None);
    }

    #[test]
    fn test_fresh_ty_var() {
        let mut env = Env::new();

        let tv1 = env.fresh_ty_var();
        let tv2 = env.fresh_ty_var();

        assert!(matches!(tv1, Ty::TyVar(0)));
        assert!(matches!(tv2, Ty::TyVar(1)));
    }

    #[test]
    fn test_builtins() {
        let env = Env::with_builtins();

        assert_eq!(env.lookup_type_def("i32"), Some(Ty::I32));
        assert_eq!(env.lookup_type_def("bool"), Some(Ty::Bool));
        assert_eq!(env.lookup_type_def("unknown"), None);
    }
}
