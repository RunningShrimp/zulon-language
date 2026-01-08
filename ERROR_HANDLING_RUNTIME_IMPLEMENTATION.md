# ZULON 错误处理端到端实现计划

**创建日期**: 2026-01-08
**优先级**: P0 (最高 - 阻塞其他功能)
**状态**: 📋 规划阶段
**预计时间**: 2周

---

## 🎯 目标

完成 ZULON 错误处理系统的端到端实现，从 Parser 到 Runtime 的完整支持。

### 当前状态

- ✅ **Parser**: 100% 完成（支持 throw, ?, | 语法）
- ✅ **HIR**: 100% 完成（error_type, effects 集成）
- ✅ **Typecheck**: 90% 完成（throw/? 验证）
- ✅ **MIR**: 100% 完成（discriminant checking）
- ✅ **LIR**: 100% 完成（字段访问 GEP）
- ⏳ **LLVM Codegen**: 90% 完成（throw 代码生成，简化版）
- ⏳ **Runtime**: 0% 完成（缺失 Outcome<T,E> 运行时）
- ⏳ **集成测试**: 0% 完成

### 目标状态

- ✅ Parser: 100%
- ✅ HIR: 100%
- ✅ Typecheck: 100%
- ✅ MIR: 100%
- ✅ LIR: 100%
- ✅ LLVM Codegen: 100%
- ✅ Runtime: 100%
- ✅ 集成测试: 100%

---

## 📋 任务分解

### Week 1: Runtime 实现

#### Day 1-2: Outcome<T,E> 运行时类型设计

**目标**: 设计并实现 Outcome<T,E> 的运行时表示

**任务**:
1. **设计数据结构**
   ```rust
   // crates/zulon-runtime-core/src/outcome.rs
   #[repr(C)]
   pub enum Outcome<T, E> {
       Ok(T),
       Err(E),
   }
   ```

2. **实现内存布局**
   - 研究 Rust 的 Result<T,E> 布局
   - 实现零成本抽象
   - 确保 FFI 兼容性

3. **实现基础方法**
   ```rust
   impl<T, E> Outcome<T, E> {
       pub fn is_ok(&self) -> bool;
       pub fn is_err(&self) -> bool;
       pub fn ok(self) -> Option<T>;
       pub fn err(self) -> Option<E>;
       pub fn unwrap(self) -> T;
       pub fn unwrap_err(self) -> E;
       pub fn map<U, F>(self, f: F) -> Outcome<U, E>
       where
           F: FnOnce(T) -> U;
       pub fn and_then<U, F>(self, f: F) -> Outcome<U, E>
       where
           F: FnOnce(T) -> Outcome<U, E>;
   }
   ```

4. **单元测试**
   - 测试 Ok 分支
   - 测试 Err 分支
   - 测试所有方法
   - 测试内存布局

**预期成果**:
- `zulon-runtime-core/src/outcome.rs` (~200 行)
- 完整的单元测试 (~100 行)
- 零编译错误和警告

---

#### Day 3-4: 错误传播机制

**目标**: 实现错误传播（? 运算符）的运行时支持

**任务**:
1. **实现 From trait**
   ```rust
   pub trait From<T> {
       fn from(t: T) -> Self;
   }

   impl<E> From<E> for Outcome<!, E> {
       fn from(err: E) -> Self {
           Outcome::Err(err)
       }
   }
   ```

2. **实现错误转换**
   ```rust
   impl<T, E> Outcome<T, E> {
       pub fn from_err<E2>(self) -> Outcome<T, E2>
       where
           E: Into<E2>,
       {
           match self {
               Outcome::Ok(v) => Outcome::Ok(v),
               Outcome::Err(e) => Outcome::Err(e.into()),
           }
       }
   }
   ```

3. **实现 ? 运算符支持**
   - 研究 Rust 的 ? 运算符去糖
   - 实现编译器集成点
   - 错误消息生成

4. **测试**
   - 测试简单错误传播
   - 测试错误转换
   - 测试嵌套错误传播

**预期成果**:
- From trait 实现
- 错误传播测试
- 与 Codegen 集成点

---

#### Day 5: 错误恢复和链追踪

**目标**: 实现错误恢复和错误链功能

**任务**:
1. **错误链支持**
   ```rust
   pub trait Error: Display + Debug {
       fn source(&self) -> Option<&(dyn Error + 'static)>;
       fn description(&self) -> &str;
       fn cause(&self) -> Option<&dyn Error> {
           self.source()
       }
   }
   ```

2. **实现 anyhow 风格的错误上下文**
   ```rust
   pub struct ContextError<M, E> {
       msg: M,
       error: E,
   }

   impl<M, E> Error for ContextError<M, E>
   where
       M: Display + Debug + 'static,
       E: Error + 'static,
   {
       fn source(&self) -> Option<&(dyn Error + 'static)> {
           Some(&self.error)
       }

       fn description(&self) -> &str {
           "error context"
       }
   }
   ```

