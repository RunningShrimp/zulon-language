// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type inference
//!
//! This module implements type inference using a unification-based algorithm.

use crate::error::{Result, TypeError};
use crate::ty::{Ty, TyVarId, Substs, subst_ty};
use std::collections::HashMap;
use zulon_parser::ast::Span;

/// Type substitution store for type inference
///
/// Maps type variables to their concrete types (or other type variables)
#[derive(Debug, Clone)]
pub struct Substitution {
    /// Mapping from type variable IDs to types
    substs: Substs,
}

impl Substitution {
    /// Create a new empty substitution
    pub fn new() -> Self {
        Substitution {
            substs: HashMap::new(),
        }
    }

    /// Add a binding: ty_var -> ty
    pub fn bind(&mut self, ty_var: TyVarId, ty: &Ty) {
        self.substs.insert(ty_var, ty.clone());
    }

    /// Look up a type variable
    pub fn lookup(&self, ty_var: TyVarId) -> Option<&Ty> {
        self.substs.get(&ty_var)
    }

    /// Apply substitution to a type
    pub fn apply(&self, ty: &Ty) -> Ty {
        subst_ty(&self.substs, ty)
    }

    /// Compose two substitutions: (self ∘ other)
    ///
    /// Returns a new substitution that applies other first, then self
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = self.clone();

        // Apply self to all bindings in other
        for (ty_var, ty) in &other.substs {
            let ty_applied = self.apply(ty);
            result.substs.insert(*ty_var, ty_applied);
        }

        result
    }

    /// Get the underlying substitutions map
    pub fn into_inner(self) -> Substs {
        self.substs
    }
}

impl Default for Substitution {
    fn default() -> Self {
        Self::new()
    }
}

/// Unify two types
///
/// This is the core of the type inference algorithm. It finds a substitution
/// that makes the two types equal, or reports an error if they cannot be unified.
///
/// # Arguments
/// * `ty1` - First type
/// * `ty2` - Second type
/// * `span` - Location for error reporting
///
/// # Returns
/// A substitution that makes ty1 and ty2 equal
pub fn unify(ty1: &Ty, ty2: &Ty, span: &Span) -> Result<Substitution> {
    let mut subst = Substitution::new();
    unify_with_subst(ty1, ty2, span, &mut subst)?;
    Ok(subst)
}

