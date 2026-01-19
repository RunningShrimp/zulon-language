// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Trait system for ZULON
//!
//! This module implements:
//! - Trait definitions with generics, super traits, associated types
//! - Trait implementations
//! - Trait constraints and bounds
//! - VTable generation for dynamic dispatch
//! - Trait object support
//! - Trait constraint solver

use crate::error::{Result, TypeError};
use crate::ty::{GenericParam, TraitBound, Ty};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use zulon_parser::ast::Span;
use zulon_parser::Position;

/// Unique identifier for a trait definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraitId(u32);

impl TraitId {
    /// Create a new trait ID
    fn new(id: u32) -> Self {
        TraitId(id)
    }

    /// Get the raw ID value
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// Self type in trait methods
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelfTy {
    /// `Self` - the implementing type
    SelfType,
    /// `&Self` - immutable reference to Self
    RefSelf,
    /// `&mut Self` - mutable reference to Self
    MutRefSelf,
}

/// Function signature for trait methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSig {
    /// The self type (if this is a method)
    pub self_ty: Option<SelfTy>,
    /// The parameters of the function
    pub params: Vec<(String, Ty)>,
    /// The return type
    pub ret_ty: Ty,
    /// Whether the function can throw
    pub can_throw: bool,
}

impl FunctionSig {
    /// Create a new function signature
    pub fn new(self_ty: Option<SelfTy>, params: Vec<(String, Ty)>, ret_ty: Ty) -> Self {
        FunctionSig {
            self_ty,
            params,
            ret_ty,
            can_throw: false,
        }
    }

    /// Create a throwing function signature
    pub fn with_throws(mut self) -> Self {
        self.can_throw = true;
        self
    }

    /// Get the full parameter types including self
    pub fn param_types(&self) -> Vec<Ty> {
        let mut types = Vec::new();

        if let Some(ref self_ty) = self.self_ty {
            // Self is represented as a type variable with ID 0
            let self_ty_var = Ty::TyVar(0);
            types.push(match self_ty {
                SelfTy::SelfType => self_ty_var,
                SelfTy::RefSelf => Ty::Ref {
                    inner: Box::new(self_ty_var),
                    mutable: false,
                },
                SelfTy::MutRefSelf => Ty::Ref {
                    inner: Box::new(self_ty_var),
                    mutable: true,
                },
            });
        }

        for (_, ty) in &self.params {
            types.push(ty.clone());
        }

        types
    }
}

/// Trait definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    /// Unique identifier
    pub id: TraitId,
    /// Name of the trait
    pub name: String,
    /// Generic parameters
    pub generics: Vec<GenericParam>,
    /// Super traits (traits this trait inherits from)
    pub super_traits: Vec<TraitBound>,
    /// Associated types
    pub associated_types: Vec<AssociatedType>,
    /// Associated constants
    pub associated_consts: Vec<AssociatedConst>,
    /// Methods defined in the trait
    pub methods: HashMap<String, FunctionSig>,
}

impl TraitDef {
    /// Create a new trait definition
    pub fn new(name: String, id: TraitId) -> Self {
        TraitDef {
            id,
            name,
            generics: Vec::new(),
            super_traits: Vec::new(),
            associated_types: Vec::new(),
            associated_consts: Vec::new(),
            methods: HashMap::new(),
        }
    }

    /// Add a generic parameter
    pub fn with_generic(mut self, param: GenericParam) -> Self {
        self.generics.push(param);
        self
    }

    /// Add a super trait
    pub fn with_super_trait(mut self, bound: TraitBound) -> Self {
        self.super_traits.push(bound);
        self
    }

    /// Add an associated type
    pub fn with_associated_type(mut self, assoc_ty: AssociatedType) -> Self {
        self.associated_types.push(assoc_ty);
        self
    }

    /// Add an associated constant
    pub fn with_associated_const(mut self, assoc_const: AssociatedConst) -> Self {
        self.associated_consts.push(assoc_const);
        self
    }

    /// Add a method
    pub fn with_method(mut self, name: String, sig: FunctionSig) -> Self {
        self.methods.insert(name, sig);
        self
    }

    /// Get a method by name
    pub fn get_method(&self, name: &str) -> Option<&FunctionSig> {
        self.methods.get(name)
    }

