# ZULON 技术详细设计文档

**版本**: 1.0
**日期**: 2026-01-07
**作者**: ZULON Language Team
**状态**: 设计阶段

---

## 目录

1. [概述](#概述)
2. [编译器架构](#编译器架构)
3. [词法与语法分析](#词法与语法分析)
4. [类型系统设计](#类型系统设计)
5. [内存管理模型](#内存管理模型)
6. [错误处理机制](#错误处理机制)
7. [并发模型设计](#并发模型设计)
8. [代码生成策略](#代码生成策略)
9. [标准库设计](#标准库设计)
10. [工具链设计](#工具链设计)

---

## 概述

### 文档目标

本文档详细描述 ZULON 编程语言的技术实现细节，包括：

- 编译器的各个阶段和实现策略
- 类型系统的形式化定义
- 内存管理的具体实现
- 错误处理和代数效应机制
- 并发编程的底层支持
- 代码生成和优化策略

### 设计原则

1. **正确性优先**: 编译器必须保证生成正确的代码
2. **性能导向**: 生成的代码应接近 C++ 性能
3. **可维护性**: 编译器代码应清晰、模块化
4. **可扩展性**: 设计应支持未来语言特性的添加
5. **用户友好**: 提供清晰的错误消息和调试信息

---

## 编译器架构

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        ZULON 编译器                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  源代码 (.zl)                                                 │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ 词法分析器   │ → Token 流                                 │
│  └─────────────┘                                             │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ 语法分析器   │ → AST (抽象语法树)                          │
│  └─────────────┘                                             │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ 语义分析器   │ → HIR (高级中间表示)                        │
│  └─────────────┘                                             │
│     │                                                         │
│     ├──────────────────┐                                     │
│     ▼                  ▼                                     │
│  ┌─────────────┐  ┌─────────────┐                           │
│  │ 类型检查器   │  │ 借用检查器   │                           │
│  └─────────────┘  └─────────────┘                           │
│     │                  │                                     │
│     └──────────────────┘                                     │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ MIR 降低     │ → MIR (中级中间表示)                        │
│  └─────────────┘                                             │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ 优化器       │ → 优化后的 MIR                              │
│  └─────────────┘                                             │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ 代码生成器   │ → LLVM IR / 机器代码                        │
│  └─────────────┘                                             │
│     │                                                         │
│     ▼                                                         │
│  ┌─────────────┐                                             │
│  │ 链接器       │ → 可执行文件                                │
│  └─────────────┘                                             │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 编译阶段详解

#### 1. 词法分析 (Lexer)

**职责**: 将源代码文本转换为 Token 流

**Token 类型**:

```rust
// Token 定义
pub enum Token {
    // 关键字
    Fn, Let, Mut, Const, If, Else, Match, Return,
    Struct, Enum, Trait, Impl, Type, Where,
    Error, Effect, Throw, Perform, Try, Defer,
    Namespace, Use, Pub, Mod,
    True, False, Null,

    // 标识符和字面量
    Identifier(String),
    IntegerLiteral(String),
    FloatLiteral(String),
    StringLiteral(String),
    CharLiteral(char),
    TemplateString(String),

    // 运算符
    Plus, Minus, Star, Slash, Percent,
    And, Or, Not, BitAnd, BitOr, BitXor,
    ShiftLeft, ShiftRight,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    Assign, PlusAssign, MinusAssign, etc.,

    // 分隔符
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Comma, Colon, Semicolon, Dot, DotDot,

    // 特殊
    Question, Pipe, Arrow, FatArrow,
    At, Dollar, Hash,

    // 结束
    EOF,
}
```

**词法规则**:

```ebnf
<identifier> ::= <letter> | "_" { <letter> | <digit> | "_" }
<integer> ::= <digit> { <digit> }
<float> ::= <digit> "." <digit> [ ("e" | "E") ["+" | "-"] <digit> ]
<string> ::= '"' { <character> } '"'
<template_string> ::= '`' { <character> | "${" <expression> "}" } '`'
<comment> ::= "//" { <character> } <newline>
            | "/*" { <character> } "*/"
```

**实现要点**:

- 使用状态机实现高性能词法分析
- 支持 Unicode 标识符
- 保留位置信息用于错误报告
- 支持源码映射 (source mapping)

#### 2. 语法分析 (Parser)

**职责**: 将 Token 流转换为 AST

**AST 节点类型**:

```rust
pub enum ASTNode {
    // 程序
    Program(Vec<ASTNode>),

    // 声明
    FunctionDecl {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        effects: Vec<Effect>,
        body: Box<ASTNode>,
    },
    StructDecl {
        name: String,
        fields: Vec<Field>,
    },
    EnumDecl {
        name: String,
        variants: Vec<Variant>,
    },
    TraitDecl {
        name: String,
        methods: Vec<FunctionDecl>,
    },
    ImplDecl {
        trait_name: Option<String>,
        type_name: String,
        methods: Vec<FunctionDecl>,
    },
    ErrorDecl {
        name: String,
        variants: Vec<Variant>,
    },
    EffectDecl {
        name: String,
        operations: Vec<Signature>,
    },

    // 语句
    LetStatement {
        name: String,
        type_annotation: Option<Type>,
        value: Box<ASTNode>,
    },
    ExprStatement(Box<ASTNode>),
    ReturnStatement(Option<Box<ASTNode>>),
    ThrowStatement(Box<ASTNode>),
    PerformStatement(Box<ASTNode>),
    DeferStatement(Box<ASTNode>),

    // 表达式
    BinaryExpr {
        left: Box<ASTNode>,
        op: BinaryOp,
        right: Box<ASTNode>,
    },
    UnaryExpr {
        op: UnaryOp,
        operand: Box<ASTNode>,
    },
    CallExpr {
        callee: Box<ASTNode>,
        args: Vec<ASTNode>,
    },
    MemberExpr {
        object: Box<ASTNode>,
        field: String,
    },
    IndexExpr {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
    },
    IfExpr {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    MatchExpr {
        scrutinee: Box<ASTNode>,
        arms: Vec<MatchArm>,
    },
    BlockExpr(Vec<ASTNode>),

    // 字面量
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BooleanLiteral(bool),
    NullLiteral,

    // 其他
    Identifier(String),
    ArrayLiteral(Vec<ASTNode>),
    TupleLiteral(Vec<ASTNode>),
    StructLiteral {
        name: String,
        fields: Vec<(String, ASTNode)>,
    },
}
```

**语法规则 (简化)**:

```ebnf
<program> ::= { <declaration> }

<declaration> ::= <fn_decl> | <struct_decl> | <enum_decl> |
                <trait_decl> | <impl_decl> | <error_decl> | <effect_decl>

<fn_decl> ::= "fn" <identifier> <params> [ "->" <type> [ "|" <effects> ] ]
              <block>

<params> ::= "(" [ <param> { "," <param> } ] ")"
<param> ::= <identifier> ":" <type>

<effects> ::= <effect> { "|" <effect> }

<block> ::= "{" { <statement> } "}"

<statement> ::= <let_stmt> | <expr_stmt> | <return_stmt> |
               <throw_stmt> | <perform_stmt> | <defer_stmt>

<let_stmt> ::= "let" [ "mut" ] <identifier> [ ":" <type> ] "=" <expr>

<expr> ::= <logic_or> | <if_expr> | <match_expr> | <block_expr>

<logic_or> ::= <logic_and> { "||" <logic_and> }
<logic_and> ::= <equality> { "&&" <equality> }
<equality> ::= <comparison> { ("==" | "!=") <comparison> }
<comparison> ::= <shift> { ("<" | ">" | "<=" | ">=") <shift> }
<shift> ::= <additive> { ("<<" | ">>") <additive> }
<additive> ::= <multiplicative> { ("+" | "-") <multiplicative> }
<multiplicative> ::= <unary> { ("*" | "/" | "%") <unary> }
<unary> ::= { ("!" | "-" | "*" | "&") } <primary>
<primary> ::= <literal> | <identifier> | <call> | <member> | <index> |
             "(" <expr> ")" | <array_literal> | <tuple_literal> |
             <struct_literal>

<match_expr> ::= "match" <expr> "{" { <match_arm> } "}"
<match_arm> ::= <pattern> "=>" <expr> [ "," ]

<pattern> ::= <literal> | <identifier> | <destruct_pattern>
```

**解析策略**:

- 使用递归下降解析 (Recursive Descent)
- 实现 Pratt 解析器处理表达式优先级
- 支持运算符重载
- 错误恢复: 同步到下一个语句/声明

#### 3. 语义分析 (Semantic Analysis)

**职责**: 类型检查、借用检查、名称解析

**HIR (高级中间表示)**:

```rust
pub enum HIR {
    Function {
        name: String,
        sig: Signature,
        body: Vec<Statement>,
    },

    Statement(Stmt),
    Expression(Expr),
}

pub enum Stmt {
    Let {
        name: String,
        type_: Type,
        value: Expr,
        mutable: bool,
    },
    Return(Expr),
    Throw(Expr),
    Perform(Expr, Effect),
    Defer(Expr),
    Expr(Expr),
}

pub enum Expr {
    Literal(Literal),
    Variable(Variable),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Lambda(Vec<Param>, Box<Expr>),
    Block(Vec<Stmt>),
    If {
        condition: Box<Expr>,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
}
```

**类型定义**:

```rust
pub enum Type {
    // 基本类型
    Void,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    String,
    Char,

    // 复合类型
    Array(Box<Type>, usize),
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Struct(String, Vec<(String, Type)>),
    Enum(String, Vec<(String, Vec<Type>)>),

    // 函数类型
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        effects: Vec<Effect>,
    },

    // 泛型
    Generic(String, Vec<TyParam>),

    // 特殊类型
    Reference(Box<Type>, Mutability),
    Optional(Box<Type>),
    Result(Box<Type>, Box<Type>),

    // Trait 对象
    TraitObject(Vec<String>),
}
```

**类型检查算法**:

```rust
// 类型检查核心接口
pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    pub type_env: TypeEnv,
    pub constraints: Vec<TypeConstraint>,
}

impl TypeChecker {
    pub fn infer_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(lit) => self.infer_literal(lit),
            Expr::Variable(var) => self.infer_variable(var),
            Expr::BinaryOp { op, left, right } => {
                self.infer_binary_op(op, left, right)
            },
            // ...
        }
    }

    pub fn unify(&mut self, ty1: &Type, ty2: &Type) -> Result<(), TypeError> {
        // 类型统一算法
    }

    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<Type, TypeError> {
        match stmt {
            Stmt::Let { name, type_, value, .. } => {
                let value_type = self.infer_expr(value)?;
                if let Some(annotated) = type_ {
                    self.unify(&value_type, annotated)?;
                }
                self.symbol_table.insert(name, value_type);
                Ok(Type::Void)
            },
            // ...
        }
    }
}
```

**借用检查器**:

```rust
pub struct BorrowChecker {
    pub scopes: Vec<Scope>,
    pub borrows: HashMap<String, Vec<Borrow>>,
}

pub enum Borrow {
    Immutable {
        lifetime: Lifetime,
        location: Location,
    },
    Mutable {
        lifetime: Lifetime,
        location: Location,
    },
}

pub struct Lifetime {
    pub start: usize,
    pub end: usize,
}

impl BorrowChecker {
    pub fn check_function(&mut self, func: &HIR::Function) -> Result<(), BorrowError> {
        // 检查借用规则
        // 1. 任何值的借用在生命周期内必须有效
        // 2. 可变借用是排他的
        // 3. 借用不能超过所有者的生命周期

        for stmt in &func.body {
            self.check_statement(stmt)?;
        }

        Ok(())
    }

    pub fn check_statement(&mut self, stmt: &Stmt) -> Result<(), BorrowError> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.check_expr(value)?;
                self.scopes.last_mut().unwrap().vars.insert(name.clone());
            },
            Stmt::Return(expr) => {
                self.check_return_expr(expr)?;
            },
            // ...
        }

        Ok(())
    }
}
```

#### 4. MIR 降低 (MIR Lowering)

**职责**: 将 HIR 降低为 MIR (中级中间表示)

**MIR 定义**:

```rust
pub enum MIR {
    Function {
        name: String,
        blocks: Vec<BasicBlock>,
        arguments: Vec<Local>,
        returns: Vec<Local>,
        locals: Vec<Local>,
    },
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub enum Statement {
    Assign(Place, Rvalue),
    SetDiscriminant(Place, VariantIdx),
    Drop(Place),
    InlineAsm(Template, Vec<Place>, Vec<Place>),
}

pub enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Aggregate(AggregateKind, Vec<Operand>),
    Ref(Region, BorrowKind, Place),
    ThreadLocalRef(ThreadLocalRef),
}

pub enum Terminator {
    Return,
    Goto(BasicBlock),
    SwitchInt { discr: Operand, values: Vec<Const>, targets: Vec<BasicBlock> },
    Call { func: Operand, args: Vec<Operand>, destination: Place, target: BasicBlock },
    Drop { place: Place, target: BasicBlock },
    Resume,
    Abort,
    Impossible,
}
```

**降低策略**:

1. **表达式分解**: 将复杂表达式分解为多个简单语句
2. **控制流标准化**: 转换为基本块和终结符
3. **模式匹配编译**: 转换为决策树或跳转表
4. **闭包转换**: 将闭包转换为结构体和函数指针

#### 5. 优化器 (Optimizer)

**优化Pass**:

```rust
pub trait OptimizerPass {
    fn name(&self) -> &str;
    fn run(&mut self, mir: &mut MIR) -> bool; // 返回是否做了修改
}

// 内联优化
pub struct Inliner;

impl OptimizerPass for Inliner {
    fn name(&self) -> &str { "inline" }

    fn run(&mut self, mir: &mut MIR) -> bool {
        let mut changed = false;

        // 内联小函数
        for block in &mut mir.blocks {
            for stmt in &mut block.statements {
                if let Statement::Assign(place, Rvalue::Call { func, args, .. }) = stmt {
                    if self.should_inline(func) {
                        self.inline_call(block, func, args, place);
                        changed = true;
                    }
                }
            }
        }

        changed
    }
}

// 常量折叠
pub struct ConstantPropagation;

impl OptimizerPass for ConstantPropagation {
    fn name(&self) -> &str { "const_prop" }

    fn run(&mut self, mir: &mut MIR) -> bool {
        // 传播常量并折叠常量表达式
    }
}

// 死代码消除
pub struct DeadCodeElimination;

impl OptimizerPass for DeadCodeElimination {
    fn name(&self) -> &str { "dce" }

    fn run(&mut self, mir: &mut MIR) -> bool {
        // 删除未使用的代码
    }
}

// 循环优化
pub struct LoopOptimizations;

impl OptimizerPass for LoopOptimizations {
    fn name(&self) -> &str { "loop_opt" }

    fn run(&mut self, mir: &mut MIR) -> bool {
        // 循环展开、向量化等
    }
}
```

**优化Pipeline**:

```rust
pub fn optimize_mir(mir: &mut MIR, level: OptimizationLevel) {
    let mut passes: Vec<Box<dyn OptimizerPass>> = match level {
        OptimizationLevel::None => vec![],
        OptimizationLevel::Basic => vec![
            Box::new(ConstantPropagation::new()),
            Box::new(DeadCodeElimination::new()),
        ],
        OptimizationLevel::Aggressive => vec![
            Box::new(Inliner::new()),
            Box::new(ConstantPropagation::new()),
            Box::new(DeadCodeElimination::new()),
            Box::new(LoopOptimizations::new()),
            Box::new(Vectorization::new()),
        ],
    };

    for pass in passes.iter_mut() {
        let mut changed = true;
        while changed {
            changed = pass.run(mir);
        }
    }
}
```

#### 6. 代码生成 (Code Generation)

**LLVM IR 生成**:

```rust
pub struct CodeGenerator {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,
}

impl CodeGenerator {
    pub fn compile_function(&mut self, mir: &MIR) -> LLVMValueRef {
        let fn_type = self.fn_type_to_llvm(&mir.sig);
        let function = unsafe {
            LLVMAddFunction(self.module, mir.name.c_str(), fn_type)
        };

        for (bb_idx, mir_bb) in mir.blocks.iter().enumerate() {
            let llvm_bb = unsafe {
                LLVMAppendBasicBlock(function, b"".as_ptr())
            };
            LLVMPositionBuilderAtEnd(self.builder, llvm_bb);

            for stmt in &mir_bb.statements {
                self.compile_statement(stmt);
            }

            self.compile_terminator(&mir_bb.terminator);
        }

        function
    }

    fn compile_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assign(place, rvalue) => {
                let value = self.compile_rvalue(rvalue);
                let ptr = self.compile_place(place);
                unsafe {
            LLVMBuildStore(self.builder, value, ptr);
        }
    },
    Statement::Drop(place) => {
        self.compile_drop(place);
    },
    // ...
}
}
```

---

## 词法与语法分析

### 词法分析实现

#### 状态机设计

```rust
pub struct Lexer {
    pub input: Vec<char>,
    pub position: usize,
    pub char: char,
    pub location: Location,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            char: '\0',
            location: Location { line: 1, column: 1 },
        };
        lexer.advance_char();
        lexer
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        match self.char {
            '\0' => Ok(Token::EOF),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
            '0'..='9' => self.read_number(),
            '"' => self.read_string(),
            '`' => self.read_template_string(),
            '\'' => self.read_char(),
            '/' => self.read_comment_or_slash(),
            _ => self.read_operator_or_punct(),
        }
    }

    fn advance_char(&mut self) {
        self.char = if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            c
        } else {
            '\0'
        };

        if self.char == '\n' {
            self.location.line += 1;
            self.location.column = 1;
        } else {
            self.location.column += 1;
        }
    }
}
```

#### Token 规范

**运算符优先级** (从高到低):

1. 路径、调用、索引、字段访问
2. 一元运算符 (`!`, `-`, `*`, `&`)
3. 乘除模 (`*`, `/`, `%`)
4. 加减 (`+`, `-`)
5. 位移 (`<<`, `>>`)
6. 比较 (`<`, `>`, `<=`, `>=`)
7. 相等 (`==`, `!=`)
8. 位与 (`&`)
9. 位异或 (`^`)
10. 位或 (`|`)
11. 逻辑与 (`&&`)
12. 逻辑或 (`||`)
13. 赋值 (`=`, `+=`, 等)

### 语法分析实现

#### Pratt 解析器

```rust
pub struct Parser {
    pub lexer: Lexer,
    pub current_token: Token,
    pub current_location: Location,
}

impl Parser {
    // 表达式解析（Pratt算法）
    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<ASTNode, ParseError> {
        let mut left = self.parse_prefix()?;

        while self.get_precedence(&self.current_token) > precedence {
            let op_token = self.current_token.clone();
            self.advance_token();
            let right = self.parse_expression(self.get_precedence(&op_token))?;
            left = ASTNode::BinaryExpr {
                left: Box::new(left),
                op: self.token_to_binop(&op_token)?,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance_token();
                Ok(ASTNode::Identifier(name))
            },
            Token::IntegerLiteral(s) => {
                let value = s.parse::<i64>()?;
                self.advance_token();
                Ok(ASTNode::IntegerLiteral(value))
            },
            Token::LeftParen => {
                self.advance_token();
                let expr = self.parse_expression(Precedence::Lowest)?;
                self.expect_token(&Token::RightParen)?;
                Ok(expr)
            },
            Token::Not | Token::Minus => {
                let op = self.token_to_unop(&self.current_token)?;
                self.advance_token();
                let operand = self.parse_expression(Precedence::Prefix)?;
                Ok(ASTNode::UnaryExpr {
                    op,
                    operand: Box::new(operand),
                })
            },
            _ => Err(ParseError::UnexpectedToken(self.current_token.clone())),
        }
    }
}
```

#### 模式匹配解析

```rust
pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
    match &self.current_token {
        Token::Identifier(name) => {
            let name = name.clone();
            self.advance_token();

            if self.check_token(&Token::Colon) {
                // 带类型标注的模式
                self.advance_token();
                let type_ = self.parse_type()?;
                Ok(Pattern::Typed(name, type_))
            } else {
                Ok(Pattern::Identifier(name))
            }
        },
        Token::LeftParen => {
            // 元组或结构体模式
            self.advance_token();
            let patterns = self.parse_pattern_list()?;
            self.expect_token(&Token::RightParen)?;
            Ok(Pattern::Tuple(patterns))
        },
        Token::Literal(lit) => {
            let lit = lit.clone();
            self.advance_token();
            Ok(Pattern::Literal(lit))
        },
        _ => Err(ParseError::ExpectedPattern(self.current_token.clone())),
    }
}
```

---

## 类型系统设计

### 类型定义

```rust
pub enum Type {
    // 基本类型
    Void,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    String,
    Char,
    Never,  // ! 类型

    // 复合类型
    Array(Box<Type>, Box<Expr>),  // [T; N]
    Slice(Box<Type>),              // [T]
    Tuple(Vec<Type>),

    // 命名类型
    Struct(StructId),
    Enum(EnumId),

    // 函数类型
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        effects: Vec<Effect>,
    },

    // 泛型
    Generic(String, Vec<TyParam>),

    // 特殊类型
    Reference(Box<Type>, Mutability),
    Optional(Box<Type>),
    Result(Box<Type>, Box<Type>),

    // Trait 约束
    TraitObject(Vec<TraitId>),

    // 类型变量
    TypeVar(usize),
}

pub enum IntegerType {
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    ISize, USize,
}

pub enum FloatType {
    F32, F64,
}

pub enum Mutability {
    Immutable,
    Mutable,
}
```

### 类型推断算法

**双向类型检查**:

```rust
pub struct TypeChecker {
    pub expected_type: Option<Type>,
    pub actual_type: Option<Type>,
}

impl TypeChecker {
    // 合成模式：从表达式推断类型
    pub fn synth(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(Literal::Integer(_)) => Ok(Type::Integer(IntegerType::I32)),
            Expr::Literal(Literal::Float(_)) => Ok(Type::Float(FloatType::F64)),
            Expr::Literal(Literal::String(_)) => Ok(Type::String),
            Expr::Variable(name) => {
                self.lookup_type(name)
            },
            Expr::BinaryOp { op, left, right } => {
                let left_ty = self.synth(left)?;
                let right_ty = self.synth(right)?;
                let result_ty = self.infer_binary_op_type(op, &left_ty, &right_ty)?;
                self.unify(&left_ty, &right_ty)?;
                Ok(result_ty)
            },
            Expr::Lambda(params, body) => {
                // 推断 lambda 类型
            },
            Expr::If { condition, then_branch, else_branch } => {
                let cond_ty = self.synth(condition)?;
                self.unify(&cond_ty, &Type::Bool)?;

                let then_ty = self.synth(then_branch)?;
                let else_ty = else_branch.as_ref()
                    .map(|e| self.synth(e))
                    .transpose()?
                    .unwrap_or(Type::Void);

                self.unify(&then_ty, &else_ty)?;
                Ok(then_ty)
            },
            _ => Err(TypeError::CannotInfer),
        }
    }

    // 检查模式：从类型检查表达式
    pub fn check(&mut self, expr: &Expr, expected: &Type) -> Result<(), TypeError> {
        match expr {
            Expr::Literal(_) => {
                let actual = self.synth(expr)?;
                self.unify(&actual, expected)
            },
            Expr::Variable(name) => {
                let actual = self.lookup_type(name)?;
                self.unify(&actual, expected)
            },
            Expr::Lambda(params, body) => {
                if let Type::Function { params: param_tys, return_type: ret_ty, .. } = expected {
                    // 检查参数类型
                    for (param, param_ty) in params.iter().zip(param_tys.iter()) {
                        self.symbol_table.insert(param.name.clone(), param_ty.clone());
                    }

                    // 检查返回类型
                    self.check(body, ret_ty)
                } else {
                    Err(TypeError::TypeMismatch {
                        expected: expected.clone(),
                        actual: self.synth(expr)?,
                    })
                }
            },
            _ => {
                let actual = self.synth(expr)?;
                self.unify(&actual, expected)
            }
        }
    }

    // 类型统一
    pub fn unify(&mut self, ty1: &Type, ty2: &Type) -> Result<(), TypeError> {
        match (ty1, ty2) {
            (Type::Int(t1), Type::Int(t2)) if t1 == t2 => Ok(()),
            (Type::Float(t1), Type::Float(t2)) if t1 == t2 => Ok(()),
            (Type::Reference(t1, m1), Type::Reference(t2, m2)) => {
                self.unify(t1, t2)?;
                if m1 != m2 {
                    return Err(TypeError::MutabilityMismatch);
                }
                Ok(())
            },
            (Type::Tuple(types1), Type::Tuple(types2)) if types1.len() == types2.len() => {
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    self.unify(t1, t2)?;
                }
                Ok(())
            },
            (Type::Function { params: p1, return_type: r1, .. },
             Type::Function { params: p2, return_type: r2, .. }) => {
                for (t1, t2) in p1.iter().zip(p2.iter()) {
                    self.unify(t1, t2)?;
                }
                self.unify(r1, r2)
            },
            (Type::TypeVar(v1), Type::TypeVar(v2)) if v1 == v2 => Ok(()),
            (Type::TypeVar(v), other) | (other, Type::TypeVar(v)) => {
                // 实例化类型变量
                self.type_constraints.insert(*v, other.clone());
                Ok(())
            },
            _ => Err(TypeError::TypeMismatch {
                expected: ty1.clone(),
                actual: ty2.clone(),
            }),
        }
    }
}
```

### Trait 系统

```rust
pub struct Trait {
    pub name: String,
    pub methods: Vec<Signature>,
    pub associated_types: Vec<String>,
    pub constraints: Vec<TraitConstraint>,
}

pub struct TraitImpl {
    pub trait_name: String,
    pub type_name: String,
    pub method_impls: Vec<Function>,
}

pub struct TraitConstraint {
    pub trait_name: String,
    pub type_params: Vec<Type>,
}

impl TypeChecker {
    pub fn check_trait_impl(&mut self, impl_: &TraitImpl) -> Result<(), TypeError> {
        let trait_ = self.lookup_trait(&impl_.trait_name)?;

        // 检查所有方法都已实现
        for method_sig in &trait_.methods {
            let impl_method = impl_.method_impls
                .iter()
                .find(|m| m.name == method_sig.name)
                .ok_or_else(|| TypeError::MissingMethod {
                    method: method_sig.name.clone(),
                    trait_: impl_.trait_name.clone(),
                })?;

            // 检查方法签名匹配
            self.check_signature_matches(impl_method, method_sig)?;
        }

        Ok(())
    }
}
```

---

## 内存管理模型

### Tree Borrows + ARC 混合模型

#### 核心概念

```rust
pub enum Value {
    // 栈分配的值
    Stack(StackValue),

    // 堆分配的值（带引用计数）
    Heap(Box<HeapValue>),
}

pub struct HeapValue {
    pub data: Vec<u8>,
    pub ref_count: AtomicUsize,
    pub weak_count: AtomicUsize,
}

pub enum Ownership {
    Owned(usize),          // 唯一所有权
    Borrowed(usize, BorrowKind),
}

pub enum BorrowKind {
    Immutable {
        lifetime: Lifetime,
    },
    Mutable {
        lifetime: Lifetime,
    },
}
```

#### 引用计数实现

```rust
pub struct Arc<T> {
    pub ptr: *mut ArcInner<T>,
}

pub struct ArcInner<T> {
    pub strong: AtomicUsize,
    pub weak: AtomicUsize,
    pub data: T,
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.ptr).strong.fetch_add(1, Ordering::Relaxed);
        }
        Arc { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.ptr).strong.fetch_sub(1, Ordering::Release) == 1 {
                self.drop_slow();
            }
        }
    }
}
```

#### 借用检查算法

```rust
pub struct BorrowChecker {
    pub borrows: HashMap<String, Vec<Borrow>>,
    pub lifetimes: Vec<Lifetime>,
}

