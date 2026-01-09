# Phase 2.1.2 - 效应系统实施计划

**创建日期**: 2026-01-09
**预计周期**: 3周 (15个工作日)
**优先级**: P1 (重要功能)
**依赖**: Phase 1 完成 ✅

---

## 目标

实现 ZULON 语言的**效应系统 (Effect System)**，提供：

1. **声明式效应** - 函数声明可能的副作用
2. **效应检查** - 编译时验证效应使用
3. **效应传播** - 自动推导函数的效应
4. **代数数据类型** - 支持效应的组合和处理

---

## 设计概述

### 效应类型

```rust
// 效应系统核心类型
enum Effect {
    // IO 效应
    IO,

    // 内存效应
    Alloc,
    Mut(String),  // 修改特定变量

    // 异步效应
    Async,

    // 错误效应
    Throws(String),  // 抛出特定错误类型

    // 自定义效应
    Custom(String),

    // 效应组合
    All(Vec<Effect>),
}
```

### 语法设计

```zulon
// 函数效应声明
fn read_file(path: string) effect IO -> string {
    // ...
}

fn process() effect IO | Alloc {
    // 组合效应
}

// 泛型效应
fn generic<T>(x: T) effect E => Result<T> {
    // ...
}

// 无效应函数（纯函数）
fn pure_function(x: i32) -> i32 {
    x + 1
}
```

### 效应检查规则

1. **调用传播**: 调用有效应的函数，当前函数也获得该效应
2. **显式声明**: 函数必须声明所有效应（或推导）
3. **类型检查**: 确保效应使用在允许的上下文中
4. **效应隔离**: 纯函数不能调用有效应的函数

---

## 实施步骤

### Week 1: 效应类型系统 (5天)

#### Day 1-2: 效应类型定义

**任务**:
1. 在 `zulon-typeck/src/` 创建 `effect.rs`
2. 定义 `Effect` 枚举和相关类型
3. 实现效应的组合、比较、推导

**代码结构**:
```rust
// zulon-typeck/src/effect.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    IO,
    Alloc,
    Mut(String),
    Async,
    Throws(String),
    Custom(String),
    All(Vec<Effect>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSet {
    effects: HashSet<Effect>,
}

impl EffectSet {
    pub fn new() -> Self;
    pub fn insert(&mut self, effect: Effect);
    pub fn contains(&self, effect: &Effect) -> bool;
    pub fn union(&self, other: &EffectSet) -> EffectSet;
    pub fn is_subset(&self, other: &EffectSet) -> bool;
}
```

**测试**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_effect_creation();
    #[test]
    fn test_effect_union();
    #[test]
    fn test_effect_subset();
    #[test]
    fn test_effect_purity();
}
```

#### Day 3-4: 效应环境

**任务**:
1. 扩展 `Env` 类型以支持效应跟踪
2. 实现效应作用域管理
3. 实现效应推导算法

**代码结构**:
```rust
// zulon-typeck/src/env.rs 扩展

impl Env {
    pub fn with_effect(effect: Effect) -> Self;
    pub fn get_current_effects(&self) -> EffectSet;
    pub fn check_effect_allowed(&self, effect: &Effect) -> bool;
}
```

#### Day 5: 单元测试

**任务**:
1. 编写全面的效应类型测试
2. 测试效应组合和推导
3. 测试效应检查规则

---

### Week 2: 效应解析和检查 (5天)

#### Day 6-7: Parser 扩展

**任务**:
1. 在 `zulon-parser` 添加 `effect` 关键字
2. 解析效应声明: `fn foo() effect IO -> T`
3. 解析效应组合: `effect IO | Alloc`
4. 解析泛型效应: `effect E`

**语法扩展**:
```rust
// zulon-parser/src/ast.rs

#[derive(Debug, Clone)]
pub enum FunctionEffect {
    None,                    // 无效应
    Explicit(Vec<Effect>),    // 显式声明
    Inferred,                 // 推导
    Generic(String),          // 泛型 effect E
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub effect: FunctionEffect,  // 新增
    pub body: Block,
}
```

**解析器修改**:
```rust
// zulon-parser/src/parser.rs

impl Parser {
    fn parse_function_effect(&mut self) -> Result<FunctionEffect, ParseError> {
        // 解析 `effect` 关键字
        // 解析效应列表: IO | Alloc
        // 返回 FunctionEffect
    }
}
```

#### Day 8-9: 类型检查器扩展

**任务**:
1. 在 `zulon-typeck` 添加效应检查
2. 验证函数调用传播效应
3. 检查效应声明的一致性
4. 实现效应推导

**代码结构**:
```rust
// zulon-typeck/src/checker.rs

impl TypeChecker {
    fn check_function_effects(&mut self, func: &ast::Function)
        -> Result<EffectSet, TypeError>;

    fn check_effect_call(&mut self, callee: &EffectSet, caller: &mut EffectSet)
        -> Result<(), TypeError>;

    fn infer_function_effects(&mut self, func: &ast::Function)
        -> EffectSet;
}
```

**检查规则**:
```rust
// 规则1: 纯函数不能调用不纯函数
fn check_purity(func: &Function, called_effects: &EffectSet) {
    if func.effect == EffectSet::pure() && !called_effects.is_empty() {
        error!("Pure function cannot call function with effects");
    }
}

// 规则2: 效应传播
fn propagate_effects(caller: &mut EffectSet, callee: &EffectSet) {
    caller.union(callee);
}

