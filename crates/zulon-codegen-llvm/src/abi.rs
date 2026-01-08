// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Function calling conventions
//!
//! Implements System V AMD64 ABI for function calls on 64-bit platforms.

use crate::error::Result;
use std::collections::HashMap;

/// Calling convention type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    /// System V AMD64 ABI (Linux, macOS, BSD)
    SystemVAMD64,
    /// Microsoft x64 ABI (Windows)
    MicrosoftX64,
    /// AArch64 ABI (ARM64)
    AArch64,
}

/// Argument location
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgLocation {
    /// Argument in register
    Register(String),
    /// Argument on stack (offset from stack pointer)
    Stack(i64),
    /// Argument returned in register (for return values)
    ReturnRegister(String),
}

/// Function call information
#[derive(Debug, Clone)]
pub struct CallInfo {
    /// CallingConvention
    pub cc: CallingConvention,
    /// Argument locations (in order)
    pub arg_locations: Vec<ArgLocation>,
    /// Return value location
    pub return_location: ArgLocation,
    /// Stack size for arguments (in bytes)
    pub stack_arg_size: i64,
    /// Stack size for locals (in bytes)
    pub stack_local_size: i64,
    /// Total stack size (in bytes)
    pub total_stack_size: i64,
    /// Register usage map
    pub register_used: HashMap<String, bool>,
}

impl CallInfo {
    /// Create new call info
    pub fn new(cc: CallingConvention) -> Self {
        let register_used = Self::init_registers(&cc);

        Self {
            cc,
            arg_locations: Vec::new(),
            return_location: ArgLocation::Register("rax".to_string()),
            stack_arg_size: 0,
            stack_local_size: 0,
            total_stack_size: 0,
            register_used,
        }
    }

    /// Initialize register availability for calling convention
    fn init_registers(cc: &CallingConvention) -> HashMap<String, bool> {
        match cc {
            CallingConvention::SystemVAMD64 => {
                // Integer argument registers (in order)
                let int_regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

                // Float argument registers (in order)
                let float_regs = vec!["xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7"];

                let mut regs = HashMap::new();
                for reg in int_regs.iter().chain(float_regs.iter()) {
                    regs.insert(reg.to_string(), false);
                }
                regs
            }

            CallingConvention::MicrosoftX64 => {
                // Microsoft x64 uses different registers
                let int_regs = vec!["rcx", "rdx", "r8", "r9"];
                let float_regs = vec!["xmm0", "xmm1", "xmm2", "xmm3"];

                let mut regs = HashMap::new();
                for reg in int_regs.iter().chain(float_regs.iter()) {
                    regs.insert(reg.to_string(), false);
                }
                regs
            }

            CallingConvention::AArch64 => {
                // ARM64 argument registers
                let regs = vec!["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"];

                let mut map = HashMap::new();
                for reg in regs {
                    map.insert(reg.to_string(), false);
                }
                map
            }
        }
    }

    /// Allocate an argument location
    pub fn allocate_arg(&mut self, ty: &zulon_lir::LirTy, is_return: bool) -> Result<ArgLocation> {
        if is_return {
            return Ok(self.allocate_return(ty));
        }

        let location = match self.cc {
            CallingConvention::SystemVAMD64 => {
                self.allocate_systemv_arg(ty)?
            }

            CallingConvention::MicrosoftX64 => {
                self.allocate_msx64_arg(ty)?
            }

            CallingConvention::AArch64 => {
                self.allocate_aarch64_arg(ty)?
            }
        };

        // Track the location
        self.arg_locations.push(location.clone());
        Ok(location)
    }

    /// Allocate argument using System V AMD64 ABI
    fn allocate_systemv_arg(&mut self, ty: &zulon_lir::LirTy) -> Result<ArgLocation> {
        // Check if type fits in register
        let size = ty.size();
        let align = ty.align();

        // Types larger than 16 bytes or with align > 8 are passed on stack
        if size > 16 || align > 8 {
            return Ok(self.allocate_stack_arg(size, align));
        }

        // Try integer registers first
        let int_regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for reg in int_regs {
            if !self.register_used.get(reg).unwrap_or(&true) {
                self.register_used.insert(reg.to_string(), true);
                return Ok(ArgLocation::Register(reg.to_string()));
            }
        }

        // If no integer registers available, use stack
        Ok(self.allocate_stack_arg(size, align))
    }

