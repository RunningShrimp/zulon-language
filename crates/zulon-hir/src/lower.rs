// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! AST to HIR lowering
//!
//! Converts typed AST into HIR, making types explicit and desugaring.

use std::collections::HashMap;

use zulon_parser::ast;
use zulon_typeck::{TypeChecker, Ty};

use super::hir::*;
use super::ty::HirTy;
use super::error::{LoweringError, Result};

/// Context for AST to HIR lowering
pub struct LoweringContext {
    /// Type checker (for getting types)
    typeck: TypeChecker,

    /// Node ID counter
    next_id: NodeId,

    /// Variable types (from type checking)
    var_types: HashMap<String, HirTy>,
}

impl LoweringContext {
    /// Create a new lowering context
    pub fn new() -> Self {
        LoweringContext {
            typeck: TypeChecker::new(),
            next_id: 0,
            var_types: HashMap::new(),
        }
    }

    /// Allocate a new node ID
    fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Lower an entire AST
    pub fn lower_ast(&mut self, ast: &ast::Ast) -> Result<HirCrate> {
        let mut items = Vec::new();

        for item in &ast.items {
            items.push(self.lower_item(item)?);
        }

        // Create a dummy span for the entire AST
        let span = if let Some(item) = items.first() {
            item.span().clone()
        } else {
            zulon_parser::Span {
                start: zulon_parser::Position { line: 0, column: 0 },
                end: zulon_parser::Position { line: 0, column: 0 },
            }
        };

        Ok(HirCrate {
            items,
            span,
        })
    }

    /// Lower a top-level item
    fn lower_item(&mut self, item: &ast::Item) -> Result<HirItem> {
        match &item.kind {
            ast::ItemKind::Function(func) => {
                Ok(HirItem::Function(self.lower_function(func)?))
            }

            ast::ItemKind::Struct(struct_def) => {
                Ok(HirItem::Struct(self.lower_struct(struct_def)?))
            }

            ast::ItemKind::Enum(enum_def) => {
                Ok(HirItem::Enum(self.lower_enum(enum_def)?))
            }

            ast::ItemKind::Trait(trait_def) => {
                Ok(HirItem::Trait(self.lower_trait(trait_def)?))
            }

            ast::ItemKind::Impl(impl_block) => {
                Ok(HirItem::Impl(self.lower_impl(impl_block)?))
            }

            ast::ItemKind::Mod(module) => {
                Ok(HirItem::Mod(self.lower_module(module)?))
            }

            _ => Err(LoweringError::UnsupportedFeature {
                feature: format!("item kind: {:?}", item.kind),
                span: item.span.clone(),
            }),
        }
    }

    /// Lower a function definition
    fn lower_function(&mut self, func: &ast::Function) -> Result<HirFunction> {
        // Type check the function first
        let func_ty = self.typeck.check_function(func)?;

        // Extract return type from function type
        let return_type = match func_ty {
            Ty::Function { return_type, .. } => {
                (*return_type).into()
            }
            _ => HirTy::Unit,
        };

        // Lower error type if present (from `-> Type | Error` syntax)
        let error_type = if let Some(ast_error_type) = &func.error_type {
            // Convert AST type to HIR type
            // TODO: Proper type conversion - for now use placeholder enum
            Some(HirTy::Enum {
                name: ast_error_type.to_string(),
                generics: Vec::new(),
            })
        } else {
            None
        };

        // Lower effects if present (from `-> Type | Error | Effect1 + Effect2` syntax)
        let mut effects = Vec::new();
        for ast_effect in &func.effects {
            // Convert AST type to HIR type
            // TODO: Proper type conversion - for now use placeholder struct
            effects.push(HirTy::Struct {
                name: ast_effect.to_string(),
                generics: Vec::new(),
            });
        }

        // Lower parameters
        let mut params = Vec::new();
        for param in &func.params {
            let param_ty: HirTy = self.typeck.env.get_binding(&param.name.name)
                .cloned()
                .unwrap_or(HirTy::Unit);

            params.push(HirParam {
                name: param.name.name.clone(),
                ty: param_ty,
                span: param.span.clone(),
            });
        }

        // Lower body
        let body = self.lower_block(&func.body)?;

        Ok(HirFunction {
            id: self.alloc_id(),
            name: func.name.name.clone(),
            generics: Vec::new(), // TODO: lower generics
            params,
            return_type,
            error_type,
            effects,
            body,
            span: func.name.span.clone(),
        })
    }

