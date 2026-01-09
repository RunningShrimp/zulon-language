// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Type checker
//!
//! This module implements type checking for ZULON.

use crate::env::Env;
use crate::error::{Result, TypeError};
use crate::ty::Ty;
use crate::infer::Substitution;
use zulon_parser::ast::{self, Ast};
use zulon_parser::ast::{Expression, Statement, Item, ItemKind, Type, Identifier};

/// Type checker with type inference support
pub struct TypeChecker {
    /// Current type environment
    env: Env,

    /// Current function return type (for return statements)
    current_return_type: Option<Ty>,

    /// Current function error type (for throw and ? statements)
    current_error_type: Option<Ty>,

    /// Current active effects (for effect operations)
    current_effects: Vec<String>,

    /// Type substitution from inference
    subst: Substitution,
}

impl TypeChecker {
    /// Create a new type checker with built-in types
    pub fn new() -> Self {
        TypeChecker {
            env: Env::with_builtins(),
            current_return_type: None,
            current_error_type: None,
            current_effects: Vec::new(),
            subst: Substitution::new(),
        }
    }

    /// Type check an entire AST
    pub fn check(&mut self, ast: &Ast) -> Result<()> {
        for item in &ast.items {
            self.check_item(item)?;
        }
        Ok(())
    }

    /// Type check an item
    fn check_item(&mut self, item: &Item) -> Result<()> {
        match &item.kind {
            ItemKind::Function(func) => self.check_function(func),
            ItemKind::ExternFunction(func) => self.check_extern_function(func),
            ItemKind::Struct(struct_def) => self.check_struct(struct_def),
            ItemKind::Enum(enum_def) => self.check_enum(enum_def),
            ItemKind::Trait(trait_def) => self.check_trait(trait_def),
            ItemKind::Impl(impl_block) => self.check_impl(impl_block),
            ItemKind::TypeAlias(type_alias) => self.check_type_alias(type_alias),
            ItemKind::Const(const_def) => self.check_const(const_def),
            ItemKind::Static(static_def) => self.check_static(static_def),
            ItemKind::Module(module) => self.check_module(module),
            ItemKind::Use(use_stmt) => self.check_use(use_stmt),
            ItemKind::ExternCrate(extern_crate) => self.check_extern_crate(extern_crate),
            ItemKind::Effect(effect) => self.check_effect(effect),
        }
    }

    /// Type check a function
    fn check_function(&mut self, func: &ast::Function) -> Result<()> {
        // Create function type from signature
        let param_types: Vec<Ty> = func.params.iter()
            .map(|p| {
                p.type_annotation.as_ref()
                    .map(|ty| self.ast_type_to_ty(ty))
                    .unwrap_or(Ty::Unit)
            })
            .collect();

        let return_type = func.return_type.as_ref()
            .map(|ty| self.ast_type_to_ty(ty))
            .unwrap_or(Ty::Unit);

        // Extract error type if present (from `-> Type | Error` syntax)
        let error_type = if let Some(ast_error_type) = &func.error_type {
            Some(self.ast_type_to_ty(ast_error_type))
        } else {
            None
        };

        let func_ty = Ty::Function {
            params: param_types.clone(),
            return_type: Box::new(return_type.clone()),
        };

        // Insert function into environment
        self.env.insert_function(func.name.name.clone(), func_ty);

        // Enter function scope
        let mut func_env = self.env.enter_scope();
        std::mem::swap(&mut self.env, &mut func_env);

        // Bind parameters
        for param in &func.params {
            let param_ty = param.type_annotation.as_ref()
                .map(|ty| self.ast_type_to_ty(ty))
                .unwrap_or(Ty::Unit);
            self.env.insert_binding(param.name.name.clone(), param_ty);
        }

        // Set current return type and error type
        let prev_return_type = self.current_return_type.take();
        let prev_error_type = self.current_error_type.take();
        let prev_effects = self.current_effects.clone();
        self.current_return_type = Some(return_type.clone());
        self.current_error_type = error_type.clone();

        // Debug logging
        // Process effects from function signature (e.g., `-> i32 | Log`)
        for effect_ty in &func.effects {
            let (effect_name, span) = match &effect_ty {
                Type::Simple(ident) => (ident.name.clone(), ident.span.clone()),
                _ => continue, // Skip complex types for now
            };

            // Verify that the effect exists in the environment
            if self.env.lookup_effect(&effect_name).is_some() {
                // Add to current effects
                self.current_effects.push(effect_name);
            } else {
                return Err(TypeError::UndefinedEffect {
                    name: effect_name,
                    span,
                });
            }
        }

        // Check function body and validate return type
        let body_result_ty = self.check_block(&func.body)?;

        // Validate that the body's result type matches the declared return type
        if &body_result_ty != &return_type {
            // Allow Never type (throw/return) in any position
            if !matches!(body_result_ty, Ty::Never) {
                return Err(TypeError::TypeMismatch {
                    expected: return_type.clone(),
                    found: body_result_ty,
                    span: func.body.span.clone(),
                });
            }
        }

        // Restore return type and error type
        self.current_return_type = prev_return_type;
        self.current_error_type = prev_error_type.clone();
        self.current_effects = prev_effects;

        // Exit function scope - swap back to parent environment
        std::mem::swap(&mut self.env, &mut func_env);

        Ok(())
    }