impl BorrowChecker {
    pub fn check(&mut self, mir: &MIR) -> Result<(), BorrowError> {
        // Tree Borrows 模型检查
        // 1. 读取时允许写入（激活状态）
        // 2. 写入时禁止其他借用
        // 3. 生命周期结束后重新激活

        for bb in &mir.blocks {
            self.check_basic_block(bb)?;
        }

        Ok(())
    }

    pub fn check_statement(&mut self, stmt: &Statement) -> Result<(), BorrowError> {
        match stmt {
            Statement::Assign(place, Rvalue::Ref(region, kind, value_place)) => {
                self.check_borrow(place, kind, value_place)?;
            },
            Statement::Assign(place, Rvalue::Use(Operand::Copy(place2))) => {
                self.check_copy(place, place2)?;
            },
            Statement::Assign(place, Rvalue::Use(Operand::Move(place2))) => {
                self.check_move(place, place2)?;
            },
            Statement::Drop(place) => {
                self.check_drop(place)?;
            },
            _ => Ok(()),
        }
    }
}
```

### 逃逸分析

```rust
pub struct EscapeAnalysis {
    pub escape_info: HashMap<Local, EscapeState>,
}

pub enum EscapeState {
    // 不逃逸：完全在栈上
    None,
    // 逃逸到返回值
    Return,
    // 逃逸到堆或全局
    Global,
}

