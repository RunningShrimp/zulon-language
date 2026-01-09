// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Struct layout computation
//!
//! Calculates field offsets, struct size, and alignment for LLVM code generation.

use crate::error::Result;
use std::collections::HashMap;

/// Field information in a struct
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: zulon_lir::LirTy,
    /// Offset from struct start (in bytes)
    pub offset: u64,
    /// Field size (in bytes)
    pub size: u64,
    /// Field alignment (in bytes)
    pub align: u64,
}

/// Struct layout information
#[derive(Debug, Clone)]
pub struct StructLayout {
    /// Struct name
    pub name: String,
    /// Fields in order of declaration
    pub fields: Vec<FieldInfo>,
    /// Total struct size (in bytes)
    pub size: u64,
    /// Struct alignment (in bytes)
    pub align: u64,
    /// Padding at end (in bytes)
    pub tail_padding: u64,
}

impl StructLayout {
    /// Create a new struct layout
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            size: 0,
            align: 1,
            tail_padding: 0,
        }
    }

    /// Add a field to the struct
    pub fn add_field(&mut self, name: String, ty: zulon_lir::LirTy) -> Result<()> {
        let field_size = ty.size();
        let field_align = ty.align();

        // Update struct alignment (max of all field alignments)
        self.align = self.align.max(field_align);

        // Calculate offset with proper alignment
        let offset = self.round_up_to_align(self.size, field_align);

        // Add padding if needed
        let padding = offset - self.size;
        if padding > 0 {
            // Will be handled in size calculation
        }

        // Add field
        self.fields.push(FieldInfo {
            name: name.clone(),
            ty,
            offset,
            size: field_size,
            align: field_align,
        });

        // Update size
        self.size = offset + field_size;

        Ok(())
    }

    /// Finalize the layout (calculate tail padding)
    pub fn finalize(&mut self) {
        // Empty struct has size 1
        if self.fields.is_empty() {
            self.size = 1;
            self.tail_padding = 0;
            return;
        }

        // Round up size to match alignment
        let rounded_size = self.round_up_to_align(self.size, self.align);
        self.tail_padding = rounded_size - self.size;
        self.size = rounded_size;
    }

    /// Get field offset by name
    pub fn field_offset(&self, name: &str) -> Option<u64> {
        self.fields.iter().find(|f| f.name == name).map(|f| f.offset)
    }

    /// Get field index by name
    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }

    /// Round up value to alignment
    fn round_up_to_align(&self, value: u64, align: u64) -> u64 {
        ((value + align - 1) / align) * align
    }

    /// Get LLVM struct type string
    pub fn to_llvm_type(&self) -> String {
        let field_types: Vec<String> = self.fields
            .iter()
            .map(|f| {
                let llvm_ty: crate::ty::LlvmType = f.ty.clone().into();
                llvm_ty.to_llvm_ir()
            })
            .collect();

        if field_types.is_empty() {
            // Empty struct - represented as i8
            "i8".to_string()
        } else {
            format!("[{}]", field_types.join(", "))
        }
    }

    /// Get LLVM struct definition
    pub fn to_llvm_definition(&self) -> String {
        let field_types: Vec<String> = self.fields
            .iter()
            .map(|f| {
                let llvm_ty: crate::ty::LlvmType = f.ty.clone().into();
                llvm_ty.to_llvm_ir()
            })
            .collect();

        if field_types.is_empty() {
            // Empty struct
            format!("%struct.{} = type {{ i8 }}", self.name)
        } else {
            format!("%struct.{} = type {{ {} }}", self.name, field_types.join(", "))
        }
    }
}

/// Struct layout cache
#[derive(Debug, Clone, Default)]
pub struct LayoutCache {
    layouts: HashMap<String, StructLayout>,
}