3. **实现 panic 支持**
   ```rust
   pub fn panic(msg: &str) -> ! {
       eprintln!("Panic: {}", msg);
       std::process::exit(1);
   }

   impl<T, E> Outcome<T, E> {
       pub fn expect(self, msg: &str) -> T {
           match self {
               Outcome::Ok(v) => v,
               Outcome::Err(e) => panic(msg),
           }
       }
   }
   ```

4. **测试**
   - 测试错误链
   - 测试错误上下文
   - 测试 panic 行为

**预期成果**:
- Error trait 定义
- 错误链实现
- panic 支持
- 完整测试

---

### Week 2: 集成和测试

#### Day 6-7: Codegen 集成

**目标**: 完善 LLVM 错误处理代码生成

**任务**:
1. **审查现有 throw 代码生成**
   - 查看 `crates/zulon-codegen-llvm/src/expr.rs`
   - 检查 throw 语句实现
   - 识别缺失部分

2. **完善 ? 运算符代码生成**
   - 实现错误传播
   - 实现提前返回
   - 生成高效的 LLVM IR

3. **实现 Outcome<T,E> 代码生成**
   - 类型映射
   - 内存布局
   - 方法调用

4. **优化**
   - 减少分支
   - 内联小函数
   - 零成本抽象

5. **测试**
   - IR 测试
   - 性能测试
   - 正确性测试

**预期成果**:
- 完整的 throw 代码生成
- 完整的 ? 代码生成
- Outcome<T,E> 代码生成

---

#### Day 8-9: 端到端集成测试

**目标**: 编写完整的端到端测试

**任务**:
1. **基础错误处理测试**
   ```zulon
   enum MathError {
       DivisionByZero,
       NegativeInput,
   }

   fn divide(a: i32, b: i32) -> i32 | MathError {
       if b == 0 {
           throw MathError::DivisionByZero;
       }
       if a < 0 || b < 0 {
           throw MathError::NegativeInput;
       }
       a / b
   }

   fn main() -> i32 {
       let result = divide(10, 2) match {
           Ok(value) => println("Result: {}", value),
           Err(MathError::DivisionByZero) => println("Error: Division by zero"),
           Err(MathError::NegativeInput) => println("Error: Negative input"),
       };
       0
   }
   ```

2. **错误传播测试**
   ```zulon
   fn parse_int(s: &str) -> i32 | ParseError {
       // ... parsing logic
       Ok(42)
   }

   fn calculate(s: &str) -> i32 | ParseError {
       let n = parse_int(s)?;  // ? 运算符
       Ok(n * 2)
   }
   ```

3. **错误链测试**
   ```zulon
   fn read_file(path: &str) -> String | IoError {
       // ... file reading
       Ok("content".into())
   }

   fn process_file(path: &str) -> String | IoError {
       let content = read_file(path)?;
       Ok(content)
   }
   ```

4. **性能测试**
   - 错误处理开销 < 5%
   - 错误传播零成本
   - Ok 路径零成本

**预期成果**:
- 10+ 端到端测试
- 性能基准测试
- 测试通过率 100%

---

#### Day 10: 文档和示例

**目标**: 编写完整的用户文档和示例

**任务**:
1. **用户指南**
   - 错误处理语法
   - throw 关键字
   - ? 运算符
   - | 分隔符
   - Outcome<T,E> 使用

2. **最佳实践**
   - 何时使用错误处理
   - 如何定义错误类型
   - 错误转换技巧
   - 性能考虑

3. **示例程序**
   - 基础错误处理
   - 错误传播
   - 错误链
   - 自定义错误类型

4. **API 文档**
   - Outcome<T,E> API
   - Error trait
   - 相关函数

**预期成果**:
- `docs/ERROR_HANDLING_GUIDE.md`
- `examples/error_handling/` 目录
- 5+ 示例程序

---

## 📊 成功标准

### 功能完整性

- [ ] throw 语句完全可用
- [ ] ? 运算符完全可用
- [ ] | 分隔符完全可用
- [ ] Outcome<T,E> 运行时支持
- [ ] 错误传播正常工作
- [ ] 错误链追踪可用

### 性能要求

- [ ] Ok 路径零成本（无分支）
- [ ] Err 路径开销 < 5%
- [ ] 错误传播零成本（仅编译时）
- [ ] 内存占用增长 < 2%

### 质量要求

- [ ] 零内存泄漏
- [ ] 零未定义行为
- [ ] 测试覆盖率 > 90%
- [ ] 所有示例通过

### 文档要求

- [ ] 用户指南完整
- [ ] API 文档完整
- [ ] 示例程序可用
- [ ] 最佳实践文档

---

## 🚀 实施步骤

### Step 1: 准备（Day 0）

```bash
# 创建工作分支
git checkout -b feature/error-handling-runtime

# 创建目录结构
mkdir -p crates/zulon-runtime-core/src
mkdir -p examples/error_handling
mkdir -p docs

# 设置开发环境
cargo build
```

### Step 2: Runtime 实现（Day 1-5）