impl EscapeAnalysis {
    pub fn analyze(&mut self, mir: &MIR) {
        for local in &mir.locals {
            let state = self.analyze_local(mir, local);
            self.escape_info.insert(local.clone(), state);
        }
    }

    pub fn analyze_local(&mut self, mir: &MIR, local: &Local) -> EscapeState {
        // 检查局部变量是否逃逸
        // 1. 是否被存储到堆？
        // 2. 是否被返回？
        // 3. 是否被传递给其他函数？

        for bb in &mir.blocks {
            for stmt in &bb.statements {
                if self.escapes_in_statement(local, stmt) {
                    return EscapeState::Global;
                }
            }

            if let Terminator::Return { values } = &bb.terminator {
                for value in values {
                    if self.refers_to_local(value, local) {
                        return EscapeState::Return;
                    }
                }
            }
        }

        EscapeState::None
    }
}
```

---

## 错误处理机制

### 代数效应实现

#### 效应类型定义

```rust
pub struct Effect {
    pub name: String,
    pub operations: Vec<EffectOperation>,
}

pub struct EffectOperation {
    pub name: String,
    pub input_types: Vec<Type>,
    pub output_type: Type,
}

pub enum EffectHandler {
    UserDefined {
        operations: HashMap<String, Handler>,
    },
    Builtin(BuiltinEffect),
}

