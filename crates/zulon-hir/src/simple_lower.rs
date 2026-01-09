// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Simple AST to HIR transformation
//!
//! This is a simplified version that demonstrates the core concepts
//! without handling all edge cases.

use zulon_parser::ast;
use zulon_typeck::TypeChecker;

use super::hir::*;
use super::ty::HirTy;
use super::error::{LoweringError, Result};

/// Simple HIR lowering - demonstrates core concepts
pub struct SimpleLoweringContext {
    typeck: TypeChecker,  // Type checker for closure type inference
    next_id: NodeId,
}

impl SimpleLoweringContext {
    /// Create a new lowering context
    pub fn new() -> Self {
        SimpleLoweringContext {
            typeck: TypeChecker::new(),
            next_id: 0,
        }
    }

    /// Allocate a new node ID
    fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Lower an AST to HIR (simplified version)
    pub fn lower_ast(&mut self, ast: &ast::Ast) -> Result<HirCrate> {
        let mut items = Vec::new();

        for item in &ast.items {
            match &item.kind {
                ast::ItemKind::Function(func) => {
                    items.push(HirItem::Function(self.lower_function(func)?));
                }
                ast::ItemKind::Struct(struct_def) => {
                    items.push(HirItem::Struct(self.lower_struct(struct_def)?));
                }
                _ => {
                    // Skip other items for now
                    continue;
                }
            }
        }

        let span = items.first()
            .map(|item| item.span().clone())
            .unwrap_or_else(|| {
                zulon_parser::Span {
                    start: zulon_parser::Position { line: 0, column: 0 },
                    end: zulon_parser::Position { line: 0, column: 0 },
                }
            });

        Ok(HirCrate {
            items,
            span,
        })
    }

    /// Lower a function (simplified)
    fn lower_function(&mut self, func: &ast::Function) -> Result<HirFunction> {
        // For now, just use I32 for all params and return type
        let mut params = Vec::new();
        for param in &func.params {
            params.push(HirParam {
                name: param.name.name.clone(),
                ty: HirTy::I32,  // TODO: Get actual type
                span: param.span.clone(),
            });
        }

        // Lower function body
        let body = self.lower_block(&func.body)?;

        // Lower error type if present
        let error_type = if let Some(_) = &func.error_type {
            // TODO: Proper type conversion
            Some(HirTy::Enum {
                name: "Error".to_string(),
                generics: Vec::new(),
            })
        } else {
            None
        };

        // Lower effects if present
        let mut effects = Vec::new();
        for effect_ty in &func.effects {
            // Convert AST effect type to HIR type, preserving effect name
            let hir_ty = match effect_ty {
                ast::Type::Simple(ident) => {
                    // For effects, store the effect name in a simple type
                    HirTy::Struct {
                        name: ident.name.clone(),
                        generics: Vec::new(),
                    }
                }
                _ => {
                    // Fallback for other effect type forms
                    HirTy::Struct {
                        name: "UnknownEffect".to_string(),
                        generics: Vec::new(),
                    }
                }
            };
            effects.push(hir_ty);
        }

        // Copy attributes (e.g., #[test], #[ignore])
        let attributes = func.attributes.clone();

        Ok(HirFunction {
            id: self.alloc_id(),
            name: func.name.name.clone(),
            generics: Vec::new(),
            params,
            return_type: HirTy::I32,  // TODO: Get actual return type
            error_type,
            effects,
            attributes,
            body,
            span: func.name.span.clone(),
        })
    }

    /// Lower a struct definition
    fn lower_struct(&mut self, struct_def: &ast::Struct) -> Result<HirStruct> {
        // Lower struct fields
        let fields: Vec<HirField> = struct_def.fields.iter()
            .map(|field| {
                let ty = HirTy::I32;  // TODO: Get actual type from field annotation
                Ok(HirField {
                    name: field.name.name.clone(),
                    ty,
                    span: field.name.span.clone(),
                })
            })
            .collect::<Result<_>>()?;

        Ok(HirStruct {
            id: self.alloc_id(),
            name: struct_def.name.name.clone(),
            generics: Vec::new(),  // TODO: Handle generics
            fields,
            span: struct_def.name.span.clone(),
        })
    }

