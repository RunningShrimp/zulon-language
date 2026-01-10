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
    /// MIR temp to LIR type mapping (tracks types of temporary variables)
    temp_types: HashMap<zulon_mir::TempVar, LirTy>,
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
    /// Track which temps hold values from mutable locals that need loading (temp -> local_name)
    temp_to_local: HashMap<zulon_mir::TempVar, String>,
}

impl LirLoweringContext {
    /// Create a new lowering context
    pub fn new() -> Self {
        LirLoweringContext {
            temp_map: HashMap::new(),
            temp_types: HashMap::new(),
            param_map: HashMap::new(),
            local_map: HashMap::new(),
            block_returns: HashMap::new(),
            block_preds: HashMap::new(),
            pending_phis: HashMap::new(),
            mutable_locals: HashSet::new(),
            local_stack_slots: HashMap::new(),
            temp_to_local: HashMap::new(),
        }
    }

    /// Lower an entire MIR body to LIR
    pub fn lower_body(&mut self, mir_body: &MirBody) -> Result<LirBody> {
        let mut lir_body = LirBody::new();

        // NOTE: External functions like printf should be declared in the source code
        // with `extern fn` declarations. They will be extracted by the compiler
        // and added to lir_body.externals, preventing duplicate declarations.

        for func in &mir_body.functions {
            let lir_func = self.lower_function(func)?;
            lir_body.push_function(lir_func);
        }

        Ok(lir_body)
    }