pub enum BuiltinEffect {
    IO,
    Database,
    Logging,
}
```

#### 效应检查

```rust
pub struct EffectChecker {
    pub effect_env: EffectEnv,
}

impl EffectChecker {
    pub fn check_function(&mut self, func: &HIR::Function) -> Result<(), EffectError> {
        for effect in &func.sig.effects {
            self.check_effect(effect)?;
        }

        self.check_block(&func.body, &func.sig.effects)?;

        Ok(())
    }

    pub fn check_statement(&mut self, stmt: &Stmt, allowed: &[Effect]) -> Result<(), EffectError> {
        match stmt {
            Stmt::Perform(expr, effect) => {
                if !allowed.contains(effect) {
                    return Err(EffectError::EffectNotAllowed {
                        effect: effect.clone(),
                        location: stmt.location(),
                    });
                }

                // 检查 effect 操作是否在 effect 定义中
                self.check_effect_operation(expr, effect)?;
            },
            _ => Ok(()),
        }
    }
}
```

#### 效应处理编译

```rust
pub struct EffectCompiler;

impl EffectCompiler {
    // 将效应调用编译为状态机
    pub fn compile_effect_handler(&self, mir: &mut MIR, handler: &EffectHandler) {
        // 1. 为每个 effect 操作创建状态
        // 2. 在 effect 调用处插入状态转换
        // 3. 在 effect handler 处恢复状态

        match handler {
            EffectHandler::UserDefined { operations } => {
                self.compile_user_handler(mir, operations);
            },
            EffectHandler::Builtin(builtin) => {
                self.compile_builtin_handler(mir, builtin);
            },
        }
    }
}
```

### 错误类型自动实现

```rust
// 宏自动实现 Error trait
macro_rules! impl_error {
    ($name:ident) => {
        impl Error for $name {
            fn display(&self) -> String {
                match self {
                    // 格式化错误消息
                }
            }

            fn source(&self) -> Option<&dyn Error> {
                None
            }
        }
    };
}