    /// Type check an extern function declaration
    fn check_extern_function(&mut self, func: &ast::Function) -> Result<()> {
        // Similar to regular function, but no body to check
        let param_types: Vec<Ty> = func.params.iter()
            .map(|p| {
                p.type_annotation.as_ref()
                    .map(|ty| self.ast_type_to_ty(ty))
                    .unwrap_or(Ty::Unit)
            })
            .collect();

        let return_type = func.return_type.as_ref()
            .map(|ty| self.ast_type_to_ty(ty))
            .unwrap_or(Ty::Unit);

        let func_ty = Ty::Function {
            params: param_types.clone(),
            return_type: Box::new(return_type.clone()),
        };

        // Insert extern function into environment
        self.env.insert_function(func.name.name.clone(), func_ty);

        Ok(())
    }

    /// Type check a struct
    fn check_struct(&mut self, struct_def: &ast::Struct) -> Result<()> {
        // For now, just register the struct type
        // TODO: Handle generics, field types, etc.
        let struct_ty = Ty::Struct {
            name: struct_def.name.clone(),
            generics: vec![],
        };

        self.env.insert_type_def(struct_def.name.name.clone(), struct_ty);
        Ok(())
    }

    /// Type check an enum
    fn check_enum(&mut self, enum_def: &ast::Enum) -> Result<()> {
        // For now, just register the enum type
        // TODO: Handle generics, variant types, etc.
        let enum_ty = Ty::Enum {
            name: enum_def.name.clone(),
            generics: vec![],
        };

        self.env.insert_type_def(enum_def.name.name.clone(), enum_ty);
        Ok(())
    }

    /// Type check a trait
    fn check_trait(&mut self, _trait_def: &ast::Trait) -> Result<()> {
        // TODO: Implement trait checking
        Ok(())
    }

    /// Type check an impl block
    fn check_impl(&mut self, _impl_block: &ast::Impl) -> Result<()> {
        // TODO: Implement impl checking
        Ok(())
    }

    /// Type check a type alias
    fn check_type_alias(&mut self, _type_alias: &ast::TypeAlias) -> Result<()> {
        // TODO: Implement type alias checking
        Ok(())
    }

    /// Type check a const
    fn check_const(&mut self, _const_def: &ast::Const) -> Result<()> {
        // TODO: Implement const checking
        Ok(())
    }

    /// Type check a static
    fn check_static(&mut self, _static_def: &ast::Static) -> Result<()> {
        // TODO: Implement static checking
        Ok(())
    }

    /// Type check a module
    fn check_module(&mut self, _module: &ast::Module) -> Result<()> {
        // TODO: Implement module checking
        Ok(())
    }

    /// Type check a use statement
    fn check_use(&mut self, _use_stmt: &ast::Use) -> Result<()> {
        // TODO: Implement use checking
        Ok(())
    }

    /// Type check an extern crate
    fn check_extern_crate(&mut self, _extern_crate: &ast::ExternCrate) -> Result<()> {
        // TODO: Implement extern crate checking
        Ok(())
    }

