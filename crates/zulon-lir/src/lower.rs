// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! MIR to LIR lowering
//!
//! This transforms MIR into SSA-based LIR.

use crate::error::Result;
use crate::lir::*;
use crate::ty::LirTy;
use zulon_mir::{MirBody, MirFunction, MirInstruction, MirNodeId, MirPlace, MirTerminator};
use std::collections::{HashMap, HashSet};

/// Context for lowering MIR to LIR
pub struct LirLoweringContext {
    /// MIR temp to LIR vreg mapping
    temp_map: HashMap<zulon_mir::TempVar, VReg>,
    /// Parameter name to LIR vreg mapping
    param_map: HashMap<String, VReg>,
    /// Local variable name to LIR vreg mapping (for let bindings)
    local_map: HashMap<String, VReg>,
    /// Block ID to return value vreg mapping (for Phi node construction)
    block_returns: HashMap<MirNodeId, VReg>,
    /// Block predecessors tracking (for Phi node detection)
    block_preds: HashMap<MirNodeId, Vec<MirNodeId>>,
    /// Pending Phi nodes to add to blocks (block_id -> (vreg, phi))
    pending_phis: HashMap<MirNodeId, Vec<(VReg, LirPhi)>>,
    /// Mutable local variables (need memory operations instead of SSA)
    mutable_locals: HashSet<String>,
    /// Stack slots for mutable locals (local_name -> vreg for alloca)
    local_stack_slots: HashMap<String, VReg>,
}

impl LirLoweringContext {
    /// Create a new lowering context
    pub fn new() -> Self {
        LirLoweringContext {
            temp_map: HashMap::new(),
            param_map: HashMap::new(),
            local_map: HashMap::new(),
            block_returns: HashMap::new(),
            block_preds: HashMap::new(),
            pending_phis: HashMap::new(),
            mutable_locals: HashSet::new(),
            local_stack_slots: HashMap::new(),
        }
    }

    /// Lower an entire MIR body to LIR
    pub fn lower_body(&mut self, mir_body: &MirBody) -> Result<LirBody> {
        let mut lir_body = LirBody::new();

        for func in &mir_body.functions {
            let lir_func = self.lower_function(func)?;
            lir_body.push_function(lir_func);
        }

        Ok(lir_body)
    }

    /// Lower a MIR function to LIR function
    pub fn lower_function(&mut self, func: &MirFunction) -> Result<LirFunction> {
        // Clear previous function's data
        self.block_returns.clear();
        self.block_preds.clear();
        self.pending_phis.clear();
        self.mutable_locals.clear();
        self.local_stack_slots.clear();
        self.local_map.clear();

        // Detect mutable local variables (those that appear in Store instructions)
        self.detect_mutable_locals(func)?;

        // Build predecessor map and collect block return values
        self.analyze_control_flow(func)?;

        // Create LIR function with parameters
        let mut params = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            let vreg = i as VReg; // Parameters start from v0
            params.push((vreg, param.ty.clone().into()));
            // Map parameter name to vreg
            self.param_map.insert(param.name.clone(), vreg);
        }

        let mut lir_func = LirFunction::new(
            func.name.clone(),
            params,
            func.return_type.clone().into(),
        );

        // Allocate stack slots for mutable locals
        // We'll emit these as allocas at the start of the entry block (block 0)
        if !self.mutable_locals.is_empty() {
            // Find the types for each mutable local by scanning for their first store
            for (_block_id, block) in &func.blocks {
                for inst in &block.instructions {
                    if let MirInstruction::Store { dest, ty: _, .. } = inst {
                        if let MirPlace::Local(name) = dest {
                            if self.mutable_locals.contains(name) && !self.local_stack_slots.contains_key(name) {
                                // Allocate a stack slot vreg for this mutable local
                                let stack_slot = lir_func.alloc_vreg();
                                self.local_stack_slots.insert(name.clone(), stack_slot);
                            }
                        }
                    }
                }
            }
        }

        // Lower all basic blocks in sorted order to ensure preds are processed first
        let mut block_ids: Vec<_> = func.blocks.keys().copied().collect();
        block_ids.sort();