// 使用示例
error DivideError {
    DivisionByZero,
    InvalidResult(f64),
}

// 自动展开为
impl Error for DivideError {
    fn display(&self) -> String {
        match self {
            DivideError::DivisionByZero => "division by zero".to_string(),
            DivideError::InvalidResult(v) => format!("invalid result: {}", v),
        }
    }
}
```

---

## 并发模型设计

### 无锁数据结构 (EPVS)

#### Epoch 保护

```rust
pub struct EpochGuard {
    pub epoch: Arc<Epoch>,
}

impl Drop for EpochGuard {
    fn drop(&mut self) {
        self.epoch.retire();
    }
}

pub struct Epoch {
    pub current: AtomicUsize,
    pub thread_local: ThreadLocal<ThreadState>,
}

pub struct ThreadState {
    pub epoch: usize,
    pub retired: Vec<Retired>,
    pub in_critical: bool,
}

impl Epoch {
    pub fn enter(&self) -> EpochGuard {
        let state = self.thread_local.get();
        state.in_critical = true;
        state.epoch = self.current.load(Ordering::Acquire);
        EpochGuard { epoch: self.clone() }
    }

    pub fn retire(&self) {
        let state = self.thread_local.get();
        state.in_critical = false;

        // 尝试回收已退役的对象
        if state.retired.len() > RETIRE_THRESHOLD {
            self.collect_garbage();
        }
    }
}
```

#### 无锁队列

```rust
pub struct LockFreeQueue<T> {
    pub head: AtomicPtr<Node<T>>,
    pub tail: AtomicPtr<Node<T>>,
}

