// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! HIR types
//!
//! HIR types are simplified versions of Ty from typeck,
//! with type variables fully resolved.

use std::fmt;

/// HIR type (fully resolved)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HirTy {
    // Primitives
    Bool,
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
    Char,
    String,

    // Special types
    Unit,
    Never,

    // Composites
    Ref {
        inner: Box<HirTy>,
        mutable: bool,
    },
    Ptr {
        inner: Box<HirTy>,
        mutable: bool,
    },
    Array {
        inner: Box<HirTy>,
        len: Option<u64>,
    },
    Slice(Box<HirTy>),
    Tuple(Vec<HirTy>),

    // Function types
    Function {
        params: Vec<HirTy>,
        return_type: Box<HirTy>,
    },

    // ADTs
    Struct {
        name: String,
        generics: Vec<HirTy>,
    },
    Enum {
        name: String,
        generics: Vec<HirTy>,
    },

    // Optional
    Optional(Box<HirTy>),

    // Traits (simplified for HIR)
    TraitObject(Vec<String>),
    ImplTrait(Vec<String>),
}

impl HirTy {
    /// Check if type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self,
            HirTy::I8 | HirTy::I16 | HirTy::I32 | HirTy::I64 |
            HirTy::I128 | HirTy::ISize | HirTy::U8 | HirTy::U16 |
            HirTy::U32 | HirTy::U64 | HirTy::U128 | HirTy::USize |
            HirTy::F32 | HirTy::F64
        )
    }

    /// Check if type is signed integer
    pub fn is_signed_integer(&self) -> bool {
        matches!(self,
            HirTy::I8 | HirTy::I16 | HirTy::I32 | HirTy::I64 |
            HirTy::I128 | HirTy::ISize
        )
    }

    /// Check if type is unsigned integer
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self,
            HirTy::U8 | HirTy::U16 | HirTy::U32 | HirTy::U64 |
            HirTy::U128 | HirTy::USize
        )
    }

    /// Check if type is integer
    pub fn is_integer(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Check if type is float
    pub fn is_float(&self) -> bool {
        matches!(self, HirTy::F32 | HirTy::F64)
    }

    /// Check if type is copy
    pub fn is_copy(&self) -> bool {
        match self {
            // Primitives are copy
            HirTy::Bool | HirTy::Char => true,
            HirTy::I8 | HirTy::I16 | HirTy::I32 | HirTy::I64 |
            HirTy::I128 | HirTy::ISize | HirTy::U8 | HirTy::U16 |
            HirTy::U32 | HirTy::U64 | HirTy::U128 | HirTy::USize => true,
            HirTy::F32 | HirTy::F64 => true,

            // References and pointers are copy
            HirTy::Ref { .. } | HirTy::Ptr { .. } => true,

            // Other types need analysis
            _ => false,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        match self {
            HirTy::Bool => "bool".to_string(),
            HirTy::I8 => "i8".to_string(),
            HirTy::I32 => "i32".to_string(),
            HirTy::I64 => "i64".to_string(),
            HirTy::ISize => "isize".to_string(),
            HirTy::U8 => "u8".to_string(),
            HirTy::U32 => "u32".to_string(),
            HirTy::U64 => "u64".to_string(),
            HirTy::USize => "usize".to_string(),
            HirTy::F32 => "f32".to_string(),
            HirTy::F64 => "f64".to_string(),
            HirTy::Char => "char".to_string(),
            HirTy::String => "String".to_string(),
            HirTy::Unit => "()".to_string(),
            HirTy::Never => "!".to_string(),
            HirTy::Ref { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", inner.display_name())
                } else {
                    format!("&{}", inner.display_name())
                }
            }
            HirTy::Tuple(tys) => {
                let inner = tys.iter()
                    .map(|t| t.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }
            HirTy::Function { params, return_type } => {
                let params = params.iter()
                    .map(|p| p.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", params, return_type.display_name())
            }
            HirTy::Struct { name, generics } => {
                if generics.is_empty() {
                    name.clone()
                } else {
                    let gens = generics.iter()
                        .map(|g| g.display_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", name, gens)
                }
            }
            _ => format!("{:?}", self),
        }
    }
}

impl fmt::Display for HirTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Convert from typeck Ty to HirTy
impl From<zulon_typeck::Ty> for HirTy {
    fn from(ty: zulon_typeck::Ty) -> Self {
        match ty {
            zulon_typeck::Ty::Bool => HirTy::Bool,
            zulon_typeck::Ty::I8 => HirTy::I8,
            zulon_typeck::Ty::I16 => HirTy::I16,
            zulon_typeck::Ty::I32 => HirTy::I32,
            zulon_typeck::Ty::I64 => HirTy::I64,
            zulon_typeck::Ty::I128 => HirTy::I128,
            zulon_typeck::Ty::ISize => HirTy::ISize,
            zulon_typeck::Ty::U8 => HirTy::U8,
            zulon_typeck::Ty::U16 => HirTy::U16,
            zulon_typeck::Ty::U32 => HirTy::U32,
            zulon_typeck::Ty::U64 => HirTy::U64,
            zulon_typeck::Ty::U128 => HirTy::U128,
            zulon_typeck::Ty::USize => HirTy::USize,
            zulon_typeck::Ty::F32 => HirTy::F32,
            zulon_typeck::Ty::F64 => HirTy::F64,
            zulon_typeck::Ty::Char => HirTy::Char,
            zulon_typeck::Ty::String => HirTy::String,
            zulon_typeck::Ty::Unit => HirTy::Unit,
            zulon_typeck::Ty::Never => HirTy::Never,

            zulon_typeck::Ty::Ref { inner, mutable } => {
                HirTy::Ref {
                    inner: Box::new((*inner).into()),
                    mutable,
                }
            }

            zulon_typeck::Ty::Ptr { inner, mutable } => {
                HirTy::Ptr {
                    inner: Box::new((*inner).into()),
                    mutable,
                }
            }

            zulon_typeck::Ty::Array { inner, len } => {
                HirTy::Array {
                    inner: Box::new((*inner).into()),
                    len,
                }
            }

            zulon_typeck::Ty::Slice(inner) => {
                HirTy::Slice(Box::new((*inner).into()))
            }

            zulon_typeck::Ty::Tuple(tys) => {
                HirTy::Tuple(tys.into_iter().map(|ty| ty.into()).collect())
            }

            zulon_typeck::Ty::Function { params, return_type } => {
                HirTy::Function {
                    params: params.into_iter().map(|ty| ty.into()).collect(),
                    return_type: Box::new((*return_type).into()),
                }
            }

            zulon_typeck::Ty::Struct { name, generics } => {
                HirTy::Struct {
                    name: name.name.clone(),
                    generics: generics.into_iter().map(|ty| ty.into()).collect(),
                }
            }

            zulon_typeck::Ty::Enum { name, generics } => {
                HirTy::Enum {
                    name: name.name.clone(),
                    generics: generics.into_iter().map(|ty| ty.into()).collect(),
                }
            }

            zulon_typeck::Ty::Optional(inner) => {
                HirTy::Optional(Box::new((*inner).into()))
            }

            zulon_typeck::Ty::TraitObject(inner) => {
                // For now, convert to simplified trait object
                HirTy::TraitObject(vec![format!("{:?}", *inner)])
            }

            zulon_typeck::Ty::ImplTrait(inner) => {
                HirTy::ImplTrait(vec![format!("{:?}", *inner)])
            }

            // Type variables should be resolved by now
            zulon_typeck::Ty::TyVar(id) => {
                panic!("Type variable ?{} not resolved during lowering", id)
            }

            // Effects should be handled at type checking time
            zulon_typeck::Ty::Effect(name) => {
                panic!("Effect '{}' not resolved during lowering", name)
            }
        }
    }
}
