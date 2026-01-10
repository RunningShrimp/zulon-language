// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Async state machine transformation
//!
//! This module transforms async functions in MIR into state machines.
//! The transformation happens after the initial MIR lowering and converts:
//! - Async function bodies into state machine switch statements
//! - Await expressions into yield points that save state and return
//! - Local variables into preserved state across suspensions
//!
//! The transformation follows this approach:
//! 1. Identify all await points in the function
//! 2. Create a state enum with variants for each await point
//! 3. Split basic blocks at await boundaries
//! 4. Generate state saving/restoration logic
//! 5. Transform the function to use a state variable

use crate::error::{MirError, Result};
use crate::mir::*;
use crate::ty::MirTy;
use std::collections::{HashMap, HashSet};

/// Information about an await point in the function
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AwaitPoint {
    /// Unique ID for this await point
    id: usize,
    /// Basic block containing the await
    block_id: MirNodeId,
    /// Temporary holding the future being awaited
    future_temp: TempVar,
    /// Temporary where the await result should be stored
    result_temp: TempVar,
    /// State ID to resume in after this await completes
    resume_state: usize,
    /// Local variables that are live across this await
    captured_locals: Vec<TempVar>,
}

/// State machine transformation context
struct AsyncTransformContext {
    /// The function being transformed
    func: MirFunction,
    /// All await points discovered in the function
    await_points: Vec<AwaitPoint>,
    /// Set of basic blocks that contain await expressions
    await_blocks: HashSet<MirNodeId>,
    /// State variable temporary (holds current state)
    state_var: TempVar,
    /// Future variable temporary (holds the future being polled)
    future_var: TempVar,
}

impl AsyncTransformContext {
    /// Create a new transformation context
    fn new(func: MirFunction) -> Self {
        AsyncTransformContext {
            func,
            await_points: Vec::new(),
            await_blocks: HashSet::new(),
            state_var: 0, // Will be allocated
            future_var: 0, // Will be allocated
        }
    }

    /// Transform an async function into a state machine
    fn transform(mut self) -> Result<MirFunction> {
        // Step 1: Identify all await points in the function
        self.identify_await_points()?;

        // Step 2: Analyze variable liveness at await points
        self.analyze_variable_capture()?;

        // Step 3: Create state machine structure
        self.create_state_machine()?;

        // Step 4: Transform basic blocks
        self.transform_blocks()?;

        // Step 5: Add state machine entry logic
        self.add_entry_logic()?;

        Ok(self.func)
    }

    /// Step 1: Identify all await points in the function
    fn identify_await_points(&mut self) -> Result<()> {
        let mut await_id = 0;

        // Scan all basic blocks for await expressions
        for (block_id, block) in &self.func.blocks {
            for (_instr_idx, instr) in block.instructions.iter().enumerate() {
                // Look for the pattern: Call to poll function or await marker
                if let MirInstruction::Call { func, .. } = instr {
                    if let MirPlace::Local(name) = func {
                        if name == "await" || name.contains("poll") {
                            // Found an await point
                            self.await_blocks.insert(*block_id);

                            let await_point = AwaitPoint {
                                id: await_id,
                                block_id: *block_id,
                                future_temp: 0, // Will be determined
                                result_temp: 0, // Will be determined
                                resume_state: await_id + 1,
                                captured_locals: Vec::new(), // Will be computed
                            };

                            self.await_points.push(await_point);
                            await_id += 1;
                        }
                    }
                }
            }
        }

        // For MVP: If we didn't find explicit await markers,
        // look for Call instructions that might be await points
        // This is a heuristic - proper implementation needs await detection in lowering

        Ok(())
    }

