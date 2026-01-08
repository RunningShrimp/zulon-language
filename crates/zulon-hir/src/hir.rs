// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! HIR node definitions
//!
//! HIR nodes are typed, desugared representations of AST nodes.

use zulon_parser::ast::Span;

use super::ty::HirTy;

/// Unique identifier for HIR nodes
pub type NodeId = usize;

/// HIR crate (compilation unit)
#[derive(Debug, Clone)]
pub struct HirCrate {
    pub items: Vec<HirItem>,
    pub span: Span,
}

/// Top-level items
#[derive(Debug, Clone)]
pub enum HirItem {
    Function(HirFunction),
    Struct(HirStruct),
    Enum(HirEnum),
    Trait(HirTrait),
    Impl(HirImpl),
    Mod(HirMod),
}

/// Function definition
#[derive(Debug, Clone)]
pub struct HirFunction {
    pub id: NodeId,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub params: Vec<HirParam>,
    pub return_type: HirTy,
    /// Error type for functions using `fn() -> T | E` syntax
    pub error_type: Option<HirTy>,
    /// Effects for functions using `fn() -> T | E | Effect1 + Effect2` syntax
    pub effects: Vec<HirTy>,
    /// Attributes on this function (e.g., #[test], #[ignore])
    pub attributes: Vec<zulon_parser::ast::Attribute>,
    pub body: HirBlock,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: HirTy,
    pub span: Span,
}

/// Closure parameter (similar to HirParam but for closures)
#[derive(Debug, Clone)]
pub struct HirClosureParam {
    pub name: String,
    pub ty: HirTy,
    pub span: Span,
}

/// Capture mode for closure captures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirCaptureMode {
    /// Immutable borrow: &x
    ImmutableRef,
    /// Mutable borrow: &mut x
    MutableRef,
    /// By value (move/Copy): x
    ByValue,
}

/// A captured variable in a closure
#[derive(Debug, Clone)]
pub struct HirCapture {
    /// Name of the captured variable
    pub name: String,
    /// How it's captured
    pub mode: HirCaptureMode,
    /// Type of the captured variable
    pub ty: HirTy,
    /// Span where it's captured
    pub span: Span,
}

/// Block expression
#[derive(Debug, Clone)]
pub struct HirBlock {
    pub id: NodeId,
    pub statements: Vec<HirStatement>,
    pub trailing_expr: Option<HirExpression>,
    pub ty: HirTy,
    pub span: Span,
}

/// Statements
#[derive(Debug, Clone)]
pub enum HirStatement {
    Local(HirLocal),
    Item(HirItem),
    Expression(HirExpression),
    Semi(HirExpression),
}

/// Local variable declaration
#[derive(Debug, Clone)]
pub struct HirLocal {
    pub id: NodeId,
    pub name: String,
    pub ty: HirTy,
    pub init: Option<HirExpression>,
    pub span: Span,
}

/// Expressions (all explicitly typed)
#[derive(Debug, Clone)]
pub enum HirExpression {
    /// Literal
    Literal(HirLiteral, NodeId, HirTy, Span),

    /// Variable reference
    Variable(String, NodeId, HirTy, Span),

    /// Binary operation
    BinaryOp {
        op: HirBinOp,
        left: Box<HirExpression>,
        right: Box<HirExpression>,
        ty: HirTy,
        span: Span,
    },

    /// Unary operation
    UnaryOp {
        op: HirUnaryOp,
        operand: Box<HirExpression>,
        ty: HirTy,
        span: Span,
    },

    /// Function call
    Call {
        func: Box<HirExpression>,
        args: Vec<HirExpression>,
        ty: HirTy,
        span: Span,
    },

    /// Method call (desugared to function call)
    MethodCall {
        receiver: Box<HirExpression>,
        method_name: String,
        args: Vec<HirExpression>,
        ty: HirTy,
        span: Span,
    },

    /// If expression
    If {
        condition: Box<HirExpression>,
        then_block: Box<HirBlock>,
        else_block: Option<Box<HirBlock>>,
        ty: HirTy,
        span: Span,
    },

    /// Loop expression
    Loop {
        body: Box<HirBlock>,
        ty: HirTy,
        span: Span,
    },

