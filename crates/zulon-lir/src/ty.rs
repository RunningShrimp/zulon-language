// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LIR types
//!
//! LIR types are simplified from MIR types, focusing on:
//! - Machine-level types
//! - Size and alignment
//! - Register allocation readiness

use std::fmt;

/// LIR type (low-level, machine-oriented)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LirTy {
    // Primitives (fixed size)
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
    Bool,

    // Special
    Unit,
    Never,

    // Pointers
    Ptr(Box<LirTy>),

    // Arrays (fixed size only)
    Array {
        inner: Box<LirTy>,
        len: u64,
    },

    // Structs (opaque, just name and size)
    Struct {
        name: String,
        size: u64,
    },
}

impl LirTy {
    /// Get size in bytes
    pub fn size(&self) -> u64 {
        match self {
            LirTy::I8 | LirTy::U8 | LirTy::Bool => 1,
            LirTy::I16 | LirTy::U16 => 2,
            LirTy::I32 | LirTy::U32 | LirTy::F32 => 4,
            LirTy::I64 | LirTy::U64 | LirTy::F64 | LirTy::Ptr(_) => 8,
            LirTy::I128 | LirTy::U128 => 16,
            LirTy::ISize | LirTy::USize => 8, // Assume 64-bit
            LirTy::Unit => 0,
            LirTy::Never => 0,
            LirTy::Array { inner, len } => inner.size() * len,
            LirTy::Struct { size, .. } => *size,
        }
    }

    /// Get alignment in bytes
    pub fn align(&self) -> u64 {
        match self {
            LirTy::I8 | LirTy::U8 | LirTy::Bool => 1,
            LirTy::I16 | LirTy::U16 => 2,
            LirTy::I32 | LirTy::U32 | LirTy::F32 => 4,
            LirTy::I64 | LirTy::U64 | LirTy::F64 | LirTy::Ptr(_) => 8,
            LirTy::I128 | LirTy::U128 => 16,
            LirTy::ISize | LirTy::USize => 8,
            LirTy::Unit => 1,
            LirTy::Never => 1,
            LirTy::Array { inner, .. } => inner.align(),
            LirTy::Struct { .. } => 8, // Default struct alignment
        }
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        match self {
            LirTy::Bool => "bool".to_string(),
            LirTy::I32 => "i32".to_string(),
            LirTy::I64 => "i64".to_string(),
            LirTy::ISize => "isize".to_string(),
            LirTy::U32 => "u32".to_string(),
            LirTy::U64 => "u64".to_string(),
            LirTy::USize => "usize".to_string(),
            LirTy::F32 => "f32".to_string(),
            LirTy::F64 => "f64".to_string(),
            LirTy::Unit => "()".to_string(),
            LirTy::Never => "!".to_string(),
            LirTy::Ptr(inner) => format!("*{}", inner.display_name()),
            LirTy::Array { inner, len } => {
                format!("[{}; {}]", inner.display_name(), len)
            }
            LirTy::Struct { name, .. } => name.clone(),
            _ => format!("{:?}", self),
        }
    }

    /// Check if this is a floating-point type
    pub fn is_float(&self) -> bool {
        matches!(self, LirTy::F32 | LirTy::F64)
    }
}

impl fmt::Display for LirTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Convert from MIR type to LIR type
impl From<zulon_mir::MirTy> for LirTy {
    fn from(mir_ty: zulon_mir::MirTy) -> Self {
        match mir_ty {
            // Primitives
            zulon_mir::MirTy::Bool => LirTy::Bool,
            zulon_mir::MirTy::I8 => LirTy::I8,
            zulon_mir::MirTy::I16 => LirTy::I16,
            zulon_mir::MirTy::I32 => LirTy::I32,
            zulon_mir::MirTy::I64 => LirTy::I64,
            zulon_mir::MirTy::I128 => LirTy::I128,
            zulon_mir::MirTy::ISize => LirTy::ISize,
            zulon_mir::MirTy::U8 => LirTy::U8,
            zulon_mir::MirTy::U16 => LirTy::U16,
            zulon_mir::MirTy::U32 => LirTy::U32,
            zulon_mir::MirTy::U64 => LirTy::U64,
            zulon_mir::MirTy::U128 => LirTy::U128,
            zulon_mir::MirTy::USize => LirTy::USize,
            zulon_mir::MirTy::F32 => LirTy::F32,
            zulon_mir::MirTy::F64 => LirTy::F64,
            zulon_mir::MirTy::Char => LirTy::U32, // Char is 4 bytes
            zulon_mir::MirTy::String => LirTy::Ptr(Box::new(LirTy::U8)), // String is *u8

            // Special
            zulon_mir::MirTy::Unit => LirTy::Unit,
            zulon_mir::MirTy::Never => LirTy::Never,

            // Pointers
            zulon_mir::MirTy::Ref { inner, .. } => {
                LirTy::Ptr(Box::new((*inner).into()))
            }
            zulon_mir::MirTy::Ptr { inner, .. } => {
                LirTy::Ptr(Box::new((*inner).into()))
            }

            // Array
            zulon_mir::MirTy::Array { inner, len } => {
                LirTy::Array {
                    inner: Box::new((*inner).into()),
                    len,
                }
            }

            // Structs (simplified - placeholder size)
            zulon_mir::MirTy::Struct { name, .. } => {
                LirTy::Struct {
                    name,
                    size: 8, // Placeholder
                }
            }

            // Simplified handling for other types
            zulon_mir::MirTy::Slice(_) => {
                LirTy::Ptr(Box::new(LirTy::U8))
            }
            zulon_mir::MirTy::Tuple(_) => {
                LirTy::Struct {
                    name: "Tuple".to_string(),
                    size: 8, // Placeholder
                }
            }
            zulon_mir::MirTy::Function { .. } => {
                LirTy::Ptr(Box::new(LirTy::Unit))
            }
            zulon_mir::MirTy::Enum { name, .. } => {
                LirTy::Struct {
                    name,
                    size: 8, // Placeholder
                }
            }
            zulon_mir::MirTy::Optional(_) => {
                LirTy::Struct {
                    name: "Option".to_string(),
                    size: 16, // Size + discriminant
                }
            }
        }
    }
}