        for mir_block_id in block_ids {
            let mir_block = &func.blocks[&mir_block_id];
            let lir_block_id = mir_block_id;
            let mut lir_block = LirBlock::new(lir_block_id);

            // Lower instructions
            for inst in &mir_block.instructions {
                let lir_insts = self.lower_instruction(inst, &mut lir_func, mir_block_id, func)?;
                lir_block.instructions.extend(lir_insts);
            }

            // Lower terminator
            if let Some(terminator) = &mir_block.terminator {
                lir_block.terminator = Some(self.lower_terminator(terminator, &lir_func)?);
            }

            lir_func.blocks.insert(lir_block_id, lir_block);
        }

        // Add pending Phi nodes to their respective blocks
        for (block_id, phis) in self.pending_phis.drain() {
            if let Some(lir_block) = lir_func.blocks.get_mut(&block_id) {
                for (vreg, phi) in phis {
                    lir_block.add_phi(vreg, phi);
                }
            }
        }

        // Emit alloca instructions for mutable locals at the start of entry block (block 0)
        let entry_block_id = 0;
        for (name, stack_slot) in &self.local_stack_slots {
            // Find the type of this local by scanning for its first store
            let mut ty = None;
            for (_block_id, block) in &func.blocks {
                for inst in &block.instructions {
                    if let MirInstruction::Store { dest, ty: inst_ty, .. } = inst {
                        if let MirPlace::Local(local_name) = dest {
                            if local_name == name {
                                ty = Some(inst_ty.clone());
                                break;
                            }
                        }
                    }
                    if ty.is_some() {
                        break;
                    }
                }
                if ty.is_some() {
                    break;
                }
            }

            let ty = ty.expect("Mutable local should have a type from Store instruction");
            let alloca_inst = LirInstruction::Alloca(crate::lir::LirAlloca {
                dest: *stack_slot,
                ty: ty.clone().into(),
            });

            // Insert alloca at the beginning of the entry block
            if let Some(entry_block) = lir_func.blocks.get_mut(&entry_block_id) {
                entry_block.instructions.insert(0, alloca_inst);
            }
        }

        // CFG Completion Pass: Ensure all blocks have terminators
        // Some blocks (like loop exits) may be created without terminators in MIR
        self.complete_cfg(&mut lir_func)?;