    /// Step 2: Analyze variable liveness at await points
    ///
    /// This computes which local variables need to be captured/saved
    /// at each await point. Variables that are live after an await
    /// must be preserved in the state machine.
    fn analyze_variable_capture(&mut self) -> Result<()> {
        // Collect block IDs first to avoid borrow issues
        let await_blocks: Vec<(usize, MirNodeId)> = self.await_points.iter()
            .map(|ap| (ap.id, ap.block_id))
            .collect();

        // For each await point, compute which temporaries are live
        let mut all_captured_locals = Vec::new();

        for (id, block_id) in await_blocks {
            // Collect all temporaries that are used after this await
            let mut live_vars = HashSet::new();

            // Analyze all blocks that can be reached after this await
            self.collect_live_variables_after(block_id, &mut live_vars)?;

            // Store captured locals
            let captured: Vec<TempVar> = live_vars.into_iter().collect();
            all_captured_locals.push((id, captured));
        }

        // Now update await_points with captured locals
        for (id, captured) in all_captured_locals {
            if let Some(await_point) = self.await_points.get_mut(id) {
                await_point.captured_locals = captured;
            }
        }

        // Update state machine's preserved_locals with all captured vars
        if let Some(ref mut sm) = self.func.state_machine {
            let all_captured: HashSet<TempVar> = self.await_points.iter()
                .flat_map(|ap| ap.captured_locals.iter().copied())
                .collect();

            sm.preserved_locals = all_captured.into_iter().collect();
        }

        Ok(())
    }

    /// Collect all live variables reachable from a given block
    fn collect_live_variables_after(&self, block_id: MirNodeId, live_vars: &mut HashSet<TempVar>) -> Result<()> {
        let mut visited = HashSet::new();
        self.collect_live_variables_recursive(block_id, live_vars, &mut visited)
    }

    /// Recursive helper for variable liveness analysis
    fn collect_live_variables_recursive(
        &self,
        block_id: MirNodeId,
        live_vars: &mut HashSet<TempVar>,
        visited: &mut HashSet<MirNodeId>,
    ) -> Result<()> {
        // Avoid infinite recursion
        if visited.contains(&block_id) {
            return Ok(());
        }
        visited.insert(block_id);

        // Get the block
        let block = match self.func.blocks.get(&block_id) {
            Some(b) => b,
            None => return Ok(()), // Block doesn't exist, skip
        };

        // Collect temporaries used in this block
        for instr in &block.instructions {
            self.collect_temporaries_from_instruction(instr, live_vars);
        }

        // Also check terminator
        if let Some(ref terminator) = block.terminator {
            self.collect_temporaries_from_terminator(terminator, live_vars);

            // Recursively process successor blocks
            match terminator {
                MirTerminator::Goto { target } => {
                    self.collect_live_variables_recursive(*target, live_vars, visited)?;
                }
                MirTerminator::If { then_block, else_block, .. } => {
                    self.collect_live_variables_recursive(*then_block, live_vars, visited)?;
                    self.collect_live_variables_recursive(*else_block, live_vars, visited)?;
                }
                MirTerminator::Switch { targets, default, .. } => {
                    for (_, target) in targets {
                        self.collect_live_variables_recursive(*target, live_vars, visited)?;
                    }
                    self.collect_live_variables_recursive(*default, live_vars, visited)?;
                }
                MirTerminator::Return(_) | MirTerminator::Throw(_) | MirTerminator::Unreachable => {
                    // No successors
                }
                MirTerminator::EffectCall { resume_block, .. } => {
                    self.collect_live_variables_recursive(*resume_block, live_vars, visited)?;
                }
            }
        }

        Ok(())
    }

    /// Collect temporaries used in an instruction
    fn collect_temporaries_from_instruction(&self, instr: &MirInstruction, live_vars: &mut HashSet<TempVar>) {
        match instr {
            MirInstruction::Const { dest, .. } => {
                live_vars.insert(*dest);
            }
            MirInstruction::Copy { dest, src } | MirInstruction::Move { dest, src } => {
                live_vars.insert(*dest);
                self.collect_temporaries_from_place(src, live_vars);
            }
            MirInstruction::BinaryOp { dest, left, right, .. } => {
                live_vars.insert(*dest);
                live_vars.insert(*left);
                live_vars.insert(*right);
            }
            MirInstruction::UnaryOp { dest, operand, .. } => {
                live_vars.insert(*dest);
                live_vars.insert(*operand);
            }
            MirInstruction::Call { dest, args, .. } => {
                if let &Some(d) = dest {
                    live_vars.insert(d);
                }
                for arg in args {
                    self.collect_temporaries_from_place(arg, live_vars);
                }
            }
            MirInstruction::Load { dest, .. } => {
                live_vars.insert(*dest);
            }
            MirInstruction::Store { src, .. } => {
                live_vars.insert(*src);
            }
            MirInstruction::Borrow { dest, src, .. } => {
                live_vars.insert(*dest);
                self.collect_temporaries_from_place(src, live_vars);
            }
            MirInstruction::FieldAccess { dest, base, .. } => {
                live_vars.insert(*dest);
                live_vars.insert(*base);
            }
            MirInstruction::Drop { .. } => {
                // Drops don't produce values
            }
            MirInstruction::PerformEffect { dest, args, .. } => {
                if let &Some(d) = dest {
                    live_vars.insert(d);
                }
                for arg in args {
                    self.collect_temporaries_from_place(arg, live_vars);
                }
            }
        }
    }

