// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! MIR types
//!
//! MIR types are simplified versions of HIR types, focused on:
//! - Memory layout
//! - Borrow checking
//! - Copy vs move semantics

use std::fmt;

/// MIR type (simplified for borrow checking)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MirTy {
    // Primitives
    Bool,
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
    Char,
    String,

    // Special
    Unit,
    Never,

    // Pointers (for borrow checking)
    Ref {
        inner: Box<MirTy>,
        mutable: bool,
    },
    Ptr {
        inner: Box<MirTy>,
        mutable: bool,
    },

    // Array and slice
    Array {
        inner: Box<MirTy>,
        len: u64,
    },
    Slice(Box<MirTy>),

    // Tuple
    Tuple(Vec<MirTy>),

    // Function (simplified)
    Function {
        params: Vec<MirTy>,
        return_type: Box<MirTy>,
    },

    // ADTs (with generics for Outcome<T, E> etc.)
    Struct {
        name: String,
        generics: Vec<MirTy>,
    },
    Enum {
        name: String,
        generics: Vec<MirTy>,
    },

    // Optional
    Optional(Box<MirTy>),
}

impl MirTy {
    /// Check if type is copy (can be duplicated without move)
    pub fn is_copy(&self) -> bool {
        match self {
            // Primitives are copy
            MirTy::Bool | MirTy::Char => true,
            MirTy::I8 | MirTy::I16 | MirTy::I32 | MirTy::I64 | MirTy::I128 | MirTy::ISize => true,
            MirTy::U8 | MirTy::U16 | MirTy::U32 | MirTy::U64 | MirTy::U128 | MirTy::USize => true,
            MirTy::F32 | MirTy::F64 => true,

            // References are copy (they copy the reference)
            MirTy::Ref { .. } | MirTy::Ptr { .. } => true,

            // Unit and Never
            MirTy::Unit | MirTy::Never => true,

            // Other types need analysis
            MirTy::Array { inner, .. } => inner.is_copy(),
            MirTy::Tuple(tys) => tys.iter().all(|t| t.is_copy()),
            MirTy::Optional(inner) => inner.is_copy(),

            // Structs and functions are not copy by default
            MirTy::Struct { .. } | MirTy::Enum { .. } | MirTy::Function { .. } => false,

            // Slices are not copy (dynamically sized)
            MirTy::Slice(_) => false,

            // String is not copy (owned data)
            MirTy::String => false,
        }
    }

    /// Check if type needs drop (has custom destructor)
    pub fn needs_drop(&self) -> bool {
        match self {
            // Primitives don't need drop
            MirTy::Bool | MirTy::Char | MirTy::Unit | MirTy::Never => false,
            MirTy::I8 | MirTy::I16 | MirTy::I32 | MirTy::I64 | MirTy::I128 | MirTy::ISize => false,
            MirTy::U8 | MirTy::U16 | MirTy::U32 | MirTy::U64 | MirTy::U128 | MirTy::USize => false,
            MirTy::F32 | MirTy::F64 => false,

            // References don't need drop
            MirTy::Ref { .. } | MirTy::Ptr { .. } => false,

            // These need drop
            MirTy::String => true,
            MirTy::Array { inner, .. } => inner.needs_drop(),
            MirTy::Tuple(tys) => tys.iter().any(|t| t.needs_drop()),
            MirTy::Optional(inner) => inner.needs_drop(),
            MirTy::Slice(_) => true,  // Fat pointer needs drop
            MirTy::Struct { .. } | MirTy::Enum { .. } => true, // Assume needs drop
            MirTy::Function { .. } => false, // Function pointers don't need drop
        }
    }