impl LayoutCache {
    /// Create a new layout cache
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
        }
    }

    /// Get or compute struct layout
    pub fn get_layout(&mut self, name: &str, fields: &[(String, zulon_lir::LirTy)]) -> Result<StructLayout> {
        // Use cached version if available
        if let Some(layout) = self.layouts.get(name) {
            return Ok(layout.clone());
        }

        // Compute new layout
        let mut layout = StructLayout::new(name.to_string());

        for (field_name, field_ty) in fields {
            layout.add_field(field_name.clone(), field_ty.clone())?;
        }

        layout.finalize();

        // Cache it
        self.layouts.insert(name.to_string(), layout.clone());

        Ok(layout)
    }

    /// Add a pre-computed layout
    pub fn insert_layout(&mut self, layout: StructLayout) {
        self.layouts.insert(layout.name.clone(), layout);
    }

    /// Check if layout exists
    pub fn has_layout(&self, name: &str) -> bool {
        self.layouts.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_struct() {
        let mut layout = StructLayout::new("Test".to_string());

        // struct { a: i32, b: i64 }
        layout.add_field("a".to_string(), zulon_lir::LirTy::I32).unwrap();
        layout.add_field("b".to_string(), zulon_lir::LirTy::I64).unwrap();
        layout.finalize();

        // i32 at offset 0 (size 4, align 4)
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].size, 4);
        assert_eq!(layout.fields[0].align, 4);

        // i64 at offset 8 (needs 8-byte alignment)
        assert_eq!(layout.fields[1].offset, 8);
        assert_eq!(layout.fields[1].size, 8);
        assert_eq!(layout.fields[1].align, 8);

        // Total size: 8 + 8 = 16
        assert_eq!(layout.size, 16);
        assert_eq!(layout.align, 8);
    }

    #[test]
    fn test_packed_struct() {
        let mut layout = StructLayout::new("Packed".to_string());

        // struct { a: i8, b: i32, c: i8 }
        layout.add_field("a".to_string(), zulon_lir::LirTy::I8).unwrap();
        layout.add_field("b".to_string(), zulon_lir::LirTy::I32).unwrap();
        layout.add_field("c".to_string(), zulon_lir::LirTy::I8).unwrap();
        layout.finalize();

        // i8 at offset 0
        assert_eq!(layout.fields[0].offset, 0);

        // i32 at offset 4 (needs 4-byte alignment)
        assert_eq!(layout.fields[1].offset, 4);

        // i8 at offset 8
        assert_eq!(layout.fields[2].offset, 8);

        // Total size: 9, rounded to 12 (alignment 4)
        assert_eq!(layout.size, 12);
        assert_eq!(layout.tail_padding, 3);
    }

    #[test]
    fn test_empty_struct() {
        let mut layout = StructLayout::new("Empty".to_string());
        layout.finalize();

        assert_eq!(layout.size, 1); // Minimum size is 1 byte
        assert_eq!(layout.fields.len(), 0);
    }

    #[test]
    fn test_nested_struct() {
        // struct Inner { a: i32, b: i32 }
        // struct Outer { inner: Inner, c: i64 }

        let inner_ty = zulon_lir::LirTy::Struct {
            name: "Inner".to_string(),
            fields: vec![zulon_lir::LirTy::I32, zulon_lir::LirTy::I32],
            size: 8, // i32 + i32
        };

        let mut layout = StructLayout::new("Outer".to_string());
        layout.add_field("inner".to_string(), inner_ty).unwrap();
        layout.add_field("c".to_string(), zulon_lir::LirTy::I64).unwrap();
        layout.finalize();

        // Inner at offset 0
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].size, 8);

        // i64 at offset 8
        assert_eq!(layout.fields[1].offset, 8);
        assert_eq!(layout.fields[1].size, 8);

        // Total size: 16
        assert_eq!(layout.size, 16);
    }

    #[test]
    fn test_layout_cache() {
        let mut cache = LayoutCache::new();

        let fields = vec![
            ("a".to_string(), zulon_lir::LirTy::I32),
            ("b".to_string(), zulon_lir::LirTy::I64),
        ];

        let layout1 = cache.get_layout("Test", &fields).unwrap();
        let layout2 = cache.get_layout("Test", &fields).unwrap();

        // Should return cached version
        assert_eq!(layout1.size, layout2.size);
        assert_eq!(cache.layouts.len(), 1);
    }
}