    /// Check if this trait has a specific method
    pub fn has_method(&self, name: &str) -> bool {
        self.methods.contains_key(name)
    }
}

/// Associated type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedType {
    /// Name of the associated type
    pub name: String,
    /// Bounds on the associated type
    pub bounds: Vec<TraitBound>,
}

impl AssociatedType {
    /// Create a new associated type
    pub fn new(name: String) -> Self {
        AssociatedType {
            name,
            bounds: Vec::new(),
        }
    }

    /// Add a bound
    pub fn with_bound(mut self, bound: TraitBound) -> Self {
        self.bounds.push(bound);
        self
    }
}

/// Associated constant definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedConst {
    /// Name of the constant
    pub name: String,
    /// Type of the constant
    pub ty: Ty,
}

impl AssociatedConst {
    /// Create a new associated constant
    pub fn new(name: String, ty: Ty) -> Self {
        AssociatedConst { name, ty }
    }
}

/// Trait implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImpl {
    /// The type implementing the trait
    pub self_ty: Ty,
    /// The trait being implemented
    pub trait_ref: TraitRef,
    /// Generic arguments for the trait
    pub generic_args: Vec<Ty>,
    /// Associated type implementations
    pub associated_tys: HashMap<String, Ty>,
    /// Method implementations (method name -> function ID)
    pub methods: HashMap<String, u32>,
}

impl TraitImpl {
    /// Create a new trait implementation
    pub fn new(self_ty: Ty, trait_ref: TraitRef) -> Self {
        TraitImpl {
            self_ty,
            trait_ref,
            generic_args: Vec::new(),
            associated_tys: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    /// Add a generic argument
    pub fn with_generic_arg(mut self, arg: Ty) -> Self {
        self.generic_args.push(arg);
        self
    }

    /// Add an associated type implementation
    pub fn with_associated_type(mut self, name: String, ty: Ty) -> Self {
        self.associated_tys.insert(name, ty);
        self
    }

    /// Add a method implementation
    pub fn with_method(mut self, name: String, function_id: u32) -> Self {
        self.methods.insert(name, function_id);
        self
    }
}

/// Reference to a trait (used in bounds and implementations)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitRef {
    /// The trait being referenced
    pub trait_id: TraitId,
    /// Generic arguments
    pub args: Vec<Ty>,
}

impl TraitRef {
    /// Create a new trait reference
    pub fn new(trait_id: TraitId) -> Self {
        TraitRef {
            trait_id,
            args: Vec::new(),
        }
    }

    /// Add a generic argument
    pub fn with_arg(mut self, arg: Ty) -> Self {
        self.args.push(arg);
        self
    }
}

/// Trait object (dynamic dispatch)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitObject {
    /// The trait(s) being objectified
    pub trait_ref: TraitRef,
    /// Optional auto traits
    pub auto_traits: Vec<TraitRef>,
}

impl TraitObject {
    /// Create a new trait object
    pub fn new(trait_ref: TraitRef) -> Self {
        TraitObject {
            trait_ref,
            auto_traits: Vec::new(),
        }
    }

    /// Add an auto trait
    pub fn with_auto_trait(mut self, trait_ref: TraitRef) -> Self {
        self.auto_traits.push(trait_ref);
        self
    }
}

/// VTable entry for dynamic dispatch
#[derive(Debug, Clone)]
pub struct VTableEntry {
    /// Method name
    pub name: String,
    /// Function pointer (represented as function ID)
    pub function_id: u32,
    /// Self type for this method
    pub self_ty: Ty,
}

impl VTableEntry {
    /// Create a new VTable entry
    pub fn new(name: String, function_id: u32, self_ty: Ty) -> Self {
        VTableEntry {
            name,
            function_id,
            self_ty,
        }
    }
}

/// Virtual function table for dynamic dispatch
#[derive(Debug, Clone)]
pub struct VTable {
    /// The trait this vtable is for
    pub trait_ref: TraitRef,
    /// The concrete type this vtable implements
    pub concrete_ty: Ty,
    /// Method entries
    pub entries: Vec<VTableEntry>,
    /// Size of the vtable in bytes
    pub size: usize,
}

impl VTable {
    /// Create a new vtable
    pub fn new(trait_ref: TraitRef, concrete_ty: Ty) -> Self {
        VTable {
            trait_ref,
            concrete_ty,
            entries: Vec::new(),
            size: 0,
        }
    }

