// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Enum layout computation
//!
//! Calculates enum size, alignment, and variant layouts for LLVM code generation.

use crate::error::Result;
use crate::layout::StructLayout;
use std::collections::HashMap;

/// Enum variant information
#[derive(Debug, Clone)]
pub struct VariantInfo {
    /// Variant discriminant value
    pub discriminant: u64,
    /// Variant name
    pub name: String,
    /// Variant fields (if data variant)
    pub fields: Vec<(String, zulon_lir::LirTy)>,
    /// Variant layout (if data variant)
    pub layout: Option<StructLayout>,
    /// Variant size (without discriminant)
    pub size: u64,
    /// Variant alignment
    pub align: u64,
}

/// Enum layout information
#[derive(Debug, Clone)]
pub struct EnumLayout {
    /// Enum name
    pub name: String,
    /// All variants
    pub variants: Vec<VariantInfo>,
    /// Discriminant type
    pub discriminant_type: zulon_lir::LirTy,
    /// Discriminant size (in bytes)
    pub discriminant_size: u64,
    /// Discriminant offset (in bytes)
    pub discriminant_offset: u64,
    /// Total enum size (in bytes)
    pub size: u64,
    /// Enum alignment (in bytes)
    pub align: u64,
    /// Offset where data starts (for data variants)
    pub data_offset: u64,
}

impl EnumLayout {
    /// Create a new enum layout
    pub fn new(name: String, discriminant_type: zulon_lir::LirTy) -> Self {
        let discriminant_size = discriminant_type.size();
        let discriminant_align = discriminant_type.align();

        Self {
            name,
            variants: Vec::new(),
            discriminant_type,
            discriminant_size,
            discriminant_offset: 0,
            size: discriminant_size,
            align: discriminant_align,
            data_offset: discriminant_size,
        }
    }

    /// Add a variant to the enum
    pub fn add_variant(
        &mut self,
        name: String,
        discriminant: u64,
        fields: Vec<(String, zulon_lir::LirTy)>,
    ) -> Result<()> {
        let (size, align, layout) = if fields.is_empty() {
            // Unit-like variant - no data
            (0, 1, None)
        } else {
            // Data variant - compute layout
            let mut struct_layout = StructLayout::new(format!("{}_{}", self.name, name));
            for (field_name, field_ty) in &fields {
                struct_layout.add_field(field_name.clone(), field_ty.clone())?;
            }
            struct_layout.finalize();

            (struct_layout.size, struct_layout.align, Some(struct_layout))
        };

        // Update alignment (max of discriminant and all variants)
        // But start with discriminant alignment
        let disc_align = self.discriminant_type.align();
        self.align = self.align.max(disc_align).max(align);

        // Add variant
        self.variants.push(VariantInfo {
            discriminant,
            name: name.clone(),
            fields,
            layout,
            size,
            align,
        });

        // Update size
        let variant_size = self.data_offset + size;
        self.size = self.size.max(variant_size);

        Ok(())
    }

    /// Finalize the layout
    pub fn finalize(&mut self) {
        // Empty enum has at least discriminant
        if self.variants.is_empty() {
            self.size = self.discriminant_size;
            self.align = self.discriminant_align();
            return;
        }

        // Round up size to match alignment
        if self.size % self.align != 0 {
            self.size = ((self.size + self.align - 1) / self.align) * self.align;
        }
    }

    /// Get discriminant alignment
    pub fn discriminant_align(&self) -> u64 {
        self.discriminant_type.align()
    }

    /// Find variant by discriminant
    pub fn variant_by_discriminant(&self, discriminant: u64) -> Option<&VariantInfo> {
        self.variants.iter().find(|v| v.discriminant == discriminant)
    }

    /// Find variant by name
    pub fn variant_by_name(&self, name: &str) -> Option<&VariantInfo> {
        self.variants.iter().find(|v| v.name == name)
    }

    /// Get variant index
    pub fn variant_index(&self, name: &str) -> Option<usize> {
        self.variants.iter().position(|v| v.name == name)
    }

    /// Check if enum has only unit variants (C-like enum)
    pub fn is_c_like(&self) -> bool {
        self.variants.iter().all(|v| v.fields.is_empty())
    }

    /// Get LLVM enum type string (simplified as opaque byte array)
    pub fn to_llvm_type(&self) -> String {
        format!("[{} x i8]", self.size)
    }

    /// Get LLVM enum definition (as opaque struct)
    pub fn to_llvm_definition(&self) -> String {
        if self.is_c_like() {
            // C-like enum - just discriminant
            let llvm_ty: crate::ty::LlvmType = self.discriminant_type.clone().into();
            format!("%enum.{} = type {}", self.name, llvm_ty.to_llvm_ir())
        } else {
            // Rust-like enum - opaque byte array
            format!("%enum.{} = type [{} x i8]", self.name, self.size)
        }
    }

    /// Generate discriminant access GEP indices
    pub fn discriminant_gep_indices(&self) -> Vec<u64> {
        vec![0, 0]  // [struct_ptr, field_0]
    }

