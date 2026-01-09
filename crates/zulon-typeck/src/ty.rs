// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type definitions for ZULON type system
//!
//! This module defines the core types used in type checking and inference.

use std::fmt;
use std::collections::HashMap;
use zulon_parser::ast;

/// A unique identifier for a type variable
pub type TyVarId = usize;

/// A unique identifier for a generic parameter
pub type GenericParamId = usize;

/// Types in ZULON
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// Boolean type
    Bool,

    /// Integer types
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,  // Platform-dependent integer

    /// Unsigned integer types
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,  // Platform-dependent unsigned integer

    /// Floating point types
    F32,
    F64,

    /// Character type
    Char,

    /// String type
    String,

    /// Unit type (empty tuple)
    Unit,

    /// Never type (for functions that never return)
    Never,

    /// Type variable (for type inference)
    TyVar(TyVarId),

    /// Reference type
    Ref {
        inner: Box<Ty>,
        mutable: bool,
    },

    /// Pointer type (raw pointers)
    Ptr {
        inner: Box<Ty>,
        mutable: bool,
    },

    /// Array type [T; N]
    Array {
        inner: Box<Ty>,
        len: Option<u64>,
    },

    /// Slice type [T]
    Slice(Box<Ty>),

    /// Tuple type (T1, T2, ...)
    Tuple(Vec<Ty>),

    /// Function type fn(T1, T2) -> ReturnType
    Function {
        params: Vec<Ty>,
        return_type: Box<Ty>,
    },

    /// Struct type
    Struct {
        name: ast::Identifier,
        generics: Vec<Ty>,
    },

    /// Enum type
    Enum {
        name: ast::Identifier,
        generics: Vec<Ty>,
    },

    /// Trait object type
    TraitObject(Box<Ty>),

    /// Impl Trait type
    ImplTrait(Box<Ty>),

    /// Optional type T?
    Optional(Box<Ty>),

    /// Effect type (for effect system)
    Effect(String),
}

impl Ty {
    /// Check if type is inhabited (can have values)
    pub fn is_inhabited(&self) -> bool {
        !matches!(self, Ty::Never)
    }

    /// Check if type is Copy (can be copied by memcpy)
    pub fn is_copy(&self) -> bool {
        match self {
            // Primitive types are Copy
            Ty::Bool | Ty::Char => true,

            // Numeric types are Copy
            Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64 | Ty::I128 | Ty::ISize => true,
            Ty::U8 | Ty::U16 | Ty::U32 | Ty::U64 | Ty::U128 | Ty::USize => true,
            Ty::F32 | Ty::F64 => true,

            // References and pointers are Copy
            Ty::Ref { .. } | Ty::Ptr { .. } => true,

            // Tuples are Copy if all elements are Copy
            Ty::Tuple(tys) => tys.iter().all(|t| t.is_copy()),

            // Arrays are Copy if element is Copy
            Ty::Array { inner, .. } => inner.is_copy(),

            // Other types are not Copy by default
            _ => false,
        }
    }