    /// Allocate argument using Microsoft x64 ABI
    fn allocate_msx64_arg(&mut self, ty: &zulon_lir::LirTy) -> Result<ArgLocation> {
        let size = ty.size();
        let align = ty.align();

        // Microsoft x64: > 8 bytes or > 8 byte align goes on stack
        if size > 8 || align > 8 {
            return Ok(self.allocate_stack_arg(size, align));
        }

        // Try integer registers
        let int_regs = vec!["rcx", "rdx", "r8", "r9"];
        for reg in int_regs {
            if !self.register_used.get(reg).unwrap_or(&true) {
                self.register_used.insert(reg.to_string(), true);
                return Ok(ArgLocation::Register(reg.to_string()));
            }
        }

        Ok(self.allocate_stack_arg(size, align))
    }

    /// Allocate argument using AArch64 ABI
    fn allocate_aarch64_arg(&mut self, ty: &zulon_lir::LirTy) -> Result<ArgLocation> {
        let size = ty.size();

        // Try integer registers
        let int_regs = vec!["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"];
        for reg in int_regs {
            if !self.register_used.get(reg).unwrap_or(&true) {
                self.register_used.insert(reg.to_string(), true);
                return Ok(ArgLocation::Register(reg.to_string()));
            }
        }

        // Use stack if no registers available
        Ok(self.allocate_stack_arg(size, ty.align()))
    }

    /// Allocate return value location
    fn allocate_return(&mut self, ty: &zulon_lir::LirTy) -> ArgLocation {
        let size = ty.size();

        // Large types returned in memory (caller allocates, pointer passed as hidden arg)
        if size > 16 {
            ArgLocation::ReturnRegister("rax".to_string())  // Pointer to return value
        } else {
            // Small types returned in rax/rax+xmm0
            if ty.is_float() {
                ArgLocation::ReturnRegister("xmm0".to_string())
            } else {
                ArgLocation::ReturnRegister("rax".to_string())
            }
        }
    }

    /// Allocate argument on stack
    fn allocate_stack_arg(&mut self, size: u64, align: u64) -> ArgLocation {
        // Calculate offset (positive from stack pointer)
        let offset = self.stack_arg_size as i64;

        // Round up to alignment
        let aligned_offset = ((offset + (align as i64 - 1)) / (align as i64)) * (align as i64);

        // Update stack size
        self.stack_arg_size = aligned_offset + size as i64;

        ArgLocation::Stack(aligned_offset)
    }

    /// Finalize stack layout
    pub fn finalize_stack(&mut self, local_size: i64) {
        self.stack_local_size = local_size;

        // Total stack = args + locals + return address + saved registers
        let stack_align = 16;  // Stack must be 16-byte aligned
        let mut total = self.stack_arg_size + self.stack_local_size + 8;  // +8 for return address

        // Round up to 16-byte alignment
        total = ((total + stack_align - 1) / stack_align) * stack_align;

        self.total_stack_size = total;
    }

    /// Get stack pointer adjustment on function entry
    pub fn get_stack_adjust(&self) -> i64 {
        self.total_stack_size
    }

    /// Get prologue code
    pub fn get_prologue(&self) -> String {
        let mut prologue = String::new();

        // Stack adjustment
        if self.total_stack_size > 0 {
            prologue.push_str(&format!("  sub rsp, {}\n", self.total_stack_size));
        }

        // Save callee-saved registers if needed
        // (This is a simplified version - real implementation would track which regs are used)

        prologue
    }

    /// Get epilogue code
    pub fn get_epilogue(&self) -> String {
        let mut epilogue = String::new();

        // Restore stack
        if self.total_stack_size > 0 {
            epilogue.push_str(&format!("  add rsp, {}\n", self.total_stack_size));
        }

        epilogue
    }
}

