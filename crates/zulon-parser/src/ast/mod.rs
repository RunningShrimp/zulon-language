// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Abstract Syntax Tree (AST) definitions for ZULON
//!
//! The AST represents the syntactic structure of ZULON code after parsing.

/// A span in source code
pub type Span = crate::lexer::Span;

/// The root of a ZULON program (compilation unit)
#[derive(Debug, Clone)]
pub struct Ast {
    /// Items in the compilation unit
    pub items: Vec<Item>,
    /// Source file path (if available)
    pub source_file: Option<String>,
}

impl Ast {
    /// Create a new AST
    pub fn new(items: Vec<Item>) -> Self {
        Ast {
            items,
            source_file: None,
        }
    }

    /// Create a new AST with source file
    pub fn with_source(items: Vec<Item>, source_file: String) -> Self {
        Ast {
            items,
            source_file: Some(source_file),
        }
    }
}

/// Top-level items in a module
#[derive(Debug, Clone)]
pub struct Item {
    pub span: Span,
    pub kind: ItemKind,
}

/// Kinds of top-level items
#[derive(Debug, Clone)]
pub enum ItemKind {
    /// Function definition: `fn name(params) -> ReturnType { body }`
    Function(Function),
    /// External function declaration: `extern fn name(params) -> ReturnType;`
    ExternFunction(Function),
    /// Struct definition: `struct Name { fields }`
    Struct(Struct),
    /// Enum definition: `enum Name { variants }`
    Enum(Enum),
    /// Trait definition: `trait Name { methods }`
    Trait(Trait),
    /// Trait implementation: `impl Trait for Type { methods }`
    Impl(Impl),
    /// Type alias: `type Name = Type;`
    TypeAlias(TypeAlias),
    /// Constant: `const NAME: Type = value;`
    Const(Const),
    /// Static: `static NAME: Type = value;`
    Static(Static),
    /// Module declaration: `mod name;` or `mod name { ... }`
    Module(Module),
    /// Use statement: `use path::to::item;`
    Use(Use),
    /// External crate import: `extern crate crate_name;`
    ExternCrate(ExternCrate),
    /// Effect declaration: `effect Name { operations }`
    Effect(Effect),
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub error_type: Option<Type>,  // Error type for | separator
    pub effects: Vec<Type>,         // Effect list for | effects
    pub is_variadic: bool,          // Variadic function (uses ...)
    pub body: Block,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub attributes: Vec<Attribute>,  // Function attributes (e.g., #[test])
}

/// Attribute: #[attribute] or #[attribute(arg)] or #[attribute(key = value)]
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Identifier,
    pub args: Vec<AttributeArg>,
}