    /// Add a method to the vtable
    pub fn add_entry(&mut self, entry: VTableEntry) {
        self.entries.push(entry);
        self.size = self.entries.len() * std::mem::size_of::<usize>();
    }

    /// Get a method entry by name
    pub fn get_entry(&self, name: &str) -> Option<&VTableEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Generate vtable layout
    pub fn layout(&self) -> VTableLayout {
        VTableLayout {
            entries: self.entries.iter().map(|e| e.name.clone()).collect(),
            size: self.size,
        }
    }
}

/// VTable layout information
#[derive(Debug, Clone)]
pub struct VTableLayout {
    /// Method names in order
    pub entries: Vec<String>,
    /// Size in bytes
    pub size: usize,
}

/// Trait constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitConstraint {
    /// A type implements a trait
    Impl { ty: Ty, trait_ref: TraitRef },
    /// Equality constraint between two types
    Equality { left: Ty, right: Ty },
}

/// Trait solver for constraint solving
#[derive(Debug)]
pub struct TraitSolver {
    /// Next trait ID
    next_trait_id: AtomicU32,
    /// Registered traits
    traits: HashMap<TraitId, TraitDef>,
    /// Registered trait implementations
    impls: Vec<TraitImpl>,
    /// Active constraints
    constraints: Vec<TraitConstraint>,
}

impl TraitSolver {
    /// Create a new trait solver
    pub fn new() -> Self {
        TraitSolver {
            next_trait_id: AtomicU32::new(0),
            traits: HashMap::new(),
            impls: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Register a trait definition
    pub fn register_trait(&mut self, trait_def: TraitDef) {
        self.traits.insert(trait_def.id, trait_def);
    }

    /// Generate a new trait ID
    pub fn fresh_trait_id(&self) -> TraitId {
        TraitId::new(self.next_trait_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Register a trait implementation
    pub fn register_impl(&mut self, impl_block: TraitImpl) {
        self.impls.push(impl_block);
    }

    /// Add a constraint to solve
    pub fn add_constraint(&mut self, constraint: TraitConstraint) {
        self.constraints.push(constraint);
    }

    /// Get a trait by ID
    pub fn get_trait(&self, id: TraitId) -> Option<&TraitDef> {
        self.traits.get(&id)
    }

    /// Check if a type implements a trait
    pub fn implements(&self, ty: &Ty, trait_ref: &TraitRef) -> bool {
        // Find a matching implementation
        self.impls
            .iter()
            .any(|impl_block| &impl_block.self_ty == ty && &impl_block.trait_ref == trait_ref)
    }

    /// Solve all active constraints
    pub fn solve(&mut self) -> Result<()> {
        for constraint in self.constraints.clone() {
            match constraint {
                TraitConstraint::Impl { ty, trait_ref } => {
                    if !self.implements(&ty, &trait_ref) {
                        let trait_name = self
                            .traits
                            .get(&trait_ref.trait_id)
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| format!("{:?}", trait_ref.trait_id));

                        return Err(TypeError::TypeNotImplTrait {
                            ty: ty.clone(),
                            trait_name,
                            span: Span::new(Position::new(0, 0), Position::new(0, 0)),
                        });
                    }
                }
                TraitConstraint::Equality { left, right } => {
                    if left != right {
                        return Err(TypeError::TypeMismatch {
                            expected: left.clone(),
                            found: right.clone(),
                            span: Span::new(Position::new(0, 0), Position::new(0, 0)),
                        });
                    }
                }
            }
        }

        self.constraints.clear();
        Ok(())
    }

    /// Find an implementation for a type and trait
    pub fn find_impl(&self, ty: &Ty, trait_ref: &TraitRef) -> Option<&TraitImpl> {
        self.impls
            .iter()
            .find(|impl_block| &impl_block.self_ty == ty && &impl_block.trait_ref == trait_ref)
    }

    /// Generate a vtable for a type and trait
    pub fn generate_vtable(&self, ty: &Ty, trait_ref: &TraitRef) -> Option<VTable> {
        let trait_def = self.get_trait(trait_ref.trait_id)?;
        let impl_block = self.find_impl(ty, trait_ref)?;

        let mut vtable = VTable::new(trait_ref.clone(), ty.clone());

        // Add methods in the order defined in the trait
        for (method_name, _sig) in &trait_def.methods {
            if let Some(&function_id) = impl_block.methods.get(method_name) {
                let entry = VTableEntry::new(method_name.clone(), function_id, ty.clone());
                vtable.add_entry(entry);
            }
        }

        Some(vtable)
    }
}

impl Default for TraitSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_def_creation() {
        let id = TraitId::new(0);
        let trait_def = TraitDef::new("Display".to_string(), id);

        assert_eq!(trait_def.name, "Display");
        assert_eq!(trait_def.id, id);
        assert!(trait_def.methods.is_empty());
    }

    #[test]
    fn test_trait_def_with_methods() {
        let id = TraitId::new(0);
        let trait_def = TraitDef::new("Display".to_string(), id).with_method(
            "to_string".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::String),
        );

        assert!(trait_def.has_method("to_string"));
        assert_eq!(trait_def.methods.len(), 1);
    }

    #[test]
    fn test_function_sig_creation() {
        let sig = FunctionSig::new(
            Some(SelfTy::SelfType),
            vec![("x".to_string(), Ty::I32)],
            Ty::Unit,
        );

        assert!(sig.self_ty.is_some());
        assert_eq!(sig.params.len(), 1);
        assert_eq!(sig.params[0].0, "x");
    }

    #[test]
    fn test_trait_solver_registration() {
        let mut solver = TraitSolver::new();
        let id = solver.fresh_trait_id();

        let trait_def = TraitDef::new("MyTrait".to_string(), id);
        solver.register_trait(trait_def);

        assert!(solver.get_trait(id).is_some());
    }

    #[test]
    fn test_trait_impl_creation() {
        let trait_ref = TraitRef::new(TraitId::new(0));
        let impl_block =
            TraitImpl::new(Ty::I32, trait_ref.clone()).with_method("method".to_string(), 123);

        assert_eq!(impl_block.self_ty, Ty::I32);
        assert_eq!(impl_block.methods.get("method"), Some(&123));
    }

    #[test]
    fn test_vtable_generation() {
        let mut solver = TraitSolver::new();
        let id = solver.fresh_trait_id();

        // Define a trait
        let trait_def = TraitDef::new("Display".to_string(), id).with_method(
            "to_string".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::String),
        );
        solver.register_trait(trait_def);

        // Implement it
        let trait_ref = TraitRef::new(id);
        let impl_block = TraitImpl::new(Ty::I32, trait_ref).with_method("to_string".to_string(), 1);
        solver.register_impl(impl_block);

        // Generate vtable
        let vtable = solver.generate_vtable(&Ty::I32, &TraitRef::new(id));

        assert!(vtable.is_some());
        let vtable = vtable.unwrap();
        assert_eq!(vtable.entries.len(), 1);
        assert_eq!(vtable.entries[0].name, "to_string");
    }

