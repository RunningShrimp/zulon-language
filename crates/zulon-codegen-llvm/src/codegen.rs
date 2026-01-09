// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LLVM IR code generation
//!
//! Converts LIR to LLVM IR (text format).

use crate::abi::{CallInfo, CallingConvention};
use crate::error::{CodegenError, Result};
use crate::enum_layout::{EnumLayout, EnumLayoutCache};
use crate::layout::{LayoutCache, StructLayout};
use crate::ty::LlvmType;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use zulon_lir::{LirBlock, LirFunction, LirInstruction, LirOperand, LirTerminator};

/// String constant data
struct StringConstant {
    name: String,
    value: String,
    len: usize,
}

/// LLVM IR code generator
pub struct CodeGenerator<W: Write> {
    writer: W,
    indent: usize,
    /// Struct layout cache
    layout_cache: Arc<LayoutCache>,
    /// Enum layout cache
    #[allow(dead_code)]
    enum_cache: Arc<EnumLayoutCache>,
    /// Struct type declarations
    struct_types: HashMap<String, StructLayout>,
    /// Enum type declarations
    enum_types: HashMap<String, EnumLayout>,
    /// Calling convention to use
    calling_convention: CallingConvention,
    /// String constants to emit at module level
    string_constants: Vec<StringConstant>,
    /// Mapping from vreg to string constant index
    string_vreg_map: HashMap<usize, usize>,
}