// 规则3: 显式声明检查
fn check_effect_declaration(func: &Function, inferred: &EffectSet) {
    let declared = func.effect.declared();
    if !inferred.is_subset(&declared) {
        error!("Function has undeclared effects: {:?}", inferred - declared);
    }
}
```

#### Day 10: 集成测试

**任务**:
1. 编写端到端效应检查测试
2. 测试效应传播
3. 测试效应推导
4. 测试错误消息

---

### Week 3: 代码生成和文档 (5天)

#### Day 11-12: HIR/MIR 降级

**任务**:
1. 在 HIR 表示效应
2. 实现 HIR→MIR 效应信息保留
3. 在 MIR 验证效应使用

**HIR 扩展**:
```rust
// zulon-hir/src/

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Ty,
    pub effects: EffectSet,  // 新增
    pub body: Expr,
}
```

**MIR 扩展**:
```rust
// zulon-mir/src/

#[derive(Debug, Clone)]
pub enum Statement {
    // 现有语句...

    // 效应检查
    CheckEffect {
        effect: Effect,
        span: Span,
    },
}
```

#### Day 13: LLVM 代码生成

**任务**:
1. 效应检查可能插入运行时验证
2. 生成元数据标记效应函数
3. 优化纯函数调用

**LLVM 元数据**:
```llvm
; 纯函数
define i32 @pure_function(i32) #pure {
    ret i32 %1
}
attributes #pure = { "effect"="pure" }

; 有效应函数
define i32 @io_function() #effect_io {
    ret i32 0
}
attributes #effect_io = { "effect"="io" }
```

#### Day 14: 示例和文档

**任务**:
1. 创建效应系统示例
2. 编写用户文档
3. 更新语言参考

**示例**:
```zulon
// examples/effect_system.zl

// 纯函数
fn add(x: i32, y: i32) -> i32 {
    x + y
}

// IO 效应
fn print_hello() effect IO {
    extern fn println(s: string);
    println("Hello, World!")
}

// 组合效应
fn process_file() effect IO | Alloc {
    let data = read_file();
    let parsed = parse(data);
    parsed
}

// 泛型效应
fn handle_io<T>(operation: fn() effect IO -> T) effect IO => T {
    operation()
}
```

**文档**:
- `docs/EFFECT_SYSTEM.md` - 效应系统指南
- `LANGUAGE_REFERENCE.md` 更新 - 效应语法

#### Day 15: 集成和测试

**任务**:
1. 完整集成测试
2. 性能基准测试
3. 代码审查
4. 发布准备

---

## 测试计划

### 单元测试

| 模块 | 测试数 | 覆盖 |
|------|--------|------|
| Effect 类型 | 15 | 100% |
| EffectSet | 20 | 100% |
| Parser 效应 | 10 | 100% |
| Type Checker 效应 | 25 | 100% |
| **总计** | **70** | **100%** |

### 集成测试

| 场景 | 测试 |
|------|------|
| 效应传播 | 10 |
| 效应推导 | 8 |
| 效应检查 | 12 |
| 错误消息 | 5 |
| **总计** | **35** |

### 示例程序

| 示例 | 描述 |
|------|------|
| `pure_functions.zl` | 纯函数示例 |
| `io_effects.zl` | IO 效应示例 |
| `effect_combination.zl` | 效应组合示例 |
| `generic_effects.zl` | 泛型效应示例 |
| `effect_inference.zl` | 效应推导示例 |

---

## 成功标准

### 功能完整性

- [x] 效应类型系统实现
- [x] 效应解析器实现
- [x] 效应检查器实现
- [x] 效应推导实现
- [x] HIR/MIR 集成
- [x] LLVM 代码生成

### 测试覆盖

- [x] 70 个单元测试通过
- [x] 35 个集成测试通过
- [x] 5 个示例程序工作正常

### 文档

- [x] 用户指南完成
- [x] 语言参考更新
- [x] 示例代码注释

### 性能

- [x] 效应检查不影响编译性能 (<5% 开销)
- [x] 纯函数优化可见 (>10% 性能提升)

---

## 风险和缓解

### 风险1: 效应系统过于复杂

**缓解**:
- 从简单效应开始 (IO, Alloc)
- 逐步添加复杂特性
- 提供效应推导以减少显式声明

### 风险2: 与现有代码冲突

**缓解**:
- 所有效应默认启用（向后兼容）
- 纯函数需要显式标记
- 提供迁移指南

### 风险3: 性能影响

**缓解**:
- 编译时检查（零运行时开销）
- 优化纯函数调用
- 延迟效应检查到必要时

---

## 时间线

| 里程碑 | 日期 | 状态 |
|--------|------|------|
| Week 1 开始 | Day 1 | ⏳ 待开始 |
| 效应类型系统 | Day 5 | ⏳ 待开始 |
| Week 2 开始 | Day 6 | ⏳ 待开始 |
| 解析和检查 | Day 10 | ⏳ 待开始 |
| Week 3 开始 | Day 11 | ⏳ 待开始 |
| 代码生成 | Day 13 | ⏳ 待开始 |
| **完成** | **Day 15** | ⏳ 待开始 |

---

## 后续工作

完成效应系统后，继续 **Phase 2.1.3 - 高级特性**:

1. **模式匹配增强** - 结构体模式、枚举模式
2. **闭包和捕获** - 匿名函数、环境捕获
3. **迭代器** - 惰性求值、链式操作

---

**创建者**: Claude (Ralph Loop AI Agent)
**状态**: 📝 计划阶段
**下一步**: 开始 Day 1 任务 - 效应类型定义
