// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LLVM type mappings
//!
//! Converts LIR types to LLVM IR types.

use zulon_lir::LirTy;

/// LLVM type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmType {
    /// Void type (for unit)
    Void,

    /// Integer type
    Integer(u32),  // bit width: 1, 8, 16, 32, 64, 128

    /// Floating point type
    Float(u32),  // bit width: 32 or 64

    /// Pointer type
    Pointer(Box<LlvmType>),

    /// Array type
    Array {
        inner: Box<LlvmType>,
        len: u64,
    },

    /// Struct type
    Struct {
        name: String,
        fields: Vec<LlvmType>,
    },

    /// Function type
    Function {
        params: Vec<LlvmType>,
        return_type: Box<LlvmType>,
        is_varargs: bool,
    },
}

impl LlvmType {
    /// Get LLVM IR type string
    pub fn to_llvm_ir(&self) -> String {
        match self {
            LlvmType::Void => "void".to_string(),

            LlvmType::Integer(bits) => format!("i{}", bits),

            LlvmType::Float(bits) => match bits {
                32 => "float".to_string(),
                64 => "double".to_string(),
                _ => format!("fp{}", bits),
            },

            LlvmType::Pointer(inner) => format!("{}*", inner.to_llvm_ir()),

            LlvmType::Array { inner, len } => {
                format!("[{} x {}]", len, inner.to_llvm_ir())
            }

            LlvmType::Struct { name, fields } => {
                let field_str = fields.iter()
                    .map(|t| t.to_llvm_ir())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("%struct.{} {{ {} }}", name, field_str)
            }

            LlvmType::Function { params, return_type, is_varargs } => {
                let param_str = params.iter()
                    .map(|t| t.to_llvm_ir())
                    .collect::<Vec<_>>()
                    .join(", ");

                let varargs = if *is_varargs { "..." } else { "" };

                format!("{} ({} {}) -> {}", return_type.to_llvm_ir(), param_str, varargs, return_type.to_llvm_ir())
            }
        }
    }
}

/// Convert LIR type to LLVM type
impl From<LirTy> for LlvmType {
    fn from(lir_ty: LirTy) -> Self {
        match lir_ty {
            // Primitives
            LirTy::I8 => LlvmType::Integer(8),
            LirTy::I16 => LlvmType::Integer(16),
            LirTy::I32 => LlvmType::Integer(32),
            LirTy::I64 => LlvmType::Integer(64),
            LirTy::I128 => LlvmType::Integer(128),
            LirTy::ISize => LlvmType::Integer(64), // Assume 64-bit
            LirTy::U8 => LlvmType::Integer(8),
            LirTy::U16 => LlvmType::Integer(16),
            LirTy::U32 => LlvmType::Integer(32),
            LirTy::U64 => LlvmType::Integer(64),
            LirTy::U128 => LlvmType::Integer(128),
            LirTy::USize => LlvmType::Integer(64), // Assume 64-bit
            LirTy::F32 => LlvmType::Float(32),
            LirTy::F64 => LlvmType::Float(64),
            LirTy::Bool => LlvmType::Integer(1),

            // Special
            // NOTE: Unit maps to i32 dummy value, not void.
            // Void is only valid for function returns in LLVM, not for values.
            // Using i32 allows unit values to be used in expressions and will be optimized away.
            LirTy::Unit => LlvmType::Integer(32),
            LirTy::Never => LlvmType::Void, // Never is bottom type

            // Pointers
            LirTy::Ptr(inner) => {
                LlvmType::Pointer(Box::new((*inner).into()))
            }

            // Arrays
            LirTy::Array { inner, len } => {
                LlvmType::Array {
                    inner: Box::new((*inner).into()),
                    len,
                }
            }

            // Structs
            LirTy::Struct { name, .. } => {
                // Simplified: struct without field info
                LlvmType::Struct {
                    name,
                    fields: vec![LlvmType::Integer(32)], // Placeholder
                }
            }
        }
    }
}

/// Get LLVM type name for a type
pub fn get_llvm_type(ty: &LirTy) -> LlvmType {
    ty.clone().into()
}