```bash
# 创建 outcome.rs
touch crates/zulon-runtime-core/src/outcome.rs

# 实现 Outcome<T,E>
# （按上述任务分解）

# 运行测试
cargo test --package zulon-runtime-core
```

### Step 3: Codegen 集成（Day 6-7）

```bash
# 完善代码生成
# （按上述任务分解）

# 运行集成测试
cargo test --package zulon-codegen-llvm
```

### Step 4: 端到端测试（Day 8-9）

```bash
# 编写测试
touch examples/error_handling/basic_error.zl
touch examples/error_handling/error_propagation.zl

# 运行测试
yan run examples/error_handling/basic_error.zl
```

### Step 5: 文档和示例（Day 10）

```bash
# 编写文档
touch docs/ERROR_HANDLING_GUIDE.md

# 创建示例
# （按上述任务分解）
```

---

## 📝 检查清单

### Week 1

- [ ] Outcome<T,E> 类型实现
- [ ] From trait 实现
- [ ] 错误传播机制
- [ ] Error trait 实现
- [ ] panic 支持
- [ ] 单元测试通过

### Week 2

- [ ] Codegen 集成完成
- [ ] 端到端测试通过
- [ ] 性能测试达标
- [ ] 用户文档完成
- [ ] 示例程序完成
- [ ] 代码审查通过

---

## 🎯 交付物

### 代码

1. `crates/zulon-runtime-core/src/outcome.rs` (~200 行)
2. `crates/zulon-runtime-core/src/error.rs` (~100 行)
3. `crates/zulon-codegen-llvm/src/error.rs` 改进
4. 测试代码 (~300 行)

### 文档

1. `docs/ERROR_HANDLING_GUIDE.md` (~500 行)
2. `docs/ERROR_HANDLING_BEST_PRACTICES.md` (~300 行)
3. API 文档（内联注释）

### 示例

1. `examples/error_handling/basic_error.zl`
2. `examples/error_handling/error_propagation.zl`
3. `examples/error_handling/error_chain.zl`
4. `examples/error_handling/custom_error.zl`
5. `examples/error_handling/advanced_usage.zl`

---

## ⏱️ 时间估算

| 任务 | 预计时间 | 缓冲时间 | 总计 |
|------|----------|----------|------|
| Outcome 实现 | 2天 | 0.5天 | 2.5天 |
| 错误传播 | 2天 | 0.5天 | 2.5天 |
| 错误恢复 | 1天 | 0.5天 | 1.5天 |
| Codegen 集成 | 2天 | 0.5天 | 2.5天 |
| 集成测试 | 2天 | 0.5天 | 2.5天 |
| 文档示例 | 1天 | 0.5天 | 1.5天 |
| **总计** | **10天** | **3天** | **13天** |

**预计完成**: 2周（包含缓冲）

---

## 🎓 技术参考

### 参考实现

1. **Rust Result<T,E>**
   - https://doc.rust-lang.org/std/result/enum.Result.html
   - 源码: library/core/src/result.rs

2. **Rust Error trait**
   - https://doc.rust-lang.org/std/error/trait.Error.html
   - 源码: library/core/src/error.rs

3. **Rust ? 运算符**
   - https://doc.rust-lang.org/std/keyword.question.html
   - RFC: https://github.com/rust-lang/rfcs/pull/243

4. **anyhow 错误处理**
   - https://docs.rs/anyhow/
   - 源码: https://github.com/dtolnay/anyhow

### 设计原则

1. **零成本抽象** - Ok 路径应该无开销
2. **类型安全** - 编译时检查错误类型
3. **内存安全** - 无泄漏，无 UB
4. **用户友好** - 清晰的 API 和错误消息

---

## 🚨 风险和缓解

### 技术风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| 性能回归 | 高 | 基准测试，优化热路径 |
| 内存布局 | 中 | 参考 Rust Result，验证 |
| ABI 兼容 | 中 | 使用 repr(C)，测试 FFI |
| 复杂度 | 中 | 简化设计，迭代开发 |

### 项目风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| 时间延误 | 中 | MVP 优先，缓冲时间 |
| 测试不足 | 高 | 90% 覆盖率目标 |
| 文档不完整 | 中 | 专职文档时间 |

---

## 📞 支持和资源

### 联系方式

- **GitHub Issues**: [github.com/RunningShrimp/zulon-language/issues](https://github.com/RunningShrimp/zulon-language/issues)
- **GitHub Discussions**: [github.com/RunningShrimp/zulon-language/discussions](https://github.com/RunningShrimp/zulon-language/discussions)

### 参考资源

- ZULON 文档索引: `docs/DOCUMENTATION_INDEX.md`
- 类型系统实现: `docs/TYPE_SYSTEM_IMPLEMENTATION.md`
- 错误处理设计: Phase 4.3 文档

---

**计划版本**: 1.0
**创建日期**: 2026-01-08
**负责人**: ZULON Language Team
**状态**: 📋 **准备开始**

**🚀 开始实施 ZULON 错误处理端到端实现！**