    /// Get size in bytes (simplified)
    pub fn size(&self) -> usize {
        match self {
            MirTy::Bool => 1,
            MirTy::I8 | MirTy::U8 => 1,
            MirTy::I16 | MirTy::U16 => 2,
            MirTy::I32 | MirTy::U32 | MirTy::F32 => 4,
            MirTy::I64 | MirTy::U64 | MirTy::F64 => 8,
            MirTy::I128 | MirTy::U128 => 16,
            MirTy::ISize | MirTy::USize => 8,  // Assume 64-bit
            MirTy::Char => 4,
            MirTy::String => 24,  // Boxed str + metadata
            MirTy::Unit => 0,
            MirTy::Never => 0,
            MirTy::Ref { .. } | MirTy::Ptr { .. } => 8,
            MirTy::Array { inner, len } => inner.size() * (*len as usize),
            MirTy::Slice(_) => 16,  // Fat pointer
            MirTy::Tuple(tys) => tys.iter().map(|t| t.size()).sum(),
            MirTy::Function { .. } => 8,  // Function pointer
            MirTy::Struct { .. } => 8,  // Placeholder
            MirTy::Enum { .. } => 8,  // Placeholder
            MirTy::Optional(inner) => inner.size() + 1,  // Size + discriminant
        }
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        match self {
            MirTy::Bool => "bool".to_string(),
            MirTy::I32 => "i32".to_string(),
            MirTy::I64 => "i64".to_string(),
            MirTy::ISize => "isize".to_string(),
            MirTy::U32 => "u32".to_string(),
            MirTy::U64 => "u64".to_string(),
            MirTy::USize => "usize".to_string(),
            MirTy::F32 => "f32".to_string(),
            MirTy::F64 => "f64".to_string(),
            MirTy::Char => "char".to_string(),
            MirTy::String => "String".to_string(),
            MirTy::Unit => "()".to_string(),
            MirTy::Never => "!".to_string(),
            MirTy::Ref { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", inner.display_name())
                } else {
                    format!("&{}", inner.display_name())
                }
            }
            MirTy::Array { inner, len } => {
                format!("[{}; {}]", inner.display_name(), len)
            }
            MirTy::Tuple(tys) => {
                let inner = tys.iter()
                    .map(|t| t.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }
            MirTy::Function { params, return_type } => {
                let params = params.iter()
                    .map(|p| p.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", params, return_type.display_name())
            }
            MirTy::Struct { name, generics } => {
                if generics.is_empty() {
                    name.clone()
                } else {
                    let gen_args = generics.iter()
                        .map(|g| g.display_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", name, gen_args)
                }
            }
            MirTy::Enum { name, generics } => {
                if generics.is_empty() {
                    name.clone()
                } else {
                    let gen_args = generics.iter()
                        .map(|g| g.display_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", name, gen_args)
                }
            }
            MirTy::Optional(inner) => format!("Option<{}>", inner.display_name()),
            _ => format!("{:?}", self),
        }
    }
}

impl fmt::Display for MirTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Convert from HIR type to MIR type
impl From<zulon_hir::HirTy> for MirTy {
    fn from(hir_ty: zulon_hir::HirTy) -> Self {
        match hir_ty {
            zulon_hir::HirTy::Bool => MirTy::Bool,
            zulon_hir::HirTy::I8 => MirTy::I8,
            zulon_hir::HirTy::I16 => MirTy::I16,
            zulon_hir::HirTy::I32 => MirTy::I32,
            zulon_hir::HirTy::I64 => MirTy::I64,
            zulon_hir::HirTy::I128 => MirTy::I128,
            zulon_hir::HirTy::ISize => MirTy::ISize,
            zulon_hir::HirTy::U8 => MirTy::U8,
            zulon_hir::HirTy::U16 => MirTy::U16,
            zulon_hir::HirTy::U32 => MirTy::U32,
            zulon_hir::HirTy::U64 => MirTy::U64,
            zulon_hir::HirTy::U128 => MirTy::U128,
            zulon_hir::HirTy::USize => MirTy::USize,
            zulon_hir::HirTy::F32 => MirTy::F32,
            zulon_hir::HirTy::F64 => MirTy::F64,
            zulon_hir::HirTy::Char => MirTy::Char,
            zulon_hir::HirTy::String => MirTy::String,
            zulon_hir::HirTy::Unit => MirTy::Unit,
            zulon_hir::HirTy::Never => MirTy::Never,

            zulon_hir::HirTy::Ref { inner, mutable } => {
                MirTy::Ref {
                    inner: Box::new((*inner).into()),
                    mutable,
                }
            }

            zulon_hir::HirTy::Ptr { inner, mutable } => {
                MirTy::Ptr {
                    inner: Box::new((*inner).into()),
                    mutable,
                }
            }

            zulon_hir::HirTy::Array { inner, len } => {
                MirTy::Array {
                    inner: Box::new((*inner).into()),
                    len: len.unwrap_or(0),
                }
            }

            zulon_hir::HirTy::Slice(inner) => {
                MirTy::Slice(Box::new((*inner).into()))
            }

            zulon_hir::HirTy::Tuple(tys) => {
                MirTy::Tuple(tys.into_iter().map(|ty| ty.into()).collect())
            }

            zulon_hir::HirTy::Function { params, return_type } => {
                MirTy::Function {
                    params: params.into_iter().map(|ty| ty.into()).collect(),
                    return_type: Box::new((*return_type).into()),
                }
            }

            zulon_hir::HirTy::Struct { name, generics } => {
                MirTy::Struct {
                    name: name.clone(),
                    generics: generics.into_iter().map(|ty| ty.into()).collect(),
                }
            }

            zulon_hir::HirTy::Enum { name, generics } => {
                MirTy::Enum {
                    name: name.clone(),
                    generics: generics.into_iter().map(|ty| ty.into()).collect(),
                }
            }

            zulon_hir::HirTy::Optional(inner) => {
                MirTy::Optional(Box::new((*inner).into()))
            }

            zulon_hir::HirTy::TraitObject(_) | zulon_hir::HirTy::ImplTrait(_) => {
                // Simplified: treat as opaque
                MirTy::Struct { name: "TraitObject".to_string(), generics: Vec::new() }
            }
        }
    }
}