/// Internal unification with existing substitution
pub fn unify_with_subst(
    ty1: &Ty,
    ty2: &Ty,
    span: &Span,
    subst: &mut Substitution,
) -> Result<()> {
    // Apply current substitution to both types
    let ty1 = subst.apply(ty1);
    let ty2 = subst.apply(ty2);

    match (ty1, ty2) {
        // Type variable - bind it
        (Ty::TyVar(id), ty) | (ty, Ty::TyVar(id)) => {
            bind_type_var(id, &ty, span, subst)?;
        }

        // Primitive types - must be exactly equal
        (Ty::Bool, Ty::Bool) |
        (Ty::I8, Ty::I8) |
        (Ty::I16, Ty::I16) |
        (Ty::I32, Ty::I32) |
        (Ty::I64, Ty::I64) |
        (Ty::I128, Ty::I128) |
        (Ty::ISize, Ty::ISize) |
        (Ty::U8, Ty::U8) |
        (Ty::U16, Ty::U16) |
        (Ty::U32, Ty::U32) |
        (Ty::U64, Ty::U64) |
        (Ty::U128, Ty::U128) |
        (Ty::USize, Ty::USize) |
        (Ty::F32, Ty::F32) |
        (Ty::F64, Ty::F64) |
        (Ty::Char, Ty::Char) |
        (Ty::String, Ty::String) |
        (Ty::Unit, Ty::Unit) |
        (Ty::Never, Ty::Never) => {
            // Equal, nothing to do
        }

        // Never type (diverging) unifies with any type
        // This allows expressions like `throw` or `return` to work in any context
        (Ty::Never, _) | (_, Ty::Never) => {
            // Never is compatible with any type - nothing to do
        }

        // References
        (Ty::Ref { inner: inner1, mutable: mut1 }, Ty::Ref { inner: inner2, mutable: mut2 }) => {
            if mut1 != mut2 {
                return Err(TypeError::TypeMismatch {
                    expected: Ty::Ref { inner: inner1.clone(), mutable: mut1 },
                    found: Ty::Ref { inner: inner2.clone(), mutable: mut2 },
                    span: span.clone(),
                });
            }
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }

        // Pointers
        (Ty::Ptr { inner: inner1, mutable: mut1 }, Ty::Ptr { inner: inner2, mutable: mut2 }) => {
            if mut1 != mut2 {
                return Err(TypeError::TypeMismatch {
                    expected: Ty::Ptr { inner: inner1.clone(), mutable: mut1 },
                    found: Ty::Ptr { inner: inner2.clone(), mutable: mut2 },
                    span: span.clone(),
                });
            }
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }

        // Arrays
        (Ty::Array { inner: inner1, len: len1 }, Ty::Array { inner: inner2, len: len2 }) => {
            if len1 != len2 {
                return Err(TypeError::TypeMismatch {
                    expected: Ty::Array { inner: inner1.clone(), len: len1 },
                    found: Ty::Array { inner: inner2.clone(), len: len2 },
                    span: span.clone(),
                });
            }
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }

        // Slices
        (Ty::Slice(inner1), Ty::Slice(inner2)) => {
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }

        // Tuples
        (Ty::Tuple(tys1), Ty::Tuple(tys2)) => {
            if tys1.len() != tys2.len() {
                return Err(TypeError::TypeMismatch {
                    expected: Ty::Tuple(tys1.clone()),
                    found: Ty::Tuple(tys2.clone()),
                    span: span.clone(),
                });
            }
            for (ty1, ty2) in tys1.iter().zip(tys2.iter()) {
                unify_with_subst(ty1, ty2, span, subst)?;
            }
        }

        // Functions
        (Ty::Function { params: params1, return_type: ret1 }, Ty::Function { params: params2, return_type: ret2 }) => {
            if params1.len() != params2.len() {
                return Err(TypeError::ArityMismatch {
                    expected: params1.len(),
                    found: params2.len(),
                    span: span.clone(),
                });
            }
            for (param1, param2) in params1.iter().zip(params2.iter()) {
                unify_with_subst(param1, param2, span, subst)?;
            }
            unify_with_subst(ret1.as_ref(), ret2.as_ref(), span, subst)?;
        }

        // Structs - nominal equality
        (Ty::Struct { name: name1, generics: gens1 }, Ty::Struct { name: name2, generics: gens2 }) => {
            if name1.name != name2.name {
                return Err(TypeError::TypeMismatch {
                    expected: Ty::Struct { name: name1.clone(), generics: gens1.clone() },
                    found: Ty::Struct { name: name2.clone(), generics: gens2.clone() },
                    span: span.clone(),
                });
            }
            if gens1.len() != gens2.len() {
                return Err(TypeError::ArityMismatch {
                    expected: gens1.len(),
                    found: gens2.len(),
                    span: span.clone(),
                });
            }
            for (gen1, gen2) in gens1.iter().zip(gens2.iter()) {
                unify_with_subst(gen1, gen2, span, subst)?;
            }
        }

        // Enums - nominal equality
        (Ty::Enum { name: name1, generics: gens1 }, Ty::Enum { name: name2, generics: gens2 }) => {
            if name1.name != name2.name {
                return Err(TypeError::TypeMismatch {
                    expected: Ty::Enum { name: name1.clone(), generics: gens1.clone() },
                    found: Ty::Enum { name: name2.clone(), generics: gens2.clone() },
                    span: span.clone(),
                });
            }
            if gens1.len() != gens2.len() {
                return Err(TypeError::ArityMismatch {
                    expected: gens1.len(),
                    found: gens2.len(),
                    span: span.clone(),
                });
            }
            for (gen1, gen2) in gens1.iter().zip(gens2.iter()) {
                unify_with_subst(gen1, gen2, span, subst)?;
            }
        }

        // Optional types
        (Ty::Optional(inner1), Ty::Optional(inner2)) => {
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }

        // Trait objects and impl trait - structural equality for now
        (Ty::TraitObject(inner1), Ty::TraitObject(inner2)) => {
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }
        (Ty::ImplTrait(inner1), Ty::ImplTrait(inner2)) => {
            unify_with_subst(inner1.as_ref(), inner2.as_ref(), span, subst)?;
        }

        // Type mismatch
        (ty1, ty2) => {
            return Err(TypeError::TypeMismatch {
                expected: ty1,
                found: ty2,
                span: span.clone(),
            });
        }
    }

    Ok(())
}

/// Bind a type variable to a type
///
/// Implements the occurs check to prevent infinite types
fn bind_type_var(ty_var: TyVarId, ty: &Ty, span: &Span, subst: &mut Substitution) -> Result<()> {
    // If the type is a type variable, check if it's already bound
    if let Ty::TyVar(other_var) = ty {
        // If they're the same, nothing to do
        if ty_var == *other_var {
            return Ok(());
        }

        // If other_var is bound, use its binding
        let binding = subst.lookup(*other_var).cloned();
        if let Some(binding) = binding {
            return bind_type_var(ty_var, &binding, span, subst);
        }
    }

    // Occurs check: make sure ty_var doesn't occur in ty
    if occurs_in(ty_var, ty) {
        return Err(TypeError::InferenceError {
            message: format!("infinite type: type variable ?{} occurs in {}", ty_var, ty),
            span: span.clone(),
        });
    }

    // Bind the type variable
    subst.bind(ty_var, ty);
    Ok(())
}

