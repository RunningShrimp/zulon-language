// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! HIR to MIR lowering
//!
//! This transforms high-level IR into mid-level IR by:
//! - Flattening expressions into temporary variables
//! - Making control flow explicit with basic blocks
//! - Converting HIR constructs to MIR instructions

use crate::error::{MirError, Result};
use crate::mir::*;
use crate::ty::MirTy;
use zulon_hir::{HirCrate, HirItem, HirFunction, HirExpression, HirBlock, HirStatement};

/// Context for lowering HIR to MIR
pub struct MirLoweringContext;

impl MirLoweringContext {
    /// Create a new lowering context
    pub fn new() -> Self {
        MirLoweringContext
    }

    /// Lower an entire HIR crate to MIR body
    pub fn lower_crate(&mut self, hir_crate: &HirCrate) -> Result<MirBody> {
        let mut body = MirBody::new();

        for item in &hir_crate.items {
            match item {
                HirItem::Function(func) => {
                    let mir_func = self.lower_function(func)?;
                    body.push_function(mir_func);
                }
                _ => {
                    // Skip non-function items for now
                    // TODO: Handle structs, enums, traits, impls
                }
            }
        }

        Ok(body)
    }

    /// Lower a HIR function to MIR function
    pub fn lower_function(&mut self, func: &HirFunction) -> Result<MirFunction> {
        // Convert return type: if function has error_type (T | E syntax),
        // convert to Outcome<T, E>
        let return_type = if let Some(_error_ty) = &func.error_type {
            // Function uses T | E syntax, convert to Outcome<T, E>
            // Represent Outcome as a struct (matches LIR's representation)
            crate::ty::MirTy::Struct {
                name: "Outcome".to_string(),
            }
        } else {
            // Normal function, just convert return type
            func.return_type.clone().into()
        };

        // Create MIR function
        let mut mir_func = MirFunction::new(
            func.name.clone(),
            func.params.iter().map(|p| MirParam {
                name: p.name.clone(),
                ty: p.ty.clone().into(),
            }).collect(),
            return_type,
        );

        // Lower function body
        let entry_block = mir_func.entry_block;
        let (return_block, return_temp) = self.lower_block(&mut mir_func, &func.body, entry_block, true)?;

        // Set return terminator
        let block = mir_func.blocks.get_mut(&return_block).unwrap();
        let return_place = return_temp.map(|t| MirPlace::Temp(t));
        block.set_terminator(MirTerminator::Return(return_place));

        Ok(mir_func)
    }

    /// Lower a HIR block to MIR basic blocks
    ///
    /// Returns (final_block_id, optional_temp_var_for_last_expr)
    fn lower_block(
        &self,
        func: &mut MirFunction,
        block: &HirBlock,
        entry_block: MirNodeId,
        is_func_body: bool,
    ) -> Result<(MirNodeId, Option<TempVar>)> {
        let mut current_block = entry_block;

        // Process statements
        for stmt in &block.statements {
            self.lower_statement(func, &mut current_block, stmt)?;
        }

        // Handle trailing expression (if any)
        let last_temp = if let Some(expr) = &block.trailing_expr {
            Some(self.lower_expression(func, &mut current_block, expr)?)
        } else if is_func_body {
            // Unit return for empty function body
            let temp = func.alloc_temp();
            let block_obj = func.blocks.get_mut(&current_block).unwrap();
            block_obj.push_instruction(MirInstruction::Const {
                dest: temp,
                value: MirConstant::Unit,
                ty: MirTy::Unit,
            });
            Some(temp)
        } else {
            None
        };

        Ok((current_block, last_temp))
    }

