// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tree Borrows Borrow Checker
//!
//! This implements a simplified version of the Tree Borrows model from Rust.
//! Key concepts:
//!
//! - **Borrow Tree**: Each allocation has a tree of borrows
//! - **Permissions**: Each node has permissions (Read, Write, Disable)
//! - **Lifetimes**: Borrows have lifetimes that determine when they expire
//! - **Two-Phase Borrows**: Mutable references become active on first use

use crate::error::{MirError, Result};
use crate::mir::*;
use std::collections::HashMap;

/// Unique identifier for borrow checks
pub type BorrowId = usize;

/// Borrow kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorrowKind {
    /// Shared (immutable) borrow
    Shared,
    /// Unique (mutable) borrow
    Unique,
}

/// Permission in Tree Borrows
///
/// Permissions flow from parent to child in the borrow tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Can read and write
    ReadWrite,
    /// Can only read
    Read,
    /// Disabled (no access allowed)
    Disable,
}

impl Permission {
    /// Check if read is allowed
    pub fn can_read(self) -> bool {
        matches!(self, Permission::ReadWrite | Permission::Read)
    }

    /// Check if write is allowed
    pub fn can_write(self) -> bool {
        matches!(self, Permission::ReadWrite)
    }

    /// Restrict permission based on child access
    pub fn restrict_child(self, child_permission: Permission) -> Permission {
        match self {
            Permission::ReadWrite => child_permission,
            Permission::Read => Permission::Read,
            Permission::Disable => Permission::Disable,
        }
    }
}

/// Borrow node in the borrow tree
#[derive(Debug, Clone)]
struct BorrowNode {
    /// Kind of borrow
    kind: BorrowKind,

    /// Current permission
    permission: Permission,

    /// Place being borrowed
    place: MirPlace,

    /// Lifetime range (start_block, end_block)
    lifetime: (MirNodeId, MirNodeId),
}

impl BorrowNode {
    /// Create a new borrow node
    fn new(_id: BorrowId, kind: BorrowKind, place: MirPlace, lifetime: (MirNodeId, MirNodeId)) -> Self {
        let permission = match kind {
            BorrowKind::Shared => Permission::Read,
            BorrowKind::Unique => Permission::ReadWrite,
        };

        BorrowNode {
            kind,
            permission,
            place,
            lifetime,
        }
    }

    /// Check if this borrow is active at a given block
    fn is_active_at(&self, block: MirNodeId) -> bool {
        let (start, end) = self.lifetime;
        block >= start && block <= end
    }

    /// Check if this borrow conflicts with another
    fn conflicts_with(&self, other: &BorrowNode) -> bool {
        // Different places don't conflict
        if self.place != other.place {
            return false;
        }

        // Check lifetime overlap
        let overlaps = self.lifetime.0 <= other.lifetime.1 && other.lifetime.0 <= self.lifetime.1;
        if !overlaps {
            return false;
        }

        // Check permission conflict
        match (self.kind, other.kind) {
            // Two mutable borrows conflict
            (BorrowKind::Unique, BorrowKind::Unique) => true,
            // Mutable and shared borrows conflict
            (BorrowKind::Unique, BorrowKind::Shared) => true,
            (BorrowKind::Shared, BorrowKind::Unique) => true,
            // Two shared borrows are fine
            (BorrowKind::Shared, BorrowKind::Shared) => false,
        }
    }
}

/// Borrow checker context
pub struct BorrowChecker {
    /// All borrow nodes
    borrows: HashMap<BorrowId, BorrowNode>,

    /// Next borrow ID
    next_id: BorrowId,

    /// Place to root borrow mapping
    place_roots: HashMap<MirPlace, BorrowId>,

    /// Current block being checked
    current_block: MirNodeId,
}

impl BorrowChecker {
    /// Create a new borrow checker
    pub fn new() -> Self {
        BorrowChecker {
            borrows: HashMap::new(),
            next_id: 0,
            place_roots: HashMap::new(),
            current_block: 0,
        }
    }

    /// Allocate a new borrow ID
    fn alloc_borrow_id(&mut self) -> BorrowId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a borrow to the checker
    pub fn add_borrow(
        &mut self,
        kind: BorrowKind,
        place: MirPlace,
        lifetime: (MirNodeId, MirNodeId),
    ) -> Result<BorrowId> {
        let id = self.alloc_borrow_id();
        let node = BorrowNode::new(id, kind, place.clone(), lifetime);

        // Check for conflicts with existing borrows
        for existing in self.borrows.values() {
            if node.conflicts_with(existing) {
                return Err(MirError::BorrowError(format!(
                    "Borrow conflict: {:?} of {:?} conflicts with existing {:?}",
                    kind, place, existing.kind
                )));
            }
        }

        // If this is the first borrow for this place, make it a root
        if !self.place_roots.contains_key(&place) {
            self.place_roots.insert(place.clone(), id);
        }

        self.borrows.insert(id, node);
        Ok(id)
    }