/// Attribute argument
#[derive(Debug, Clone)]
pub enum AttributeArg {
    /// Identifier: #[attribute(name)]
    Ident(Identifier),
    /// Key-value pair: #[attribute(key = "value")]
    KeyValue { key: Identifier, value: String },
    /// String literal: #[attribute("value")]
    String(String),
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub span: Span,
    pub name: Identifier,
    pub type_annotation: Option<Type>,
    pub default_value: Option<Box<Expression>>, // Box to break recursion
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct Struct {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub fields: Vec<StructField>,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct StructField {
    pub span: Span,
    pub name: Identifier,
    pub type_annotation: Type,
    pub default_value: Option<Box<Expression>>, // Box to break recursion
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub variants: Vec<EnumVariant>,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub span: Span,
    pub name: Identifier,
    pub fields: Vec<VariantField>,
}

/// Variant field (either named or tuple-style)
#[derive(Debug, Clone)]
pub enum VariantField {
    /// Named field: `field: Type`
    Named(Identifier, Type),
    /// Unnamed field: `Type`
    Unnamed(Type),
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct Trait {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub super_traits: Vec<Type>,
    pub items: Vec<TraitItem>,
}

/// Trait item (method or associated type)
#[derive(Debug, Clone)]
pub enum TraitItem {
    /// Method signature (optional body provided in impl)
    Method(Box<Function>),
    /// Associated type: `type Name;`
    AssociatedType(Identifier, Vec<Type>),
    /// Constant: `const NAME: Type = value;`
    Const(Box<Const>),
}

/// Trait implementation
#[derive(Debug, Clone)]
pub struct Impl {
    pub impl_span: Span,
    pub generics: Option<Generics>,
    pub trait_name: Option<Type>,
    pub self_type: Type,
    pub items: Vec<Box<Function>>,
}

/// Type alias
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub type_annotation: Type,
}

/// Constant definition
#[derive(Debug, Clone)]
pub struct Const {
    pub name: Identifier,
    pub type_annotation: Type,
    pub value: Expression,
    pub is_mutable: bool,
}

/// Static variable
#[derive(Debug, Clone)]
pub struct Static {
    pub name: Identifier,
    pub type_annotation: Type,
    pub value: Expression,
    pub is_mutable: bool,
}

/// Module
#[derive(Debug, Clone)]
pub struct Module {
    pub name: Identifier,
    pub items: Option<Vec<Box<Item>>>, // None for `mod name;`
}

/// Use statement
#[derive(Debug, Clone)]
pub struct Use {
    pub path: UsePath,
    pub is_pub: bool,
    pub alias: Option<Identifier>,
}

/// Path in use statement
#[derive(Debug, Clone)]
pub enum UsePath {
    /// Simple path: `use path::to::item;`
    Simple(Vec<Identifier>),
    /// Glob: `use path::to::*;`
    Glob(Vec<Identifier>),
    /// List: `use path::{a, b, c};`
    List(Vec<Identifier>, Vec<Identifier>),
}

/// External crate
#[derive(Debug, Clone)]
pub struct ExternCrate {
    pub name: Identifier,
    pub rename: Option<Identifier>,
}

/// Effect declaration: `effect Name { operations }`
#[derive(Debug, Clone)]
pub struct Effect {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub operations: Vec<EffectOperation>,
}

/// Effect operation
#[derive(Debug, Clone)]
pub struct EffectOperation {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

/// Generic parameters
#[derive(Debug, Clone)]
pub struct Generics {
    pub span: Span,
    pub params: Vec<GenericParam>,
    pub where_clause: Vec<WhereClause>,
}

/// Generic parameter
#[derive(Debug, Clone)]
pub enum GenericParam {
    /// Type parameter: `T`
    Type(Identifier),
    /// Const parameter: `const N: usize`
    Const(Identifier, Type),
    /// Lifetime parameter: `'a`
    Lifetime(Identifier),
}

/// Where clause
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub span: Span,
    pub type_param: Identifier,
    pub bounds: Vec<TraitBound>,
}

/// Trait bound
#[derive(Debug, Clone)]
pub enum TraitBound {
    /// Simple trait bound: `T: Display`
    Trait(Type),
    /// Lifetime bound: `T: 'a`
    Lifetime(Identifier),
}

/// Statements (declarations and expressions with semicolon)
#[derive(Debug, Clone)]
pub struct Statement {
    pub span: Span,
    pub kind: StatementKind,
}

/// Statement kinds
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// Local variable: `let x: Type = value;`
    Local(Local),
    /// Item declaration (function, struct, etc.)
    Item(Box<Item>),
    /// Expression statement: `expr;`
    Expr(Expression),
    /// Semi-colicon (empty statement)
    Empty,
}

/// Local variable declaration
#[derive(Debug, Clone)]
pub struct Local {
    pub name: Identifier,
    pub type_annotation: Option<Type>,
    pub init: Option<Box<Expression>>, // 使用 Box 避免递归
    pub is_mutable: bool,
}

/// Expressions
#[derive(Debug, Clone)]
pub struct Expression {
    pub span: Span,
    pub kind: ExpressionKind,
}

/// Expression kinds
#[derive(Debug, Clone)]
pub enum ExpressionKind {
    /// Literal value
    Literal(Literal),

    /// Path (variable, function, etc.)
    Path(Vec<Identifier>),

    /// Block expression: `{ statements }`
    Block(Block),

    /// Binary operation: `a + b`, `a == b`, etc.
    Binary(BinaryOp, Box<Expression>, Box<Expression>),

    /// Unary operation: `-x`, `!x`, `*x`, `&x`
    Unary(UnaryOp, Box<Expression>),

    /// Function call: `func(args)`
    Call(Box<Expression>, Vec<Box<Expression>>),