    /// Type check an effect declaration
    fn check_effect(&mut self, effect: &ast::Effect) -> Result<()> {
        // For now, effects are just declarations - no type checking needed yet
        // Future work: check operation signatures, validate generic parameters

        // Pre-collect all operations to avoid borrow issues
        let operations: Vec<crate::ty::EffectOperation> = effect.operations.iter()
            .map(|op| {
                // Convert parameter types
                let param_types: Vec<Ty> = op.params.iter()
                    .map(|p| {
                        p.type_annotation.as_ref()
                            .map(|ty| self.ast_type_to_ty(ty))
                            .unwrap_or_else(|| Ty::TyVar(0)) // Temporary, will be replaced
                    })
                    .collect();

                // Convert return type
                let return_type = op.return_type.as_ref()
                    .map(|ty| self.ast_type_to_ty(ty))
                    .unwrap_or(Ty::Unit);

                crate::ty::EffectOperation {
                    name: op.name.name.clone(),
                    param_types,
                    return_type,
                }
            })
            .collect();

        // Register effect in environment
        self.env.insert_effect(
            effect.name.name.clone(),
            crate::ty::Effect {
                name: effect.name.name.clone(),
                operations,
            }
        );

        Ok(())
    }

    /// Type check a block
    fn check_block(&mut self, block: &ast::Block) -> Result<Ty> {
        // Enter block scope
        let mut block_env = self.env.enter_scope();
        std::mem::swap(&mut self.env, &mut block_env);

        // Check statements
        for stmt in &block.statements {
            self.check_statement(stmt)?;
        }

        // Check trailing expression
        let result_ty = if let Some(expr) = &block.trailing_expr {
            self.check_expression(expr)?
        } else {
            Ty::Unit
        };

        // Exit block scope - swap back to parent environment
        std::mem::swap(&mut self.env, &mut block_env);

        Ok(result_ty)
    }