struct Node<T> {
    pub data: Option<T>,
    pub next: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let node = Box::into_raw(Box::new(Node {
            data: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        LockFreeQueue {
            head: AtomicPtr::new(node),
            tail: AtomicPtr::new(node),
        }
    }

    pub fn push(&self, value: T) {
        let _epoch = self.epoch.enter();

        let new_node = Box::into_raw(Box::new(Node {
            data: Some(value),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };

            if next.is_null() {
                let res = unsafe {
                    (*tail).next.compare_exchange(
                        next,
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    )
                };

                if res.is_ok() {
                    let _ = self.tail.compare_exchange(
                        tail,
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                    return;
                }
            } else {
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let _epoch = self.epoch.enter();

        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if head == tail {
                return None;
            }

            if next.is_null() {
                return None;  // 不应该发生
            }

            let res = unsafe {
                self.head.compare_exchange(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
            };

            if res.is_ok() {
                unsafe {
                    let data = (*next).data.take();
                    self.retire_node(head);
                    return data;
                }
            }
        }
    }
}
```

### 结构化并发

```rust
pub struct TaskScope<F>
where
    F: FnOnce(&mut Scope),
{
    pub tasks: Vec<Task>,
    pub callback: F,
}

pub struct Scope {
    pub tasks: Vec<TaskHandle>,
}

impl<F> TaskScope<F>
where
    F: FnOnce(&mut Scope),
{
    pub fn new(callback: F) -> Self {
        TaskScope {
            tasks: Vec::new(),
            callback,
        }
    }

    pub fn spawn(&mut self, f: impl FnOnce() + Send + 'static) {
        let task = Task::new(f);
        self.tasks.push(task);
    }
}

impl<F> Drop for TaskScope<F>
where
    F: FnOnce(&mut Scope),
{
    fn drop(&mut self) {
        // 等待所有任务完成
        for task in &self.tasks {
            task.join();
        }
    }
}
```

---

## 代码生成策略

### LLVM IR 生成

```rust
pub struct LLVMGenerator {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,
    pub value_map: HashMap<String, LLVMValueRef>,
}

impl LLVMGenerator {
    pub fn compile_type(&self, ty: &Type) -> LLVMTypeRef {
        match ty {
            Type::Void => unsafe { LLVMVoidType() },
            Type::Bool => unsafe { LLVMInt1Type() },
            Type::Integer(IntegerType::I32) => unsafe { LLVMInt32Type() },
            Type::Float(FloatType::F64) => unsafe { LLVMDoubleType() },
            Type::Reference(inner, _) => {
                let base = self.compile_type(inner);
                unsafe { LLVMPointerType(base, 0) }
            },
            Type::Array(inner, size) => {
                let base = self.compile_type(inner);
                unsafe { LLVMArrayType(base, *size) }
            },
            Type::Function { params, return_type, .. } => {
                let param_types: Vec<LLVMTypeRef> = params
                    .iter()
                    .map(|p| self.compile_type(p))
                    .collect();

                let ret_type = self.compile_type(return_type);

                unsafe {
                    LLVMFunctionType(
                        ret_type,
                        param_types.as_ptr(),
                        param_types.len(),
                        false,
                    )
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn compile_function(&mut self, func: &HIR::Function) -> LLVMValueRef {
        let fn_name = func.name.clone();
        let fn_type = self.compile_sig(&func.sig);

        let function = unsafe {
            LLVMAddFunction(self.module, fn_name.as_str(), fn_type)
        };

        let entry = unsafe {
            LLVMAppendBasicBlock(function, b"entry\0".as_ptr())
        };

        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, entry);
        }

        // 编译函数体
        for stmt in &func.body {
            self.compile_statement(stmt);
        }

        function
    }
}
```

### 优化Pass

```rust
pub fn run_optimization_passes(module: LLVMModuleRef, level: OptimizationLevel) {
    let pass_manager = unsafe {
        if level == OptimizationLevel::Aggressive {
            let pm = LLVMCreatePassManager();
            LLVMAddAGgressiveInstCombinPass(pm);
            LLVMAddInstructionCombiningPass(pm);
            LLVMAddReassociatePass(pm);
            LLVMAddGVNPass(pm);
            LLVMAddCFGSimplificationPass(pm);
            LLVMAddLICMPass(pm);
            LLVMAddLoopVectorizePass(pm);
            LLVMAddSLPVectorizePass(pm);
            LLVMAddSCCPPass(pm);
            LLVMAddDeadCodeEliminationPass(pm);
            pm
        } else {
            let pm = LLVMCreatePassManager();
            LLVMAddInstructionCombiningPass(pm);
            LLVMAddDeadCodeEliminationPass(pm);
            pm
        }
    };

    unsafe {
        LLVMRunPassManager(pass_manager, module);
        LLVMDisposePassManager(pass_manager);
    }
}
```

---

## 标准库设计

### 核心模块

```
std::
├── core           // 核心类型和trait
│   ├── clone       // Clone trait
│   ├── fmt         // Display, Debug traits
│   ├── iter        // Iterator trait
│   ├── option      // Option 类型
│   └── result      // Result 类型
├── collections    // 集合
│   ├── vec         // Vec<T>
│   ├── hashmap     // HashMap<K, V>
│   ├── hashset     // HashSet<T>
│   └── btreemap    // BTreeMap<K, V>
├── sync           // 同步原语
│   ├── mutex       // Mutex<T>
│   ├── rwlock      // RwLock<T>
│   ├── arc         // Arc<T>
│   └── lockfree    // 无锁数据结构
├── io             // 输入输出
│   ├── fs          // 文件系统
│   ├── net         // 网络
│   └── process     // 进程管理
├── async          // 异步运行时
│   ├── task        // Task
│   ├── future      // Future<T>
│   └── await       // await! 宏
└── testing        // 测试框架
    ├── assert      // 断言宏
    ├── bench       // 性能测试
    └── mock        // Mock框架
```

### 核心Trait定义

```rust
// Clone trait
trait Clone {
    fn clone(&self) -> Self;
}

// Display trait
trait Display {
    fn fmt(&self) -> String;
}

// Iterator trait
trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    fn map<F, R>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> R,
    { /* ... */ }

    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        P: Fn(&Self::Item) -> bool,
    { /* ... */ }
}
```

---

## 工具链设计

### 包管理器 (zpm)

```rust
pub struct Package {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<Dependency>,
    pub source: Source,
}

pub struct Dependency {
    pub name: String,
    pub version_req: VersionReq,
    pub source: Source,
}

impl Package {
    pub fn download(&self) -> Result<(), Error> {
        // 从源下载包
    }

    pub fn build(&self) -> Result<(), Error> {
        // 编译包
    }
}
```

### 构建系统 (zbuild)

```rust
pub struct BuildConfig {
    pub optimization: OptimizationLevel,
    pub target: TargetTriple,
    pub features: Vec<String>,
}

pub struct Builder {
    pub config: BuildConfig,
}

impl Builder {
    pub fn build(&self, project: &Project) -> Result<(), Error> {
        // 编译项目
        let modules = self.compile_modules(project)?;
        let binary = self.link_modules(&modules)?;

        Ok(())
    }
}
```

---

**文档版本**: 1.0
**最后更新**: 2026-01-07
**维护者**: ZULON Language Team