        Ok(lir_func)
    }

    /// Analyze control flow to build predecessor map and collect block return values
    /// Detect mutable local variables by scanning for Store instructions
    fn detect_mutable_locals(&mut self, func: &MirFunction) -> Result<()> {
        for (_block_id, block) in &func.blocks {
            for inst in &block.instructions {
                if let MirInstruction::Store { dest, .. } = inst {
                    // If we're storing to a Local, it's mutable
                    if let MirPlace::Local(name) = dest {
                        self.mutable_locals.insert(name.clone());
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_control_flow(&mut self, func: &MirFunction) -> Result<()> {
        // First pass: build predecessor map
        for (block_id, block) in &func.blocks {
            if let Some(terminator) = &block.terminator {
                let targets = self.get_terminator_targets(terminator);
                for target in targets {
                    self.block_preds
                        .entry(target)
                        .or_insert_with(Vec::new)
                        .push(*block_id);
                }
            }
        }

        // Second pass: collect block return values (last temp before terminator)
        for (block_id, block) in &func.blocks {
            // Find the last instruction that produces a value
            if let Some(last_inst) = block.instructions.last() {
                let return_temp = match last_inst {
                    MirInstruction::Call { dest: Some(d), .. } => Some(*d),
                    MirInstruction::Load { dest, .. } => Some(*dest),
                    MirInstruction::BinaryOp { dest, .. } => Some(*dest),
                    MirInstruction::Const { dest, .. } => Some(*dest),
                    _ => None,
                };

                if let Some(temp) = return_temp {
                    // We'll map this to vreg later during instruction lowering
                    // For now, just mark that this block has a return value
                    self.block_returns.insert(*block_id, temp as VReg);
                }
            }
        }

        Ok(())
    }

    /// Get the target blocks of a terminator
    fn get_terminator_targets(&self, terminator: &MirTerminator) -> Vec<MirNodeId> {
        match terminator {
            MirTerminator::Goto { target } => vec![*target],
            MirTerminator::If { then_block, else_block, .. } => vec![*then_block, *else_block],
            MirTerminator::Return { .. } => vec![],
            MirTerminator::Unreachable => vec![],
            _ => vec![],
        }
    }

    /// Check if a block is a join block (has multiple predecessors)
    fn is_join_block(&self, block_id: MirNodeId) -> bool {
        self.block_preds
            .get(&block_id)
            .map(|preds| preds.len() > 1)
            .unwrap_or(false)
    }

    /// Complete CFG by adding terminators to blocks that are missing them
    /// This handles cases where MIR creates blocks (like loop exits) without terminators
    fn complete_cfg(&mut self, func: &mut LirFunction) -> Result<()> {
        let mut blocks_to_fix: Vec<MirNodeId> = Vec::new();

        // Find all blocks without terminators
        for (block_id, block) in &func.blocks {
            if block.terminator.is_none() {
                blocks_to_fix.push(*block_id);
            }
        }

        // Fix each unterminated block
        for block_id in blocks_to_fix {
            if let Some(block) = func.blocks.get_mut(&block_id) {
                // Check if this block has phi nodes
                let has_phi = !block.phi_nodes.is_empty();

                if has_phi {
                    // Block with phi but no terminator is likely a loop exit
                    // Return the last phi value (or first if multiple)
                    let phi_vregs: Vec<VReg> = block.phi_nodes.keys().copied().collect();

                    if !phi_vregs.is_empty() {
                        // Use the first phi value as the return value
                        let return_vreg = phi_vregs[0];

                        block.terminator = Some(LirTerminator::Return(Some(return_vreg)));
                    } else {
                        // No phi values, return default (0 or undef)
                        block.terminator = Some(LirTerminator::Return(None));
                    }
                } else {
                    // Block without phi and without terminator
                    // This is likely dead code or unreachable
                    // Add unreachable as a safe default
                    block.terminator = Some(LirTerminator::Unreachable);
                }
            }
        }

        Ok(())
    }

    /// Lower a MIR instruction to LIR instruction(s)
    fn lower_instruction(
        &mut self,
        inst: &MirInstruction,
        func: &mut LirFunction,
        current_block: MirNodeId,
        _mir_func: &MirFunction,
    ) -> Result<Vec<LirInstruction>> {
        match inst {
            MirInstruction::Const { dest, value, ty } => {
                let vreg = func.alloc_vreg();
                self.temp_map.insert(*dest, vreg);

                let lir_value = match value {
                    zulon_mir::MirConstant::Bool(b) => LirConstant::Bool(*b),
                    zulon_mir::MirConstant::Integer(i) => LirConstant::Integer(*i as u64),
                    zulon_mir::MirConstant::Float(f) => LirConstant::Float(*f),
                    zulon_mir::MirConstant::String(s) => LirConstant::String(s.clone()),
                    zulon_mir::MirConstant::Char(c) => LirConstant::Integer(*c as u64),
                    zulon_mir::MirConstant::Unit => LirConstant::Unit,
                };

                Ok(vec![LirInstruction::Const {
                    dest: vreg,
                    value: lir_value,
                    ty: ty.clone().into(),
                }])
            }

            MirInstruction::BinaryOp { dest, op, left, right, ty } => {
                let dest_vreg = func.alloc_vreg();
                let left_vreg = self.temp_map.get(left).copied().unwrap_or_else(|| *left as VReg);
                let right_vreg = self.temp_map.get(right).copied().unwrap_or_else(|| *right as VReg);

                self.temp_map.insert(*dest, dest_vreg);

                // Check if this is a comparison operation
                let is_comparison = matches!(
                    *op,
                    zulon_mir::MirBinOp::Eq | zulon_mir::MirBinOp::NotEq |
                    zulon_mir::MirBinOp::Less | zulon_mir::MirBinOp::LessEq |
                    zulon_mir::MirBinOp::Greater | zulon_mir::MirBinOp::GreaterEq
                );

                if is_comparison {
                    // Generate comparison instruction
                    let lir_cmp_op = self.lower_cmp_op(*op);
                    Ok(vec![LirInstruction::Cmp {
                        dest: dest_vreg,
                        op: lir_cmp_op,
                        left: left_vreg,
                        right: right_vreg,
                    }])
                } else {
                    // Regular binary operation (arithmetic, bitwise, or logical)
                    let lir_op = self.lower_bin_op(*op);
                    Ok(vec![LirInstruction::BinaryOp {
                        dest: dest_vreg,
                        op: lir_op,
                        left: left_vreg,
                        right: right_vreg,
                        ty: ty.clone().into(),
                    }])
                }
            }

            MirInstruction::UnaryOp { dest, op, operand, ty } => {
                let dest_vreg = func.alloc_vreg();
                let operand_vreg = self.temp_map.get(operand).copied().unwrap_or_else(|| *operand as VReg);

                self.temp_map.insert(*dest, dest_vreg);

                // Convert MIR unary op to LIR unary op
                let lir_op = match op {
                    zulon_mir::MirUnaryOp::Neg => LirUnaryOp::Neg,
                    zulon_mir::MirUnaryOp::Not => LirUnaryOp::Not,
                    _ => LirUnaryOp::Neg, // Default to Neg for other ops (Ref, Deref, etc.)
                };

                Ok(vec![LirInstruction::UnaryOp {
                    dest: dest_vreg,
                    op: lir_op,
                    operand: operand_vreg,
                    ty: ty.clone().into(),
                }])
            }

            MirInstruction::Copy { dest, src } => {
                let dest_vreg = func.alloc_vreg();
                let src_vreg = self.get_or_alloc_vreg(src, func);

                self.temp_map.insert(*dest, dest_vreg);

                // Infer type from context (placeholder)
                let ty = LirTy::I32;

                Ok(vec![LirInstruction::Copy {
                    dest: dest_vreg,
                    src: src_vreg,
                    ty,
                }])
            }

            MirInstruction::Move { dest, src } => {
                // Move in SSA form - check if this represents a Phi node
                let dest_vreg = func.alloc_vreg();
                self.temp_map.insert(*dest, dest_vreg);

                // Check if this block is a join point (multiple predecessors)
                // and we need to generate a Phi node
                if self.is_join_block(current_block) {
                    // Generate Phi node - store in pending_phis
                    let mut phi_sources = Vec::new();

                    if let Some(preds) = self.block_preds.get(&current_block) {
                        for &pred_block_id in preds {
                            // Get the return value from this predecessor
                            if let Some(&return_temp) = self.block_returns.get(&pred_block_id) {
                                // Map MIR temp to LIR vreg - cast to usize
                                let src_vreg = self.temp_map.get(&(return_temp as zulon_mir::TempVar))
                                    .copied()
                                    .unwrap_or(return_temp);

                                phi_sources.push((src_vreg, pred_block_id));
                            } else {
                                // This predecessor doesn't produce a value
                                // Use undef (represented by vreg 0, which is always undef)
                                // In practice, this means the predecessor takes a different path
                                phi_sources.push((0, pred_block_id)); // 0 = undef
                            }
                        }
                    }

                    // Create Phi node and store for later
                    let phi = LirPhi {
                        def: dest_vreg,
                        sources: phi_sources,
                        ty: LirTy::I32, // TODO: Infer proper type
                    };

                    self.pending_phis
                        .entry(current_block)
                        .or_insert_with(Vec::new)
                        .push((dest_vreg, phi));

                    // No instruction needed - Phi is separate
                    Ok(vec![])
                } else {
                    // Not a join block - treat as regular Copy
                    let src_vreg = if let MirPlace::Temp(src_temp) = src {
                        self.temp_map.get(src_temp).copied()
                            .unwrap_or_else(|| *src_temp as VReg)
                    } else {
                        self.get_or_alloc_vreg(src, func)
                    };

                    Ok(vec![LirInstruction::Copy {
                        dest: dest_vreg,
                        src: src_vreg,
                        ty: LirTy::I32, // Placeholder
                    }])
                }
            }

            MirInstruction::Call { dest, func: mir_func, args, return_type } => {
                let dest_vreg = if dest.is_some() {
                    Some(func.alloc_vreg())
                } else {
                    None
                };

                let arg_vregs: Vec<VReg> = args
                    .iter()
                    .map(|arg| self.get_or_alloc_vreg(arg, func))
                    .collect();

                if let Some(d) = dest {
                    self.temp_map.insert(*d, dest_vreg.unwrap());
                }

                // Extract function name
                let func_name = match mir_func {
                    zulon_mir::MirPlace::Local(name) => name.clone(),
                    _ => "unknown".to_string(),
                };

                // Get argument types from the places
                let arg_types: Vec<LirTy> = args
                    .iter()
                    .map(|arg| self.get_place_type(arg))
                    .collect();

                // Track external function
                if !func.external_funcs.contains(&func_name) {
                    func.external_funcs.push(func_name.clone());
                }

                Ok(vec![LirInstruction::CallExternal {
                    dest: dest_vreg,
                    func_name,
                    args: arg_vregs,
                    arg_types,
                    return_type: return_type.clone().into(),
                }])
            }

            MirInstruction::Load { dest, src, ty } => {
                // Check if this is a load from a field (struct/enum field access)
                if let MirPlace::Field { base, field } = src {
                    // Generate GEP for field access
                    let base_vreg = self.get_or_alloc_vreg(base, func);
                    let dest_vreg = func.alloc_vreg();
                    let gep_vreg = func.alloc_vreg();
                    self.temp_map.insert(*dest, dest_vreg);

                    // Calculate field index
                    // For Outcome<T, E>: discriminant=0, data=1
                    let field_index = match field.as_str() {
                        "discriminant" => 0u64,
                        "data" => 1u64,
                        _ => {
                            // Unknown field - use 0 as fallback
                            // TODO: Proper error handling
                            0
                        }
                    };

                    // Generate GEP + Load
                    Ok(vec![
                        LirInstruction::Gep {
                            dest: gep_vreg,
                            base: base_vreg,
                            indices: vec![
                                LirOperand::Imm(0),  // struct pointer
                                LirOperand::Imm(field_index),  // field index
                            ],
                            ty: ty.clone().into(),
                        },
                        LirInstruction::Load {
                            dest: dest_vreg,
                            src: LirOperand::Reg(gep_vreg),  // Load from GEP result
                            ty: ty.clone().into(),
                        },
                    ])
                } else if let MirPlace::Local(name) = src {
                    // Check if this is a load from a mutable local
                    if self.mutable_locals.contains(name) {
                        // Mutable local: generate actual Load from stack slot
                        let stack_slot = *self.local_stack_slots.get(name)
                            .expect("Mutable local should have stack slot");

                        self.temp_map.insert(*dest, func.alloc_vreg());

                        Ok(vec![LirInstruction::Load {
                            dest: self.temp_map[dest],
                            src: LirOperand::Reg(stack_slot),
                            ty: ty.clone().into(),
                        }])
                    } else {
                        // Immutable local: SSA rename (no instruction needed)
                        let src_vreg = self.get_or_alloc_vreg(src, func);
                        self.temp_map.insert(*dest, src_vreg);
                        Ok(vec![])
                    }
                } else {
                    // Load from non-local (Temp, Param): SSA rename
                    let src_vreg = self.get_or_alloc_vreg(src, func);
                    self.temp_map.insert(*dest, src_vreg);
                    Ok(vec![])
                }
            }

            MirInstruction::Store { dest, src, ty } => {
                let src_vreg = self.temp_map.get(src).copied().unwrap_or_else(|| *src as VReg);

                // Check if this is a store to a mutable local
                if let MirPlace::Local(name) = dest {
                    if self.mutable_locals.contains(name) {
                        // Mutable local: generate actual Store to stack slot
                        let stack_slot = *self.local_stack_slots.get(name)
                            .expect("Mutable local should have stack slot");

                        // Update local_map for potential subsequent SSA uses
                        self.local_map.insert(name.clone(), src_vreg);

                        Ok(vec![LirInstruction::Store {
                            dest: LirOperand::Reg(stack_slot),
                            src: src_vreg,
                            ty: ty.clone().into(),
                        }])
                    } else {
                        // Immutable local: SSA rename (no instruction needed)
                        self.local_map.insert(name.clone(), src_vreg);
                        Ok(vec![])
                    }
                } else {
                    // Store to non-local (Temp, Param): need actual copy
                    let dest_vreg = self.get_or_alloc_vreg(dest, func);
                    Ok(vec![LirInstruction::Copy {
                        dest: dest_vreg,
                        src: src_vreg,
                        ty: ty.clone().into(),
                    }])
                }
            }

            _ => {
                // Placeholder for other instructions
                Ok(vec![])
            }
        }
    }

    /// Lower a MIR terminator to LIR terminator
    fn lower_terminator(&self, terminator: &MirTerminator, _func: &LirFunction) -> Result<LirTerminator> {
        match terminator {
            MirTerminator::Return(place) => {
                let vreg = place.as_ref().and_then(|p| {
                    match p {
                        zulon_mir::MirPlace::Temp(t) => {
                            // Look up the temp in temp_map to get the actual vreg
                            self.temp_map.get(t).copied()
                        }
                        _ => None,
                    }
                });

                Ok(LirTerminator::Return(vreg))
            }

            MirTerminator::Goto { target } => {
                Ok(LirTerminator::Jump { target: *target })
            }

            MirTerminator::If { condition, then_block, else_block } => {
                let cond_vreg = self.temp_map.get(condition).copied().unwrap_or(*condition as VReg);

                Ok(LirTerminator::Branch {
                    condition: cond_vreg,
                    then_block: *then_block,
                    else_block: *else_block,
                })
            }

            _ => {
                // Placeholder for other terminators
                Ok(LirTerminator::Unreachable)
            }
        }
    }

    /// Get or allocate a virtual register for a place
    fn get_or_alloc_vreg(&mut self, place: &zulon_mir::MirPlace, func: &mut LirFunction) -> VReg {
        match place {
            zulon_mir::MirPlace::Temp(temp) => {
                if let Some(&vreg) = self.temp_map.get(temp) {
                    vreg
                } else {
                    let vreg = func.alloc_vreg();
                    self.temp_map.insert(*temp, vreg);
                    vreg
                }
            }
            zulon_mir::MirPlace::Param(name) => {
                // Parameters should already be mapped
                if let Some(&vreg) = self.param_map.get(name) {
                    vreg
                } else {
                    // Fallback: allocate a new vreg (shouldn't happen)
                    func.alloc_vreg()
                }
            }
            zulon_mir::MirPlace::Local(name) => {
                // Check if this local refers to a parameter
                if let Some(&vreg) = self.param_map.get(name) {
                    vreg
                } else if let Some(&vreg) = self.local_map.get(name) {
                    // Use the vreg from the Store that created this local
                    vreg
                } else {
                    // Allocate a new vreg for the local
                    let vreg = func.alloc_vreg();
                    self.local_map.insert(name.clone(), vreg);
                    vreg
                }
            }
            _ => func.alloc_vreg(), // Placeholder for other places
        }
    }

    /// Get the type of a MirPlace
    fn get_place_type(&self, place: &zulon_mir::MirPlace) -> LirTy {
        match place {
            zulon_mir::MirPlace::Temp(_) => {
                // Look up the type from temp_types map if available
                // For now, default to I32
                LirTy::I32
            }
            zulon_mir::MirPlace::Param(_name) => {
                // TODO: Look up parameter type from function signature
                // For now, default to I32
                LirTy::I32
            }
            zulon_mir::MirPlace::Local(_name) => {
                // TODO: Look up local variable type
                // For now, default to I32
                LirTy::I32
            }
            _ => LirTy::I32, // Placeholder for other places
        }
    }

    /// Lower a binary operator (arithmetic and bitwise only)
    fn lower_bin_op(&self, op: zulon_mir::MirBinOp) -> LirBinOp {
        match op {
            zulon_mir::MirBinOp::Add => LirBinOp::Add,
            zulon_mir::MirBinOp::Sub => LirBinOp::Sub,
            zulon_mir::MirBinOp::Mul => LirBinOp::Mul,
            zulon_mir::MirBinOp::Div => LirBinOp::Div,
            zulon_mir::MirBinOp::Mod => LirBinOp::Mod,
            zulon_mir::MirBinOp::BitAnd => LirBinOp::BitAnd,
            zulon_mir::MirBinOp::BitOr => LirBinOp::BitOr,
            zulon_mir::MirBinOp::BitXor => LirBinOp::BitXor,
            zulon_mir::MirBinOp::LeftShift => LirBinOp::LeftShift,
            zulon_mir::MirBinOp::RightShift => LirBinOp::RightShift,
            // Logical And/Or map to bitwise for primitive types
            zulon_mir::MirBinOp::And => LirBinOp::BitAnd,
            zulon_mir::MirBinOp::Or => LirBinOp::BitOr,
            // Comparison operators are handled separately
            _ => LirBinOp::Add, // Placeholder for unhandled ops
        }
    }

    /// Lower a comparison operator
    fn lower_cmp_op(&self, op: zulon_mir::MirBinOp) -> LirCmpOp {
        match op {
            zulon_mir::MirBinOp::Eq => LirCmpOp::Eq,
            zulon_mir::MirBinOp::NotEq => LirCmpOp::NotEq,
            zulon_mir::MirBinOp::Less => LirCmpOp::Less,
            zulon_mir::MirBinOp::LessEq => LirCmpOp::LessEq,
            zulon_mir::MirBinOp::Greater => LirCmpOp::Greater,
            zulon_mir::MirBinOp::GreaterEq => LirCmpOp::GreaterEq,
            _ => LirCmpOp::Eq, // Placeholder
        }
    }
}

impl Default for LirLoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Public API for lowering MIR to LIR
pub fn lower_mir(mir_body: &MirBody) -> Result<LirBody> {
    let mut ctx = LirLoweringContext::new();
    ctx.lower_body(mir_body)
}