/// Check if a type variable occurs in a type (occurs check)
fn occurs_in(ty_var: TyVarId, ty: &Ty) -> bool {
    match ty {
        Ty::TyVar(id) => *id == ty_var,

        Ty::Ref { inner, .. } | Ty::Ptr { inner, .. } => occurs_in(ty_var, inner),
        Ty::Array { inner, .. } => occurs_in(ty_var, inner),
        Ty::Slice(inner) => occurs_in(ty_var, inner),
        Ty::Tuple(tys) => tys.iter().any(|t| occurs_in(ty_var, t)),
        Ty::Function { params, return_type } => {
            params.iter().any(|t| occurs_in(ty_var, t))
                || occurs_in(ty_var, return_type)
        }
        Ty::Struct { generics, .. } | Ty::Enum { generics, .. } => {
            generics.iter().any(|t| occurs_in(ty_var, t))
        }
        Ty::TraitObject(inner) | Ty::ImplTrait(inner) | Ty::Optional(inner) => {
            occurs_in(ty_var, inner)
        }

        // Primitive types don't contain type variables
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_parser::Position;

    #[test]
    fn test_unify_primitives() {
        let span = Span {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 2 },
        };

        // Same types should unify
        assert!(unify(&Ty::I32, &Ty::I32, &span).is_ok());
        assert!(unify(&Ty::Bool, &Ty::Bool, &span).is_ok());

        // Different types should not unify
        assert!(unify(&Ty::I32, &Ty::Bool, &span).is_err());
        assert!(unify(&Ty::I32, &Ty::I64, &span).is_err());
    }

    #[test]
    fn test_unify_type_var() {
        let span = Span {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 2 },
        };

        // Type variable should unify with concrete type
        let subst = unify(&Ty::TyVar(0), &Ty::I32, &span).unwrap();
        assert_eq!(subst.lookup(0), Some(&Ty::I32));

        // Type variable should unify with another type variable
        let subst = unify(&Ty::TyVar(0), &Ty::TyVar(1), &span).unwrap();
        // Either ?0 -> ?1 or ?1 -> ?0 is valid
        assert!(subst.lookup(0).is_some() || subst.lookup(1).is_some());
    }

    #[test]
    fn test_unify_refs() {
        let span = Span {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 2 },
        };

        // Same ref types should unify
        assert!(unify(
            &Ty::Ref { inner: Box::new(Ty::I32), mutable: false },
            &Ty::Ref { inner: Box::new(Ty::I32), mutable: false },
            &span
        ).is_ok());

        // Different mutability should not unify
        assert!(unify(
            &Ty::Ref { inner: Box::new(Ty::I32), mutable: false },
            &Ty::Ref { inner: Box::new(Ty::I32), mutable: true },
            &span
        ).is_err());

        // Type vars in refs should unify
        let subst = unify(
            &Ty::Ref { inner: Box::new(Ty::TyVar(0)), mutable: false },
            &Ty::Ref { inner: Box::new(Ty::I32), mutable: false },
            &span
        ).unwrap();
        assert_eq!(subst.lookup(0), Some(&Ty::I32));
    }

    #[test]
    fn test_occurs_check() {
        let span = Span {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 2 },
        };

        // ?0 = Vec<?0> should fail occurs check
        let result = unify(
            &Ty::TyVar(0),
            &Ty::Optional(Box::new(Ty::TyVar(0))),
            &span
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_substitution_compose() {
        let mut s1 = Substitution::new();
        s1.bind(0, &Ty::I32);

        let mut s2 = Substitution::new();
        s2.bind(1, &Ty::Bool);

        // Compose: s1 ∘ s2
        let composed = s1.compose(&s2);
        assert_eq!(composed.lookup(0), Some(&Ty::I32));
        assert_eq!(composed.lookup(1), Some(&Ty::Bool));
    }

    #[test]
    fn test_apply_substitution() {
        let mut subst = Substitution::new();
        subst.bind(0, &Ty::I32);
        subst.bind(1, &Ty::Bool);

        // Apply to type variable
        assert_eq!(subst.apply(&Ty::TyVar(0)), Ty::I32);
        assert_eq!(subst.apply(&Ty::TyVar(1)), Ty::Bool);

        // Apply to complex type
        let ty = Ty::Tuple(vec![Ty::TyVar(0), Ty::TyVar(1)]);
        let expected = Ty::Tuple(vec![Ty::I32, Ty::Bool]);
        assert_eq!(subst.apply(&ty), expected);
    }
}