    /// Lower a block (simplified)
    fn lower_block(&mut self, block: &ast::Block) -> Result<HirBlock> {
        let mut statements = Vec::new();

        // Process all statements in the block
        for stmt in &block.statements {
            match &stmt.kind {
                ast::StatementKind::Local(local) => {
                    let local_ty = HirTy::I32;  // TODO: Get actual type

                    let init = if let Some(init) = &local.init {
                        Some(self.lower_expression(init)?)
                    } else {
                        None
                    };

                    statements.push(HirStatement::Local(HirLocal {
                        id: self.alloc_id(),
                        name: local.name.name.clone(),
                        ty: local_ty,
                        init,
                        span: local.name.span.clone(),
                    }));
                }

                ast::StatementKind::Expr(expr) => {
                    // Expressions in the statements array have semicolons
                    // They become Semi statements in HIR
                    let lowered_expr = self.lower_expression(expr)?;
                    statements.push(HirStatement::Semi(lowered_expr));
                }

                ast::StatementKind::Empty => {
                    // Skip empty statements
                }

                ast::StatementKind::Item(_) => {
                    // TODO: Handle nested items
                }
            }
        }

        // Handle trailing expression (if any)
        let trailing_expr = if let Some(expr) = &block.trailing_expr {
            Some(self.lower_expression(expr)?)
        } else {
            None
        };

        Ok(HirBlock {
            id: self.alloc_id(),
            statements,
            trailing_expr,
            ty: HirTy::Unit,  // TODO: Get actual block type
            span: block.span.clone(),
        })
    }