    /// Collect temporaries used in a terminator
    fn collect_temporaries_from_terminator(&self, term: &MirTerminator, live_vars: &mut HashSet<TempVar>) {
        match term {
            MirTerminator::Return(ret_place) => {
                if let Some(place) = ret_place {
                    self.collect_temporaries_from_place(place, live_vars);
                }
            }
            MirTerminator::Throw(place) => {
                self.collect_temporaries_from_place(place, live_vars);
            }
            MirTerminator::If { condition, .. } => {
                live_vars.insert(*condition);
            }
            MirTerminator::Switch { scrutinee, .. } => {
                live_vars.insert(*scrutinee);
            }
            MirTerminator::EffectCall { args, .. } => {
                for arg in args {
                    self.collect_temporaries_from_place(arg, live_vars);
                }
            }
            MirTerminator::Goto { .. } | MirTerminator::Unreachable => {
                // No temporaries
            }
        }
    }

    /// Collect temporaries from a place
    fn collect_temporaries_from_place(&self, place: &MirPlace, live_vars: &mut HashSet<TempVar>) {
        match place {
            MirPlace::Temp(temp) => {
                live_vars.insert(*temp);
            }
            MirPlace::Field { base, .. } => {
                self.collect_temporaries_from_place(base, live_vars);
            }
            MirPlace::Index { base, index } => {
                self.collect_temporaries_from_place(base, live_vars);
                live_vars.insert(*index);
            }
            MirPlace::Deref(inner) => {
                self.collect_temporaries_from_place(inner, live_vars);
            }
            MirPlace::Ref { place, .. } => {
                self.collect_temporaries_from_place(place, live_vars);
            }
            MirPlace::Local(_) | MirPlace::Param(_) => {
                // Locals and params are not temporaries
            }
        }
    }

    /// Step 3: Create the state machine structure
    fn create_state_machine(&mut self) -> Result<()> {
        // Allocate state variable
        self.state_var = self.func.alloc_temp();

        // Allocate future variable (if needed)
        if !self.await_points.is_empty() {
            self.future_var = self.func.alloc_temp();
        }

        // Update the state machine metadata
        if let Some(ref mut sm) = self.func.state_machine {
            // Create states for each await point plus start and complete states
            // State 0: Start (before first await)
            // State 1..n: After each await (resume points)
            // State n+1: Complete

            let start_block = self.func.entry_block;

            // Add start state (state 0)
            sm.add_state(start_block, Vec::new());

            // Add states for each await point
            for await_point in &self.await_points {
                // The resume block will be created during block transformation
                // For now, use the block containing the await
                sm.add_state(await_point.block_id, await_point.captured_locals.clone());
            }

            // Note: We don't add a "complete" state - the function just returns
        }

        Ok(())
    }