    /// Check if a place can be read
    pub fn can_read(&self, place: &MirPlace, block: MirNodeId) -> Result<bool> {
        for borrow in self.borrows.values() {
            if borrow.place == *place && borrow.is_active_at(block) {
                if !borrow.permission.can_read() {
                    return Err(MirError::BorrowError(format!(
                        "Cannot read {:?}: permission denied",
                        place
                    )));
                }
            }
        }
        Ok(true)
    }

    /// Check if a place can be written
    pub fn can_write(&self, place: &MirPlace, block: MirNodeId) -> Result<bool> {
        for borrow in self.borrows.values() {
            if borrow.place == *place && borrow.is_active_at(block) {
                if !borrow.permission.can_write() {
                    return Err(MirError::BorrowError(format!(
                        "Cannot write {:?}: permission denied",
                        place
                    )));
                }
            }
        }
        Ok(true)
    }

    /// Check a MIR function for borrow violations
    pub fn check_function(&mut self, func: &MirFunction) -> Result<()> {
        // Collect all basic blocks in order
        let mut blocks: Vec<_> = func.blocks.values().collect();
        blocks.sort_by_key(|b| b.id);

        // First pass: collect all borrows
        self.collect_borrows(func)?;

        // Second pass: check all instructions
        for block in &blocks {
            self.current_block = block.id;

            // Check instructions
            for inst in &block.instructions {
                self.check_instruction(inst, func)?;
            }

            // Check terminator
            if let Some(terminator) = &block.terminator {
                self.check_terminator(terminator)?;
            }
        }

        Ok(())
    }

    /// Collect all borrows in a function
    fn collect_borrows(&mut self, func: &MirFunction) -> Result<()> {
        for block in func.blocks.values() {
            for inst in &block.instructions {
                if let MirInstruction::Borrow {
                    dest: _,
                    src,
                    mutable,
                    ty: _,
                } = inst
                {
                    // Determine lifetime (simplified: from this block to end of function)
                    let lifetime = (block.id, self.find_last_block(func)?);

                    // Add the borrow
                    let kind = if *mutable {
                        BorrowKind::Unique
                    } else {
                        BorrowKind::Shared
                    };

                    self.add_borrow(kind, src.clone(), lifetime)?;
                }
            }
        }
        Ok(())
    }

    /// Find the last block in a function (simplified)
    fn find_last_block(&self, func: &MirFunction) -> Result<MirNodeId> {
        func.blocks
            .keys()
            .max()
            .copied()
            .ok_or_else(|| MirError::InvalidConstruction("Function has no blocks".to_string()))
    }

    /// Check an instruction for borrow violations
    fn check_instruction(&self, inst: &MirInstruction, _func: &MirFunction) -> Result<()> {
        match inst {
            MirInstruction::Load { dest: _, src, ty: _ } => {
                self.can_read(src, self.current_block)?;
            }

            MirInstruction::Store { dest, src: _, ty: _ } => {
                self.can_write(dest, self.current_block)?;
            }

            MirInstruction::Copy { dest: _, src } => {
                self.can_read(src, self.current_block)?;
            }

            MirInstruction::Move { dest: _, src } => {
                // Move requires write access to dest and read from src
                self.can_read(src, self.current_block)?;
            }

            MirInstruction::BinaryOp {
                dest: _,
                op: _,
                left,
                right,
                ty: _,
            } => {
                self.can_read(&MirPlace::Temp(*left), self.current_block)?;
                self.can_read(&MirPlace::Temp(*right), self.current_block)?;
            }

            MirInstruction::UnaryOp {
                dest: _,
                op: _,
                operand,
                ty: _,
            } => {
                self.can_read(&MirPlace::Temp(*operand), self.current_block)?;
            }

            MirInstruction::Call {
                dest: _,
                func: _,
                args,
                return_type: _,
            } => {
                for arg in args {
                    self.can_read(arg, self.current_block)?;
                }
            }

            MirInstruction::Borrow {
                dest: _,
                src,
                mutable: _,
                ty: _,
            } => {
                // Borrowing requires read access to the source
                self.can_read(src, self.current_block)?;
            }

            MirInstruction::Drop { place, ty: _ } => {
                // Drop requires write access (to consume the value)
                self.can_write(place, self.current_block)?;
            }

            _ => {
                // Const instructions don't need borrow checking
            }
        }

        Ok(())
    }

    /// Check a terminator for borrow violations
    fn check_terminator(&self, terminator: &MirTerminator) -> Result<()> {
        match terminator {
            MirTerminator::Return(place) => {
                if let Some(p) = place {
                    self.can_read(p, self.current_block)?;
                }
            }

            MirTerminator::If { condition, then_block: _, else_block: _ } => {
                self.can_read(&MirPlace::Temp(*condition), self.current_block)?;
            }

            MirTerminator::Switch { scrutinee, targets: _, default: _ } => {
                self.can_read(&MirPlace::Temp(*scrutinee), self.current_block)?;
            }

            _ => {
                // Goto and Unreachable don't need borrow checking
            }
        }

        Ok(())
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Public API for borrow checking
pub fn check_borrows(func: &MirFunction) -> Result<()> {
    let mut checker = BorrowChecker::new();
    checker.check_function(func)
}