    /// Check if type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64 | Ty::I128 | Ty::ISize |
                Ty::U8 | Ty::U16 | Ty::U32 | Ty::U64 | Ty::U128 | Ty::USize |
                Ty::F32 | Ty::F64
        )
    }

    /// Check if type is signed integer
    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64 | Ty::I128 | Ty::ISize
        )
    }

    /// Check if type is unsigned integer
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Ty::U8 | Ty::U16 | Ty::U32 | Ty::U64 | Ty::U128 | Ty::USize
        )
    }

    /// Check if type is integer (signed or unsigned)
    pub fn is_integer(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Check if type is a reference
    pub fn is_ref(&self) -> bool {
        matches!(self, Ty::Ref { .. })
    }

    /// Get inner type of reference or pointer
    pub fn inner_ty(&self) -> Option<&Ty> {
        match self {
            Ty::Ref { inner, .. } | Ty::Ptr { inner, .. } => Some(inner),
            Ty::Array { inner, .. } => Some(inner),
            Ty::Slice(inner) => Some(inner),
            Ty::Optional(inner) => Some(inner),
            _ => None,
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Bool => write!(f, "bool"),
            Ty::I8 => write!(f, "i8"),
            Ty::I16 => write!(f, "i16"),
            Ty::I32 => write!(f, "i32"),
            Ty::I64 => write!(f, "i64"),
            Ty::I128 => write!(f, "i128"),
            Ty::ISize => write!(f, "isize"),
            Ty::U8 => write!(f, "u8"),
            Ty::U16 => write!(f, "u16"),
            Ty::U32 => write!(f, "u32"),
            Ty::U64 => write!(f, "u64"),
            Ty::U128 => write!(f, "u128"),
            Ty::USize => write!(f, "usize"),
            Ty::F32 => write!(f, "f32"),
            Ty::F64 => write!(f, "f64"),
            Ty::Char => write!(f, "char"),
            Ty::String => write!(f, "str"),
            Ty::Unit => write!(f, "()"),
            Ty::Never => write!(f, "!"),
            Ty::TyVar(id) => write!(f, "?{}", id),
            Ty::Ref { inner, mutable: false } => write!(f, "&{}", inner),
            Ty::Ref { inner, mutable: true } => write!(f, "&mut {}", inner),
            Ty::Ptr { inner, mutable: false } => write!(f, "*const {}", inner),
            Ty::Ptr { inner, mutable: true } => write!(f, "*mut {}", inner),
            Ty::Array { inner, len: Some(n) } => write!(f, "[{}; {}]", inner, n),
            Ty::Array { inner, len: None } => write!(f, "[{}; _]", inner),
            Ty::Slice(inner) => write!(f, "[{}]", inner),
            Ty::Tuple(tys) => {
                write!(f, "(")?;
                for (i, ty) in tys.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Ty::Function { params, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Ty::Struct { name, generics } => {
                write!(f, "{}", name.name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, gen) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", gen)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Ty::Enum { name, generics } => {
                write!(f, "{}", name.name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, gen) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", gen)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Ty::TraitObject(inner) => write!(f, "dyn {}", inner),
            Ty::ImplTrait(inner) => write!(f, "impl {}", inner),
            Ty::Optional(inner) => write!(f, "{}?", inner),
            Ty::Effect(name) => write!(f, "{}", name),
        }
    }
}

/// Generic parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericParam {
    /// Type parameter: T
    Type {
        name: String,
        id: GenericParamId,
        bounds: Vec<Ty>,
    },
    /// Const parameter: const N: usize
    Const {
        name: String,
        id: GenericParamId,
        ty: Box<Ty>,
    },
    /// Lifetime parameter: 'a
    Lifetime {
        name: String,
        id: GenericParamId,
    },
}

/// Trait bound
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitBound {
    /// Trait bound: T: Display
    Trait(Ty),
    /// Lifetime bound: T: 'a
    Lifetime(String),
}

/// Substitution for type variables
pub type Substs = HashMap<TyVarId, Ty>;

/// Create a new substitution
pub fn empty_substs() -> Substs {
    HashMap::new()
}

/// Apply substitution to a type
pub fn subst_ty(substs: &Substs, ty: &Ty) -> Ty {
    match ty {
        // Type variable - look up in substitution
        Ty::TyVar(id) => substs.get(id).cloned().unwrap_or_else(|| Ty::TyVar(*id)),

        // Recursive types
        Ty::Ref { inner, mutable } => Ty::Ref {
            inner: Box::new(subst_ty(substs, inner)),
            mutable: *mutable,
        },
        Ty::Ptr { inner, mutable } => Ty::Ptr {
            inner: Box::new(subst_ty(substs, inner)),
            mutable: *mutable,
        },
        Ty::Array { inner, len } => Ty::Array {
            inner: Box::new(subst_ty(substs, inner)),
            len: *len,
        },
        Ty::Slice(inner) => Ty::Slice(Box::new(subst_ty(substs, inner))),
        Ty::Tuple(tys) => Ty::Tuple(tys.iter().map(|t| subst_ty(substs, t)).collect()),
        Ty::Function { params, return_type } => Ty::Function {
            params: params.iter().map(|t| subst_ty(substs, t)).collect(),
            return_type: Box::new(subst_ty(substs, return_type)),
        },
        Ty::Struct { name, generics } => Ty::Struct {
            name: name.clone(),
            generics: generics.iter().map(|t| subst_ty(substs, t)).collect(),
        },
        Ty::Enum { name, generics } => Ty::Enum {
            name: name.clone(),
            generics: generics.iter().map(|t| subst_ty(substs, t)).collect(),
        },
        Ty::TraitObject(inner) => Ty::TraitObject(Box::new(subst_ty(substs, inner))),
        Ty::ImplTrait(inner) => Ty::ImplTrait(Box::new(subst_ty(substs, inner))),
        Ty::Optional(inner) => Ty::Optional(Box::new(subst_ty(substs, inner))),

        // Non-recursive types - return as is
        Ty::Bool | Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64 | Ty::I128 | Ty::ISize |
        Ty::U8 | Ty::U16 | Ty::U32 | Ty::U64 | Ty::U128 | Ty::USize |
        Ty::F32 | Ty::F64 | Ty::Char | Ty::String | Ty::Unit | Ty::Never |
        Ty::Effect(_) => ty.clone(),
    }
}

/// Effect declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub name: String,
    pub operations: Vec<EffectOperation>,
}

/// Effect operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectOperation {
    pub name: String,
    pub param_types: Vec<Ty>,
    pub return_type: Ty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(Ty::I32.to_string(), "i32");
        assert_eq!(Ty::Bool.to_string(), "bool");
        assert_eq!(Ty::Unit.to_string(), "()");
        assert_eq!(Ty::Never.to_string(), "!");
    }

    #[test]
    fn test_is_numeric() {
        assert!(Ty::I32.is_numeric());
        assert!(Ty::F64.is_numeric());
        assert!(!Ty::Bool.is_numeric());
        assert!(!Ty::String.is_numeric());
    }

    #[test]
    fn test_is_copy() {
        assert!(Ty::I32.is_copy());
        assert!(Ty::Bool.is_copy());
        assert!(!Ty::String.is_copy());
    }

    #[test]
    fn test_subst_ty() {
        let mut substs = empty_substs();
        substs.insert(0, Ty::I32);

        let ty_var = Ty::TyVar(0);
        let result = subst_ty(&substs, &ty_var);
        assert_eq!(result, Ty::I32);
    }
}