    /// Lower a MIR function to LIR function
    pub fn lower_function(&mut self, func: &MirFunction) -> Result<LirFunction> {
        // Clear previous function's data
        self.temp_map.clear();
        self.temp_types.clear();
        self.block_returns.clear();
        self.block_preds.clear();
        self.pending_phis.clear();
        self.mutable_locals.clear();
        self.local_stack_slots.clear();
        self.local_map.clear();
        self.temp_to_local.clear();

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

        // Inject Load instructions before Return terminators for mutable locals
        self.inject_loads_before_returns(&mut lir_func)?;

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
                    MirInstruction::UnaryOp { dest, .. } => Some(*dest),
                    MirInstruction::Const { dest, .. } => Some(*dest),
                    MirInstruction::FieldAccess { dest, .. } => Some(*dest),
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
            MirTerminator::Switch { targets, default, .. } => {
                let mut result = targets.iter().map(|(_, block_id)| *block_id).collect::<Vec<_>>();
                result.push(*default);
                result
            }
            MirTerminator::Return { .. } => vec![],
            MirTerminator::Throw(_) => vec![],
            MirTerminator::Unreachable => vec![],
            MirTerminator::EffectCall { resume_block, .. } => vec![*resume_block],
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
        // First, eliminate empty join blocks that would become unreachable
        self.eliminate_empty_join_blocks(func)?;

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
                    // Check if there's a value-producing instruction that should be returned
                    let last_value_vreg = block.instructions.iter()
                        .rev()
                        .find_map(|inst| match inst {
                            LirInstruction::Copy { dest, .. } => Some(*dest),
                            LirInstruction::BinaryOp { dest, .. } => Some(*dest),
                            LirInstruction::UnaryOp { dest, .. } => Some(*dest),
                            LirInstruction::Load { dest, .. } => Some(*dest),
                            LirInstruction::Call { dest, .. } => *dest,
                            LirInstruction::Gep { dest, .. } => Some(*dest),
                            LirInstruction::Const { dest, .. } => Some(*dest),
                            _ => None,
                        });

                    if let Some(vreg) = last_value_vreg {
                        // Found a value to return
                        block.terminator = Some(LirTerminator::Return(Some(vreg)));
                    } else {
                        // No value found, this is likely dead code or unreachable
                        block.terminator = Some(LirTerminator::Unreachable);
                    }
                }
            }
        }

        Ok(())
    }

    /// Eliminate empty join blocks by redirecting predecessors to successors
    fn eliminate_empty_join_blocks(&mut self, func: &mut LirFunction) -> Result<()> {
        // Find empty blocks (no instructions, no phi) that would become unreachable
        // These blocks may or may not have terminators yet
        let empty_join_blocks: Vec<MirNodeId> = func.blocks.iter()
            .filter(|(_, block)| {
                block.instructions.is_empty()
                    && block.phi_nodes.is_empty()
            })
            .map(|(id, _)| *id)
            .collect();

        eprintln!("DEBUG: Found {} empty blocks", empty_join_blocks.len());

        for empty_block in empty_join_blocks {
            // Find all blocks that branch to this empty block
            let predecessors: Vec<MirNodeId> = func.blocks.iter()
                .filter(|(_, block)| {
                    self.branches_to(block, empty_block)
                })
                .map(|(id, _)| *id)
                .collect();

            eprintln!("DEBUG: Empty block {} has {} predecessors", empty_block, predecessors.len());

            if predecessors.is_empty() {
                // No predecessors, this block is truly dead
                continue;
            }

            // Find a suitable successor block
            // Look for blocks that come after this one in ID order
            let mut block_ids: Vec<_> = func.blocks.keys().copied().collect();
            block_ids.sort();

            let empty_idx = block_ids.iter().position(|&id| id == empty_block);
            let successor = if let Some(idx) = empty_idx {
                // Try the next block after this one
                if idx + 1 < block_ids.len() {
                    Some(block_ids[idx + 1])
                } else {
                    None
                }
            } else {
                None
            };

            eprintln!("DEBUG: Empty block {}, successor: {:?}", empty_block, successor);

            if let Some(succ_block) = successor {
                // Redirect all predecessors to the successor
                for pred_id in predecessors {
                    eprintln!("DEBUG: Redirecting {} from {} to {}", pred_id, empty_block, succ_block);
                    if let Some(pred_block) = func.blocks.get_mut(&pred_id) {
                        match &mut pred_block.terminator {
                            Some(LirTerminator::Jump { target }) if *target == empty_block => {
                                *target = succ_block;
                            }
                            Some(LirTerminator::Branch { then_block, else_block, .. }) => {
                                if *then_block == empty_block {
                                    *then_block = succ_block;
                                }
                                if *else_block == empty_block {
                                    *else_block = succ_block;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Mark the empty block as removed by clearing its terminator
                // This prevents it from getting Unreachable later
                if let Some(empty_block_obj) = func.blocks.get_mut(&empty_block) {
                    empty_block_obj.terminator = None;
                }
            }
        }

        Ok(())
    }

    /// Check if a block's terminator branches to the target block
    fn branches_to(&self, block: &LirBlock, target: MirNodeId) -> bool {
        match &block.terminator {
            Some(LirTerminator::Jump { target: t }) => t == &target,
            Some(LirTerminator::Branch { then_block, else_block, .. }) => {
                then_block == &target || else_block == &target
            }
            _ => false,
        }
    }

    /// Inject Load instructions before Return terminators that return mutable locals
    /// This ensures that variables stored to stack are loaded before being returned
    fn inject_loads_before_returns(&self, func: &mut LirFunction) -> Result<()> {
        // Collect the blocks that need load injection first to avoid borrow checker issues
        let mut blocks_needing_load: Vec<(MirNodeId, VReg)> = Vec::new();

        for (block_id, block) in &func.blocks {
            if let Some(LirTerminator::Return(return_vreg)) = block.terminator {
                // Check if this return vreg corresponds to a mutable local stack slot
                let needs_load = self.local_stack_slots.values().any(|&slot| Some(slot) == return_vreg);

                if needs_load {
                    if let Some(stack_slot) = return_vreg {
                        blocks_needing_load.push((*block_id, stack_slot));
                    }
                }
            }
        }

        // Now inject the loads
        for (block_id, stack_slot) in blocks_needing_load {
            // Find the type for this stack slot by looking up the local
            let mut ty = LirTy::I32; // Default

            for (local_name, &slot) in &self.local_stack_slots {
                if slot == stack_slot {
                    // Get the type from temp_types by finding a temp that was stored to this local
                    for (temp, local) in &self.temp_to_local {
                        if local == local_name {
                            if let Some(t) = self.temp_types.get(temp) {
                                ty = t.clone();
                                break;
                            }
                        }
                    }
                    break;
                }
            }

            // Allocate a new vreg for the loaded value BEFORE borrowing block
            let loaded_vreg = func.alloc_vreg();

            // Generate Load instruction
            let load_inst = LirInstruction::Load {
                dest: loaded_vreg,
                src: LirOperand::Reg(stack_slot),
                ty,
            };

            // Now borrow the block and insert the instruction
            if let Some(block) = func.blocks.get_mut(&block_id) {

                // Insert Load before the terminator
                block.instructions.push(load_inst);

                // Update the Return to use the loaded vreg
                block.terminator = Some(LirTerminator::Return(Some(loaded_vreg)));
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
                self.temp_types.insert(*dest, ty.clone().into());

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
                let mut instructions = Vec::new();

                // Helper to get operand vreg, generating Load if needed for mutable locals
                let get_operand = |temp: &zulon_mir::TempVar, func: &mut LirFunction, ctx: &mut Self| -> (VReg, Vec<LirInstruction>) {
                    // Check if this temp corresponds to a mutable local that needs loading
                    if let Some(local_name) = ctx.temp_to_local.get(temp) {
                        // This temp was stored to a mutable local - we need to load it back
                        let stack_slot = *ctx.local_stack_slots.get(local_name)
                            .expect("Mutable local should have stack slot");

                        // Get the type for this temp
                        let operand_ty = ctx.temp_types.get(temp)
                            .cloned()
                            .unwrap_or(LirTy::I32);

                        // Allocate a new vreg for the loaded value
                        let loaded_vreg = func.alloc_vreg();

                        // Generate Load instruction
                        let load_inst = LirInstruction::Load {
                            dest: loaded_vreg,
                            src: LirOperand::Reg(stack_slot),
                            ty: operand_ty,
                        };

                        // Update temp_map to point to the loaded value
                        ctx.temp_map.insert(*temp, loaded_vreg);

                        (loaded_vreg, vec![load_inst])
                    } else if let Some(&vreg) = ctx.temp_map.get(temp) {
                        // Temp has a vreg mapping - use it directly
                        (vreg, vec![])
                    } else {
                        // No mapping - use temp number as vreg (fallback)
                        (*temp as VReg, vec![])
                    }
                };

                let (left_vreg, left_loads) = get_operand(left, func, self);
                let (right_vreg, right_loads) = get_operand(right, func, self);

                // Add any Load instructions that were generated
                instructions.extend(left_loads);
                instructions.extend(right_loads);

                self.temp_map.insert(*dest, dest_vreg);
                self.temp_types.insert(*dest, ty.clone().into());

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
                    instructions.push(LirInstruction::Cmp {
                        dest: dest_vreg,
                        op: lir_cmp_op,
                        left: left_vreg,
                        right: right_vreg,
                    });
                } else {
                    // Regular binary operation (arithmetic, bitwise, or logical)
                    let lir_op = self.lower_bin_op(*op);
                    instructions.push(LirInstruction::BinaryOp {
                        dest: dest_vreg,
                        op: lir_op,
                        left: left_vreg,
                        right: right_vreg,
                        ty: ty.clone().into(),
                    });
                }

                Ok(instructions)
            }

            MirInstruction::UnaryOp { dest, op, operand, ty } => {
                let dest_vreg = func.alloc_vreg();
                let mut instructions = Vec::new();

                // Get operand, generating Load if needed for mutable locals
                let (operand_vreg, load_insts) = if let Some(local_name) = self.temp_to_local.get(operand) {
                    // This temp was stored to a mutable local - we need to load it back
                    let stack_slot = *self.local_stack_slots.get(local_name)
                        .expect("Mutable local should have stack slot");

                    let operand_ty = self.temp_types.get(operand)
                        .cloned()
                        .unwrap_or(LirTy::I32);

                    let loaded_vreg = func.alloc_vreg();

                    let load_inst = LirInstruction::Load {
                        dest: loaded_vreg,
                        src: LirOperand::Reg(stack_slot),
                        ty: operand_ty,
                    };

                    self.temp_map.insert(*operand, loaded_vreg);
                    (loaded_vreg, vec![load_inst])
                } else if let Some(&vreg) = self.temp_map.get(operand) {
                    (vreg, vec![])
                } else {
                    (*operand as VReg, vec![])
                };

                instructions.extend(load_insts);

                self.temp_map.insert(*dest, dest_vreg);
                self.temp_types.insert(*dest, ty.clone().into());

                // Convert MIR unary op to LIR unary op
                let lir_op = match op {
                    zulon_mir::MirUnaryOp::Neg => LirUnaryOp::Neg,
                    zulon_mir::MirUnaryOp::Not => LirUnaryOp::Not,
                    _ => LirUnaryOp::Neg, // Default to Neg for other ops (Ref, Deref, etc.)
                };

                instructions.push(LirInstruction::UnaryOp {
                    dest: dest_vreg,
                    op: lir_op,
                    operand: operand_vreg,
                    ty: ty.clone().into(),
                });

                Ok(instructions)
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
                    self.temp_types.insert(*d, return_type.clone().into());
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
                    self.temp_types.insert(*dest, ty.clone().into());

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

                    // Check if base is a struct value (not a pointer)
                    // If so, we need to store it to an alloca first before doing GEP
                    let base_type = self.get_place_type(base);
                    let mut instructions = Vec::new();

                    let (base_for_gep, gep_ty) = if matches!(base_type, LirTy::Struct { .. }) {
                        // Base is a struct value - need to store it first
                        let temp_slot = func.alloc_vreg();
                        let base_type_lir: LirTy = base_type.clone();

                        // Allocate stack slot
                        instructions.push(LirInstruction::Alloca(crate::lir::LirAlloca {
                            dest: temp_slot,
                            ty: base_type_lir.clone(),
                        }));

                        // Store struct value to stack slot
                        instructions.push(LirInstruction::Store {
                            dest: LirOperand::Reg(temp_slot),
                            src: base_vreg,
                            ty: base_type_lir.clone(),
                        });

                        // Use the stack slot for GEP, and use struct type for GEP
                        (temp_slot, base_type_lir)
                    } else {
                        // Base is already a pointer
                        (base_vreg, ty.clone().into())
                    };

                    // Generate GEP + Load
                    instructions.push(LirInstruction::Gep {
                        dest: gep_vreg,
                        base: base_for_gep,
                        indices: vec![
                            LirOperand::Imm(0),  // struct pointer
                            LirOperand::Imm(field_index),  // field index
                        ],
                        ty: gep_ty,
                    });
                    instructions.push(LirInstruction::Load {
                        dest: dest_vreg,
                        src: LirOperand::Reg(gep_vreg),  // Load from GEP result
                        ty: ty.clone().into(),
                    });

                    Ok(instructions)
                } else if let MirPlace::Local(name) = src {
                    // Check if this is a load from a mutable local
                    if self.mutable_locals.contains(name) {
                        // Mutable local: generate actual Load from stack slot
                        let stack_slot = *self.local_stack_slots.get(name)
                            .expect("Mutable local should have stack slot");

                        let dest_vreg = func.alloc_vreg();
                        self.temp_map.insert(*dest, dest_vreg);
                        self.temp_types.insert(*dest, ty.clone().into());

                        Ok(vec![LirInstruction::Load {
                            dest: self.temp_map[dest],
                            src: LirOperand::Reg(stack_slot),
                            ty: ty.clone().into(),
                        }])
                    } else {
                        // Immutable local: SSA rename (no instruction needed)
                        let src_vreg = self.get_or_alloc_vreg(src, func);
                        self.temp_map.insert(*dest, src_vreg);
                        self.temp_types.insert(*dest, ty.clone().into());
                        Ok(vec![])
                    }
                } else {
                    // Load from non-local (Temp, Param): SSA rename
                    let src_vreg = self.get_or_alloc_vreg(src, func);
                    self.temp_map.insert(*dest, src_vreg);
                    self.temp_types.insert(*dest, ty.clone().into());
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

                        // Track that this src temp corresponds to this mutable local
                        // This allows us to generate Load instructions when the temp is used later
                        self.temp_to_local.insert(*src, name.clone());

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

            MirInstruction::FieldAccess { dest, base, field_name: _, field_index, ty } => {
                // Lower MIR FieldAccess to LIR GEP + Load
                let base_vreg = self.temp_map.get(base).copied().unwrap_or_else(|| *base as VReg);
                let dest_vreg = func.alloc_vreg();
                let gep_vreg = func.alloc_vreg();

                self.temp_map.insert(*dest, dest_vreg);
                self.temp_types.insert(*dest, ty.clone().into());

                // Generate GEP to get pointer to field, then Load the value
                Ok(vec![
                    LirInstruction::Gep {
                        dest: gep_vreg,
                        base: base_vreg,
                        indices: vec![
                            LirOperand::Imm(0),  // struct pointer
                            LirOperand::Imm(*field_index as u64),  // field index
                        ],
                        ty: ty.clone().into(),
                    },
                    LirInstruction::Load {
                        dest: dest_vreg,
                        src: LirOperand::Reg(gep_vreg),
                        ty: ty.clone().into(),
                    },
                ])
            }

            MirInstruction::PerformEffect { dest, effect_name: _, operation_name: _, args: _, return_type } => {
                // Effect operations are currently stubbed
                // In a full implementation, this would:
                // 1. Pack the effect operation into a continuation
                // 2. Call the effect handler
                // 3. Resume with the result

                if let Some(d) = dest {
                    let dest_vreg = func.alloc_vreg();
                    self.temp_map.insert(*d, dest_vreg);
                    self.temp_types.insert(*d, return_type.clone().into());

                    // For now, just return a placeholder value
                    // This will be replaced by proper effect handler dispatch
                    Ok(vec![
                        LirInstruction::Const {
                            dest: dest_vreg,
                            value: LirConstant::Unit,
                            ty: return_type.clone().into(),
                        }
                    ])
                } else {
                    Ok(vec![])
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
                        zulon_mir::MirPlace::Local(name) => {
                            // Check if this is a mutable local

                            if self.mutable_locals.contains(name) {
                                // Return the stack slot - will be loaded by inject_loads_before_returns
                                let slot = self.local_stack_slots.get(name).copied();
                                slot
                            } else {
                                // Immutable local - look up in local_map
                                let imm = self.local_map.get(name).copied();
                                imm
                            }
                        }
                        _ => None,
                    }
                });

                Ok(LirTerminator::Return(vreg))
            }

            MirTerminator::Throw(place) => {
                let vreg = match place {
                    zulon_mir::MirPlace::Temp(t) => {
                        // Look up the temp in temp_map to get the actual vreg
                        self.temp_map.get(t).copied()
                    }
                    zulon_mir::MirPlace::Local(name) => {
                        // Check if this is a mutable local
                        if self.mutable_locals.contains(name) {
                            // Return the stack slot
                            self.local_stack_slots.get(name).copied()
                        } else {
                            // Immutable local - look up in local_map
                            self.local_map.get(name).copied()
                        }
                    }
                    _ => None,
                }.ok_or_else(|| crate::error::LirError::LoweringError("Throw terminator has invalid place".to_string()))?;

                Ok(LirTerminator::Throw(vreg))
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

            MirTerminator::Switch { scrutinee, targets, default } => {
                let scrutinee_vreg = self.temp_map.get(scrutinee).copied().unwrap_or(*scrutinee as VReg);

                // Convert MIR constants to u64 values for LIR
                let lir_targets = targets.iter().map(|(constant, block_id)| {
                    let value = match constant {
                        zulon_mir::MirConstant::Bool(b) => {
                            if *b { 1 } else { 0 }
                        }
                        zulon_mir::MirConstant::Integer(i) => {
                            *i as u64
                        }
                        zulon_mir::MirConstant::Char(c) => {
                            *c as u64
                        }
                        _ => {
                            // For other constants (float, string, unit), use 0 as default
                            // These shouldn't appear in match patterns for now
                            0
                        }
                    };
                    (value, *block_id)
                }).collect();

                Ok(LirTerminator::Switch {
                    scrutinee: scrutinee_vreg,
                    targets: lir_targets,
                    default: *default,
                })
            }

            MirTerminator::Unreachable => {
                Ok(LirTerminator::Unreachable)
            }

            MirTerminator::EffectCall {
                effect_name: _effect_name,
                operation_name: _operation_name,
                args: _args,
                return_type: _return_type,
                resume_block,
                dest: _dest,
            } => {
                // For now, just jump to the resume block
                // This simulates a handler that immediately returns
                // TODO: Implement proper handler block generation and dispatch
                Ok(LirTerminator::Jump {
                    target: *resume_block,
                })
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
            zulon_mir::MirPlace::Temp(temp) => {
                // Look up the type from temp_types map
                if let Some(ty) = self.temp_types.get(temp) {
                    ty.clone()
                } else {
                    // Default to I32 if type not found
                    LirTy::I32
                }
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
