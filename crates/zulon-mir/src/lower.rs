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
use zulon_hir::{HirCrate, HirItem, HirFunction, HirExpression, HirBlock, HirStatement, HirTy};

/// Loop context for tracking break/continue targets
struct LoopContext {
    /// Exit block for break statements
    exit_block: MirNodeId,
    /// Head block for continue statements (loop header or while condition)
    head_block: MirNodeId,
}

/// Defer context for tracking deferred statements
struct DeferContext {
    /// Block ID where the defer was registered
    block_id: MirNodeId,
    /// The deferred statement
    statement: HirStatement,
}

/// Context for lowering HIR to MIR
pub struct MirLoweringContext {
    /// Struct definitions: name -> (field_names, field_indices)
    struct_defs: std::collections::HashMap<String, Vec<String>>,
    /// Loop context stack (for nested loops)
    loop_stack: Vec<LoopContext>,
    /// Defer statement stack (for cleanup blocks)
    defer_stack: Vec<DeferContext>,
}

impl MirLoweringContext {
    /// Create a new lowering context
    pub fn new() -> Self {
        MirLoweringContext {
            struct_defs: std::collections::HashMap::new(),
            loop_stack: Vec::new(),
            defer_stack: Vec::new(),
        }
    }

    /// Lower an entire HIR crate to MIR body
    pub fn lower_crate(&mut self, hir_crate: &HirCrate) -> Result<MirBody> {
        let mut body = MirBody::new();

        // First pass: collect struct definitions
        for item in &hir_crate.items {
            if let HirItem::Struct(struct_def) = item {
                let field_names: Vec<String> = struct_def.fields.iter()
                    .map(|field| field.name.clone())
                    .collect();
                self.struct_defs.insert(struct_def.name.clone(), field_names);
            }
        }

        // Second pass: lower functions
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
        // Convert return type from HIR to MIR
        // Note: HIR lowering already converts T | E to Outcome<T, E>,
        // so we just need to convert the HIR type to MIR type
        let return_type: crate::ty::MirTy = func.return_type.clone().into();

        // Create MIR function
        let mut mir_func = MirFunction::new(
            func.name.clone(),
            func.params.iter().map(|p| MirParam {
                name: p.name.clone(),
                ty: p.ty.clone().into(),
            }).collect(),
            return_type.clone(),
        );

        // Set async flag
        mir_func.is_async = func.is_async;

        // For async functions, create a state machine
        if func.is_async {
            let state_machine = AsyncStateMachine::new(return_type);
            mir_func.state_machine = Some(state_machine);
        }

        // Extract effect names from function's effects
        // These are the effects this function performs (e.g., Log in fn() -> i32 | Log)
        let effect_names: Vec<String> = func.effects.iter()
            .filter_map(|ty| match ty {
                HirTy::Struct { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // Store effect names in MIR function for use during lowering
        mir_func.effects = effect_names.clone();

        // Lower function body
        let entry_block = mir_func.entry_block;
        let (return_block, return_temp) = self.lower_block(&mut mir_func, &func.body, entry_block, true)?;

        // Set return terminator ONLY if the trailing expression didn't already set one
        // (e.g., Return or Throw expressions set their own terminators)
        let block = mir_func.blocks.get_mut(&return_block).unwrap();
        if block.terminator.is_none() {
            // No terminator set yet, set it based on trailing expression
            let return_place = return_temp.map(|t| MirPlace::Temp(t));
            block.set_terminator(MirTerminator::Return(return_place));
        }
        // Else: terminator already set by Return/Throw, don't override it

        Ok(mir_func)
    }

    /// Lower a HIR block to MIR basic blocks
    ///
    /// Returns (final_block_id, optional_temp_var_for_last_expr)
    fn lower_block(
        &mut self,
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

        // Execute deferred statements in reverse order (LIFO)
        // For MVP: Execute at block end
        // Limitation: Doesn't handle early returns, errors, or nested blocks
        let defers_to_execute: Vec<HirStatement> = self.defer_stack.iter()
            .filter(|d| d.block_id == entry_block)
            .rev()
            .map(|d| d.statement.clone())
            .collect();

        for defer_stmt in defers_to_execute {
            // Lower the deferred statement
            // Note: We use current_block which may have changed during the block
            // This is a simplification - proper implementation needs cleanup blocks
            drop(self.lower_statement(func, &mut current_block, &defer_stmt));
        }

        Ok((current_block, last_temp))
    }

    /// Lower a HIR statement to MIR instructions
    fn lower_statement(
        &mut self,
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
                // Semicolon expression - lower it
                let old_block = *current_block;
                self.lower_expression(func, current_block, expr)?;

                // If the expression created new blocks (Loop/If/Try/EffectCall), the current_block changed
                // Effect operations create continuation blocks, so we handle them here
                if *current_block != old_block {
                    // The expression changed the current_block, which means it created new blocks
                    // The old_block should already have a terminator (set by the expression)
                    // For example, effect operations set a Goto to handler, so we don't need to do anything
                }
            }
            HirStatement::Item(_item) => {
                // TODO: Handle nested items
            }
            HirStatement::Defer(stmt) => {
                // Defer statements are handled by registering cleanup actions
                // The cleanup will be executed when exiting the scope
                //
                // For MVP: Track defer statements and execute them at block end
                // Full implementation: Handle early returns, errors, panics

                // Register the defer statement for the current block
                self.defer_stack.push(DeferContext {
                    block_id: *current_block,
                    statement: (**stmt).clone(),
                });

                // The defer statement itself doesn't generate any code here
                // The cleanup code will be generated when we exit the block
            }
        }
        Ok(())
    }

    /// Lower a HIR expression to MIR instructions
    ///
    /// Returns the temporary variable containing the result
    fn lower_expression(
        &mut self,
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
                // Check if this is an enum variant (e.g., "DivideError::Zero")
                if name.contains("::") {
                    // For MVP: Treat enum variants as constant discriminant values
                    // Extract the variant name and use a simple heuristic
                    let variant_name = name.split("::").last().unwrap_or("");

                    // Simple heuristic: "Zero" or similar -> 0, can be improved later
                    let discriminant = if variant_name == "Zero" || variant_name == "None" {
                        0
                    } else if variant_name == "One" || variant_name == "Some" {
                        1
                    } else {
                        // For other variants, use a hash-based or sequential approach
                        // For now, default to 0
                        0
                    };

                    // Generate a constant instruction
                    let temp = func.alloc_temp();
                    let mir_ty = expr.ty().clone().into();
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.push_instruction(MirInstruction::Const {
                        dest: temp,
                        value: MirConstant::Integer(discriminant as i128),
                        ty: mir_ty,
                    });
                    Ok(temp)
                } else {
                    // Normal variable - load from local
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

                // Check if this function has any declared effects OR has handlers registered
                // Effects can be declared in function signature (fn() -> i32 | Log)
                // OR handlers can be present in try...with blocks
                let has_effects = !func.effects.is_empty();
                let has_handlers = !func.handlers.is_empty();

                // Check if this call is to an effect operation
                // We use a heuristic: if the function has effects/handlers and the
                // function name matches common effect operation names, treat it as an effect call
                let is_effect_operation = (has_effects || has_handlers)
                    && self.is_effect_operation_name(&func_name);

                if is_effect_operation {
                    // Look for a handler for this effect operation
                    // Handlers are registered in the function's handlers list
                    let handler_info = func.handlers.iter()
                        .find(|h| h.methods.contains_key(&func_name))
                        .and_then(|h| h.methods.get(&func_name))
                        .map(|(handler_block, resume_block)| (*handler_block, *resume_block));

                    if let Some((handler_block, resume_block)) = handler_info {
                        // Handler exists - jump to it
                        let block_obj = func.blocks.get_mut(current_block).unwrap();
                        block_obj.set_terminator(MirTerminator::Goto {
                            target: handler_block,
                        });

                        // After handler completes, execution resumes at resume_block
                        // The trailing expression should be lowered into resume_block
                        *current_block = resume_block;

                        // For unit effect operations (dest_temp is None), don't create a temp
                        // For operations with return values, return the destination temp
                        if let Some(dt) = dest_temp {
                            Ok(dt)
                        } else {
                            // Unit operation - return None so statement handling discards it
                            // and the next expression in the block provides the value
                            Ok(func.alloc_temp())  // Placeholder, will be discarded
                        }
                    } else {
                        // No handler found - jump directly to resume (like before)
                        let resume_block = func.alloc_block();
                        let block_obj = func.blocks.get_mut(current_block).unwrap();
                        block_obj.set_terminator(MirTerminator::Goto {
                            target: resume_block,
                        });

                        *current_block = resume_block;
                        Ok(dest_temp.unwrap_or_else(|| func.alloc_temp()))
                    }
                } else {
                    // Regular function call - generate Call instruction
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.push_instruction(MirInstruction::Call {
                        dest: dest_temp,
                        func: MirPlace::Local(func_name),
                        args: arg_temps.into_iter().map(|t| MirPlace::Temp(t)).collect(),
                        return_type: return_ty,
                    });

                    Ok(dest_temp.unwrap_or_else(|| func.alloc_temp()))
                }
            }

            // Field access (e.g., object.field_name)
            HirExpression::Field { base, field_name, ty, span: _ } => {
                // Lower the base expression
                let base_temp = self.lower_expression(func, current_block, base)?;

                // Get the base expression's type (which should be a struct)
                let base_ty = base.ty();

                // Get the field index from the struct type
                let field_index = self.get_field_index(base_ty, field_name)?;

                // Allocate result temp
                let result_temp = func.alloc_temp();
                let mir_ty = ty.clone().into();

                // Generate FieldAccess instruction
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.push_instruction(MirInstruction::FieldAccess {
                    dest: result_temp,
                    base: base_temp,
                    field_name: field_name.clone(),
                    field_index,
                    ty: mir_ty,
                });

                Ok(result_temp)
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
            HirExpression::If { condition, then_block, else_block, ty, span: _ } => {
                let cond_temp = self.lower_expression(func, current_block, condition)?;

                // Check if this is a Unit-type if without an explicit else branch
                let mir_ty: MirTy = ty.clone().into();
                let is_unit_statement = matches!(mir_ty, MirTy::Unit);
                let has_explicit_else = else_block.is_some();

                eprintln!("DEBUG: If expression - is_unit_statement={}, has_explicit_else={}, ty={:?}", is_unit_statement, has_explicit_else, ty);

                // Special case: Unit-type if without else branch
                // Don't create join block - let then fall through to continuation
                if is_unit_statement && !has_explicit_else {
                    eprintln!("DEBUG: Special case triggered for Unit if without else");
                    // For "if cond { stmt }" statements:
                    // - then branch: executes stmt, then falls through to continuation
                    // - else branch (implicit): jumps directly to continuation (skipping then)
                    // - No join block needed - continuation block serves as the merge point
                    
                    let then_block_id = func.alloc_block();
                    let continuation_block = func.alloc_block();

                    // Set conditional terminator: then_block vs continuation
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.set_terminator(MirTerminator::If {
                        condition: cond_temp,
                        then_block: then_block_id,
                        else_block: continuation_block,  // Implicit else skips to continuation
                    });

                    // Lower then block (no terminator - falls through to continuation)
                    *current_block = then_block_id;
                    let (then_final_block, _then_temp) = self.lower_block(func, then_block, then_block_id, false)?;

                    // Ensure then block's final block branches to continuation
                    let then_final_block_obj = func.blocks.get_mut(&then_final_block).unwrap();
                    if then_final_block_obj.terminator.is_none() {
                        // Then block falls through to continuation
                        then_final_block_obj.set_terminator(MirTerminator::Goto { 
                            target: continuation_block 
                        });
                    }

                    // Set continuation as current block for subsequent statements
                    *current_block = continuation_block;
                    return Ok(func.alloc_temp());
                }

                // General case: value-producing if or if with explicit else
                // Allocate the standard three blocks: then, else, join
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
                let (then_final_block, then_temp) = self.lower_block(func, then_block, then_block_id, false)?;

                // For Unit-type if expressions (statements), check the final block for terminators
                // For value-producing if expressions, check the initial block
                let mir_ty: MirTy = ty.clone().into();
                let check_final_block = matches!(mir_ty, MirTy::Unit);

                let then_block_obj = if check_final_block {
                    func.blocks.get_mut(&then_final_block).unwrap()
                } else {
                    func.blocks.get_mut(&then_block_id).unwrap()
                };
                let then_has_term = then_block_obj.terminator.is_some();
                // Only set terminator if block doesn't already have one (e.g., from break/continue)
                if !then_has_term {
                    then_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });
                }

                // Lower else block if present
                let (_else_final_block, _else_temp, else_has_term): (MirNodeId, TempVar, bool) = if let Some(else_blk) = else_block {
                    *current_block = else_block_id;
                    let (final_block, et) = self.lower_block(func, else_blk, else_block_id, false)?;
                    let et = et.unwrap_or_else(|| func.alloc_temp());

                    let else_block_obj = if check_final_block {
                        func.blocks.get_mut(&final_block).unwrap()
                    } else {
                        func.blocks.get_mut(&else_block_id).unwrap()
                    };
                    let has_term = else_block_obj.terminator.is_some();
                    // Only set terminator if block doesn't already have one (e.g., from break/continue)
                    if !has_term {
                        else_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });
                    }
                    (final_block, et, has_term)
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
                    (else_block_id, et, false) // (final_block, temp, has_term) - implicit else doesn't count as having a term
                };