    /// Lower an expression (simplified)
    fn lower_expression(&mut self, expr: &ast::Expression) -> Result<HirExpression> {
        match &expr.kind {
            ast::ExpressionKind::Literal(lit) => {
                let hir_lit = self.lower_literal(lit)?;
                let ty = self.literal_type(&hir_lit);

                Ok(HirExpression::Literal(
                    hir_lit,
                    self.alloc_id(),
                    ty,
                    expr.span.clone(),
                ))
            }

            ast::ExpressionKind::Path(path) => {
                // Simple variable reference
                if path.len() == 1 {
                    // For now, use a placeholder type
                    // TODO: Look up variable type from symbol table
                    Ok(HirExpression::Variable(
                        path[0].name.clone(),
                        self.alloc_id(),
                        HirTy::I32,  // Placeholder - will be fixed when proper type propagation is implemented
                        expr.span.clone(),
                    ))
                } else {
                    Err(LoweringError::UnsupportedFeature {
                        feature: format!("qualified path: {:?}", path),
                        span: expr.span.clone(),
                    })
                }
            }

            ast::ExpressionKind::Binary(op, left, right) => {
                let left_expr = self.lower_expression(left)?;
                let right_expr = self.lower_expression(right)?;
                let hir_op = self.lower_bin_op(op)?;
                let ty = HirTy::I32;  // TODO: Infer actual type

                Ok(HirExpression::BinaryOp {
                    op: hir_op,
                    left: Box::new(left_expr),
                    right: Box::new(right_expr),
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Unary(op, operand) => {
                let operand_expr = self.lower_expression(operand)?;
                let hir_op = self.lower_unary_op(op)?;
                let ty = HirTy::I32;  // TODO: Infer actual type

                Ok(HirExpression::UnaryOp {
                    op: hir_op,
                    operand: Box::new(operand_expr),
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Assign(target, value) => {
                // Assignment: target = value
                // Lower both sides, then represent as a BinaryOp with Assign operator
                let target_expr = self.lower_expression(target)?;
                let value_expr = self.lower_expression(value)?;
                let ty = HirTy::I32;  // TODO: Infer actual type

                Ok(HirExpression::BinaryOp {
                    op: HirBinOp::Assign,
                    left: Box::new(target_expr),
                    right: Box::new(value_expr),
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::MacroInvocation { macro_name, args, delimiter: _ } => {
                // Handle builtin macros
                match macro_name.name.as_str() {
                    "assert_eq" => {
                        // assert_eq!(left, right) expands to:
                        // if left != right {
                        //     panic("assertion failed: left != right")
                        // }
                        if args.len() != 2 {
                            return Err(LoweringError::UnsupportedFeature {
                                feature: format!("assert_eq! requires exactly 2 arguments, got {}", args.len()),
                                span: expr.span.clone(),
                            });
                        }

                        let left = Box::new(self.lower_expression(&args[0])?);
                        let right = Box::new(self.lower_expression(&args[1])?);

                        // Create the comparison: left != right
                        let comparison = HirExpression::BinaryOp {
                            op: HirBinOp::NotEq,
                            left,
                            right,
                            ty: HirTy::Bool,
                            span: expr.span.clone(),
                        };

                        // Create the panic block (simplified - just return error code)
                        let panic_block = HirBlock {
                            id: self.alloc_id(),
                            statements: vec![],
                            trailing_expr: Some(HirExpression::Literal(
                                HirLiteral::Integer(1),
                                self.alloc_id(),
                                HirTy::I32,
                                expr.span.clone(),
                            )),
                            ty: HirTy::I32,
                            span: expr.span.clone(),
                        };

                        // Wrap in if statement
                        Ok(HirExpression::If {
                            condition: Box::new(comparison),
                            then_block: Box::new(panic_block),
                            else_block: None,
                            ty: HirTy::Unit,
                            span: expr.span.clone(),
                        })
                    }
                    "assert" => {
                        // assert!(condition) expands to:
                        // if !condition {
                        //     panic("assertion failed")
                        // }
                        if args.len() != 1 {
                            return Err(LoweringError::UnsupportedFeature {
                                feature: format!("assert! requires exactly 1 argument, got {}", args.len()),
                                span: expr.span.clone(),
                            });
                        }

                        let condition = Box::new(self.lower_expression(&args[0])?);

                        // Create the negation: !condition
                        let negated_condition = HirExpression::UnaryOp {
                            op: HirUnaryOp::Not,
                            operand: condition,
                            ty: HirTy::Bool,
                            span: expr.span.clone(),
                        };

                        // Create the panic block
                        let panic_block = HirBlock {
                            id: self.alloc_id(),
                            statements: vec![],
                            trailing_expr: Some(HirExpression::Literal(
                                HirLiteral::Integer(1),
                                self.alloc_id(),
                                HirTy::I32,
                                expr.span.clone(),
                            )),
                            ty: HirTy::I32,
                            span: expr.span.clone(),
                        };

                        // Wrap in if statement
                        Ok(HirExpression::If {
                            condition: Box::new(negated_condition),
                            then_block: Box::new(panic_block),
                            else_block: None,
                            ty: HirTy::Unit,
                            span: expr.span.clone(),
                        })
                    }
                    _ => {
                        // Unknown macro - for now, return error
                        Err(LoweringError::UnsupportedFeature {
                            feature: format!("macro {}", macro_name.name),
                            span: expr.span.clone(),
                        })
                    }
                }
            }

            ast::ExpressionKind::Call(func, args) => {
                let func_expr = self.lower_expression(func)?;
                let lowered_args: Result<Vec<_>> = args.iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect();
                let args = lowered_args?;
                let ty = HirTy::I32;  // TODO: Get actual return type

                Ok(HirExpression::Call {
                    func: Box::new(func_expr),
                    args,
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Block(block) => {
                let lowered_block = self.lower_block(block)?;
                Ok(HirExpression::Block(Box::new(lowered_block)))
            }

            ast::ExpressionKind::If(condition, then_block, else_block) => {
                let condition_expr = self.lower_expression(condition)?;
                let then_lowered = self.lower_block(then_block)?;
                let else_lowered = match else_block {
                    Some(block) => Some(Box::new(self.lower_block(block)?)),
                    None => None,
                };
                let ty = HirTy::I32;  // TODO: Unify branch types

                Ok(HirExpression::If {
                    condition: Box::new(condition_expr),
                    then_block: Box::new(then_lowered),
                    else_block: else_lowered,
                    ty,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Return(value) => {
                let lowered_value = match value {
                    Some(v) => Some(Box::new(self.lower_expression(v)?)),
                    None => None,
                };

                Ok(HirExpression::Return(lowered_value, expr.span.clone()))
            }

            ast::ExpressionKind::Break(_label) => {
                // TODO: Handle break labels and values
                Ok(HirExpression::Break(None, expr.span.clone()))
            }

            ast::ExpressionKind::Continue(_label) => {
                // TODO: Handle loop labels
                Ok(HirExpression::Continue(expr.span.clone()))
            }

            ast::ExpressionKind::Loop(body, _label) => {
                let lowered_body = Box::new(self.lower_block(body)?);
                Ok(HirExpression::Loop {
                    body: lowered_body,
                    ty: HirTy::Unit,  // Loops return unit unless broken
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::While(condition, body, _label) => {
                let lowered_condition = Box::new(self.lower_expression(condition)?);
                let lowered_body = Box::new(self.lower_block(body)?);
                Ok(HirExpression::While {
                    condition: lowered_condition,
                    body: lowered_body,
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::For(_local, _iter, _body, _label) => {
                // For loops need to be desugared into while loops or match expressions
                // For now, mark as unsupported
                Err(LoweringError::UnsupportedFeature {
                    feature: "for loop (will be desugared to while loop)".to_string(),
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Closure { params, return_type, body } => {
                // Type check the closure to get inferred types
                use zulon_typeck::Ty;
                let closure_ty = self.typeck.check_closure(params, return_type, body)?;

                // Extract parameter and return types from inferred closure type
                let (inferred_param_tys, inferred_return_ty) = match &closure_ty {
                    Ty::Function { params, return_type } => (params, return_type.as_ref()),
                    _ => {
                        // Fallback - shouldn't happen in normal operation
                        return Err(LoweringError::UnsupportedFeature {
                            feature: format!("closure without function type: {:?}", closure_ty),
                            span: expr.span.clone(),
                        });
                    }
                };

                // Lower closure parameters with inferred types
                let mut hir_params = Vec::new();
                for (i, param) in params.iter().enumerate() {
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

                // Capture analysis will be done during type checking
                let captures = Vec::new();

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

            ast::ExpressionKind::FieldAccess(object, field_name) => {
                let lowered_object = Box::new(self.lower_expression(object)?);

                // Get the type of the field access from type checker
                let field_ty = self.typeck.check_expression(expr)?;

                Ok(HirExpression::Field {
                    base: lowered_object,
                    field_name: field_name.name.clone(),
                    ty: HirTy::from(field_ty),
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Match(scrutinee_expr, arms) => {
                // Lower scrutinee (note: scrutinee_expr is Box<Expression>, so dereference it)
                let lowered_scrutinee = Box::new(self.lower_expression(&scrutinee_expr)?);

                // Lower match arms
                let mut hir_arms = Vec::new();
                for arm in arms {
                    // Lower pattern (simplified - just use the first pattern for now)
                    let hir_pattern = self.lower_pattern(&arm.patterns[0], &arm.span)?;

                    // Lower guard if present
                    let hir_guard = if let Some(guard_expr) = &arm.guard {
                        Some(self.lower_expression(guard_expr)?)
                    } else {
                        None
                    };

                    // Lower body
                    let hir_body = self.lower_expression(&arm.body)?;

                    hir_arms.push(HirMatchArm {
                        pattern: hir_pattern,
                        guard: hir_guard,
                        body: hir_body,
                        span: arm.span.clone(),
                    });
                }

                // Get the type of the match expression from type checker
                let match_ty = self.typeck.check_expression(expr)?;

                Ok(HirExpression::Match {
                    scrutinee: lowered_scrutinee,
                    arms: hir_arms,
                    ty: HirTy::from(match_ty),
                    span: expr.span.clone(),
                })
            }

            ast::ExpressionKind::Try(block, handlers) => {
                // Lower try block
                let try_block = self.lower_block(block)?;

                // Lower effect handlers
                let mut hir_handlers = Vec::new();
                for handler in handlers {
                    let mut hir_methods = Vec::new();
                    for method in &handler.methods {
                        let method_body = self.lower_block(&method.body)?;

                        hir_methods.push(HirEffectMethod {
                            name: method.name.name.clone(),
                            params: method.params.iter().map(|p| HirParam {
                                name: p.name.name.clone(),
                                ty: HirTy::I32,  // TODO: Get actual type
                                span: p.name.span.clone(),
                            }).collect(),
                            return_type: HirTy::I32,  // TODO: Get actual type
                            body: method_body,
                            span: method.name.span.clone(),
                        });
                    }

                    hir_handlers.push(HirEffectHandler {
                        effect_name: handler.effect_name.name.clone(),
                        methods: hir_methods,
                        span: handler.effect_name.span.clone(),
                    });
                }

                Ok(HirExpression::Try(HirTryBlock {
                    try_block: Box::new(try_block),
                    handlers: hir_handlers,
                    span: expr.span.clone(),
                }))
            }

            _ => Err(LoweringError::UnsupportedFeature {
                feature: format!("expression: {:?}", expr.kind),
                span: expr.span.clone(),
            }),
        }
    }

    /// Lower a pattern (simplified - literal patterns only)
    fn lower_pattern(&mut self, pattern: &ast::Pattern, parent_span: &zulon_parser::Span) -> Result<HirPattern> {
        match pattern {
            ast::Pattern::Wildcard => {
                Ok(HirPattern::Wildcard(parent_span.clone()))
            }
            ast::Pattern::Literal(lit) => {
                let hir_lit = self.lower_literal(lit)?;
                Ok(HirPattern::Literal(hir_lit, parent_span.clone()))
            }
            ast::Pattern::Identifier(ident) => {
                // Identifier pattern is a binding
                // For now, use i32 as default type
                Ok(HirPattern::Binding(ident.name.clone(), HirTy::I32, ident.span.clone()))
            }
            _ => {
                // For now, only support simple patterns
                Err(LoweringError::UnsupportedFeature {
                    feature: format!("pattern: {:?}", pattern),
                    span: parent_span.clone(),
                })
            }
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

    /// Get type of a literal
    fn literal_type(&self, lit: &HirLiteral) -> HirTy {
        match lit {
            HirLiteral::Bool(_) => HirTy::Bool,
            HirLiteral::Integer(_) => HirTy::I32,
            HirLiteral::Float(_) => HirTy::F64,
            HirLiteral::Char(_) => HirTy::Char,
            HirLiteral::String(_) => HirTy::String,
            HirLiteral::Unit => HirTy::Unit,
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
            ast::UnaryOp::BitNot => Ok(HirUnaryOp::Not),
            ast::UnaryOp::Deref => Ok(HirUnaryOp::Deref),
            ast::UnaryOp::Ref => Ok(HirUnaryOp::Ref),
            ast::UnaryOp::Borrow => Ok(HirUnaryOp::Ref),
            ast::UnaryOp::BorrowMut => Ok(HirUnaryOp::RefMut),
        }
    }

    /// Lower a type annotation (simplified version)
    fn lower_type(&self, ty: Option<&ast::Type>) -> Result<HirTy> {
        match ty {
            None => Ok(HirTy::Unit),  // TODO: Use proper type variable
            Some(ast_type) => {
                // For now, just map common types
                // TODO: Implement proper type lowering
                match ast_type {
                    ast::Type::Simple(ident) => match ident.name.as_str() {
                        "i32" => Ok(HirTy::I32),
                        "i64" => Ok(HirTy::I64),
                        "f64" => Ok(HirTy::F64),
                        "bool" => Ok(HirTy::Bool),
                        "string" => Ok(HirTy::String),
                        "char" => Ok(HirTy::Char),
                        _ => Ok(HirTy::Unit),  // TODO: Unknown types
                    },
                    ast::Type::Path(path) if !path.is_empty() => {
                        // For now, just use the last component
                        let last = path.last().unwrap();
                        match last.name.as_str() {
                            "i32" => Ok(HirTy::I32),
                            "i64" => Ok(HirTy::I64),
                            "f64" => Ok(HirTy::F64),
                            "bool" => Ok(HirTy::Bool),
                            "string" => Ok(HirTy::String),
                            "char" => Ok(HirTy::Char),
                            _ => Ok(HirTy::Unit),  // TODO: Unknown types
                        }
                    }
                    _ => Ok(HirTy::Unit),  // TODO: Handle other types
                }
            }
        }
    }
}

/// Convenience function to lower AST to HIR
pub fn lower_ast_simple(ast: &ast::Ast) -> Result<HirCrate> {
    let mut ctx = SimpleLoweringContext::new();
    ctx.lower_ast(ast)
}