    #[test]
    fn test_trait_object() {
        let trait_ref = TraitRef::new(TraitId::new(0));
        let trait_obj = TraitObject::new(trait_ref.clone());

        assert_eq!(trait_obj.trait_ref, trait_ref);
        assert!(trait_obj.auto_traits.is_empty());
    }

    #[test]
    fn test_associated_type() {
        let assoc_ty = AssociatedType::new("Item".to_string())
            .with_bound(TraitBound::Lifetime("a".to_string()));

        assert_eq!(assoc_ty.name, "Item");
        assert_eq!(assoc_ty.bounds.len(), 1);
    }

    #[test]
    fn test_self_ty_variants() {
        assert_ne!(SelfTy::SelfType, SelfTy::RefSelf);
        assert_ne!(SelfTy::RefSelf, SelfTy::MutRefSelf);
    }

    #[test]
    fn test_constraint_solving() {
        let mut solver = TraitSolver::new();
        let id = solver.fresh_trait_id();

        // Register trait
        let trait_def = TraitDef::new("Simple".to_string(), id);
        solver.register_trait(trait_def);

        // Register impl
        let trait_ref = TraitRef::new(id);
        let impl_block = TraitImpl::new(Ty::I32, trait_ref.clone());
        solver.register_impl(impl_block);

        // Add constraint
        solver.add_constraint(TraitConstraint::Impl {
            ty: Ty::I32,
            trait_ref,
        });

        // Solve
        let result = solver.solve();
        assert!(result.is_ok());
    }
}