    /// Method call: `obj.method(args)`
    MethodCall(Box<Expression>, Identifier, Vec<Box<Expression>>),

    /// Field access: `obj.field`
    FieldAccess(Box<Expression>, Identifier),

    /// Tuple index access: `tuple.0`, `tuple.1`
    TupleIndex(Box<Expression>, usize),

    /// Array indexing: `arr[index]`
    Index(Box<Expression>, Box<Expression>),

    /// Array literal: `[a, b, c]`
    Array(Vec<Box<Expression>>),

    /// Tuple literal: `(a, b, c)`
    Tuple(Vec<Box<Expression>>),

    /// Struct literal: `Point { x: 1.0, y: 2.0 }`
    Struct(StructLiteral),

    /// If expression: `if cond { then } else { else }`
    If(Box<Expression>, Block, Option<Block>),

    /// Match expression: `match value { patterns }`
    Match(Box<Expression>, Vec<MatchArm>),

    /// Loop expression: `loop { body }`
    Loop(Block, Option<Identifier>),

    /// While loop: `while cond { body }`
    While(Box<Expression>, Block, Option<Identifier>),

    /// For loop: `for item in iter { body }`
    For(Local, Box<Expression>, Block, Option<Identifier>),

    /// Break statement: `break;` or `break label;`
    Break(Option<Identifier>),

    /// Continue statement: `continue;` or `continue label;`
    Continue(Option<Identifier>),

    /// Return statement: `return;` or `return value;`
    Return(Option<Box<Expression>>),

    /// Closure (lambda function): `|params| body` or `|params: Type| -> Type { body }`
    Closure {
        /// Closure parameters
        params: Vec<Local>,
        /// Return type annotation (optional)
        return_type: Option<Type>,
        /// Closure body (can be expression or block)
        body: Box<Expression>,
    },

    /// Defer statement: `defer statement;`
    Defer(Box<Statement>),

    /// Throw statement: `throw error;`
    Throw(Box<Expression>),

    /// Question mark operator for error propagation: `expr?`
    QuestionMark(Box<Expression>),

    /// Try block: `try { block } with handler { }`
    Try(Box<Block>, Vec<EffectHandler>),

    /// Perform effect: `perform Effect::method()`
    Perform(Identifier, Vec<Box<Expression>>),

    /// Cast expression: `value as Type`
    Cast(Box<Expression>, Type),

    /// Grouped expression: `(expr)`
    Grouped(Box<Expression>),

    /// Assignment: `target = value`
    Assign(Box<Expression>, Box<Expression>),

    /// Compound assignment: `target += value`
    AssignOp(BinaryOp, Box<Expression>, Box<Expression>),

    /// Range expression: `start..end` or `start..=end`
    Range(Box<Expression>, RangeKind, Box<Expression>),

    /// Template string with interpolation
    TemplateString(TemplateString),

    /// Macro invocation: `macro_name!(args)` or `macro_name! { args }` or `macro_name![ args ]`
    MacroInvocation {
        /// Macro name
        macro_name: Identifier,
        /// Macro arguments
        args: Vec<Box<Expression>>,
        /// Delimiter used: '(', '{', or '['
        delimiter: MacroDelimiter,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Bitwise
    BitAnd, BitOr, BitXor,
    LeftShift, RightShift,
    // Comparison
    Eq, NotEq, Less, LessEq, Greater, GreaterEq,
    // Logical
    And, Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    // Arithmetic
    Neg, // -
    // Logical
    Not, // !
    // Bitwise
    BitNot, // ^
    // Reference/Dereference
    Ref,    // &
    Deref,  // *
    // Borrow
    Borrow,     // &
    BorrowMut,  // &mut
}

/// Range kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeKind {
    Exclusive, // ..
    Inclusive, // ..=
}

/// Macro delimiter kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroDelimiter {
    Paren,   // ( )
    Brace,   // { }
    Bracket, // [ ]
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    /// Integer: `42`
    Int(i64),
    /// Float: `3.14`
    Float(f64),
    /// String: `"hello"`
    String(String),
    /// Character: `'a'`
    Char(char),
    /// Boolean: `true`, `false`
    Bool(bool),
    /// Null: `null`
    Null,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub span: Span,
    pub statements: Vec<Statement>,
    pub trailing_expr: Option<Box<Expression>>,
}