    /// Step 3: Transform basic blocks to implement state machine
    fn transform_blocks(&mut self) -> Result<()> {
        // For each await point, we need to:
        // 1. Split the block at the await point
        // 2. Insert state saving logic before the await
        // 3. Insert yield logic (save and return)
        // 4. Create resume block that restores state and continues

        let mut new_blocks = HashMap::new();
        let mut block_transformation = HashMap::new(); // old_block -> (pre_block_id, resume_block_id)

        // Collect block IDs first to avoid borrow checker issues
        let block_ids: Vec<MirNodeId> = self.func.blocks.keys().copied().collect();

        // Build a map from block_id to await_point (cloned to avoid borrow issues)
        let block_to_await: HashMap<MirNodeId, AwaitPoint> = self.await_points.iter()
            .map(|ap| (ap.block_id, ap.clone()))
            .collect();

        for block_id in block_ids {
            let block = self.func.blocks.get(&block_id).unwrap().clone();

            if self.await_blocks.contains(&block_id) {
                // This block contains an await - need to split it
                // Get the await point for this block
                let await_point = block_to_await.get(&block_id).unwrap();

                let (pre_block, resume_block) = self.split_await_block(block_id, &block, await_point)?;

                block_transformation.insert(block_id, (pre_block.id, resume_block.id));
                new_blocks.insert(pre_block.id, pre_block);
                new_blocks.insert(resume_block.id, resume_block);
            } else {
                // Regular block - just copy it
                new_blocks.insert(block_id, block);
            }
        }

        // Replace the function's blocks with the transformed ones
        self.func.blocks = new_blocks;

        // Update block references in terminators
        self.update_terminators(&block_transformation)?;

        Ok(())
    }

    /// Split a block containing an await into pre-await and resume blocks
    fn split_await_block(&mut self, block_id: MirNodeId, block: &MirBasicBlock, await_point: &AwaitPoint) -> Result<(MirBasicBlock, MirBasicBlock)> {
        // Find the await instruction
        let await_idx = block.instructions.iter().position(|instr| {
            matches!(instr, MirInstruction::Call { func, .. } if matches!(func, MirPlace::Local(name) if name == "await" || name.contains("poll")))
        });

        if await_idx.is_none() {
            return Err(MirError::TransformError(
                format!("Block {} marked as await block but no await found", block_id)
            ));
        }

        let await_idx = await_idx.unwrap();

        // Create pre-await block (instructions before the await)
        let mut pre_block = MirBasicBlock::new(self.func.alloc_block());
        for instr in block.instructions.iter().take(await_idx) {
            pre_block.push_instruction(instr.clone());
        }

        // Generate state saving code before the await
        self.generate_state_saving(&mut pre_block, await_point)?;

        // Update state variable to indicate we're about to await
        let next_state = await_point.resume_state;
        pre_block.push_instruction(MirInstruction::Const {
            dest: self.state_var,
            value: MirConstant::Integer(next_state as i128),
            ty: MirTy::I32,
        });

        // Set terminator to return (yield point)
        // For MVP: Return state to indicate pending
        pre_block.set_terminator(MirTerminator::Return(Some(MirPlace::Temp(self.state_var))));

        // Create resume block (instructions after the await)
        let mut resume_block = MirBasicBlock::new(self.func.alloc_block());

        // Generate state restoration code at resume point
        self.generate_state_restoration(&mut resume_block, await_point)?;

        // Copy remaining instructions after the await
        for instr in block.instructions.iter().skip(await_idx + 1) {
            resume_block.push_instruction(instr.clone());
        }

        // Copy terminator from original block to resume block
        if let Some(ref term) = block.terminator {
            resume_block.set_terminator(term.clone());
        }

        Ok((pre_block, resume_block))
    }

    /// Generate code to save captured variables before an await
    fn generate_state_saving(&mut self, block: &mut MirBasicBlock, await_point: &AwaitPoint) -> Result<()> {
        // For each captured temporary, generate a store instruction
        // In a full implementation, this would store to a state machine struct
        // For MVP: Generate comments/nops to show where saving would happen

        for &temp in &await_point.captured_locals {
            // TODO: Generate actual store to state machine struct
            // For now, we generate a Copy instruction to a preserved temporary

            // Create a temporary in the preserved area
            let preserved_temp = self.func.alloc_temp();

            // Copy the live temporary to preserved storage
            block.push_instruction(MirInstruction::Copy {
                dest: preserved_temp,
                src: MirPlace::Temp(temp),
            });

            // Note: In a full implementation, these preserved temporaries
            // would be stored in a state machine struct that lives across
            // the await boundary. For now, we're just demonstrating
            // the mechanism.
        }

        Ok(())
    }