    /// Lower a HIR statement to MIR instructions
    fn lower_statement(
        &self,
        func: &mut MirFunction,
        current_block: &mut MirNodeId,
        stmt: &HirStatement,
    ) -> Result<()> {
        match stmt {
            HirStatement::Local(local) => {
                // Handle local variable declaration
                if let Some(init) = &local.init {
                    let temp = self.lower_expression(func, current_block, init)?;
                    // Store to local
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.push_instruction(MirInstruction::Store {
                        dest: MirPlace::Local(local.name.clone()),
                        src: temp,
                        ty: init.ty().clone().into(),
                    });
                }
            }
            HirStatement::Expression(expr) => {
                // Expression statement - just lower it
                self.lower_expression(func, current_block, expr)?;
            }
            HirStatement::Semi(expr) => {
                // Semicolon expression - lower it, but need to handle control flow specially
                // Check if this is a Loop or If expression that creates new blocks
                let is_control_flow = matches!(expr, HirExpression::Loop { .. } | HirExpression::If { .. });

                let old_block = *current_block;
                self.lower_expression(func, current_block, expr)?;

                // If the expression created new blocks (Loop/If), we need to connect the old block
                // to the new control flow's entry point
                if is_control_flow && *current_block != old_block {
                    // The expression changed the current_block, which means it created new blocks
                    // We need to add a terminator from old_block to the new entry point
                    let block_obj = func.blocks.get_mut(&old_block).unwrap();
                    if block_obj.terminator.is_none() {
                        // Add Goto to the new entry point
                        block_obj.set_terminator(MirTerminator::Goto { target: *current_block });
                    }
                }
            }
            HirStatement::Item(_item) => {
                // TODO: Handle nested items
            }
        }
        Ok(())
    }

    /// Lower a HIR expression to MIR instructions
    ///
    /// Returns the temporary variable containing the result
    fn lower_expression(
        &self,
        func: &mut MirFunction,
        current_block: &mut MirNodeId,
        expr: &HirExpression,
    ) -> Result<TempVar> {
        match expr {
            // Literals
            HirExpression::Literal(lit, _id, _ty, _span) => {
                let temp = func.alloc_temp();
                let (value, ty) = self.lower_literal(lit)?;
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.push_instruction(MirInstruction::Const {
                    dest: temp,
                    value,
                    ty,
                });
                Ok(temp)
            }

            // Variables
            HirExpression::Variable(name, _id, _ty, _span) => {
                let temp = func.alloc_temp();
                let ty = expr.ty().clone().into();
                let block_obj = func.blocks.get_mut(current_block).unwrap();

                // Load from local
                block_obj.push_instruction(MirInstruction::Load {
                    dest: temp,
                    src: MirPlace::Local(name.clone()),
                    ty,
                });
                Ok(temp)
            }

            // Binary operations
            HirExpression::BinaryOp { op, left, right, ty, span: _ } => {
                // Special handling for assignment: x = expr
                if *op == zulon_hir::HirBinOp::Assign {
                    // Lower the right-hand side (the value being assigned)
                    let value_temp = self.lower_expression(func, current_block, right)?;

                    // For the left-hand side, we need to extract the variable name
                    // Currently, this expects left to be a simple variable reference
                    if let HirExpression::Variable(name, ..) = &**left {
                        // Store the value to the variable
                        let mir_ty = ty.clone().into();
                        let block_obj = func.blocks.get_mut(current_block).unwrap();
                        block_obj.push_instruction(MirInstruction::Store {
                            dest: MirPlace::Local(name.clone()),
                            src: value_temp,
                            ty: mir_ty,
                        });

                        // Assignment returns the assigned value (in ZULON)
                        Ok(value_temp)
                    } else {
                        return Err(MirError::LoweringError(
                            format!("Assignment left-hand side must be a variable, found: {:?}", left)
                        ));
                    }
                } else {
                    // Regular binary operation
                    let left_temp = self.lower_expression(func, current_block, left)?;
                    let right_temp = self.lower_expression(func, current_block, right)?;

                    let result_temp = func.alloc_temp();
                    let mir_ty = ty.clone().into();
                    let mir_op = self.lower_bin_op(*op);

                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.push_instruction(MirInstruction::BinaryOp {
                        dest: result_temp,
                        op: mir_op,
                        left: left_temp,
                        right: right_temp,
                        ty: mir_ty,
                    });
                    Ok(result_temp)
                }
            }

            // Unary operations
            HirExpression::UnaryOp { op, operand, ty, span: _ } => {
                let operand_temp = self.lower_expression(func, current_block, operand)?;

                let result_temp = func.alloc_temp();
                let mir_ty = ty.clone().into();
                let mir_op = self.lower_unary_op(*op);

                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.push_instruction(MirInstruction::UnaryOp {
                    dest: result_temp,
                    op: mir_op,
                    operand: operand_temp,
                    ty: mir_ty,
                });
                Ok(result_temp)
            }

            // Function calls
            HirExpression::Call { func: func_expr, args, ty, span: _ } => {
                // Lower function name
                let func_name = match func_expr.as_ref() {
                    HirExpression::Variable(name, _id, _ty, _span) => name.clone(),
                    _ => return Err(MirError::LoweringError(
                        "Complex function expressions not supported yet".to_string()
                    )),
                };

                // Lower arguments
                let mut arg_temps = Vec::new();
                for arg in args {
                    let arg_temp = self.lower_expression(func, current_block, arg)?;
                    arg_temps.push(arg_temp);
                }

                // Allocate result temp (or None if unit)
                let return_ty = ty.clone().into();
                let dest_temp = if return_ty != MirTy::Unit {
                    Some(func.alloc_temp())
                } else {
                    None
                };

                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.push_instruction(MirInstruction::Call {
                    dest: dest_temp,
                    func: MirPlace::Local(func_name),
                    args: arg_temps.into_iter().map(|t| MirPlace::Temp(t)).collect(),
                    return_type: return_ty,
                });

                Ok(dest_temp.unwrap_or_else(|| func.alloc_temp()))
            }

            // Blocks
            HirExpression::Block(block) => {
                let new_block = func.alloc_block();
                let old_block = *current_block;

                // Jump to new block
                let block_obj = func.blocks.get_mut(&old_block).unwrap();
                block_obj.set_terminator(MirTerminator::Goto { target: new_block });

                *current_block = new_block;
                let (_, temp) = self.lower_block(func, block, new_block, false)?;

                Ok(temp.unwrap_or_else(|| {
                    // Empty block, return unit
                    let temp = func.alloc_temp();
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.push_instruction(MirInstruction::Const {
                        dest: temp,
                        value: MirConstant::Unit,
                        ty: MirTy::Unit,
                    });
                    temp
                }))
            }

            // If expressions
            HirExpression::If { condition, then_block, else_block, ty: _, span: _ } => {
                let cond_temp = self.lower_expression(func, current_block, condition)?;

                let then_block_id = func.alloc_block();
                let else_block_id = if else_block.is_some() {
                    func.alloc_block()
                } else {
                    func.alloc_block()
                };
                let join_block_id = func.alloc_block();

                // Set conditional terminator
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::If {
                    condition: cond_temp,
                    then_block: then_block_id,
                    else_block: else_block_id,
                });

                // Lower then block
                *current_block = then_block_id;
                let (_, then_temp) = self.lower_block(func, then_block, then_block_id, false)?;
                let then_temp = then_temp.unwrap_or_else(|| func.alloc_temp());
                let then_block_obj = func.blocks.get_mut(&then_block_id).unwrap();
                then_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });

                // Lower else block if present
                let else_temp: TempVar = if let Some(else_blk) = else_block {
                    *current_block = else_block_id;
                    let (_, et) = self.lower_block(func, else_blk, else_block_id, false)?;
                    let et = et.unwrap_or_else(|| func.alloc_temp());
                    let else_block_obj = func.blocks.get_mut(&else_block_id).unwrap();
                    else_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });
                    et
                } else {
                    // No else block, just goto join (returns unit)
                    let et = func.alloc_temp();
                    let else_block_obj = func.blocks.get_mut(&else_block_id).unwrap();
                    else_block_obj.push_instruction(MirInstruction::Const {
                        dest: et,
                        value: MirConstant::Unit,
                        ty: MirTy::Unit,
                    });
                    else_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });
                    et
                };

                // For now, create a Move that will be converted to Phi during LIR lowering
                *current_block = join_block_id;
                let result_temp = func.alloc_temp();
                let join_block_obj = func.blocks.get_mut(&join_block_id).unwrap();

                // TODO: Implement proper Phi nodes
                // For now, just move the then value - this is incorrect for else paths
                join_block_obj.push_instruction(MirInstruction::Move {
                    dest: result_temp,
                    src: MirPlace::Temp(then_temp),
                });

                // Note: else_temp is available but not used (would need Phi)
                let _ = else_temp; // Suppress unused warning

                Ok(result_temp)
            }

            // Loop
            HirExpression::Loop { body, ty: _, span: _ } => {
                let loop_head = func.alloc_block();
                let loop_body = func.alloc_block();
                let exit_block = func.alloc_block();

                // Jump from current to loop head
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Goto { target: loop_head });

                // Loop head: just goto body for now
                *current_block = loop_head;
                let head_block_obj = func.blocks.get_mut(&loop_head).unwrap();
                head_block_obj.set_terminator(MirTerminator::Goto { target: loop_body });

                // Lower body (this returns the final block ID after all statements)
                let (final_block_id, body_temp) = self.lower_block(func, body, loop_body, false)?;
                let _body_temp = body_temp.unwrap_or_else(|| func.alloc_temp());

                // After lowering the body, final_block_id might be different from loop_body
                // (e.g., if the body had an If expression or nested loop that created new blocks)
                let final_block_obj = func.blocks.get_mut(&final_block_id).unwrap();

                // Check if final_block already has a terminator
                if final_block_obj.terminator.is_none() {
                    // No terminator - add loop-back to create the actual loop
                    final_block_obj.set_terminator(MirTerminator::Goto { target: loop_head });
                } else {
                    // Block already has a terminator - this means the trailing expression
                    // was a control flow construct (If, Loop, etc.)
                    //
                    // For nested loops:
                    // - final_block_id is the nested loop's exit_block
                    // - The nested loop set current_block = exit_block but didn't add a terminator
                    // - We need to add a terminator to loop back to the outer loop head
                    //
                    // For If expressions:
                    // - final_block_id is the join block where then/else converge
                    // - It should already loop back or have appropriate terminator
                }

                // Set current to exit (though loop never exits in this simple version)
                *current_block = exit_block;

                // Loop never returns, so return never type temp
                let never_temp = func.alloc_temp();
                Ok(never_temp)
            }

            // Break
            HirExpression::Break(_expr, _span) => {
                // TODO: Handle proper loop exit
                // For now, just unreachable
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Unreachable);
                Ok(func.alloc_temp())
            }

            // Continue
            HirExpression::Continue(_span) => {
                // TODO: Handle proper loop continue
                // For now, just unreachable
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Unreachable);
                Ok(func.alloc_temp())
            }

            HirExpression::Return(_expr, _span) => {
                // TODO: Handle return properly
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Return(None));
                Ok(func.alloc_temp())
            }

            // Throw statement (error handling)
            HirExpression::Throw(error_expr, _span) => {
                // Lower the error expression to get its temporary
                let error_temp = self.lower_expression(func, current_block, error_expr)?;

                // Throw returns the error value (similar to return but with error)
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Return(Some(MirPlace::Temp(error_temp))));

                // Throw doesn't produce a value (Never type), but we need to return something
                // This temp will never be used since throw ends execution
                Ok(func.alloc_temp())
            }

            // Question mark operator (error propagation)
            HirExpression::QuestionMark(inner_expr, _ty, _span) => {
                // Lower the inner expression (should be Outcome<T, E>)
                let outcome_temp = self.lower_expression(func, current_block, inner_expr)?;

                // Create basic blocks for:
                // - Success path (extract T and continue)
                // - Error path (return E)
                let success_block = func.alloc_block();
                let error_block = func.alloc_block();

                // Allocate all temps first (to avoid borrow checker issues)
                let discriminant_temp = func.alloc_temp();
                let zero_temp = func.alloc_temp();
                let is_ok_temp = func.alloc_temp();
                let result_temp = func.alloc_temp();
                let error_temp = func.alloc_temp();
                let continue_block = func.alloc_block();

                // Current block: Load discriminant from Outcome and check it
                // Outcome<T, E> layout: { discriminant (i8), data }
                // We load the discriminant to check if it's Ok (0) or Err (1)
                {
                    let block_obj = func.blocks.get_mut(current_block).unwrap();

                    // Load discriminant (first field of Outcome)
                    block_obj.push_instruction(MirInstruction::Load {
                        dest: discriminant_temp,
                        src: MirPlace::Field {
                            base: Box::new(MirPlace::Temp(outcome_temp)),
                            field: "discriminant".to_string(),  // Convention: discriminant field
                        },
                        ty: MirTy::I8,
                    });

                    // Create constant 0 for comparison
                    block_obj.push_instruction(MirInstruction::Const {
                        dest: zero_temp,
                        value: MirConstant::Integer(0),
                        ty: MirTy::I8,
                    });

                    // Compare discriminant to 0 (Ok variant)
                    block_obj.push_instruction(MirInstruction::BinaryOp {
                        dest: is_ok_temp,
                        op: MirBinOp::Eq,
                        left: discriminant_temp,
                        right: zero_temp,
                        ty: MirTy::Bool,
                    });

                    // Branch: if discriminant == 0 goto success_block else goto error_block
                    block_obj.set_terminator(MirTerminator::If {
                        condition: is_ok_temp,
                        then_block: success_block,
                        else_block: error_block,
                    });
                }

                // Success block: extract T from Outcome::Ok(T)
                {
                    *current_block = success_block;
                    let success_block_obj = func.blocks.get_mut(&success_block).unwrap();
                    // Load the data field (field 1, after discriminant)
                    success_block_obj.push_instruction(MirInstruction::Load {
                        dest: result_temp,
                        src: MirPlace::Field {
                            base: Box::new(MirPlace::Temp(outcome_temp)),
                            field: "data".to_string(),  // Convention: data field
                        },
                        ty: _ty.clone().into(),  // Success type T (HirTy → MirTy)
                    });

                    // Continue to next statement
                    success_block_obj.set_terminator(MirTerminator::Goto { target: continue_block });
                }

                // Error block: return E from Outcome::Err(E)
                {
                    *current_block = error_block;
                    let error_block_obj = func.blocks.get_mut(&error_block).unwrap();
                    // Load the error data and return it
                    error_block_obj.push_instruction(MirInstruction::Load {
                        dest: error_temp,
                        src: MirPlace::Field {
                            base: Box::new(MirPlace::Temp(outcome_temp)),
                            field: "data".to_string(),  // Same data field, but contains E
                        },
                        ty: MirTy::I32,  // TODO: Get actual error type
                    });
                    error_block_obj.set_terminator(MirTerminator::Return(Some(MirPlace::Temp(error_temp))));
                }

                // Set current to continue block for subsequent code
                *current_block = continue_block;

                Ok(result_temp)
            }

            // While loop
            HirExpression::While { condition, body, span: _ } => {
                // Create blocks for header, body, and exit
                let header_block = func.alloc_block();
                let body_block = func.alloc_block();
                let exit_block = func.alloc_block();

                // Current block jumps to header
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Goto { target: header_block });

                // Header block: check condition
                *current_block = header_block;
                let cond_temp = self.lower_expression(func, current_block, condition)?;
                let header_obj = func.blocks.get_mut(&header_block).unwrap();
                header_obj.set_terminator(MirTerminator::If {
                    condition: cond_temp,
                    then_block: body_block,
                    else_block: exit_block,
                });

                // Body block: execute loop body, then jump back to header
                *current_block = body_block;
                let (final_block_id, _) = self.lower_block(func, body, body_block, false)?;

                // Add loop-back terminator to the final block of the body
                // This handles the case where the body contains nested control flow
                let final_body_obj = func.blocks.get_mut(&final_block_id).unwrap();
                if final_body_obj.terminator.is_none() {
                    // No terminator yet - add loop-back to header
                    final_body_obj.set_terminator(MirTerminator::Goto { target: header_block });
                }
                // If final_block already has a terminator, it's from a nested construct
                // In that case, we need to ensure the nested construct loops back properly

                // Exit block: continue after loop
                *current_block = exit_block;

                // While loops return unit
                let unit_temp = func.alloc_temp();
                let exit_obj = func.blocks.get_mut(&exit_block).unwrap();
                exit_obj.push_instruction(MirInstruction::Const {
                    dest: unit_temp,
                    value: MirConstant::Unit,
                    ty: MirTy::Unit,
                });
                Ok(unit_temp)
            }

            // For loop: for pattern in iterator { body }
            // Desugars to: match iterator.next() { Some(pattern) => { body; continue }, None => break }
            // For MVP: Emit a helpful error message
            HirExpression::For { pattern, iter, body: _, span: _ } => {
                // Simplified for loop: desugar to while loop
                // Temporarily supports a simpler form until Range is fully implemented
                // For now, we'll return a helpful error message

                return Err(MirError::LoweringError(
                    format!("For loops are not yet fully implemented in this version. \
                            Please use 'while' loops instead. \
                            Example: 'let mut i = start; while i < end {{ ...; i = i + 1 }};'. \
                            Pattern: {:?}, Iterator: {:?}", pattern, iter)
                ));
            }

            _ => {
                return Err(MirError::LoweringError(
                    format!("Unsupported expression: {:?}", expr)
                ))
            }
        }
    }

    /// Lower a literal to MIR constant
    fn lower_literal(&self, lit: &zulon_hir::HirLiteral) -> Result<(MirConstant, MirTy)> {
        match lit {
            zulon_hir::HirLiteral::Bool(b) => Ok((MirConstant::Bool(*b), MirTy::Bool)),
            zulon_hir::HirLiteral::Integer(i) => Ok((MirConstant::Integer(*i as i128), MirTy::I32)),
            zulon_hir::HirLiteral::Float(f) => Ok((MirConstant::Float(*f), MirTy::F64)),
            zulon_hir::HirLiteral::String(s) => Ok((MirConstant::String(s.clone()), MirTy::String)),
            zulon_hir::HirLiteral::Char(c) => Ok((MirConstant::Char(*c), MirTy::Char)),
            zulon_hir::HirLiteral::Unit => Ok((MirConstant::Unit, MirTy::Unit)),
        }
    }

    /// Lower a binary operator
    fn lower_bin_op(&self, op: zulon_hir::HirBinOp) -> MirBinOp {
        match op {
            zulon_hir::HirBinOp::Add => MirBinOp::Add,
            zulon_hir::HirBinOp::Sub => MirBinOp::Sub,
            zulon_hir::HirBinOp::Mul => MirBinOp::Mul,
            zulon_hir::HirBinOp::Div => MirBinOp::Div,
            zulon_hir::HirBinOp::Mod => MirBinOp::Mod,
            zulon_hir::HirBinOp::BitAnd => MirBinOp::BitAnd,
            zulon_hir::HirBinOp::BitOr => MirBinOp::BitOr,
            zulon_hir::HirBinOp::BitXor => MirBinOp::BitXor,
            zulon_hir::HirBinOp::LeftShift => MirBinOp::LeftShift,
            zulon_hir::HirBinOp::RightShift => MirBinOp::RightShift,
            zulon_hir::HirBinOp::And => MirBinOp::And,
            zulon_hir::HirBinOp::Or => MirBinOp::Or,
            zulon_hir::HirBinOp::Eq => MirBinOp::Eq,
            zulon_hir::HirBinOp::NotEq => MirBinOp::NotEq,
            zulon_hir::HirBinOp::Less => MirBinOp::Less,
            zulon_hir::HirBinOp::LessEq => MirBinOp::LessEq,
            zulon_hir::HirBinOp::Greater => MirBinOp::Greater,
            zulon_hir::HirBinOp::GreaterEq => MirBinOp::GreaterEq,
            // Assign should never reach here - it's handled specially in BinaryOp lowering
            zulon_hir::HirBinOp::Assign => unreachable!("Assign operator should be handled before lower_bin_op"),
        }
    }

    /// Lower a unary operator
    fn lower_unary_op(&self, op: zulon_hir::HirUnaryOp) -> MirUnaryOp {
        match op {
            zulon_hir::HirUnaryOp::Neg => MirUnaryOp::Neg,
            zulon_hir::HirUnaryOp::Not => MirUnaryOp::Not,
            zulon_hir::HirUnaryOp::Deref => MirUnaryOp::Deref,
            zulon_hir::HirUnaryOp::Ref => MirUnaryOp::Ref,
            zulon_hir::HirUnaryOp::RefMut => MirUnaryOp::RefMut,
        }
    }
}

impl Default for MirLoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Public API for lowering HIR to MIR
pub fn lower_hir(hir_crate: &HirCrate) -> Result<MirBody> {
    let mut ctx = MirLoweringContext::new();
    ctx.lower_crate(hir_crate)
}