    /// While loop
    While {
        condition: Box<HirExpression>,
        body: Box<HirBlock>,
        span: Span,
    },

    /// For loop (desugared)
    For {
        pattern: HirPattern,
        iter: Box<HirExpression>,
        body: Box<HirBlock>,
        span: Span,
    },

    /// Block expression
    Block(Box<HirBlock>),

    /// Match expression
    Match {
        scrutinee: Box<HirExpression>,
        arms: Vec<HirMatchArm>,
        ty: HirTy,
        span: Span,
    },

    /// Tuple expression
    Tuple(Vec<HirExpression>, HirTy, Span),

    /// Array expression
    Array {
        elements: Vec<HirExpression>,
        ty: HirTy,
        span: Span,
    },

    /// Index expression
    Index {
        base: Box<HirExpression>,
        index: Box<HirExpression>,
        ty: HirTy,
        span: Span,
    },

    /// Field access
    Field {
        base: Box<HirExpression>,
        field_name: String,
        ty: HirTy,
        span: Span,
    },

    /// Return expression
    Return(Option<Box<HirExpression>>, Span),

    /// Break expression
    Break(Option<Box<HirExpression>>, Span),

    /// Continue expression
    Continue(Span),

    /// Closure (lambda function)
    Closure {
        /// Closure parameters
        params: Vec<HirClosureParam>,
        /// Return type
        return_ty: HirTy,
        /// Closure body
        body: Box<HirExpression>,
        /// Captured variables (analysis filled in during type checking)
        captures: Vec<HirCapture>,
        /// Closure type (function pointer type)
        ty: HirTy,
        span: Span,
    },

    /// Struct literal
    Struct {
        name: String,
        fields: Vec<(String, HirExpression)>,
        ty: HirTy,
        span: Span,
    },

    /// Throw statement (error handling)
    Throw(Box<HirExpression>, Span),

    /// Question mark operator (error propagation)
    QuestionMark(Box<HirExpression>, HirTy, Span),
}

impl HirExpression {
    /// Get the type of this expression
    pub fn ty(&self) -> &HirTy {
        match self {
            HirExpression::Literal(_, _, ty, _) => ty,
            HirExpression::Variable(_, _, ty, _) => ty,
            HirExpression::BinaryOp { ty, .. } => ty,
            HirExpression::UnaryOp { ty, .. } => ty,
            HirExpression::Call { ty, .. } => ty,
            HirExpression::MethodCall { ty, .. } => ty,
            HirExpression::If { ty, .. } => ty,
            HirExpression::Loop { ty, .. } => ty,
            HirExpression::Match { ty, .. } => ty,
            HirExpression::Tuple(_, ty, _) => ty,
            HirExpression::Array { ty, .. } => ty,
            HirExpression::Index { ty, .. } => ty,
            HirExpression::Field { ty, .. } => ty,
            HirExpression::Struct { ty, .. } => ty,
            HirExpression::Block(block) => &block.ty,
            HirExpression::Return(..) | HirExpression::Break(..) => &HirTy::Never,
            HirExpression::Continue(_) => &HirTy::Never,
            HirExpression::While { .. } | HirExpression::For { .. } => &HirTy::Unit,
            HirExpression::Closure { ty, .. } => ty,
            HirExpression::Throw(..) => &HirTy::Never,  // throw doesn't return normally
            HirExpression::QuestionMark(_, ty, _) => ty,  // ? returns the success type
        }
    }

    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            HirExpression::Literal(_, _, _, span) => span,
            HirExpression::Variable(_, _, _, span) => span,
            HirExpression::BinaryOp { span, .. } => span,
            HirExpression::UnaryOp { span, .. } => span,
            HirExpression::Call { span, .. } => span,
            HirExpression::MethodCall { span, .. } => span,
            HirExpression::If { span, .. } => span,
            HirExpression::Loop { span, .. } => span,
            HirExpression::While { span, .. } => span,
            HirExpression::For { span, .. } => span,
            HirExpression::Block(block) => &block.span,
            HirExpression::Match { span, .. } => span,
            HirExpression::Tuple(_, _, span) => span,
            HirExpression::Array { span, .. } => span,
            HirExpression::Index { span, .. } => span,
            HirExpression::Field { span, .. } => span,
            HirExpression::Struct { span, .. } => span,
            HirExpression::Return(_, span) => span,
            HirExpression::Break(_, span) => span,
            HirExpression::Continue(span) => span,
            HirExpression::Closure { span, .. } => span,
            HirExpression::Throw(_, span) => span,
            HirExpression::QuestionMark(_, _, span) => span,
        }
    }
}