    /// Lower a block
    fn lower_block(&mut self, block: &ast::Block) -> Result<HirBlock> {
        let mut statements = Vec::new();
        let mut trailing_expr = None;

        // Type check the block
        let block_ty = self.typeck.check_block(block)?;
        let block_ty: HirTy = block_ty.into();

        for stmt in &block.statements {
            match stmt {
                ast::Statement::Local(local) => {
                    statements.push(HirStatement::Local(self.lower_local(local)?));
                }

                ast::Statement::Expression(expr) => {
                    // This could be a trailing expression
                    trailing_expr = Some(self.lower_expression(expr)?);
                }

                ast::Statement::Semi(expr) => {
                    statements.push(HirStatement::Semi(self.lower_expression(expr)?));
                }
            }
        }

        Ok(HirBlock {
            id: self.alloc_id(),
            statements,
            trailing_expr,
            ty: block_ty,
            span: block.span.clone(),
        })
    }

    /// Lower a local variable declaration
    fn lower_local(&mut self, local: &ast::Local) -> Result<HirLocal> {
        // Get the type from type checking
        let local_ty: HirTy = self.typeck.env.get_binding(&local.name.name)
            .cloned()
            .ok_or_else(|| LoweringError::MissingTypeAnnotation {
                name: local.name.name.clone(),
                span: local.name.span.clone(),
            })?;

        // Lower initializer if present
        let init = if let Some(init) = &local.init {
            Some(self.lower_expression(init)?)
        } else {
            None
        };

        Ok(HirLocal {
            id: self.alloc_id(),
            name: local.name.name.clone(),
            ty: local_ty,
            init,
            span: local.name.span.clone(),
        })
    }

