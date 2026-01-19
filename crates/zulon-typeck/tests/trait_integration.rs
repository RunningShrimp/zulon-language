// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the trait system
//!
//! These tests demonstrate that trait system components work together
//! in realistic scenarios.

use zulon_typeck::traits::*;
use zulon_typeck::{GenericParam, Ty};

#[test]
fn test_trait_definition_and_registration_workflow() {
    // Simulate defining a Display trait with methods
    let mut solver = TraitSolver::new();
    let display_id = solver.fresh_trait_id();

    let display_trait = TraitDef::new("Display".to_string(), display_id).with_method(
        "to_string".to_string(),
        FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::String),
    );

    solver.register_trait(display_trait);

    // Verify trait was registered
    let retrieved = solver.get_trait(display_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Display");
}

#[test]
fn test_trait_implementation_workflow() {
    let mut solver = TraitSolver::new();

    // Define a Debug trait
    let debug_id = solver.fresh_trait_id();
    let debug_trait = TraitDef::new("Debug".to_string(), debug_id).with_method(
        "debug".to_string(),
        FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::String),
    );
    solver.register_trait(debug_trait);

    // Implement Debug for i32
    let debug_ref = TraitRef::new(debug_id);
    let impl_block = TraitImpl::new(Ty::I32, debug_ref.clone()).with_method("debug".to_string(), 1);
    solver.register_impl(impl_block);

    // Verify implementation exists
    let found_impl = solver.find_impl(&Ty::I32, &debug_ref);
    assert!(found_impl.is_some());
    assert_eq!(found_impl.unwrap().methods.get("debug"), Some(&1));
}

#[test]
fn test_vtable_generation_workflow() {
    let mut solver = TraitSolver::new();

    // Define Clone trait
    let clone_id = solver.fresh_trait_id();
    let clone_trait = TraitDef::new("Clone".to_string(), clone_id).with_method(
        "clone".to_string(),
        FunctionSig::new(
            Some(SelfTy::RefSelf),
            vec![],
            Ty::TyVar(0), // Returns Self type
        ),
    );
    solver.register_trait(clone_trait);

    // Implement Clone for String
    let clone_ref = TraitRef::new(clone_id);
    let impl_block =
        TraitImpl::new(Ty::String, clone_ref.clone()).with_method("clone".to_string(), 1);
    solver.register_impl(impl_block);

    // Generate vtable for String + Clone
    let vtable = solver.generate_vtable(&Ty::String, &clone_ref);
    assert!(vtable.is_some());

    let vtable = vtable.unwrap();
    assert_eq!(vtable.concrete_ty, Ty::String);
    assert_eq!(vtable.entries.len(), 1);
    assert_eq!(vtable.entries[0].name, "clone");
}

#[test]
fn test_constraint_solving_workflow() {
    let mut solver = TraitSolver::new();

    // Define a simple trait
    let trait_id = solver.fresh_trait_id();
    let simple_trait = TraitDef::new("Sized".to_string(), trait_id);
    solver.register_trait(simple_trait);

    // Add an implementation
    let trait_ref = TraitRef::new(trait_id);
    let impl_block = TraitImpl::new(Ty::I32, trait_ref.clone());
    solver.register_impl(impl_block);

    // Add a constraint that i32 implements Sized
    solver.add_constraint(TraitConstraint::Impl {
        ty: Ty::I32,
        trait_ref: trait_ref.clone(),
    });

    // Solve constraint
    let result = solver.solve();
    assert!(result.is_ok());
}

#[test]
fn test_trait_object_workflow() {
    // Create a trait object from a solver-created trait reference
    let mut solver = TraitSolver::new();
    let trait_id = solver.fresh_trait_id();

    // Create a simple trait
    let simple_trait = TraitDef::new("Simple".to_string(), trait_id);
    solver.register_trait(simple_trait);

    // Create a trait object from the trait
    let trait_ref = TraitRef::new(trait_id);
    let trait_obj = TraitObject::new(trait_ref.clone());

    assert_eq!(trait_obj.trait_ref, trait_ref);
    assert!(trait_obj.auto_traits.is_empty());
}

#[test]
fn test_vtable_with_multiple_methods() {
    let mut solver = TraitSolver::new();

    // Define a trait with multiple methods
    let trait_id = solver.fresh_trait_id();
    let multi_trait = TraitDef::new("MultiMethod".to_string(), trait_id)
        .with_method(
            "method1".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::I32),
        )
        .with_method(
            "method2".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::String),
        )
        .with_method(
            "method3".to_string(),
            FunctionSig::new(
                Some(SelfTy::MutRefSelf),
                vec![("x".to_string(), Ty::I32)],
                Ty::Unit,
            ),
        );
    solver.register_trait(multi_trait);

    // Implement the trait
    let trait_ref = TraitRef::new(trait_id);
    let impl_block = TraitImpl::new(Ty::String, trait_ref.clone())
        .with_method("method1".to_string(), 1)
        .with_method("method2".to_string(), 2)
        .with_method("method3".to_string(), 3);
    solver.register_impl(impl_block);

    // Generate vtable and verify all methods are present
    let vtable = solver.generate_vtable(&Ty::String, &trait_ref);
    assert!(vtable.is_some());

    let vtable = vtable.unwrap();
    assert_eq!(vtable.entries.len(), 3);
    assert_eq!(vtable.entries[0].name, "method1");
    assert_eq!(vtable.entries[1].name, "method2");
    assert_eq!(vtable.entries[2].name, "method3");
}

#[test]
fn test_trait_with_generics_workflow() {
    let mut solver = TraitSolver::new();

    // Define a generic trait
    let trait_id = solver.fresh_trait_id();
    let generic_trait = TraitDef::new("Comparable".to_string(), trait_id)
        .with_generic(GenericParam::Type {
            name: "T".to_string(),
            id: 0,
            bounds: vec![],
        })
        .with_method(
            "compare".to_string(),
            FunctionSig::new(
                Some(SelfTy::RefSelf),
                vec![("other".to_string(), Ty::TyVar(1))],
                Ty::I32,
            ),
        );
    solver.register_trait(generic_trait);

    // Verify trait has generics
    let retrieved = solver.get_trait(trait_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().generics.len(), 1);
}