impl<W: Write> CodeGenerator<W> {
    /// Create a new code generator
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            indent: 0,
            layout_cache: Arc::new(LayoutCache::new()),
            enum_cache: Arc::new(EnumLayoutCache::new()),
            struct_types: HashMap::new(),
            enum_types: HashMap::new(),
            calling_convention: CallingConvention::SystemVAMD64, // Default
            string_constants: Vec::new(),
            string_vreg_map: HashMap::new(),
        }
    }

    /// Create a new code generator with shared caches
    pub fn with_caches(
        writer: W,
        layout_cache: Arc<LayoutCache>,
        enum_cache: Arc<EnumLayoutCache>,
    ) -> Self {
        Self {
            writer,
            indent: 0,
            layout_cache,
            enum_cache,
            struct_types: HashMap::new(),
            enum_types: HashMap::new(),
            calling_convention: CallingConvention::SystemVAMD64,
            string_constants: Vec::new(),
            string_vreg_map: HashMap::new(),
        }
    }

    /// Set the calling convention
    pub fn with_calling_convention(mut self, cc: CallingConvention) -> Self {
        self.calling_convention = cc;
        self
    }

    /// Register a struct type
    pub fn register_struct(&mut self, layout: StructLayout) {
        let name = layout.name.clone();
        self.struct_types.insert(name, layout);
    }

    /// Register an enum type
    pub fn register_enum(&mut self, layout: EnumLayout) {
        let name = layout.name.clone();
        self.enum_types.insert(name, layout);
    }

    /// Get the layout cache
    pub fn layout_cache(&self) -> &LayoutCache {
        &self.layout_cache
    }

    /// Generate LLVM IR for a function
    pub fn generate_function(&mut self, func: &LirFunction) -> Result<()> {
        // Function declaration
        self.write_function_header(func)?;
        writeln!(self.writer, " {{").unwrap();

        // Generate blocks
        self.indent += 1;
        let mut block_ids: Vec<_> = func.blocks.keys().copied().collect();
        block_ids.sort();  // Generate in ID order for determinism

        for block_id in block_ids {
            if let Some(block) = func.blocks.get(&block_id) {
                self.generate_block(func, block)?;
            }
        }

        self.indent -= 1;
        writeln!(self.writer, "}}").unwrap();
        writeln!(self.writer).unwrap();

        Ok(())
    }

    /// Write function header
    fn write_function_header(&mut self, func: &LirFunction) -> Result<()> {
        // Define return type
        let return_type: LlvmType = func.return_type.clone().into();
        write!(self.writer, "define {}", return_type.to_llvm_ir()).unwrap();

        // Function name
        write!(self.writer, " @{}", func.name).unwrap();

        // Parameters
        write!(self.writer, "(").unwrap();
        for (i, param_ty) in func.param_types.iter().enumerate() {
            if i > 0 {
                write!(self.writer, ", ").unwrap();
            }
            let llvm_ty: LlvmType = param_ty.clone().into();
            write!(self.writer, "{} %v{}", llvm_ty.to_llvm_ir(), func.params[i]).unwrap();
        }
        write!(self.writer, ")").unwrap();

        Ok(())
    }

    /// Generate a basic block
    fn generate_block(&mut self, func: &LirFunction, block: &LirBlock) -> Result<()> {
        // Block label
        writeln!(self.writer, "{}block{}:", "  ".repeat(self.indent), block.id)
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        self.indent += 1;

        // Phi nodes
        for (vreg, phi) in &block.phi_nodes {
            self.generate_phi(vreg, phi)?;
        }

        // Instructions
        for instr in &block.instructions {
            self.generate_instruction(instr)?;
        }

        // Terminator
        if let Some(terminator) = &block.terminator {
            self.generate_terminator(func, terminator)?;
        }

        self.indent -= 1;
        Ok(())
    }

    /// Generate phi node
    fn generate_phi(&mut self, vreg: &zulon_lir::VReg, phi: &zulon_lir::LirPhi) -> Result<()> {
        let ty: LlvmType = phi.ty.clone().into();
        write!(self.writer, "{}  %v{} = phi {}", "  ".repeat(self.indent), vreg, ty.to_llvm_ir()).unwrap();

        let mut sources = phi.sources.iter().peekable();
        while let Some((reg, block_id)) = sources.next() {
            // Special case: vreg 0 represents undef (no value from this predecessor)
            if *reg == 0 {
                write!(self.writer, "[ undef, %block{} ]", block_id).unwrap();
            } else {
                write!(self.writer, "[ %v{}, %block{} ]", reg, block_id).unwrap();
            }
            if sources.peek().is_some() {
                write!(self.writer, ", ").unwrap();
            }
        }
        writeln!(self.writer).unwrap();

        Ok(())
    }

    /// Generate instruction
    fn generate_instruction(&mut self, instr: &LirInstruction) -> Result<()> {
        match instr {
            LirInstruction::Alloca(alloca) => {
                self.generate_alloca(alloca)?;
            }

            LirInstruction::Const { dest, value, ty } => {
                self.generate_const(*dest, value, ty)?;
            }

            LirInstruction::Copy { dest, src, ty: _ } => {
                let ty = LlvmType::Integer(32);  // Placeholder
                writeln!(
                    self.writer,
                    "{}  %v{} = add {} %v{}, 0",
                    "  ".repeat(self.indent),
                    dest,
                    ty.to_llvm_ir(),
                    src
                ).unwrap();
            }

            LirInstruction::BinaryOp { dest, op, left, right, ty } => {
                self.generate_binary_op(*dest, op, *left, *right, ty)?;
            }

            LirInstruction::UnaryOp { dest, op, operand, ty } => {
                self.generate_unary_op(*dest, op, *operand, ty)?;
            }

            LirInstruction::Load { dest, src, ty } => {
                self.generate_load(*dest, src, ty)?;
            }

            LirInstruction::Store { dest, src, ty } => {
                self.generate_store(dest, *src, ty)?;
            }

            LirInstruction::Gep { dest, base, indices, ty } => {
                self.generate_gep(*dest, *base, indices, ty)?;
            }

            LirInstruction::Call { dest, func: _func_vreg, args, return_type } => {
                // Placeholder: use function name "unknown"
                self.generate_call(*dest, "unknown", args, return_type, &[])?;
            }

            LirInstruction::CallExternal { dest, func_name, args, arg_types, return_type } => {
                self.generate_call(*dest, func_name, args, return_type, arg_types)?;
            }

            LirInstruction::Cmp { dest, op, left, right } => {
                // Assume i32 for now
                let ty = zulon_lir::LirTy::I32;
                self.generate_cmp(*dest, op, *left, *right, &ty)?;
            }

            LirInstruction::Cast { dest, src, from, to } => {
                self.generate_cast(*dest, *src, from, to)?;
            }

            LirInstruction::RefInc { ptr, ty } => {
                self.generate_ref_inc(*ptr, ty)?;
            }

            LirInstruction::RefDec { ptr, ty } => {
                self.generate_ref_dec(*ptr, ty)?;
            }
        }

        Ok(())
    }

    /// Generate alloca instruction
    fn generate_alloca(&mut self, alloca: &zulon_lir::LirAlloca) -> Result<()> {
        let llvm_ty: LlvmType = alloca.ty.clone().into();

        // Use struct reference if this is a declared struct type
        let type_str = if matches!(alloca.ty, zulon_lir::LirTy::Struct { .. }) {
            llvm_ty.to_llvm_ref()
        } else {
            llvm_ty.to_llvm_ir()
        };

        writeln!(
            self.writer,
            "{}  %v{} = alloca {}",
            "  ".repeat(self.indent),
            alloca.dest,
            type_str
        ).map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        Ok(())
    }

    /// Generate constant instruction
    fn generate_const(&mut self, dest: zulon_lir::VReg, value: &zulon_lir::LirConstant, ty: &zulon_lir::LirTy) -> Result<()> {
        let llvm_ty: LlvmType = ty.clone().into();

        match value {
            zulon_lir::LirConstant::Integer(val) => {
                if ty.is_float() {
                    writeln!(
                        self.writer,
                        "{}  %v{} = fadd {} 0.0, {}",
                        "  ".repeat(self.indent),
                        dest,
                        llvm_ty.to_llvm_ir(),
                        val
                    ).unwrap();
                } else {
                    writeln!(
                        self.writer,
                        "{}  %v{} = add {} 0, {}",
                        "  ".repeat(self.indent),
                        dest,
                        llvm_ty.to_llvm_ir(),
                        val
                    ).unwrap();
                }
            }

            zulon_lir::LirConstant::Float(val) => {
                writeln!(
                    self.writer,
                    "{}  %v{} = fadd {} 0.0, {}",
                    "  ".repeat(self.indent),
                    dest,
                    llvm_ty.to_llvm_ir(),
                    val
                ).unwrap();
            }

            zulon_lir::LirConstant::String(s) => {
                // String constants are collected at module level
                // Here we just generate the getelementptr to get the address
                let str_id = self.string_vreg_map.get(&(dest as usize)).unwrap();
                let global_name = format!("@.str{}", str_id);

                writeln!(
                    self.writer,
                    "{}  %v{} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
                    "  ".repeat(self.indent),
                    dest,
                    s.len() + 1, // +1 for null terminator
                    s.len() + 1,
                    global_name
                ).unwrap();
            }

            zulon_lir::LirConstant::Bool(val) => {
                writeln!(
                    self.writer,
                    "{}  %v{} = add {} 0, {}",
                    "  ".repeat(self.indent),
                    dest,
                    llvm_ty.to_llvm_ir(),
                    if *val { 1 } else { 0 }
                ).unwrap();
            }

            zulon_lir::LirConstant::Unit => {
                writeln!(self.writer, "{}  %v{} = add {} 0, 0", "  ".repeat(self.indent), dest, llvm_ty.to_llvm_ir()).unwrap();
            }
        }

        Ok(())
    }

    /// Collect string constants from a function
    fn collect_string_constants(&mut self, func: &LirFunction) {
        for (_block_id, block) in &func.blocks {
            for instr in &block.instructions {
                if let zulon_lir::LirInstruction::Const { dest, value, .. } = instr {
                    if let zulon_lir::LirConstant::String(s) = value {
                        // Find next available string ID
                        let str_id = self.string_constants.len();

                        self.string_constants.push(StringConstant {
                            name: format!(".str{}", str_id),
                            value: s.clone(),
                            len: s.len() + 1, // +1 for null terminator
                        });

                        // Map vreg to string constant index
                        self.string_vreg_map.insert(*dest as usize, str_id);
                    }
                }
            }
        }
    }

    fn escape_string_for_llvm(s: &str) -> String {
        let mut result = String::from("\"");

        for c in s.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\0A"),
                '\r' => result.push_str("\\0D"),
                '\t' => result.push_str("\\09"),
                _ => result.push(c),
            }
        }

        result.push_str("\\00\"");
        result
    }

    /// Generate binary operation
    fn generate_binary_op(
        &mut self,
        dest: zulon_lir::VReg,
        op: &zulon_lir::LirBinOp,
        left: zulon_lir::VReg,
        right: zulon_lir::VReg,
        ty: &zulon_lir::LirTy,
    ) -> Result<()> {
        let llvm_ty: LlvmType = ty.clone().into();
        let llvm_op = self.binary_op_to_llvm(op, ty.is_float())?;

        if ty.is_float() {
            writeln!(
                self.writer,
                "{}  %v{} = f{} {} %v{}, %v{}",
                "  ".repeat(self.indent),
                dest,
                llvm_op,
                llvm_ty.to_llvm_ir(),
                left,
                right
            ).unwrap();
        } else {
            writeln!(
                self.writer,
                "{}  %v{} = {} {} %v{}, %v{}",
                "  ".repeat(self.indent),
                dest,
                llvm_op,
                llvm_ty.to_llvm_ir(),
                left,
                right
            ).unwrap();
        }

        Ok(())
    }

    /// Generate unary operation
    fn generate_unary_op(
        &mut self,
        dest: zulon_lir::VReg,
        op: &zulon_lir::LirUnaryOp,
        operand: zulon_lir::VReg,
        ty: &zulon_lir::LirTy,
    ) -> Result<()> {
        let llvm_ty: LlvmType = ty.clone().into();

        match op {
            zulon_lir::LirUnaryOp::Neg => {
                if ty.is_float() {
                    writeln!(
                        self.writer,
                        "{}  %v{} = fsub {} 0.0, %v{}",
                        "  ".repeat(self.indent),
                        dest,
                        llvm_ty.to_llvm_ir(),
                        operand
                    ).unwrap();
                } else {
                    writeln!(
                        self.writer,
                        "{}  %v{} = sub {} 0, %v{}",
                        "  ".repeat(self.indent),
                        dest,
                        llvm_ty.to_llvm_ir(),
                        operand
                    ).unwrap();
                }
            }

            zulon_lir::LirUnaryOp::Not => {
                writeln!(
                    self.writer,
                    "{}  %v{} = xor {} %v{}, -1",
                    "  ".repeat(self.indent),
                    dest,
                    llvm_ty.to_llvm_ir(),
                    operand
                ).unwrap();
            }
        }

        Ok(())
    }

    /// Generate load instruction
    fn generate_load(
        &mut self,
        dest: zulon_lir::VReg,
        src: &LirOperand,
        ty: &zulon_lir::LirTy,
    ) -> Result<()> {
        let llvm_ty: LlvmType = ty.clone().into();
        let src_str = self.operand_to_llvm(src)?;

        // Use struct reference if this is a declared struct type
        let type_str = if matches!(ty, zulon_lir::LirTy::Struct { .. }) {
            llvm_ty.to_llvm_ref()
        } else {
            llvm_ty.to_llvm_ir()
        };

        writeln!(
            self.writer,
            "{}  %v{} = load {}, {}* {}",
            "  ".repeat(self.indent),
            dest,
            type_str,
            type_str,
            src_str
        ).unwrap();

        Ok(())
    }

    /// Generate store instruction
    fn generate_store(
        &mut self,
        dest: &LirOperand,
        src: zulon_lir::VReg,
        ty: &zulon_lir::LirTy,
    ) -> Result<()> {
        let llvm_ty: LlvmType = ty.clone().into();
        let dest_str = self.operand_to_llvm(dest)?;

        // Use struct reference if this is a declared struct type
        let type_str = if matches!(ty, zulon_lir::LirTy::Struct { .. }) {
            llvm_ty.to_llvm_ref()
        } else {
            llvm_ty.to_llvm_ir()
        };

        writeln!(
            self.writer,
            "{}  store {} %v{}, {}* {}",
            "  ".repeat(self.indent),
            type_str,
            src,
            type_str,
            dest_str
        ).unwrap();

        Ok(())
    }

    /// Generate GEP instruction
    fn generate_gep(
        &mut self,
        dest: zulon_lir::VReg,
        base: zulon_lir::VReg,
        indices: &[LirOperand],
        ty: &zulon_lir::LirTy,
    ) -> Result<()> {
        let indices_str: Vec<String> = indices
            .iter()
            .map(|op| self.operand_to_llvm(op))
            .collect::<Result<Vec<_>>>()?;

        // Convert LIR type to LLVM type string
        use crate::ty::LlvmType;
        let llvm_type = LlvmType::from(ty.clone());
        let type_str = llvm_type.to_llvm_ir();

        // Use ptr for the base pointer type (modern LLVM style)
        let base_type = match ty {
            zulon_lir::LirTy::Struct { .. } => "ptr".to_string(),
            _ => format!("{}*", type_str),
        };

        writeln!(
            self.writer,
            "{}  %v{} = getelementptr {}, {} %v{}, {}",
            "  ".repeat(self.indent),
            dest,
            type_str,
            base_type,
            base,
            indices_str.join(", ")
        ).unwrap();

        Ok(())
    }

    /// Generate call instruction
    fn generate_call(
        &mut self,
        dest: Option<zulon_lir::VReg>,
        func_name: &str,
        args: &[zulon_lir::VReg],
        return_ty: &zulon_lir::LirTy,
        arg_types: &[zulon_lir::LirTy],
    ) -> Result<()> {
        let return_llvm_ty: LlvmType = return_ty.clone().into();

        // Format arguments with types
        let args_str: Vec<String> = args.iter().enumerate().map(|(i, &arg_reg)| {
            if i < arg_types.len() {
                let arg_ty: LlvmType = arg_types[i].clone().into();
                format!("{} %v{}", arg_ty.to_llvm_ir(), arg_reg)
            } else {
                // Default to i32 if no type info
                format!("i32 %v{}", arg_reg)
            }
        }).collect();

        if let Some(dest_vreg) = dest {
            writeln!(
                self.writer,
                "{}  %v{} = call {} @{}({})",
                "  ".repeat(self.indent),
                dest_vreg,
                return_llvm_ty.to_llvm_ir(),
                func_name,
                args_str.join(", ")
            ).unwrap();
        } else {
            writeln!(
                self.writer,
                "{}  call {} @{}({})",
                "  ".repeat(self.indent),
                return_llvm_ty.to_llvm_ir(),
                func_name,
                args_str.join(", ")
            ).unwrap();
        }

        Ok(())
    }

    /// Generate comparison instruction
    fn generate_cmp(
        &mut self,
        dest: zulon_lir::VReg,
        op: &zulon_lir::LirCmpOp,
        left: zulon_lir::VReg,
        right: zulon_lir::VReg,
        ty: &zulon_lir::LirTy,
    ) -> Result<()> {
        let llvm_ty: LlvmType = ty.clone().into();
        let (pred, is_float) = self.cmp_op_to_llvm(op, ty.is_float())?;

        if is_float {
            writeln!(
                self.writer,
                "{}  %v{} = fcmp {} {} %v{}, %v{}",
                "  ".repeat(self.indent),
                dest,
                pred,
                llvm_ty.to_llvm_ir(),
                left,
                right
            ).unwrap();
        } else {
            writeln!(
                self.writer,
                "{}  %v{} = icmp {} {} %v{}, %v{}",
                "  ".repeat(self.indent),
                dest,
                pred,
                llvm_ty.to_llvm_ir(),
                left,
                right
            ).unwrap();
        }

        Ok(())
    }

    /// Generate cast instruction
    fn generate_cast(
        &mut self,
        dest: zulon_lir::VReg,
        src: zulon_lir::VReg,
        from: &zulon_lir::LirTy,
        to: &zulon_lir::LirTy,
    ) -> Result<()> {
        let from_llvm: LlvmType = from.clone().into();
        let to_llvm: LlvmType = to.clone().into();

        // Simple cast logic (placeholder)
        let op = "bitcast";

        writeln!(
            self.writer,
            "{}  %v{} = {} {} %v{} to {}",
            "  ".repeat(self.indent),
            dest,
            op,
            from_llvm.to_llvm_ir(),
            src,
            to_llvm.to_llvm_ir()
        ).unwrap();

        Ok(())
    }

    /// Generate reference count increment
    /// Calls runtime function: void zulon_ref_inc(void* ptr)
    fn generate_ref_inc(&mut self, ptr: zulon_lir::VReg, _ty: &zulon_lir::LirTy) -> Result<()> {
        writeln!(
            self.writer,
            "{}  call void @zulon_ref_inc(i8* %v{})",
            "  ".repeat(self.indent),
            ptr
        ).unwrap();
        Ok(())
    }

    /// Generate reference count decrement
    /// Calls runtime function: void zulon_ref_dec(void* ptr)
    fn generate_ref_dec(&mut self, ptr: zulon_lir::VReg, _ty: &zulon_lir::LirTy) -> Result<()> {
        writeln!(
            self.writer,
            "{}  call void @zulon_ref_dec(i8* %v{})",
            "  ".repeat(self.indent),
            ptr
        ).unwrap();
        Ok(())
    }

    /// Convert operand to LLVM IR string
    fn operand_to_llvm(&self, operand: &LirOperand) -> Result<String> {
        match operand {
            LirOperand::Reg(vreg) => Ok(format!("%v{}", vreg)),
            LirOperand::Imm(val) => Ok(val.to_string()),
            LirOperand::ImmFloat(val) => Ok(val.to_string()),
        }
    }

    /// Generate terminator
    fn generate_terminator(&mut self, func: &LirFunction, terminator: &LirTerminator) -> Result<()> {
        match terminator {
            LirTerminator::Return(value) => {
                let ret_ty: LlvmType = func.return_type.clone().into();

                // Check if function returns Outcome type (error handling)
                let is_outcome = match &func.return_type {
                    zulon_lir::LirTy::Struct { name, .. } => name == "Outcome",
                    _ => false,
                };

                if let Some(vreg) = value {
                    // Return with value
                    // If this is an error-returning function and the value isn't already wrapped,
                    // we need to construct Outcome::Err
                    if is_outcome && !self.is_outcome_value(*vreg) {
                        // Construct Outcome::Err(error_value)
                        self.generate_error_return(*vreg, &ret_ty)?;
                    } else {
                        // Normal return (value is already Outcome or not an error function)
                        writeln!(
                            self.writer,
                            "{}  ret {} %v{}",
                            "  ".repeat(self.indent),
                            ret_ty.to_llvm_ir(),
                            vreg
                        ).unwrap();
                    }
                } else {
                    // Return without value
                    // For void functions, use ret void
                    // For non-void functions, return a default value (0 or undef)
                    if matches!(ret_ty, LlvmType::Void) {
                        writeln!(self.writer, "{}  ret void", "  ".repeat(self.indent)).unwrap();
                    } else {
                        // Return zero for the appropriate type
                        writeln!(
                            self.writer,
                            "{}  ret {} 0",
                            "  ".repeat(self.indent),
                            ret_ty.to_llvm_ir()
                        ).unwrap();
                    }
                }
            }

            LirTerminator::Jump { target } => {
                writeln!(
                    self.writer,
                    "{}  br label %block{}",
                    "  ".repeat(self.indent),
                    target
                ).unwrap();
            }

            LirTerminator::Branch { condition, then_block, else_block } => {
                writeln!(
                    self.writer,
                    "{}  br i1 %v{}, label %block{}, label %block{}",
                    "  ".repeat(self.indent),
                    condition,
                    then_block,
                    else_block
                ).unwrap();
            }

            LirTerminator::Switch { scrutinee, targets, default } => {
                writeln!(
                    self.writer,
                    "{}  switch i32 %v{}, label %block{} [",
                    "  ".repeat(self.indent),
                    scrutinee,
                    default
                ).unwrap();

                self.indent += 1;
                for (val, block) in targets {
                    writeln!(
                        self.writer,
                        "{}    i32 {}, label %block{}",
                        "  ".repeat(self.indent),
                        val,
                        block
                    ).unwrap();
                }
                self.indent -= 1;

                writeln!(self.writer, "{}  ]", "  ".repeat(self.indent)).unwrap();
            }

            LirTerminator::Unreachable => {
                writeln!(self.writer, "{}  unreachable", "  ".repeat(self.indent)).unwrap();
            }
        }

        Ok(())
    }

    /// Convert binary operation to LLVM IR
    fn binary_op_to_llvm(&self, op: &zulon_lir::LirBinOp, is_float: bool) -> Result<&'static str> {
        match (op, is_float) {
            // Integer operations
            (zulon_lir::LirBinOp::Add, false) => Ok("add"),
            (zulon_lir::LirBinOp::Sub, false) => Ok("sub"),
            (zulon_lir::LirBinOp::Mul, false) => Ok("mul"),
            (zulon_lir::LirBinOp::Div, false) => Ok("sdiv"),
            (zulon_lir::LirBinOp::Mod, false) => Ok("srem"),
            (zulon_lir::LirBinOp::BitAnd, false) => Ok("and"),
            (zulon_lir::LirBinOp::BitOr, false) => Ok("or"),
            (zulon_lir::LirBinOp::BitXor, false) => Ok("xor"),
            (zulon_lir::LirBinOp::LeftShift, false) => Ok("shl"),
            (zulon_lir::LirBinOp::RightShift, false) => Ok("ashr"),

            // Float operations
            (zulon_lir::LirBinOp::Add, true) => Ok("add"),
            (zulon_lir::LirBinOp::Sub, true) => Ok("sub"),
            (zulon_lir::LirBinOp::Mul, true) => Ok("mul"),
            (zulon_lir::LirBinOp::Div, true) => Ok("fdiv"),
            (zulon_lir::LirBinOp::Mod, true) => Ok("frem"),

            _ => Err(CodegenError::Unsupported(format!("{:?}", op))),
        }
    }

    /// Convert comparison operation to LLVM IR
    fn cmp_op_to_llvm(&self, op: &zulon_lir::LirCmpOp, is_float: bool) -> Result<(&'static str, bool)> {
        match (op, is_float) {
            // Integer comparisons
            (zulon_lir::LirCmpOp::Eq, false) => Ok(("eq", false)),
            (zulon_lir::LirCmpOp::NotEq, false) => Ok(("ne", false)),
            (zulon_lir::LirCmpOp::Less, false) => Ok(("slt", false)),
            (zulon_lir::LirCmpOp::LessEq, false) => Ok(("sle", false)),
            (zulon_lir::LirCmpOp::Greater, false) => Ok(("sgt", false)),
            (zulon_lir::LirCmpOp::GreaterEq, false) => Ok(("sge", false)),

            // Float comparisons
            (zulon_lir::LirCmpOp::Eq, true) => Ok(("oeq", true)),
            (zulon_lir::LirCmpOp::NotEq, true) => Ok(("une", true)),
            (zulon_lir::LirCmpOp::Less, true) => Ok(("olt", true)),
            (zulon_lir::LirCmpOp::LessEq, true) => Ok(("ole", true)),
            (zulon_lir::LirCmpOp::Greater, true) => Ok(("ogt", true)),
            (zulon_lir::LirCmpOp::GreaterEq, true) => Ok(("oge", true)),
        }
    }

    /// Generate module-level type declarations
    pub fn generate_type_declarations(&mut self) -> Result<()> {
        // Declare struct types
        for (_name, layout) in &self.struct_types {
            writeln!(self.writer, "{}", layout.to_llvm_definition())
                .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
        }

        // Declare enum types
        for (_name, layout) in &self.enum_types {
            writeln!(self.writer, "{}", layout.to_llvm_definition())
                .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
        }

        writeln!(self.writer)
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        Ok(())
    }

    /// Generate a complete LLVM IR module
    pub fn generate_module(&mut self, functions: &[LirFunction]) -> Result<()> {
        // Module header
        writeln!(self.writer, "; Generated by ZULON compiler")
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
        writeln!(self.writer)
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        // Type declarations
        self.generate_type_declarations()?;

        // Generate each function
        for func in functions {
            self.generate_function(func)?;
        }

        Ok(())
    }

    /// Generate a complete LLVM IR module with external declarations
    pub fn generate_module_with_externals(
        &mut self,
        functions: &[LirFunction],
        externals: &[zulon_lir::LirExternal],
    ) -> Result<()> {
        // Module header
        writeln!(self.writer, "; Generated by ZULON compiler")
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
        writeln!(self.writer)
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        // Type declarations
        self.generate_type_declarations()?;

        // External function declarations
        for external in externals {
            self.generate_external_decl(external)?;
        }

        writeln!(self.writer)
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        // First pass: collect string constants from all functions
        for func in functions {
            self.collect_string_constants(func);
        }

        // Emit string constants at module level
        if !self.string_constants.is_empty() {
            for sc in &self.string_constants {
                writeln!(
                    self.writer,
                    "@{} = private unnamed_addr constant [{} x i8] c{}",
                    sc.name,
                    sc.len,
                    Self::escape_string_for_llvm(&sc.value)
                )
                .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
            }
            writeln!(self.writer)
                .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
        }

        // Second pass: generate each function
        for func in functions {
            self.generate_function(func)?;
        }

        Ok(())
    }

    /// Generate external function declaration
    fn generate_external_decl(&mut self, external: &zulon_lir::LirExternal) -> Result<()> {
        let return_llvm_ty: LlvmType = external.return_type.clone().into();

        write!(self.writer, "declare {} @{}(",
            return_llvm_ty.to_llvm_ir(),
            external.name
        ).map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        // Parameter types
        for (i, param_ty) in external.param_types.iter().enumerate() {
            if i > 0 {
                write!(self.writer, ", ").map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
            }
            let llvm_ty: LlvmType = param_ty.clone().into();
            write!(self.writer, "{}", llvm_ty.to_llvm_ir())
                .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;
        }

        writeln!(self.writer, ")")
            .map_err(|e| CodegenError::InstructionError(format!("IO error: {}", e)))?;

        Ok(())
    }

    /// Generate function with ABI-aware prologue/epilogue
    pub fn generate_function_with_abi(&mut self, func: &LirFunction) -> Result<()> {
        // Calculate call info
        let mut call_info = CallInfo::new(self.calling_convention);

        // Allocate return value
        call_info.allocate_arg(&func.return_type, true)?;

        // Allocate arguments
        for param_ty in &func.param_types {
            call_info.allocate_arg(param_ty, false)?;
        }

        // Finalize stack (calculate locals size)
        let local_size = self.calculate_locals_size(func)?;
        call_info.finalize_stack(local_size);

        // Function header
        self.write_function_header(func)?;
        writeln!(self.writer, " {{").unwrap();

        // Prologue
        let prologue = call_info.get_prologue();
        if !prologue.is_empty() {
            writeln!(self.writer, "entry:").unwrap();
            for line in prologue.lines() {
                writeln!(self.writer, "  {}", line).unwrap();
            }
            writeln!(self.writer, "  br label %block{}", func.entry_block).unwrap();
        }

        // Generate blocks
        self.indent += 1;
        let mut block_ids: Vec<_> = func.blocks.keys().copied().collect();
        block_ids.sort();

        for block_id in block_ids {
            if let Some(block) = func.blocks.get(&block_id) {
                // Skip entry block if we generated prologue
                if block_id == func.entry_block && !prologue.is_empty() {
                    continue;
                }
                self.generate_block(func, block)?;
            }
        }

        self.indent -= 1;
        writeln!(self.writer, "}}").unwrap();
        writeln!(self.writer).unwrap();

        Ok(())
    }

    /// Calculate total size of local variables
    fn calculate_locals_size(&self, func: &LirFunction) -> Result<i64> {
        let mut total_size = 0i64;

        // Scan all instructions to find allocations
        for block in func.blocks.values() {
            for instr in &block.instructions {
                if let LirInstruction::Load { dest: _, src: _, ty } = instr {
                    // Simplified: assume each local needs stack space
                    // Real implementation would do liveness analysis
                    total_size += ty.size() as i64;
                }
            }
        }

        Ok(total_size)
    }

    /// Check if a value is already an Outcome (vs raw error value)
    fn is_outcome_value(&self, _vreg: zulon_lir::VReg) -> bool {
        // TODO: Proper type tracking for vregs
        // For now, assume all values are NOT Outcome (raw values)
        // This means we'll always wrap in Outcome::Err for error functions
        false
    }

    /// Generate Outcome::Err construction for throw statements
    ///
    /// Constructs an Outcome::Err(error_value) by:
    /// 1. Allocating stack space for Outcome
    /// 2. Setting discriminant to 1 (Err variant)
    /// 3. Storing error value in data field
    /// 4. Loading and returning the Outcome
    ///
    /// Outcome layout: { i32 discriminant, <error_type> data }
    /// - discriminant = 0 for Ok, 1 for Err
    /// - data field contains the error value
    fn generate_error_return(
        &mut self,
        error_vreg: zulon_lir::VReg,
        ret_ty: &LlvmType,
    ) -> Result<()> {
        // We need to generate new temporary registers
        // Start from a high number to avoid conflicts with LIR vregs
        let mut temp_reg = 1000usize;

        // Step 1: Allocate stack space for Outcome
        // Outcome is represented as: { i32, <error_type> }
        let outcome_reg = temp_reg;
        temp_reg = temp_reg.wrapping_add(1);
        writeln!(
            self.writer,
            "{}  %v{} = alloca {}",
            "  ".repeat(self.indent),
            outcome_reg,
            ret_ty.to_llvm_ir()
        ).unwrap();

        // Step 2: Get pointer to discriminant field (field 0)
        let disc_ptr_reg = temp_reg;
        temp_reg = temp_reg.wrapping_add(1);
        writeln!(
            self.writer,
            "{}  %v{} = getelementptr {}, ptr %v{}, i32 0, i32 0",
            "  ".repeat(self.indent),
            disc_ptr_reg,
            ret_ty.to_llvm_ir(),
            outcome_reg
        ).unwrap();

        // Step 3: Store discriminant value = 1 (Err variant)
        writeln!(
            self.writer,
            "{}  store i32 1, ptr %v{}",
            "  ".repeat(self.indent),
            disc_ptr_reg
        ).unwrap();

        // Step 4: Get pointer to data field (field 1)
        let data_ptr_reg = temp_reg;
        temp_reg = temp_reg.wrapping_add(1);
        writeln!(
            self.writer,
            "{}  %v{} = getelementptr {}, ptr %v{}, i32 0, i32 1",
            "  ".repeat(self.indent),
            data_ptr_reg,
            ret_ty.to_llvm_ir(),
            outcome_reg
        ).unwrap();

        // Step 5: Store error value in data field
        // Need to load the error value first
        let error_loaded_reg = temp_reg;
        let _ = temp_reg.wrapping_add(1); // Increment but don't need the value
        writeln!(
            self.writer,
            "{}  %v{} = load i32, ptr %v{}",
            "  ".repeat(self.indent),
            error_loaded_reg,
            error_vreg
        ).unwrap();

        // Store the error value
        writeln!(
            self.writer,
            "{}  store i32 %v{}, ptr %v{}",
            "  ".repeat(self.indent),
            error_loaded_reg,
            data_ptr_reg
        ).unwrap();

        // Step 6: Load the entire Outcome and return it
        let outcome_loaded_reg = temp_reg;
        writeln!(
            self.writer,
            "{}  %v{} = load {}, ptr %v{}",
            "  ".repeat(self.indent),
            outcome_loaded_reg,
            ret_ty.to_llvm_ir(),
            outcome_reg
        ).unwrap();

        // Return the constructed Outcome::Err
        writeln!(
            self.writer,
            "{}  ret {} %v{}",
            "  ".repeat(self.indent),
            ret_ty.to_llvm_ir(),
            outcome_loaded_reg
        ).unwrap();

        Ok(())
    }
}