    /// Generate code to restore captured variables after an await
    fn generate_state_restoration(&mut self, block: &mut MirBasicBlock, await_point: &AwaitPoint) -> Result<()> {
        // For each captured temporary, generate a load instruction
        // In a full implementation, this would load from the state machine struct

        for &temp in &await_point.captured_locals {
            // TODO: Generate actual load from state machine struct
            // For now, the Copy instruction in generate_state_saving
            // is sufficient to show the mechanism

            // In the full implementation, we would:
            // 1. Load the value from the state machine struct
            // 2. Move it back to the original temporary
            // 3. Ensure the temporary is live for subsequent code

            // For MVP demonstration:
            block.push_instruction(MirInstruction::Const {
                dest: temp,
                value: MirConstant::Integer(0), // Placeholder
                ty: MirTy::I32,
            });
        }

        Ok(())
    }

    /// Update terminators to point to new block IDs after transformation
    fn update_terminators(&mut self, transformation: &HashMap<MirNodeId, (MirNodeId, MirNodeId)>) -> Result<()> {
        for (_block_id, block) in &mut self.func.blocks {
            if let Some(ref mut terminator) = block.terminator {
                match terminator {
                    MirTerminator::Goto { target } => {
                        if let Some((_, resume_id)) = transformation.get(target) {
                            *target = *resume_id;
                        }
                    }
                    MirTerminator::If { then_block, else_block, .. } => {
                        if let Some((_, resume_id)) = transformation.get(then_block) {
                            *then_block = *resume_id;
                        }
                        if let Some((_, resume_id)) = transformation.get(else_block) {
                            *else_block = *resume_id;
                        }
                    }
                    MirTerminator::Switch { targets, default, .. } => {
                        if let Some((_, resume_id)) = transformation.get(default) {
                            *default = *resume_id;
                        }
                        for (_, target) in targets.iter_mut() {
                            if let Some((_, resume_id)) = transformation.get(target) {
                                *target = *resume_id;
                            }
                        }
                    }
                    _ => {
                        // Return, EffectCall, Unreachable don't need updating
                    }
                }
            }
        }

        Ok(())
    }

    /// Step 4: Add state machine entry logic
    fn add_entry_logic(&mut self) -> Result<()> {
        if self.await_points.is_empty() {
            // No awaits in this function - no state machine needed
            return Ok(());
        }

        // Create a new entry block that switches on state
        let mut entry_block = MirBasicBlock::new(self.func.alloc_block());

        // Initialize state variable to 0 (start state)
        entry_block.push_instruction(MirInstruction::Const {
            dest: self.state_var,
            value: MirConstant::Integer(0),
            ty: MirTy::I32,
        });

        // Add switch statement on state variable
        let mut targets = Vec::new();

        // State 0: Start (go to original entry block)
        targets.push((MirConstant::Integer(0), self.func.entry_block));

        // State 1..n: Resume after each await
        for (i, await_point) in self.await_points.iter().enumerate() {
            let state_val = (i + 1) as i128;
            // The resume block will be the transformed block
            // For now, use the block containing the await (will be updated by block transformation)
            targets.push((MirConstant::Integer(state_val), await_point.block_id));
        }

        // Create a default block (shouldn't happen - return error)
        let error_block = self.func.alloc_block();
        let mut error_block_obj = MirBasicBlock::new(error_block);
        error_block_obj.set_terminator(MirTerminator::Unreachable);
        self.func.blocks.insert(error_block, error_block_obj);

        entry_block.set_terminator(MirTerminator::Switch {
            scrutinee: self.state_var,
            targets,
            default: error_block,
        });

        // Replace entry block with new switch block
        let _old_entry = self.func.entry_block;
        self.func.entry_block = entry_block.id;
        self.func.blocks.insert(entry_block.id, entry_block);

        Ok(())
    }
}

/// Public API: Transform an async function into a state machine
pub fn transform_async_function(func: MirFunction) -> Result<MirFunction> {
    if !func.is_async {
        // Not an async function - return as-is
        return Ok(func);
    }

    let ctx = AsyncTransformContext::new(func);
    ctx.transform()
}

/// Transform all async functions in a MIR body
pub fn transform_async_functions(body: MirBody) -> Result<MirBody> {
    let mut transformed = MirBody::new();

    for func in body.functions {
        let transformed_func = transform_async_function(func)?;
        transformed.push_function(transformed_func);
    }

    Ok(transformed)
}