/// Struct literal
#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub path: Vec<Identifier>,
    pub fields: Vec<StructExprField>,
    pub base: Option<Box<Expression>>,
}

/// Struct expression field
#[derive(Debug, Clone)]
pub struct StructExprField {
    pub span: Span,
    pub name: Identifier,
    pub value: Expression,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub guard: Option<Box<Expression>>, // Box to break recursion
    pub body: Box<Expression>, // Box to break recursion
}

/// Patterns for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard: `_`
    Wildcard,
    /// Literal pattern: `42`, `"hello"`
    Literal(Literal),
    /// Identifier pattern: `x`
    Identifier(Identifier),
    /// Struct pattern: `Point { x, y }`
    Struct(Vec<Identifier>, Vec<StructPatternField>),
    /// Tuple pattern: `(a, b, c)`
    Tuple(Vec<Pattern>),
    /// Array pattern: `[a, b, c]`
    Array(Vec<Pattern>),
    /// Slice pattern: `[first, .., last]`
    Slice(Vec<Pattern>, Vec<Pattern>, Vec<Pattern>),
    /// Range pattern: `start..end`
    Range(Box<Pattern>, RangeKind, Box<Pattern>),
    /// Or pattern: `A | B`
    Or(Vec<Pattern>),
}

/// Struct pattern field
#[derive(Debug, Clone)]
pub enum StructPatternField {
    /// Field with pattern: `field: pattern`
    Field(Identifier, Box<Pattern>),
    /// Field shorthand: `field`
    Shorthand(Identifier),
}

/// Effect handler for try blocks
#[derive(Debug, Clone)]
pub struct EffectHandler {
    pub effect_name: Identifier,
    pub methods: Vec<EffectMethod>,
}

/// Effect method handler
#[derive(Debug, Clone)]
pub struct EffectMethod {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub body: Block,
}

/// Closure expression
#[derive(Debug, Clone)]
pub struct Closure {
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Box<Expression>, // 使用 Box 打破循环
    pub is_move: bool,
}

/// Template string with interpolation
#[derive(Debug, Clone)]
pub struct TemplateString {
    pub parts: Vec<TemplateStringPart>,
}

/// Template string part
#[derive(Debug, Clone)]
pub enum TemplateStringPart {
    /// Static string
    Static(String),
    /// Interpolated expression
    Expr(Expression),
}

/// Types
#[derive(Debug, Clone)]
pub enum Type {
    /// Simple type: `i32`, `str`, etc.
    Simple(Identifier),
    /// Tuple type: `(i32, str, bool)`
    Tuple(Vec<Type>),
    /// Array type: `[T; N]`
    Array(Box<Type>, Option<Box<Expression>>), // Box to break recursion
    /// Slice type: `[T]`
    Slice(Box<Type>),
    /// Reference type: `&T` or `&mut T`
    Ref(Box<Type>, bool),
    /// Pointer type: `*const T` or `*mut T`
    Pointer(Box<Type>, bool),
    /// Function type: `fn(params) -> ReturnType`
    Function(Vec<Type>, Box<Type>),
    /// Trait object type: `dyn Trait`
    TraitObject(Box<Type>),
    /// Impl Trait type: `impl Trait`
    ImplTrait(Box<Type>),
    /// Never type: `!`
    Never,
    /// Tuple unit: `()`
    Unit,
    /// Optional type: `T?`
    Optional(Box<Type>),
    /// Path type: `std::collections::HashMap`
    Path(Vec<Identifier>),
}

/// Identifier
#[derive(Debug, Clone)]
pub struct Identifier {
    pub span: Span,
    pub name: String,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Identifier {}

impl Identifier {
    pub fn new(span: Span, name: impl Into<String>) -> Self {
        Identifier {
            span,
            name: name.into(),
        }
    }
}

/// Visibility modifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// Public: `pub`
    Public,
    /// Private (default)
    Private,
    /// Crate-visible: `pub(crate)`
    Crate,
    /// Restricted to path: `pub(self)`, `pub(super)`, `pub(in path)`
    Restricted(Vec<Identifier>),
}
