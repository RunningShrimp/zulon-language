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
        for _ in &func.effects {
            // TODO: Proper type conversion
            effects.push(HirTy::Struct {
                name: "Effect".to_string(),
                generics: Vec::new(),
            });
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
                    Ok(HirExpression::Variable(
                        path[0].name.clone(),
                        self.alloc_id(),
                        HirTy::I32,  // TODO: Get actual type
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