/// Helper functions for calling conventions
impl CallInfo {
    /// Check if type is passed in integer register
    pub fn is_integer_aggregate(ty: &zulon_lir::LirTy) -> bool {
        match ty {
            zulon_lir::LirTy::I8 | zulon_lir::LirTy::I16 | zulon_lir::LirTy::I32 |
            zulon_lir::LirTy::I64 | zulon_lir::LirTy::I128 | zulon_lir::LirTy::ISize |
            zulon_lir::LirTy::U8 | zulon_lir::LirTy::U16 | zulon_lir::LirTy::U32 |
            zulon_lir::LirTy::U64 | zulon_lir::LirTy::U128 | zulon_lir::LirTy::USize |
            zulon_lir::LirTy::Bool | zulon_lir::LirTy::Ptr(_) => true,

            zulon_lir::LirTy::Struct { .. } => {
                // Check if all fields are integers
                // Simplified - should recurse into fields
                false
            }

            _ => false,
        }
    }

    /// Check if type is passed in float register
    pub fn is_float_aggregate(ty: &zulon_lir::LirTy) -> bool {
        matches!(ty, zulon_lir::LirTy::F32 | zulon_lir::LirTy::F64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemv_small_args() {
        let mut call_info = CallInfo::new(CallingConvention::SystemVAMD64);

        // Allocate i32 args
        let loc1 = call_info.allocate_arg(&zulon_lir::LirTy::I32, false).unwrap();
        let loc2 = call_info.allocate_arg(&zulon_lir::LirTy::I32, false).unwrap();
        let loc3 = call_info.allocate_arg(&zulon_lir::LirTy::I32, false).unwrap();

        // First 6 args should be in registers
        assert_eq!(loc1, ArgLocation::Register("rdi".to_string()));
        assert_eq!(loc2, ArgLocation::Register("rsi".to_string()));
        assert_eq!(loc3, ArgLocation::Register("rdx".to_string()));
    }

    #[test]
    fn test_systemv_many_args() {
        let mut call_info = CallInfo::new(CallingConvention::SystemVAMD64);

        // Allocate 7 integer args (more than available registers)
        for _ in 0..7 {
            call_info.allocate_arg(&zulon_lir::LirTy::I32, false).unwrap();
        }

        // First 6 in registers, 7th on stack
        assert_eq!(call_info.arg_locations[0], ArgLocation::Register("rdi".to_string()));
        assert_eq!(call_info.arg_locations[5], ArgLocation::Register("r9".to_string()));
        assert!(matches!(call_info.arg_locations[6], ArgLocation::Stack(_)));
    }

    #[test]
    fn test_systemv_return_value() {
        let mut call_info = CallInfo::new(CallingConvention::SystemVAMD64);

        // Small return type
        let loc1 = call_info.allocate_arg(&zulon_lir::LirTy::I32, true).unwrap();
        assert!(matches!(loc1, ArgLocation::ReturnRegister(_)));

        // Float return type
        let loc2 = call_info.allocate_arg(&zulon_lir::LirTy::F64, true).unwrap();
        assert_eq!(loc2, ArgLocation::ReturnRegister("xmm0".to_string()));
    }

    #[test]
    fn test_stack_alignment() {
        let mut call_info = CallInfo::new(CallingConvention::SystemVAMD64);

        // Allocate args that exceed registers
        for _ in 0..10 {
            call_info.allocate_arg(&zulon_lir::LirTy::I64, false).unwrap();
        }

        call_info.finalize_stack(32);

        // Stack should be 16-byte aligned
        assert_eq!(call_info.total_stack_size % 16, 0);
        assert!(call_info.total_stack_size > 0);
    }

    #[test]
    fn test_msx64_registers() {
        let mut call_info = CallInfo::new(CallingConvention::MicrosoftX64);

        // Microsoft x64 uses different registers
        let loc1 = call_info.allocate_arg(&zulon_lir::LirTy::I32, false).unwrap();
        let loc2 = call_info.allocate_arg(&zulon_lir::LirTy::I32, false).unwrap();

        assert_eq!(loc1, ArgLocation::Register("rcx".to_string()));
        assert_eq!(loc2, ArgLocation::Register("rdx".to_string()));
    }
}