    /// Lower an expression
    fn lower_expression(&mut self, expr: &ast::Expression) -> Result<HirExpression> {
        // Type check the expression
        let expr_ty = self.typeck.check_expression(expr)?;
        let ty: HirTy = expr_ty.into();

        match &expr.kind {
            ast::ExpressionKind::Literal(lit) => {
                Ok(HirExpression::Literal(
                    self.lower_literal(lit)?,
                    self.alloc_id(),
                    ty,
                    expr.span.clone(),
                ))
            }

            ast::ExpressionKind::Variable(name) => {
                Ok(HirExpression::Variable(
                    name.clone(),
                    self.alloc_id(),
                    ty,
                    expr.span.clone(),
                ))
            }

            ast::ExpressionKind::BinaryOp { op, left, right } => {
                let left_expr = self.lower_expression(left)?;
                let right_expr = self.lower_expression(right)?;
                let hir_op = self.lower_bin_op(op)?;

                Ok(HirExpression::BinaryOp {
                    op: hir_op,
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::UnaryOp { op, operand } => {
                let operand_expr = self.lower_expression(operand)?;
                let hir_op = self.lower_unary_op(op)?;

                Ok(HirExpression::UnaryOp {
                    op: hir_op,
                    operand: Box::new(operand_expr),
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Call { func, args } => {
                let func_expr = self.lower_expression(func)?;
                let lowered_args: Result<Vec<_>> = args.iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect();
                let args = lowered_args?;

                Ok(HirExpression::Call {
                    func: Box::new(func_expr),
                    args,
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::If { condition, then_block, else_block } => {
                let condition_expr = self.lower_expression(condition)?;
                let then_lowered = self.lower_block(then_block)?;
                let else_lowered = match else_block {
                    Some(block) => Some(Box::new(self.lower_block(block)?)),
                    None => None,
                };

                Ok(HirExpression::If {
                    condition: Box::new(condition_expr),
                    then_block: Box::new(then_lowered),
                    else_block: else_lowered,
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Block(block) => {
                let lowered_block = self.lower_block(block)?;
                Ok(HirExpression::Block(Box::new(lowered_block)))
            }

            ast::ExpressionKind::Return(value) => {
                let lowered_value = match value {
                    Some(v) => Some(Box::new(self.lower_expression(v)?)),
                    None => None,
                };

                Ok(HirExpression::Return(lowered_value, expr.span.clone()))
            }

            ast::ExpressionKind::Break(value) => {
                let lowered_value = match value {
                    Some(v) => Some(Box::new(self.lower_expression(v)?)),
                    None => None,
                };

                Ok(HirExpression::Break(lowered_value, expr.span.clone()))
            }

            ast::ExpressionKind::Continue(_label) => {
                // Ignore label for now
                Ok(HirExpression::Continue(expr.span.clone()))
            }

            // Throw statement (error handling)
            ast::ExpressionKind::Throw(error_expr) => {
                let lowered_error = self.lower_expression(error_expr)?;
                Ok(HirExpression::Throw(Box::new(lowered_error), expr.span.clone()))
            }

            // Question mark operator (error propagation)
            ast::ExpressionKind::QuestionMark(inner_expr) => {
                let lowered_inner = self.lower_expression(inner_expr)?;
                // TODO: Proper type inference for the success type
                // For now, use a placeholder type
                let ty = HirTy::I32;  // Will be replaced by type checker
                Ok(HirExpression::QuestionMark(Box::new(lowered_inner), ty, expr.span.clone()))
            }

            // Loop expression: loop { body }
            ast::ExpressionKind::Loop(body, _label) => {
                let lowered_body = self.lower_block(body)?;
                Ok(HirExpression::Loop {
                    body: Box::new(lowered_body),
                    ty: HirTy::Unit, // Loops never return a value (unless breaking with one)
                    span: expr.span.clone(),
                })
            }

            // While loop: while cond { body }
            ast::ExpressionKind::While(condition, body, _label) => {
                let lowered_condition = self.lower_expression(condition)?;
                let lowered_body = self.lower_block(body)?;
                Ok(HirExpression::While {
                    condition: Box::new(lowered_condition),
                    body: Box::new(lowered_body),
                    span: expr.span.clone(),
                })
            }

            // For loop: for pat in iter { body }
            // This will be desugared later in MIR/LIR lowering
            ast::ExpressionKind::For(pattern, iter, body, _label) => {
                let lowered_pattern = self.lower_pattern(pattern)?;
                let lowered_iter = self.lower_expression(iter)?;
                let lowered_body = self.lower_block(body)?;
                Ok(HirExpression::For {
                    pattern: lowered_pattern,
                    iter: Box::new(lowered_iter),
                    body: Box::new(lowered_body),
                    span: expr.span.clone(),
                })
            }

            // Closure: |params| body or |params: Type| -> Type { body }
            ast::ExpressionKind::Closure { params, return_type, body } => {
                // Type check the closure to get inferred types
                let closure_ty = self.typeck.check_closure(params, return_type, body)?;

                // Extract parameter and return types from inferred closure type
                let (inferred_param_tys, inferred_return_ty) = match &closure_ty {
                    Ty::Function { params, return_type } => (params, return_type.as_ref()),
                    _ => {
                        // Fallback if type checking didn't produce a function type
                        // This shouldn't happen in normal operation
                        return Err(LoweringError::UnsupportedFeature {
                            feature: format!("closure without function type: {:?}", closure_ty),
                            span: expr.span.clone(),
                        });
                    }
                };

                // Lower closure parameters with inferred types
                let mut hir_params = Vec::new();
                let mut param_names = Vec::new();
                for (i, param) in params.iter().enumerate() {
                    param_names.push(param.name.name.clone());

                    // Use inferred type if no explicit annotation, otherwise use annotation
                    let param_ty = if let Some(type_ann) = &param.type_annotation {
                        self.lower_type(Some(type_ann))?
                    } else {
                        // Use inferred type from type checker
                        inferred_param_tys.get(i)
                            .map(|ty| HirTy::from(ty.clone()))
                            .unwrap_or(HirTy::Unit)
                    };

                    hir_params.push(HirClosureParam {
                        name: param.name.name.clone(),
                        ty: param_ty,
                        span: param.name.span.clone(),
                    });
                }

                // Lower return type with inferred type
                let return_ty = if let Some(ty) = return_type {
                    self.lower_type(Some(ty))?
                } else {
                    // Use inferred return type from type checker
                    HirTy::from(inferred_return_ty.clone())
                };

                // Lower closure body
                let lowered_body = Box::new(self.lower_expression(body)?);

                // Perform capture analysis using a simple environment
                // TODO: Integrate with proper type checker environment
                use super::capture::{SimpleEnvironment, analyze_captures};
                let mut env = SimpleEnvironment::new();

                // Add outer scope variables to environment (from var_types)
                for (name, ty) in &self.var_types {
                    env.add(name.clone(), ty.clone());
                }

                let capture_analysis = analyze_captures(&env, &lowered_body, param_names);

                // Extract captures from the analysis
                let captures = capture_analysis.captures;

                // Use the inferred closure type
                let closure_ty_hir = HirTy::from(closure_ty);

                Ok(HirExpression::Closure {
                    params: hir_params,
                    return_ty,
                    body: lowered_body,
                    captures,
                    ty: closure_ty_hir,
                    span: expr.span.clone(),
                })
            }

            _ => Err(LoweringError::UnsupportedFeature {
                feature: format!("expression: {:?}", expr.kind),
                span: expr.span.clone(),
            }),
        }
    }

    /// Lower a literal
    fn lower_literal(&mut self, lit: &ast::Literal) -> Result<HirLiteral> {
        match lit {
            ast::Literal::Bool(b) => Ok(HirLiteral::Bool(*b)),
            ast::Literal::Int(n) => Ok(HirLiteral::Integer(*n as u64)),
            ast::Literal::Float(f) => Ok(HirLiteral::Float(*f)),
            ast::Literal::Char(c) => Ok(HirLiteral::Char(*c)),
            ast::Literal::String(s) => Ok(HirLiteral::String(s.clone())),
            ast::Literal::Null => Ok(HirLiteral::Unit),
        }
    }

    /// Lower a binary operator
    fn lower_bin_op(&mut self, op: &ast::BinaryOp) -> Result<HirBinOp> {
        match op {
            ast::BinaryOp::Add => Ok(HirBinOp::Add),
            ast::BinaryOp::Sub => Ok(HirBinOp::Sub),
            ast::BinaryOp::Mul => Ok(HirBinOp::Mul),
            ast::BinaryOp::Div => Ok(HirBinOp::Div),
            ast::BinaryOp::Mod => Ok(HirBinOp::Mod),

            ast::BinaryOp::BitAnd => Ok(HirBinOp::BitAnd),
            ast::BinaryOp::BitOr => Ok(HirBinOp::BitOr),
            ast::BinaryOp::BitXor => Ok(HirBinOp::BitXor),
            ast::BinaryOp::LeftShift => Ok(HirBinOp::LeftShift),
            ast::BinaryOp::RightShift => Ok(HirBinOp::RightShift),

            ast::BinaryOp::And => Ok(HirBinOp::And),
            ast::BinaryOp::Or => Ok(HirBinOp::Or),

            ast::BinaryOp::Eq => Ok(HirBinOp::Eq),
            ast::BinaryOp::NotEq => Ok(HirBinOp::NotEq),
            ast::BinaryOp::Less => Ok(HirBinOp::Less),
            ast::BinaryOp::LessEq => Ok(HirBinOp::LessEq),
            ast::BinaryOp::Greater => Ok(HirBinOp::Greater),
            ast::BinaryOp::GreaterEq => Ok(HirBinOp::GreaterEq),
        }
    }

    /// Lower a unary operator
    fn lower_unary_op(&mut self, op: &ast::UnaryOp) -> Result<HirUnaryOp> {
        match op {
            ast::UnaryOp::Neg => Ok(HirUnaryOp::Neg),
            ast::UnaryOp::Not => Ok(HirUnaryOp::Not),
            ast::UnaryOp::BitNot => Ok(HirUnaryOp::Not), // Map BitNot to Not
            ast::UnaryOp::Deref => Ok(HirUnaryOp::Deref),
            ast::UnaryOp::Ref => Ok(HirUnaryOp::Ref),
            ast::UnaryOp::Borrow => Ok(HirUnaryOp::Ref),
            ast::UnaryOp::BorrowMut => Ok(HirUnaryOp::RefMut),
        }
    }

    /// Lower a pattern
    fn lower_pattern(&mut self, pattern: &ast::Pattern) -> Result<HirPattern> {
        use zulon_parser::Span;

        // Helper to create a dummy span
        let dummy_span = || Span {
            start: zulon_parser::lexer::Position { line: 0, column: 0 },
            end: zulon_parser::lexer::Position { line: 0, column: 0 },
        };

        match pattern {
            ast::Pattern::Wildcard => {
                Ok(HirPattern::Wildcard(dummy_span()))
            }

            ast::Pattern::Identifier(ident) => {
                // For simple identifier patterns in for loops, use Binding
                // We need to infer the type from the iterator
                let ty = HirTy::Infer(
                    Box::new(std::any::TypeId::of::<()>())
                ); // Placeholder - will be inferred later

                Ok(HirPattern::Binding(
                    ident.name.clone(),
                    ty,
                    ident.span.clone(),
                ))
            }

            ast::Pattern::Literal(lit) => {
                Ok(HirPattern::Literal(self.lower_literal(lit)?, dummy_span()))
            }

            ast::Pattern::Tuple(patterns) => {
                let lowered_patterns: Result<Vec<_>> = patterns
                    .iter()
                    .map(|p| self.lower_pattern(p))
                    .collect();
                Ok(HirPattern::Tuple(lowered_patterns?, dummy_span()))
            }

            ast::Pattern::TupleVariant(path, patterns) => {
                // Convert tuple-like variant pattern to HIR
                // For example: Outcome::Ok(value) -> EnumVariant { ... }
                let enum_name = path.first().map(|ident| ident.name.clone()).unwrap_or_default();
                let variant_name = path.last().map(|ident| ident.name.clone()).unwrap_or_default();

                // For MVP: Use first pattern as inner (handles single-field variants like Outcome::Ok)
                let inner = if patterns.len() == 1 {
                    Some(Box::new(self.lower_pattern(&patterns[0])?))
                } else if patterns.is_empty() {
                    None
                } else {
                    // Multiple patterns - create a tuple pattern as inner
                    let lowered_patterns: Result<Vec<_>> = patterns
                        .iter()
                        .map(|p| self.lower_pattern(p))
                        .collect();
                    Some(Box::new(HirPattern::Tuple(lowered_patterns?, dummy_span())))
                };

                Ok(HirPattern::EnumVariant {
                    enum_name,
                    variant_name,
                    inner,
                    ty: HirTy::I32, // TODO: Get actual type
                    span: dummy_span(),
                })
            }

            // For now, treat complex patterns as unsupported
            _ => Err(LoweringError::UnsupportedFeature {
                feature: format!("pattern: {:?}", pattern),
                span: dummy_span(),
            }),
        }
    }

    /// Lower a struct definition
    fn lower_struct(&mut self, struct_def: &ast::Struct) -> Result<HirStruct> {
        let mut fields = Vec::new();

        for field in &struct_def.fields {
            // Type check to get the field type
            let field_ty: HirTy = HirTy::I32; // Placeholder - TODO: get actual type

            fields.push(HirField {
                name: field.name.name.clone(),
                ty: field_ty,
                span: field.span.clone(),
            });
        }

        Ok(HirStruct {
            id: self.alloc_id(),
            name: struct_def.name.name.clone(),
            generics: Vec::new(), // TODO: lower generics
            fields,
            span: struct_def.name.span.clone(),
        })
    }

    /// Lower an enum definition
    fn lower_enum(&mut self, enum_def: &ast::Enum) -> Result<HirEnum> {
        let mut variants = Vec::new();

        for variant in &enum_def.variants {
            let fields = variant.fields.iter().map(|field| {
                // TODO: get actual type from field
                HirField {
                    name: match field {
                        ast::VariantField::Named(name, _ty) => name.name.clone(),
                        ast::VariantField::Unnamed(_) => String::from("unnamed"),
                    },
                    ty: HirTy::Unit,
                    span: enum_def.name.span.clone(),
                }
            }).collect();

            variants.push(HirVariant {
                name: variant.name.name.clone(),
                fields,
                span: variant.span.clone(),
            });
        }

        Ok(HirEnum {
            id: self.alloc_id(),
            name: enum_def.name.name.clone(),
            generics: Vec::new(), // TODO: lower generics
            variants,
            span: enum_def.name.span.clone(),
        })
    }

    /// Lower a trait definition
    fn lower_trait(&mut self, trait_def: &ast::Trait) -> Result<HirTrait> {
        let items = Vec::new(); // TODO: lower trait items

        Ok(HirTrait {
            id: self.alloc_id(),
            name: trait_def.name.name.clone(),
            generics: Vec::new(),
            items,
            span: trait_def.name.span.clone(),
        })
    }

    /// Lower an impl block
    fn lower_impl(&mut self, impl_block: &ast::Impl) -> Result<HirImpl> {
        let target_type: HirTy = HirTy::I32; // Placeholder - TODO: get actual type
        let mut items = Vec::new();

        for item in &impl_block.items {
            items.push(self.lower_item(item)?);
        }

        Ok(HirImpl {
            id: self.alloc_id(),
            generics: Vec::new(),
            target_trait: None, // TODO: extract trait name
            target_type,
            items,
            span: impl_block.span.clone(),
        })
    }

    /// Lower a module
    fn lower_module(&mut self, module: &ast::Module) -> Result<HirMod> {
        let mut items = Vec::new();

        for item in &module.items {
            items.push(self.lower_item(item)?);
        }

        Ok(HirMod {
            id: self.alloc_id(),
            name: module.name.name.clone(),
            items,
            span: module.name.span.clone(),
        })
    }
}

/// Convenience function to lower an entire AST to HIR
pub fn lower_ast(ast: &ast::Ast) -> Result<HirCrate> {
    let mut ctx = LoweringContext::new();
    ctx.lower_ast(ast)
}