                // If both branches have terminators (e.g., break/continue), the if expression
                // doesn't produce a value and we don't need a join block with PHI
                if then_has_term && else_has_term {
                    // Both branches have terminators
                    // For Unit-type ifs (statements), the join block is always the continuation
                    // For value-producing ifs (expressions), we need to check the terminators
                    if check_final_block {
                        // Unit type: join block is always the continuation point
                        // Ensure final blocks actually branch to the join block
                        let then_final_block_obj = func.blocks.get_mut(&then_final_block).unwrap();
                        let then_gotos_join = matches!(then_final_block_obj.terminator,
                            Some(MirTerminator::Goto { target }) if target == join_block_id);
                        if !then_gotos_join {
                            then_final_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });
                        }

                        // Same for else final block if it exists
                        let else_final_block_obj = func.blocks.get_mut(&_else_final_block).unwrap();
                        let else_gotos_join = matches!(else_final_block_obj.terminator,
                            Some(MirTerminator::Goto { target }) if target == join_block_id);
                        if !else_gotos_join {
                            else_final_block_obj.set_terminator(MirTerminator::Goto { target: join_block_id });
                        }

                        *current_block = join_block_id;
                        let dummy_temp = func.alloc_temp();
                        Ok(dummy_temp)
                    } else {
                        // Value-producing type: check if both branches end with Return
                        let then_block_obj = func.blocks.get(&then_block_id).unwrap();
                        let else_block_obj = func.blocks.get(&else_block_id).unwrap();

                        let then_has_return = matches!(then_block_obj.terminator, Some(MirTerminator::Return(_)));
                        let else_has_return = matches!(else_block_obj.terminator, Some(MirTerminator::Return(_)));

                        if then_has_return && else_has_return {
                            // Both branches return - the join block is unreachable
                            let dummy_temp = func.alloc_temp();
                            Ok(dummy_temp)
                        } else {
                            // At least one branch doesn't return
                            // The join block is the continuation
                            *current_block = join_block_id;
                            let dummy_temp = func.alloc_temp();
                            Ok(dummy_temp)
                        }
                    }
                } else {
                    // At least one branch doesn't have a terminator, create join block with Move/Phi
                    *current_block = join_block_id;
                    let result_temp = func.alloc_temp();
                    let join_block_obj = func.blocks.get_mut(&join_block_id).unwrap();

                    // Check if this if expression produces a meaningful value (non-Unit)
                    // If it's Unit, we don't need PHI nodes - just return a dummy temp
                    let mir_ty: MirTy = ty.clone().into();
                    if matches!(mir_ty, MirTy::Unit) {
                        // If expression doesn't produce a meaningful value
                        // No need for PHI nodes in the join block
                        Ok(result_temp)
                    } else if then_temp.is_some() {
                        // At least one branch produces a value
                        // Use then_temp if it exists, else use else_temp
                        let src_temp = then_temp.unwrap();

                        join_block_obj.push_instruction(MirInstruction::Move {
                            dest: result_temp,
                            src: MirPlace::Temp(src_temp),
                        });

                        Ok(result_temp)
                    } else {
                        // Neither branch produces a value (or only else produces value)
                        Ok(result_temp)
                    }
                }
            }

            // Loop
            HirExpression::Loop { body, ty: _, span: _ } => {
                let loop_head = func.alloc_block();
                let loop_body = func.alloc_block();
                let exit_block = func.alloc_block();

                // Push loop context onto stack (for break/continue)
                self.loop_stack.push(LoopContext {
                    exit_block,
                    head_block: loop_head,
                });

                // Jump from current to loop head
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Goto { target: loop_head });

                // Loop head: just goto body for now
                *current_block = loop_head;
                let head_block_obj = func.blocks.get_mut(&loop_head).unwrap();
                head_block_obj.set_terminator(MirTerminator::Goto { target: loop_body });

                // Lower body (this returns the final block ID after all statements)
                let (final_block_id, _body_temp) = self.lower_block(func, body, loop_body, false)?;

                // After lowering the body, final_block_id might be different from loop_body
                // (e.g., if the body had an If expression or nested loop that created new blocks)
                let final_block_obj = func.blocks.get_mut(&final_block_id).unwrap();

                // Check if final_block already has a terminator
                if final_block_obj.terminator.is_none() {
                    // No terminator - add loop-back to create the actual loop
                    final_block_obj.set_terminator(MirTerminator::Goto { target: loop_head });
                }

                // Pop loop context from stack
                self.loop_stack.pop();

                // Set current to exit block
                *current_block = exit_block;

                // Loop never returns without break, so return a dummy temp
                // (break will jump to exit_block, so code after loop won't be reached)
                let dummy_temp = func.alloc_temp();
                Ok(dummy_temp)
            }

            // Match expression (simplified - literal patterns only)
            HirExpression::Match { scrutinee, arms, ty: _, span: _ } => {
                // Lower the scrutinee expression
                let scrutinee_temp = self.lower_expression(func, current_block, scrutinee)?;

                // Allocate blocks for each arm
                let mut arm_blocks = Vec::new();
                for _arm in arms {
                    arm_blocks.push(func.alloc_block());
                }
                let default_block = func.alloc_block();
                let join_block = func.alloc_block();

                // Collect targets for switch (literal patterns only)
                let mut switch_targets = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    match &arm.pattern {
                        zulon_hir::HirPattern::Literal(lit, _span) => {
                            // Convert literal to MIR constant
                            let mir_const = self.lower_literal(lit)?.0;
                            switch_targets.push((mir_const, arm_blocks[i]));
                        }
                        zulon_hir::HirPattern::Wildcard(_span) => {
                            // Wildcard goes to default
                            // (handled by not adding to switch_targets)
                        }
                        _ => {
                            // For now, ignore complex patterns
                            // They'll match the default case
                        }
                    }
                }

                // Set switch terminator in current block
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Switch {
                    scrutinee: scrutinee_temp,
                    targets: switch_targets,
                    default: default_block,
                });

                // Lower each arm
                let mut arm_temps = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    *current_block = arm_blocks[i];

                    // TODO: Handle pattern bindings (ignore for now)
                    // Lower the arm body
                    let body_temp = self.lower_expression(func, current_block, &arm.body)?;
                    arm_temps.push(body_temp);

                    // Add goto to join block
                    let arm_block_obj = func.blocks.get_mut(&arm_blocks[i]).unwrap();
                    if arm_block_obj.terminator.is_none() {
                        arm_block_obj.set_terminator(MirTerminator::Goto { target: join_block });
                    }
                }

                // Handle default block (for wildcard or non-literal patterns)
                *current_block = default_block;
                // For now, just use the last arm as default
                let default_temp = if let Some(last_arm) = arms.last() {
                    self.lower_expression(func, current_block, &last_arm.body)?
                } else {
                    func.alloc_temp()
                };

                let default_block_obj = func.blocks.get_mut(&default_block).unwrap();
                if default_block_obj.terminator.is_none() {
                    default_block_obj.set_terminator(MirTerminator::Goto { target: join_block });
                }

                // Join block: create Phi-like value (simplified - use first arm temp)
                *current_block = join_block;
                let result_temp = func.alloc_temp();
                let join_block_obj = func.blocks.get_mut(&join_block).unwrap();

                // TODO: Proper Phi nodes - for now, just move first arm value
                let first_arm_temp = arm_temps.first().copied().unwrap_or(default_temp);
                join_block_obj.push_instruction(MirInstruction::Move {
                    dest: result_temp,
                    src: MirPlace::Temp(first_arm_temp),
                });

                Ok(result_temp)
            }

            // Break
            HirExpression::Break(_expr, _span) => {
                // Get the innermost loop's exit block from the context stack
                if let Some(loop_ctx) = self.loop_stack.last() {
                    let exit_block = loop_ctx.exit_block;
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.set_terminator(MirTerminator::Goto { target: exit_block });
                    // Return a dummy temp (break doesn't produce a value in the current block)
                    Ok(func.alloc_temp())
                } else {
                    // No loop context - this is a compile error, but for now we make it unreachable
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.set_terminator(MirTerminator::Unreachable);
                    Ok(func.alloc_temp())
                }
            }

            // Continue
            HirExpression::Continue(_span) => {
                // Get the innermost loop's head block from the context stack
                if let Some(loop_ctx) = self.loop_stack.last() {
                    let head_block = loop_ctx.head_block;
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.set_terminator(MirTerminator::Goto { target: head_block });
                    // Return a dummy temp (continue doesn't produce a value in the current block)
                    Ok(func.alloc_temp())
                } else {
                    // No loop context - this is a compile error, but for now we make it unreachable
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.set_terminator(MirTerminator::Unreachable);
                    Ok(func.alloc_temp())
                }
            }

            HirExpression::Return(expr_opt, _span) => {
                // Lower the return expression to get its temporary (if present)
                let return_place = match expr_opt {
                    Some(expr) => {
                        let return_temp = self.lower_expression(func, current_block, expr)?;
                        Some(MirPlace::Temp(return_temp))
                    }
                    None => None,
                };

                // Set return terminator with the expression's value
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Return(return_place));

                // Return doesn't produce a value (Never type)
                // Allocate a dummy temp (will be ignored since we already set the terminator)
                Ok(func.alloc_temp())
            }

            // Throw statement (error handling)
            HirExpression::Throw(error_expr, _span) => {
                // Lower the error expression to get its temporary
                let error_temp = self.lower_expression(func, current_block, error_expr)?;

                // Throw creates a Throw terminator (distinguishable from normal Return)
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Throw(MirPlace::Temp(error_temp)));

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
                        ty: MirTy::I32,  // Discriminant is i32 in Outcome struct
                    });

                    // Create constant 0 for comparison
                    block_obj.push_instruction(MirInstruction::Const {
                        dest: zero_temp,
                        value: MirConstant::Integer(0),
                        ty: MirTy::I32,
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

                // Push loop context onto stack (for break/continue)
                self.loop_stack.push(LoopContext {
                    exit_block,
                    head_block: header_block,
                });

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

                // Pop loop context from stack
                self.loop_stack.pop();

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
            // Desugars to: loop { match iterator.next() { Some(pattern) => { body }, None => break } }
            // For MVP: Basic implementation using loop + match
            HirExpression::For { pattern: _, iter: _, body, span: _ } => {
                // Allocate blocks for the for loop structure
                let loop_head = func.alloc_block();
                let loop_body = func.alloc_block();
                let exit_block = func.alloc_block();

                // Push loop context onto stack (for break/continue)
                self.loop_stack.push(LoopContext {
                    exit_block,
                    head_block: loop_head,
                });

                // Jump from current to loop head
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.set_terminator(MirTerminator::Goto { target: loop_head });

                // For now, implement a simple infinite loop
                // TODO: Implement proper iterator protocol with .next() calls
                *current_block = loop_head;
                let head_block_obj = func.blocks.get_mut(&loop_head).unwrap();
                head_block_obj.set_terminator(MirTerminator::Goto { target: loop_body });

                // Lower body (this returns the final block ID after all statements)
                let (final_block_id, _body_temp) = self.lower_block(func, body, loop_body, false)?;

                // After lowering the body, final_block_id might be different from loop_body
                let final_block_obj = func.blocks.get_mut(&final_block_id).unwrap();

                // Check if final_block already has a terminator
                if final_block_obj.terminator.is_none() {
                    // No terminator - add loop-back to create the actual loop
                    final_block_obj.set_terminator(MirTerminator::Goto { target: loop_head });
                }

                // Pop loop context from stack
                self.loop_stack.pop();

                // Set current to exit block
                *current_block = exit_block;

                // For loop returns Unit type
                let dummy_temp = func.alloc_temp();
                Ok(dummy_temp)
            }

            HirExpression::Try(try_block) => {
                // First, register handler blocks so they're available during try block lowering
                for handler in &try_block.handlers {
                    let mut mir_handler = MirEffectHandler {
                        effect_name: handler.effect_name.clone(),
                        methods: std::collections::HashMap::new(),
                    };

                    // Pre-allocate handler blocks (will be filled later)
                    for method in &handler.methods {
                        let handler_block = func.alloc_block();
                        let resume_block = func.alloc_block();

                        // Store placeholder mapping (blocks will be filled after try block)
                        mir_handler.methods.insert(
                            method.name.clone(),
                            (handler_block, resume_block)
                        );
                    }

                    // Register handler in function (before lowering try block)
                    func.handlers.push(mir_handler);
                }

                // Now lower the try block (handlers are now registered)
                let new_block = func.alloc_block();
                let old_block = *current_block;

                // Jump to try block
                let block_obj = func.blocks.get_mut(&old_block).unwrap();
                block_obj.set_terminator(MirTerminator::Goto { target: new_block });

                *current_block = new_block;
                let (try_end_block, try_result_temp) = self.lower_block(func, &try_block.try_block, new_block, false)?;

                // Update current_block to where the try block ended
                // This is important if the try block contained effect operations
                *current_block = try_end_block;

                // Now lower handler method bodies (after try block, can use resume blocks)
                let handler_index_offset = func.handlers.len() - try_block.handlers.len();

                // Collect all handler information first to avoid borrow issues
                let mut handlers_to_lower = Vec::new();
                for (i, handler) in try_block.handlers.iter().enumerate() {
                    let mir_handler = &func.handlers[handler_index_offset + i];

                    for method in &handler.methods {
                        if let Some((handler_block, resume_block)) = mir_handler.methods.get(&method.name) {
                            handlers_to_lower.push((
                                handler_index_offset + i,
                                method.name.clone(),
                                method.body.clone(),
                                *handler_block,
                                *resume_block,
                            ));
                        }
                    }
                }

                // Now lower handler methods
                for (_handler_idx, _method_name, method_body, handler_block, resume_block) in handlers_to_lower {
                    // Lower handler method body
                    let (handler_end_block, _handler_result_temp) =
                        self.lower_block(func, &method_body, handler_block, false)?;

                    // Handler should jump to resume block after execution
                    let end_block_obj = func.blocks.get_mut(&handler_end_block).unwrap();
                    if end_block_obj.terminator.is_none() {
                        end_block_obj.set_terminator(MirTerminator::Goto {
                            target: resume_block,
                        });
                    }
                }

                // Return the result of evaluating the try block
                Ok(try_result_temp.unwrap_or_else(|| func.alloc_temp()))
            }

            // Tuple expression: (a, b, c)
            HirExpression::Tuple(elements, _ty, _span) => {
                // Tuples are represented as anonymous structs in MIR
                // For MVP, we create a struct-like representation using FieldAccess

                if elements.is_empty() {
                    // Unit tuple - just return unit constant
                    let temp = func.alloc_temp();
                    let _block_obj = func.blocks.get_mut(current_block).unwrap();
                    _block_obj.push_instruction(MirInstruction::Const {
                        dest: temp,
                        value: MirConstant::Unit,
                        ty: MirTy::Unit,
                    });
                    return Ok(temp);
                }

                // Lower each element and store in temps
                let mut elem_temps = Vec::new();
                for elem in elements {
                    let elem_temp = self.lower_expression(func, current_block, elem)?;
                    elem_temps.push(elem_temp);
                }

                // For MVP: Store tuple elements as consecutive temps
                // Return the first temp as the "tuple" value
                // Index operations will use offset to access other elements
                //
                // Example: tuple = (1, 2, 3)
                // - elem_temps = [t1, t2, t3]
                // - Return t1 as the tuple value
                // - tuple.0 → t1 (offset 0)
                // - tuple.1 → t2 (offset 1) - need to track this
                // - tuple.2 → t3 (offset 2) - need to track this
                //
                // Limitation: This only works if the tuple is immediately indexed
                // A full implementation would create a proper tuple struct

                let result_temp = elem_temps[0];

                // Store tuple metadata for later field access
                // For MVP: We track which temps belong to tuples
                // This is a simplification - a full implementation would
                // create proper tuple struct values
                //
                // TODO: Add tuple metadata table to MirFunction
                // For now, tuples are limited to immediate indexing

                Ok(result_temp)
            }

            // Array literal: [a, b, c]
            HirExpression::Array { elements, ty: _, span: _ } => {
                // For now, arrays are lowered by evaluating each element
                // and returning the first one as a placeholder
                // TODO: Implement proper array handling with allocation
                let mut result_temp = func.alloc_temp();
                for (i, elem) in elements.iter().enumerate() {
                    let elem_temp = self.lower_expression(func, current_block, elem)?;
                    if i == 0 {
                        result_temp = elem_temp;
                    }
                    // TODO: Store elements in array memory
                }
                Ok(result_temp)
            }

            // Index operation: arr[index] or tuple.0
            HirExpression::Index { base, index, ty: _, span: _ } => {
                // Index can be used for:
                // 1. Tuple access: tuple.0, tuple.1, etc. (index is literal)
                // 2. Array access: arr[i] (index is expression)
                //
                // For MVP: Support tuple field access with literal indices
                // Array access will come later with proper array types

                let base_temp = self.lower_expression(func, current_block, base)?;
                let _index_temp = self.lower_expression(func, current_block, index)?;

                // Check if index is a constant integer (tuple field access)
                let _block_obj = func.blocks.get_mut(current_block).unwrap();

                // Try to get the constant value of the index
                // For MVP: Assume it's a constant (tuple field access)
                // In full implementation, we'd check the actual value

                // For now, just return base_temp as placeholder
                // TODO: Implement GEP for tuple field access
                // TODO: Implement array bounds checking and indexing

                Ok(base_temp)
            }

            // Template string with interpolation
            HirExpression::TemplateString { parts, ty, span: _ } => {
                // Desugar template strings to runtime string_concat calls
                // `Hello ${name}!` desugars to: string_concat("Hello ", name, "!")
                // For efficiency, we chain binary calls: string_concat(string_concat("Hello ", name), "!")

                if parts.is_empty() {
                    // Empty template string, return empty string constant
                    let temp = func.alloc_temp();
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    block_obj.push_instruction(MirInstruction::Const {
                        dest: temp,
                        value: MirConstant::String(String::new()),
                        ty: MirTy::String,
                    });
                    return Ok(temp);
                }

                // Lower each part and collect the temps
                let mut part_temps = Vec::new();
                for part in parts {
                    match part {
                        zulon_hir::HirTemplateStringPart::Static(s) => {
                            // Create a constant string for static parts
                            let temp = func.alloc_temp();
                            let block_obj = func.blocks.get_mut(current_block).unwrap();
                            block_obj.push_instruction(MirInstruction::Const {
                                dest: temp,
                                value: MirConstant::String(s.clone()),
                                ty: MirTy::String,
                            });
                            part_temps.push(temp);
                        }
                        zulon_hir::HirTemplateStringPart::Expr(expr) => {
                            // Lower the interpolated expression
                            let expr_temp = self.lower_expression(func, current_block, expr)?;
                            part_temps.push(expr_temp);
                        }
                    }
                }

                // Chain string_concat calls: concat(concat(part1, part2), part3), ...
                // Start with the first part
                let mut result_temp = part_temps[0];

                // Iteratively concatenate remaining parts
                for part_temp in &part_temps[1..] {
                    let concat_temp = func.alloc_temp();
                    let block_obj = func.blocks.get_mut(current_block).unwrap();
                    
                    // Generate call to string_concat(result_so_far, next_part)
                    block_obj.push_instruction(MirInstruction::Call {
                        dest: Some(concat_temp),
                        func: MirPlace::Local("string_concat".to_string()),
                        args: vec![
                            MirPlace::Temp(result_temp),
                            MirPlace::Temp(*part_temp),
                        ],
                        return_type: ty.clone().into(),  // Return type is String
                    });
                    
                    result_temp = concat_temp;
                }

                Ok(result_temp)
            }

            // Await expression: future.await
            HirExpression::Await { future, ty, span: _ } => {
                // For now, await is treated as a yield point in the async state machine
                // The full implementation would:
                // 1. Lower the future expression
                // 2. Create a new state in the state machine
                // 3. Generate yield logic
                // 4. Store captured locals
                //
                // For MVP: Just lower the future expression and return it
                // The actual await semantics will be implemented in the state machine transformation

                let _future_temp = self.lower_expression(func, current_block, future)?;

                // TODO: Generate proper await semantics with state machine yield
                // For now, just return the future's temp (this is incorrect but allows compilation)
                // A proper implementation needs to:
                // - Create a yield point
                // - Save current state
                // - Store the future for later polling
                // - Return Pending to the executor
                //
                // This will be implemented in a follow-up task as it requires
                // significant infrastructure (state machine transformation, etc.)

                // For MVP: Return a placeholder temp with the await result type
                let result_temp = func.alloc_temp();
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.push_instruction(MirInstruction::Const {
                    dest: result_temp,
                    value: MirConstant::Integer(0),  // Placeholder
                    ty: ty.clone().into(),
                });

                Ok(result_temp)
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

    /// Get the field index from a struct type
    ///
    /// This looks up the struct definition and finds the index of the field by name.
    /// If the base type is not a struct type (e.g., due to placeholder I32 type),
    /// we search all struct definitions to find one with a matching field name.
    fn get_field_index(&self, ty: &HirTy, field_name: &str) -> Result<usize> {
        // First, try to extract struct name from the type
        let struct_name = match ty {
            HirTy::Struct { name, .. } => {
                // Type info is available - use it
                return self.get_field_index_in_struct(name, field_name);
            }
            _ => {
                // Type info is not available (placeholder type)
                // Search all struct definitions for one with this field
                self.find_field_in_any_struct(field_name)?
            }
        };

        Ok(struct_name)
    }

    /// Get field index in a specific struct
    fn get_field_index_in_struct(&self, struct_name: &str, field_name: &str) -> Result<usize> {
        if let Some(field_names) = self.struct_defs.get(struct_name) {
            if let Some(index) = field_names.iter().position(|name| name == field_name) {
                Ok(index)
            } else {
                Err(MirError::InvalidFieldAccess {
                    field: field_name.to_string(),
                    reason: format!("field '{}' not found in struct '{}'", field_name, struct_name),
                })
            }
        } else {
            Err(MirError::InvalidFieldAccess {
                field: field_name.to_string(),
                reason: format!("struct '{}' not found", struct_name),
            })
        }
    }

    /// Find a field in any struct definition
    ///
    /// This is a fallback when the base expression's type is not available
    fn find_field_in_any_struct(&self, field_name: &str) -> Result<usize> {
        // Search through all struct definitions
        for (_struct_name, field_names) in &self.struct_defs {
            if let Some(index) = field_names.iter().position(|name| name == field_name) {
                // Found the field in this struct
                return Ok(index);
            }
        }

        // Field not found in any struct
        Err(MirError::InvalidFieldAccess {
            field: field_name.to_string(),
            reason: format!("field '{}' not found in any struct definition", field_name),
        })
    }

    /// Check if a function name is likely an effect operation
    ///
    /// This is a heuristic-based check for common effect operation names.
    /// TODO: Replace with proper effect system lookup
    fn is_effect_operation_name(&self, name: &str) -> bool {
        // Common effect operation names
        matches!(name,
            "log" | "print" | "println" |
            "get" | "set" | "update" |
            "read" | "write" |
            "fail" | "raise" | "throw"
        )
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