    /// Generate data field access GEP indices
    pub fn data_gep_indices(&self, field_index: usize) -> Vec<u64> {
        vec![0, 1, field_index as u64]  // [struct_ptr, data_field, actual_field]
    }
}

/// Enum layout cache
#[derive(Debug, Clone, Default)]
pub struct EnumLayoutCache {
    layouts: HashMap<String, EnumLayout>,
}

impl EnumLayoutCache {
    /// Create a new enum layout cache
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
        }
    }

    /// Get or compute enum layout
    pub fn get_layout(&mut self, name: &str, discriminant_type: zulon_lir::LirTy) -> Result<EnumLayout> {
        if let Some(layout) = self.layouts.get(name) {
            return Ok(layout.clone());
        }

        let layout = EnumLayout::new(name.to_string(), discriminant_type);
        self.layouts.insert(name.to_string(), layout.clone());

        Ok(layout)
    }

    /// Insert a computed layout
    pub fn insert_layout(&mut self, layout: EnumLayout) {
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
    fn test_c_like_enum() {
        // enum Option { None, Some }
        let mut layout = EnumLayout::new("Option".to_string(), zulon_lir::LirTy::I8);

        layout.add_variant("None".to_string(), 0, vec![]).unwrap();
        layout.add_variant("Some".to_string(), 1, vec![]).unwrap();
        layout.finalize();

        // C-like enum: only discriminant
        assert!(layout.is_c_like());
        assert_eq!(layout.size, 1);  // i8
        assert_eq!(layout.align, 1);
        assert_eq!(layout.variants.len(), 2);

        assert_eq!(layout.variants[0].discriminant, 0);
        assert_eq!(layout.variants[1].discriminant, 1);
    }

    #[test]
    fn test_enum_with_data() {
        // enum Option<T> { None, Some(T) }
        let mut layout = EnumLayout::new("Option".to_string(), zulon_lir::LirTy::I8);

        layout.add_variant("None".to_string(), 0, vec![]).unwrap();
        layout.add_variant("Some".to_string(), 1, vec![
            ("value".to_string(), zulon_lir::LirTy::I32)
        ]).unwrap();
        layout.finalize();

        // Not C-like - has data variant
        assert!(!layout.is_c_like());
        // Size: 1 (disc) + 4 (data) = 5, rounded to align 4 (from i32)
        assert_eq!(layout.size, 8);
        assert_eq!(layout.align, 4);  // i32 alignment
        assert_eq!(layout.variants.len(), 2);

        // Some variant has data
        assert!(layout.variants[1].layout.is_some());
        assert_eq!(layout.variants[1].size, 4);
    }

    #[test]
    fn test_multi_variant_enum() {
        // enum Result { Ok(i32), Error(String), Pending }
        let mut layout = EnumLayout::new("Result".to_string(), zulon_lir::LirTy::U8);

        layout.add_variant("Ok".to_string(), 0, vec![
            ("value".to_string(), zulon_lir::LirTy::I32)
        ]).unwrap();

        layout.add_variant("Error".to_string(), 1, vec![
            ("msg".to_string(), zulon_lir::LirTy::Ptr(Box::new(zulon_lir::LirTy::U8)))
        ]).unwrap();

        layout.add_variant("Pending".to_string(), 2, vec![]).unwrap();
        layout.finalize();

        assert!(!layout.is_c_like());
        assert_eq!(layout.variants.len(), 3);

        // Find variant by name
        assert!(layout.variant_by_name("Ok").is_some());
        assert!(layout.variant_by_name("Error").is_some());
        assert!(layout.variant_by_name("Pending").is_some());
        assert!(layout.variant_by_name("Unknown").is_none());

        // Variant indices
        assert_eq!(layout.variant_index("Ok"), Some(0));
        assert_eq!(layout.variant_index("Error"), Some(1));
        assert_eq!(layout.variant_index("Pending"), Some(2));
    }

    #[test]
    fn test_empty_enum() {
        let mut layout = EnumLayout::new("Empty".to_string(), zulon_lir::LirTy::I32);
        layout.finalize();

        assert_eq!(layout.size, 4);  // Only discriminant
        assert_eq!(layout.align, 4);
        assert_eq!(layout.variants.len(), 0);
    }

    #[test]
    fn test_enum_alignment() {
        // enum with large alignment requirements
        let mut layout = EnumLayout::new("AlignTest".to_string(), zulon_lir::LirTy::I8);

        // Unit variant
        layout.add_variant("Unit".to_string(), 0, vec![]).unwrap();

        // Data variant with i64 (8-byte alignment)
        layout.add_variant("Data".to_string(), 1, vec![
            ("value".to_string(), zulon_lir::LirTy::I64)
        ]).unwrap();

        layout.finalize();

        // Alignment should be 8 (from i64 variant)
        assert_eq!(layout.align, 8);
        // Size should be aligned to 8
        assert_eq!(layout.size % 8, 0);
    }
}