    /// Type check a statement
    fn check_statement(&mut self, stmt: &Statement) -> Result<()> {
        match &stmt.kind {
            ast::StatementKind::Local(local) => self.check_local(local),
            ast::StatementKind::Item(item) => self.check_item(item),
            ast::StatementKind::Expr(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            ast::StatementKind::Defer(stmt) => {
                // Defer statements are checked normally
                // The runtime behavior (execution at scope exit) is handled later
                self.check_statement(stmt)
            }
            ast::StatementKind::Empty => Ok(()),
        }
    }

    /// Type check a local variable with type inference
    fn check_local(&mut self, local: &ast::Local) -> Result<()> {
        // Type check initializer if present
        let init_ty = if let Some(init) = &local.init {
            self.check_expression(init)?
        } else {
            // No initializer, use type annotation or create fresh type variable
            local.type_annotation.as_ref()
                .map(|ty| self.ast_type_to_ty(ty))
                .unwrap_or_else(|| self.env.fresh_ty_var())
        };

        // Handle type annotation
        if let Some(type_ann) = &local.type_annotation {
            let declared_ty = self.ast_type_to_ty(type_ann);

            // Unify declared type with inferred type
            self.unify(&declared_ty, &init_ty, &local.name.span)?;

            // Use the declared type (after unification)
            let final_ty = self.apply_subst(&declared_ty);
            self.env.insert_binding(local.name.name.clone(), final_ty);
        } else {
            // No type annotation - use inferred type
            let final_ty = self.apply_subst(&init_ty);
            self.env.insert_binding(local.name.name.clone(), final_ty);
        }

        Ok(())
    }

    /// Type check an expression
    pub fn check_expression(&mut self, expr: &Expression) -> Result<Ty> {
        match &expr.kind {
            ast::ExpressionKind::Literal(literal) => self.check_literal(literal),
            ast::ExpressionKind::Path(path) => self.check_path(path),
            ast::ExpressionKind::Block(block) => self.check_block(block),
            ast::ExpressionKind::Binary(op, left, right) => {
                self.check_binary_op(op, left, right)
            }
            ast::ExpressionKind::Unary(op, operand) => {
                self.check_unary_op(op, operand)
            }
            ast::ExpressionKind::Call(func, args) => {
                self.check_call(func, args)
            }
            ast::ExpressionKind::FieldAccess(obj, field) => {
                self.check_field_access(obj, field)
            }
            ast::ExpressionKind::Index(obj, index) => {
                self.check_index(obj, index)
            }
            ast::ExpressionKind::Array(elements) => {
                self.check_array(elements)
            }
            ast::ExpressionKind::Tuple(elements) => {
                self.check_tuple(elements)
            }
            ast::ExpressionKind::If(condition, then_block, else_block) => {
                self.check_if(condition, then_block, else_block)
            }
            ast::ExpressionKind::Match(scrutinee, arms) => {
                self.check_match(scrutinee, arms)
            }
            ast::ExpressionKind::Loop(body, _) => {
                self.check_loop(body)
            }
            ast::ExpressionKind::While(condition, body, _) => {
                self.check_while(condition, body)
            }
            ast::ExpressionKind::For(local, iter, body, _) => {
                self.check_for(local, iter, body)
            }
            ast::ExpressionKind::Closure { params, return_type, body } => {
                self.check_closure(params, return_type, body)
            }
            ast::ExpressionKind::Break(_) => Ok(Ty::Never),
            ast::ExpressionKind::Continue(_) => Ok(Ty::Never),
            ast::ExpressionKind::Return(value) => self.check_return(value),
            ast::ExpressionKind::Throw(error_expr) => self.check_throw(error_expr),
            ast::ExpressionKind::QuestionMark(expr) => self.check_question_mark(expr),
            ast::ExpressionKind::Struct(struct_lit) => self.check_struct_literal(struct_lit),
            ast::ExpressionKind::Assign(target, value) => self.check_assign(target, value),
            ast::ExpressionKind::AssignOp(op, target, value) => {
                self.check_assign_op(op, target, value)
            }
            ast::ExpressionKind::MacroInvocation { macro_name, args, .. } => {
                // For builtin macros, check the arguments
                match macro_name.name.as_str() {
                    "assert_eq" | "assert" => {
                        // Type check macro arguments
                        for arg in args {
                            self.check_expression(arg)?;
                        }
                        // Macros expand to unit type (for now)
                        Ok(Ty::Unit)
                    }
                    _ => {
                        // Unknown macro - assume unit type for now
                        Ok(Ty::Unit)
                    }
                }
            }
            _ => {
                // TODO: Implement remaining expression kinds
                Ok(Ty::Unit)
            }
        }
    }

    /// Type check a literal
    fn check_literal(&mut self, literal: &ast::Literal) -> Result<Ty> {
        match literal {
            ast::Literal::Int(_) => Ok(Ty::I32),  // Default to i32
            ast::Literal::Float(_) => Ok(Ty::F64),
            // String literals are pointers to u8 (for C compatibility)
            ast::Literal::String(_) => Ok(Ty::Ref {
                inner: Box::new(Ty::U8),
                mutable: false,
            }),
            ast::Literal::Char(_) => Ok(Ty::Char),
            ast::Literal::Bool(_) => Ok(Ty::Bool),
            ast::Literal::Null => Ok(Ty::Unit),
        }
    }

    /// Type check a path (variable or function reference)
    fn check_path(&mut self, path: &[Identifier]) -> Result<Ty> {
        if path.len() == 1 {
            // Simple identifier - existing logic
            let name = &path[0].name;

            // Look up as variable
            if let Some(ty) = self.env.lookup_binding(name) {
                return Ok(ty);
            }

            // Look up as function
            if let Some(ty) = self.env.lookup_function(name) {
                return Ok(ty);
            }

            // Look up as type
            if let Some(ty) = self.env.lookup_type_def(name) {
                return Ok(ty);
            }

            // Look up as effect operation in current effects
            for effect_name in &self.current_effects {
                if let Some(effect) = self.env.lookup_effect(effect_name) {
                    if let Some(op) = effect.operations.iter().find(|op| &op.name == name) {
                        // Found the effect operation - return its type
                        return Ok(Ty::Function {
                            params: op.param_types.clone(),
                            return_type: Box::new(op.return_type.clone()),
                        });
                    }
                }
            }

            Err(TypeError::UndefinedVariable {
                name: name.clone(),
                span: path[0].span,
            })
        } else if path.len() == 2 {
            // Qualified path: Type::Variant or Type::Field
            let type_name = &path[0].name;
            let _variant_name = &path[1].name;

            // Look up as enum type
            if let Some(enum_ty) = self.env.lookup_type_def(type_name) {
                return Ok(enum_ty);
            }

            // Not found
            Err(TypeError::UndefinedVariable {
                name: type_name.clone(),
                span: path[0].span.clone(),
            })
        } else {
            // Longer paths (module::Type::Variant, etc.) - not yet supported
            Err(TypeError::UndefinedVariable {
                name: path.last().unwrap().name.clone(),
                span: path.last().unwrap().span.clone(),
            })
        }
    }

    /// Type check a binary operation with type inference
    fn check_binary_op(
        &mut self,
        op: &ast::BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> Result<Ty> {
        let left_ty = self.check_expression(left)?;
        let right_ty = self.check_expression(right)?;

        // Unify operand types based on operator
        let result_ty = match op {
            // Arithmetic operators: require numeric types, return same type
            ast::BinaryOp::Add |
            ast::BinaryOp::Sub |
            ast::BinaryOp::Mul |
            ast::BinaryOp::Div |
            ast::BinaryOp::Mod => {
                // Both operands must be the same numeric type
                self.unify(&left_ty, &right_ty, &left.span)?;

                // Return the unified type
                let unified = self.apply_subst(&left_ty);

                // Check that it's numeric
                if !unified.is_numeric() {
                    return Err(TypeError::TypeMismatch {
                        expected: Ty::I32, // Any numeric type
                        found: unified.clone(),
                        span: left.span,
                    });
                }

                unified
            }

            // Comparison operators: return bool
            ast::BinaryOp::Eq |
            ast::BinaryOp::NotEq |
            ast::BinaryOp::Less |
            ast::BinaryOp::LessEq |
            ast::BinaryOp::Greater |
            ast::BinaryOp::GreaterEq => {
                // Operands must be the same type
                self.unify(&left_ty, &right_ty, &left.span)?;

                // Check that operands are comparable (numeric or other comparable types)
                let unified = self.apply_subst(&left_ty);
                if !unified.is_numeric() && !matches!(unified, Ty::Bool | Ty::Char | Ty::String) {
                    return Err(TypeError::InferenceError {
                        message: format!("cannot compare {} values", unified),
                        span: left.span,
                    });
                }

                Ty::Bool
            }

            // Logical operators: require bool, return bool
            ast::BinaryOp::And |
            ast::BinaryOp::Or => {
                self.unify(&left_ty, &Ty::Bool, &left.span)?;
                self.unify(&right_ty, &Ty::Bool, &right.span)?;

                Ty::Bool
            }

            // Bitwise operators: require integer types
            ast::BinaryOp::BitAnd |
            ast::BinaryOp::BitOr |
            ast::BinaryOp::BitXor |
            ast::BinaryOp::LeftShift |
            ast::BinaryOp::RightShift => {
                self.unify(&left_ty, &right_ty, &left.span)?;

                let unified = self.apply_subst(&left_ty);
                if !unified.is_integer() {
                    return Err(TypeError::TypeMismatch {
                        expected: Ty::I32, // Any integer type
                        found: unified.clone(),
                        span: left.span,
                    });
                }

                unified
            }
        };

        Ok(result_ty)
    }

    /// Type check a unary operation
    fn check_unary_op(&mut self, _op: &ast::UnaryOp, operand: &Expression) -> Result<Ty> {
        self.check_expression(operand)?;
        Ok(Ty::I32)
    }

    /// Type check a function call with type inference
    fn check_call(&mut self, func: &Expression, args: &[Box<Expression>]) -> Result<Ty> {
        let func_ty = self.check_expression(func)?;

        match func_ty {
            Ty::Function { params, return_type } => {
                // Check arity
                if params.len() != args.len() {
                    return Err(TypeError::ArityMismatch {
                        expected: params.len(),
                        found: args.len(),
                        span: func.span,
                    });
                }

                // Check arguments and unify with parameter types
                for (arg, param_ty) in args.iter().zip(params.iter()) {
                    let arg_ty = self.check_expression(arg)?;
                    self.unify(&arg_ty, param_ty, &arg.span)?;
                }

                // Apply substitution to return type
                Ok(self.apply_subst(&*return_type))
            }
            _ => Err(TypeError::NotCallable {
                ty: func_ty,
                span: func.span,
            }),
        }
    }

    /// Type check field access
    fn check_field_access(&mut self, _obj: &Expression, _field: &Identifier) -> Result<Ty> {
        // TODO: Implement field access checking
        Ok(Ty::I32)
    }

    /// Type check array indexing
    fn check_index(&mut self, _obj: &Expression, _index: &Expression) -> Result<Ty> {
        // TODO: Implement index checking
        Ok(Ty::I32)
    }

    /// Type check an array literal
    fn check_array(&mut self, elements: &[Box<Expression>]) -> Result<Ty> {
        if elements.is_empty() {
            return Ok(Ty::Array {
                inner: Box::new(Ty::Unit),
                len: Some(0),
            });
        }

        // Check first element to get element type
        let elem_ty = self.check_expression(&elements[0])?;

        // Check all elements have the same type
        for elem in &elements[1..] {
            let ty = self.check_expression(elem)?;
            if ty != elem_ty {
                return Err(TypeError::TypeMismatch {
                    expected: elem_ty,
                    found: ty,
                    span: elem.span,
                });
            }
        }

        Ok(Ty::Array {
            inner: Box::new(elem_ty),
            len: Some(elements.len() as u64),
        })
    }

    /// Type check a tuple literal
    fn check_tuple(&mut self, elements: &[Box<Expression>]) -> Result<Ty> {
        let mut elem_tys = Vec::new();
        for elem in elements {
            elem_tys.push(self.check_expression(elem)?);
        }

        Ok(Ty::Tuple(elem_tys))
    }

    /// Type check an if expression with type inference
    fn check_if(
        &mut self,
        condition: &Expression,
        then_block: &ast::Block,
        else_block: &Option<ast::Block>,
    ) -> Result<Ty> {
        // Condition must be bool
        let cond_ty = self.check_expression(condition)?;
        self.unify(&cond_ty, &Ty::Bool, &condition.span)?;

        // Check both branches
        let then_ty = self.check_block(then_block)?;

        let else_ty = match else_block {
            Some(block) => self.check_block(block)?,
            None => Ty::Unit,
        };

        // Special handling for Never type (diverging expressions)
        // If one branch never returns, the result type is the other branch
        if matches!(then_ty, Ty::Never) {
            // Then branch diverges (throw/return), so result is else branch type
            return Ok(else_ty);
        }
        if matches!(else_ty, Ty::Never) {
            // Else branch diverges, so result is then branch type
            return Ok(then_ty);
        }

        // Normal case: unify branch types
        self.unify(&then_ty, &else_ty, &then_block.span)?;

        // Return the unified type
        Ok(self.apply_subst(&then_ty))
    }

    /// Type check a match expression
    fn check_match(&mut self, _scrutinee: &Expression, _arms: &[ast::MatchArm]) -> Result<Ty> {
        // TODO: Implement match expression checking
        Ok(Ty::I32)
    }

    /// Type check a loop expression
    fn check_loop(&mut self, _body: &ast::Block) -> Result<Ty> {
        // TODO: Implement loop checking
        Ok(Ty::Unit)
    }

    /// Type check a while loop
    fn check_while(&mut self, condition: &Expression, _body: &ast::Block) -> Result<Ty> {
        let cond_ty = self.check_expression(condition)?;
        if cond_ty != Ty::Bool {
            return Err(TypeError::TypeMismatch {
                expected: Ty::Bool,
                found: cond_ty,
                span: condition.span,
            });
        }
        Ok(Ty::Unit)
    }

    /// Type check a for loop
    fn check_for(&mut self, _local: &ast::Local, _iter: &Expression, _body: &ast::Block) -> Result<Ty> {
        // TODO: Implement for loop checking
        Ok(Ty::Unit)
    }

    /// Type check a closure expression with type inference
    ///
    /// Closures are typed as function pointers: fn(params) -> return_type
    /// This method infers the types of parameters, return type, and the overall closure type.
    pub fn check_closure(
        &mut self,
        params: &[ast::Local],
        return_type: &Option<Type>,
        body: &Expression,
    ) -> Result<Ty> {
        // Enter a new scope for the closure body
        let mut closure_env = self.env.enter_scope();
        std::mem::swap(&mut self.env, &mut closure_env);

        // Process each parameter
        let mut param_tys = Vec::new();

        for param in params {
            // Get or infer the parameter type
            let param_ty = if let Some(type_ann) = &param.type_annotation {
                // Explicit type annotation - convert to Ty
                let ty = self.ast_type_to_ty(type_ann);

                // If it's a type variable, create a fresh one
                let ty = match &ty {
                    Ty::TyVar(_) => self.env.fresh_ty_var(),
                    _ => ty,
                };

                ty
            } else {
                // No type annotation - create fresh type variable for inference
                self.env.fresh_ty_var()
            };

            // Bind parameter name to its type in the closure's environment
            self.env.insert_binding(param.name.name.clone(), param_ty.clone());
            param_tys.push(param_ty);
        }

        // Type check the closure body
        let body_ty = self.check_expression(body)?;

        // Process return type annotation
        let return_ty = if let Some(ret_type_ann) = return_type {
            // Explicit return type annotation
            let ty = self.ast_type_to_ty(ret_type_ann);

            // Unify with inferred body type
            self.unify(&ty, &body_ty, &body.span)?;

            // Apply substitution to get final return type
            self.apply_subst(&ty)
        } else {
            // No return type annotation - use inferred body type
            self.apply_subst(&body_ty)
        };

        // Exit closure scope - swap back to parent environment
        std::mem::swap(&mut self.env, &mut closure_env);

        // Construct the closure's function type
        // Closures are represented as function types: fn(params) -> return_type
        let closure_ty = Ty::Function {
            params: param_tys,
            return_type: Box::new(return_ty),
        };

        Ok(closure_ty)
    }

    /// Type check a return statement
    fn check_return(&mut self, value: &Option<Box<Expression>>) -> Result<Ty> {
        let value_ty = match value {
            Some(expr) => self.check_expression(expr)?,
            None => Ty::Unit,
        };

        // Check against current return type
        if let Some(expected_ty) = &self.current_return_type {
            if &value_ty != expected_ty {
                let span = value.as_ref()
                    .map(|v| v.span)
                    .unwrap_or_else(|| {
                        // Use a default span if expression is None
                        zulon_parser::lexer::Span::new(
                            zulon_parser::lexer::Position::new(1, 1),
                            zulon_parser::lexer::Position::new(1, 1),
                        )
                    });

                return Err(TypeError::TypeMismatch {
                    expected: expected_ty.clone(),
                    found: value_ty,
                    span,
                });
            }
        }

        Ok(Ty::Never)
    }

    /// Type check a throw statement
    fn check_throw(&mut self, error_expr: &Expression) -> Result<Ty> {
        // Type check the error expression
        let error_ty = self.check_expression(error_expr)?;

        // Check against current function's error type
        if let Some(expected_error_ty) = &self.current_error_type {
            if &error_ty != expected_error_ty {
                return Err(TypeError::TypeMismatch {
                    expected: expected_error_ty.clone(),
                    found: error_ty,
                    span: error_expr.span.clone(),
                });
            }
        } else {
            // Function doesn't have an error type but we're trying to throw
            return Err(TypeError::InferenceError {
                message: "throw statement used in function without error type".to_string(),
                span: error_expr.span.clone(),
            });
        }

        // throw statements never return normally (they always return an error)
        Ok(Ty::Never)
    }

    /// Type check a question mark operator (error propagation)
    fn check_question_mark(&mut self, expr: &Expression) -> Result<Ty> {
        // Type check the operand expression
        let _operand_ty = self.check_expression(expr)?;

        // Check if current function has an error type
        let _error_ty = match &self.current_error_type {
            Some(ty) => ty.clone(),
            None => {
                return Err(TypeError::InferenceError {
                    message: "? operator used in function without error type".to_string(),
                    span: expr.span.clone(),
                });
            }
        };

        // For now, we assume the operand is an Outcome<T, E>
        // TODO: Properly destructure Outcome type to extract T
        // For placeholder implementation, we'll return the function's return type
        if let Some(return_ty) = &self.current_return_type {
            Ok(return_ty.clone())
        } else {
            Ok(Ty::Unit)
        }
    }

    /// Type check a struct literal
    fn check_struct_literal(&mut self, _struct_lit: &ast::StructLiteral) -> Result<Ty> {
        // TODO: Implement struct literal checking
        Ok(Ty::Unit)
    }

    /// Type check an assignment
    fn check_assign(&mut self, _target: &Expression, _value: &Expression) -> Result<Ty> {
        // TODO: Implement assignment checking
        Ok(Ty::Unit)
    }

    /// Type check a compound assignment
    fn check_assign_op(
        &mut self,
        _op: &ast::BinaryOp,
        _target: &Expression,
        _value: &Expression,
    ) -> Result<Ty> {
        // TODO: Implement compound assignment checking
        Ok(Ty::Unit)
    }

    /// Apply current substitution to a type
    fn apply_subst(&self, ty: &Ty) -> Ty {
        self.subst.apply(ty)
    }

    /// Unify two types and update substitution
    fn unify(&mut self, ty1: &Ty, ty2: &Ty, span: &ast::Span) -> Result<()> {
        use crate::infer::unify_with_subst;

        // Apply current substitution first
        let ty1 = self.apply_subst(ty1);
        let ty2 = self.apply_subst(ty2);

        // Unify and update substitution
        unify_with_subst(&ty1, &ty2, span, &mut self.subst)
    }

    /// Convert AST type to Ty
    fn ast_type_to_ty(&self, ty: &Type) -> Ty {
        match ty {
            Type::Simple(ident) => {
                // Check if this is an effect type (by looking up in effects)
                if self.env.lookup_effect(&ident.name).is_some() {
                    return Ty::Effect(ident.name.clone());
                }

                // Look up type in environment
                self.env.lookup_type_def(&ident.name)
                    .unwrap_or(Ty::TyVar(self.env.peek_next_ty_var()))
            }
            Type::Tuple(types) => {
                let elem_tys: Vec<Ty> = types.iter()
                    .map(|t| self.ast_type_to_ty(t))
                    .collect();
                Ty::Tuple(elem_tys)
            }
            Type::Array(inner, size) => {
                Ty::Array {
                    inner: Box::new(self.ast_type_to_ty(inner)),
                    len: size.as_ref().map(|_s| {
                        // TODO: Evaluate constant expression
                        42  // Placeholder
                    }),
                }
            }
            Type::Slice(inner) => {
                Ty::Slice(Box::new(self.ast_type_to_ty(inner)))
            }
            Type::Ref(inner, mutable) => {
                Ty::Ref {
                    inner: Box::new(self.ast_type_to_ty(inner)),
                    mutable: *mutable,
                }
            }
            Type::Function(params, return_type) => {
                Ty::Function {
                    params: params.iter().map(|p| self.ast_type_to_ty(p)).collect(),
                    return_type: Box::new(self.ast_type_to_ty(return_type)),
                }
            }
            Type::Pipe(left, right) => {
                // T | E syntax is desugared to Outcome<T, E>
                // Create Outcome struct with type parameters
                use zulon_parser::ast::Identifier;
                use zulon_parser::{Span, Position};

                Ty::Struct {
                    name: Identifier {
                        span: Span::new(Position::new(0, 0), Position::new(0, 0)),
                        name: "Outcome".to_string(),
                    },
                    generics: vec![self.ast_type_to_ty(left), self.ast_type_to_ty(right)],
                }
            }
            Type::Optional(inner) => {
                // T? syntax is desugared to Optional<T>
                use zulon_parser::ast::Identifier;
                use zulon_parser::{Span, Position};

                Ty::Struct {
                    name: Identifier {
                        span: Span::new(Position::new(0, 0), Position::new(0, 0)),
                        name: "Optional".to_string(),
                    },
                    generics: vec![self.ast_type_to_ty(inner)],
                }
            }
            Type::Never => Ty::Never,
            Type::Unit => Ty::Unit,
            Type::TraitObject(inner) => Ty::TraitObject(Box::new(self.ast_type_to_ty(inner))),
            Type::ImplTrait(inner) => Ty::ImplTrait(Box::new(self.ast_type_to_ty(inner))),
            Type::Pointer(inner, mutable) => {
                Ty::Ref {
                    inner: Box::new(self.ast_type_to_ty(inner)),
                    mutable: *mutable,
                }
            }
            Type::Path(path) => {
                // For now, treat paths as simple types (first component)
                if let Some(ident) = path.first() {
                    self.env.lookup_type_def(&ident.name)
                        .unwrap_or(Ty::TyVar(self.env.peek_next_ty_var()))
                } else {
                    Ty::TyVar(self.env.peek_next_ty_var())
                }
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zulon_parser::Parser;

    #[test]
    fn test_basic_type_checking() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }

    #[test]
    fn test_type_inference() {
        let source = r#"
            fn main() {
                let x = 42;
                let y: i32 = x;
                let z = x + y;
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }

    #[test]
    fn test_type_inference_with_annotations() {
        let source = r#"
            fn add(a: i32, b: i32) -> i32 {
                let result: i32 = a + b;
                result
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }

    #[test]
    fn test_expression_inference() {
        let source = r#"
            fn test() {
                let x = 10 + 20;
                let y = x * 2;
                let z = x == y;
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }

    #[test]
    fn test_function_call_inference() {
        let source = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            fn test() {
                let result = add(10, 20);
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }

    #[test]
    fn test_if_expression_inference() {
        let source = r#"
            fn test() {
                let x = if true { 10 } else { 20 };
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }

    #[test]
    fn test_block_trailing_inference() {
        let source = r#"
            fn test() -> i32 {
                let a = 10;
                let b = 20;
                a + b
            }
        "#;

        let mut parser = Parser::from_source(source);
        let ast = parser.parse().unwrap();

        let mut checker = TypeChecker::new();
        assert!(checker.check(&ast).is_ok());
    }
}