/// Literal values
#[derive(Debug, Clone)]
pub enum HirLiteral {
    Bool(bool),
    Integer(u64),
    Float(f64),
    Char(char),
    String(String),
    Unit,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HirBinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,

    // Bitwise
    BitAnd, BitOr, BitXor, LeftShift, RightShift,

    // Logical
    And, Or,

    // Comparison
    Eq, NotEq, Less, LessEq, Greater, GreaterEq,

    // Assignment (desugared)
    Assign,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HirUnaryOp {
    Neg,   // -x
    Not,   // !x
    Deref, // *x
    Ref,   // &x
    RefMut, // &mut x
}

/// Patterns (for match, let, etc.)
#[derive(Debug, Clone)]
pub enum HirPattern {
    /// Wildcard pattern (_)
    Wildcard(Span),

    /// Variable binding
    Binding(String, HirTy, Span),

    /// Literal pattern
    Literal(HirLiteral, Span),

    /// Tuple pattern
    Tuple(Vec<HirPattern>, Span),

    /// Struct pattern
    Struct {
        name: String,
        fields: Vec<String>,
        ty: HirTy,
        span: Span,
    },

    /// Enum variant pattern
    EnumVariant {
        enum_name: String,
        variant_name: String,
        inner: Option<Box<HirPattern>>,
        ty: HirTy,
        span: Span,
    },
}

/// Match arm
#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub pattern: HirPattern,
    pub guard: Option<HirExpression>,
    pub body: HirExpression,
    pub span: Span,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct HirStruct {
    pub id: NodeId,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub fields: Vec<HirField>,
    pub span: Span,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct HirField {
    pub name: String,
    pub ty: HirTy,
    pub span: Span,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct HirEnum {
    pub id: NodeId,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub variants: Vec<HirVariant>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct HirVariant {
    pub name: String,
    pub fields: Vec<HirField>,
    pub span: Span,
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct HirTrait {
    pub id: NodeId,
    pub name: String,
    pub generics: Vec<HirGenericParam>,
    pub items: Vec<HirTraitItem>,
    pub span: Span,
}

/// Trait item (method or associated type)
#[derive(Debug, Clone)]
pub enum HirTraitItem {
    Method {
        name: String,
        sig: HirFunctionSig,
        span: Span,
    },
    AssocType {
        name: String,
        bounds: Vec<String>,
        span: Span,
    },
}

/// Function signature (separate from body for traits)
#[derive(Debug, Clone)]
pub struct HirFunctionSig {
    pub generics: Vec<HirGenericParam>,
    pub params: Vec<HirParam>,
    pub return_type: HirTy,
}

/// Impl block
#[derive(Debug, Clone)]
pub struct HirImpl {
    pub id: NodeId,
    pub generics: Vec<HirGenericParam>,
    pub target_trait: Option<String>,
    pub target_type: HirTy,
    pub items: Vec<HirItem>,
    pub span: Span,
}

/// Module
#[derive(Debug, Clone)]
pub struct HirMod {
    pub id: NodeId,
    pub name: String,
    pub items: Vec<HirItem>,
    pub span: Span,
}

/// Generic parameter
#[derive(Debug, Clone)]
pub struct HirGenericParam {
    pub name: String,
    pub bounds: Vec<String>,
}

// Add span() method to HirItem
impl HirItem {
    pub fn span(&self) -> &Span {
        match self {
            HirItem::Function(f) => &f.span,
            HirItem::Struct(s) => &s.span,
            HirItem::Enum(e) => &e.span,
            HirItem::Trait(t) => &t.span,
            HirItem::Impl(i) => &i.span,
            HirItem::Mod(m) => &m.span,
        }
    }
}
