// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LIR optimizations
//!
//! This module implements various optimization passes on LIR.

use crate::error::Result;
use crate::lir::*;

/// Constant folding optimization
pub fn constant_fold(func: &mut LirFunction) -> Result<()> {
    for block in func.blocks.values_mut() {
        let mut i = 0;
        while i < block.instructions.len() {
            if let Some(folded) = try_fold(&block.instructions[i]) {
                block.instructions[i] = folded;
            }
            i += 1;
        }
    }
    Ok(())
}

/// Try to fold an instruction
fn try_fold(inst: &LirInstruction) -> Option<LirInstruction> {
    match inst {
        LirInstruction::BinaryOp { dest: _, op: _, left: _, right: _, ty: _ } => {
            // Only fold if both operands are constants
            // (In real SSA, this requires reaching definitions analysis)
            None
        }
        _ => None,
    }
}

/// Dead code elimination
pub fn dead_code_elimination(func: &mut LirFunction) -> Result<()> {
    // Collect used registers
    let mut used = std::collections::HashSet::new();

    // Mark return values as used
    for block in func.blocks.values() {
        if let Some(LirTerminator::Return(Some(vreg))) = &block.terminator {
            used.insert(*vreg);
        }
    }

    // Mark operands in terminators as used
    for block in func.blocks.values() {
        if let Some(terminator) = &block.terminator {
            match terminator {
                LirTerminator::Branch { condition, .. } => {
                    used.insert(*condition);
                }
                _ => {}
            }
        }
    }

    // Collect live instructions
    for block in func.blocks.values_mut() {
        block.instructions.retain(|inst| {
            let dest = match inst {
                LirInstruction::Const { dest, .. } => Some(*dest),
                LirInstruction::BinaryOp { dest, .. } => Some(*dest),
                LirInstruction::Copy { dest, .. } => Some(*dest),
                LirInstruction::Call { dest, .. } => *dest,
                _ => None,
            };

            if let Some(d) = dest {
                used.contains(&d)
            } else {
                true // Keep instructions without destinations (stores, etc.)
            }
        });
    }

    Ok(())
}
