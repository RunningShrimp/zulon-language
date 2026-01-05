# ZULON 语言集成式设计与增强规划（工程可执行版）

**版本**: 2.2（最终整合版）  
**日期**: 2026-01-05  
**作者**: Zulon Language Design Team  
**状态**: 集成设计规范（Integrated Design Specification）

> 本文档面向编译器、运行时与标准库工程落地：把愿景转化为可实现的语义、IR 约束、验收口径与 API 草案。

---

## 目录

1. [愿景与边界](#1-愿景与边界)
2. [语言总览与入口模型（脚本/系统统一）](#2-语言总览与入口模型脚本系统统一)
3. [类型系统深度设计规范（安全与并发的基石）](#3-类型系统深度设计规范安全与并发的基石)
4. [核心安全与性能机制（基于类型系统的实现）](#4-核心安全与性能机制基于类型系统的实现)
5. [并发模型：无锁优先与结构化并发](#5-并发模型无锁优先与结构化并发)
6. [错误处理与代数效应：统一异常/异步/注入](#6-错误处理与代数效应统一异常异步注入)
7. [开发体验与诊断系统（含学习模式）](#7-开发体验与诊断系统含学习模式)
8. [语法体系与高级特性（函数式、泛型、反射、元编程）](#8-语法体系与高级特性函数式泛型反射元编程)
9. [标准库架构：Core/Std/Ext](#9-标准库架构corestdext)
10. [多目标编译与工具链：`yan` 统一入口](#10-多目标编译与工具链yan-统一入口)
11. [六大领域能力：特性与标准库 API 草案](#11-六大领域能力特性与标准库-api-草案)
12. [默认控安全：类型安全 + 内存安全 + 访问控制](#12-默认控安全类型安全--内存安全--访问控制)
13. [编译器工程规格：IR、Pass、后端与路线图](#13-编译器工程规格irpass后端与路线图)
14. [附录：语义权衡与前沿参考](#14-附录语义权衡与前沿参考)

---

## 1. 愿景与边界

### 1.1 愿景

ZULON 的目标是在不牺牲系统级性能的前提下，把“正确并发 + 内存安全 + 可预期错误处理”变成默认路径；同时提供脚本式迭代体验与跨平台生态连接能力。

工程承诺以可量化验收口径表达（详见 [7.1](#71-量化开发体验验收指标)）：

- 默认空安全：空值不属于 `T`，仅属于 `T?`。
- 默认内存安全：无悬垂指针、无 double-free、无 use-after-free。
- 默认并发安全：跨并发边界的可传输/可共享能力由类型系统与编译器推断/验证。
- 多目标可行：同一前端语义可落地到 Native/Wasm/JS/JVM/Rust 源码（详见 [10](#10-多目标编译与工具链yan-统一入口)）。

### 1.2 五大统一（The Power of 5）

为降低认知成本，ZULON 将核心叙事收敛为五个统一概念：

1. **统一所有权**：值的默认语义为 `owned`；并发/逃逸驱动自动提升为 `local` 或 `shared`。
2. **统一任务**：所有异步都运行于 `task`；取消、超时、错误传播有明确语义。
3. **统一隔离**：共享可变默认禁止；可变状态推荐封装在 `actor` 内部。
4. **统一副作用**：错误、取消、I/O、注入都以类型化副作用（effects）显式呈现。
5. **统一工具链**：`yan` 一体化管理依赖、编译、运行、测试、基准与诊断。

### 1.3 非目标（P0/P1 约束）

为保证可交付性，P0/P1 明确不做：

- 不以“全能 GC 语言”作为默认路线：P0 仅要求确定性内存（栈/区域/共享冻结）。
- 不把“锁”作为默认并发模型：锁仅在底层库/FFI/unsafe 边界内允许。
- 不引入“无限语法糖”：核心语法保持小而正交；高级能力通过库与编译期反射实现。

---

## 2. 语言总览与入口模型（脚本/系统统一）

### 2.1 文件、模块与包

- 文件扩展名：`.zl`。
- 包（package）是分发与版本单元；模块（module）是命名空间单元。
- `yan.toml` 定义包名、版本、目标、依赖、特性开关。

### 2.2 两种入口：`start` 与 `main`

ZULON 同时支持脚本式与系统式编程，但用同一语言表达：

- **脚本模式入口**：`start`（适合快速运行、解释/JIT、或轻量 AOT）。
- **系统模式入口**：`main`（生成可执行文件、无运行时依赖、支持 `--release` 优化）。

规则：

- 一个包最多存在一个 `start` 与一个 `main`。
- `start` 允许依赖运行时能力（例如可选 GC、动态模块加载、热重载）。
- `main` 必须可在 `no_runtime` 配置下通过（除非显式开启）。

示例：

```zulon
fn start() ! AppError {
  let cfg = config::load()?;
  println(cfg.name);
  Ok(())
}

fn main() ! AppError {
  return server::run()?;
}
```

---

## 3. 类型系统深度设计规范（安全与并发的基石）

### 3.1 设计哲学

#### 3.1.1 推断先行（Inference First）

- 局部变量、闭包参数、私有函数参数允许推断。
- 公共 API（`pub`）默认要求显式类型，保证可读性与稳定性。

#### 3.1.2 默认空安全（Null-Safety by Default）

- 非空类型 `T` 为默认。
- 可空类型显式写作 `T?`。
- `null` 仅属于 `T?`，不属于 `T`。

#### 3.1.3 所有权注记隐喻化（Ownership as Capability）

ZULON 以权限修饰符表达能力边界，并由编译器尽量推断：

- `owned`：独占所有权（默认）。
- `shared`：共享不可变（冻结后可跨并发共享）。
- `local`：作用域/区域绑定（不可逃逸）。

#### 3.1.4 代数效应（Algebraic Effects as Unifier）

副作用（I/O、取消、异常恢复、依赖注入等）在类型中显式出现，可组合、可推导、可在库中扩展。P0 以“效果集合（effects set）”为主，优先保证可推导与可实现；P1+ 再演进到行多态/关联效应等更强表达（参考 Koka 与近年的“关联效应”路线）[^koka][^associated-effects]。

#### 3.1.5 渐进式动态（Local Gradual Typing）

局部允许 `dynamic`，但动态性必须是边界化的：

- `dynamic` 只能出现在显式标注位置。
- 从 `dynamic` 回到静态类型必须通过 `cast`（可失败）或模式匹配。
- `yan vet` 对 `dynamic` 传播出包边界给出警告（可配置为错误）。

---

### 3.2 核心类型架构

#### 3.2.1 基础数值类型

- 有符号整数：`i8 i16 i32 i64 i128 isize`
- 无符号整数：`u8 u16 u32 u64 u128 usize`
- 浮点：`f16 f32 f64`
- 布尔：`bool`
- 字符：`char`（Unicode scalar）

整数溢出规则：

- 默认：溢出检查（触发 `panic` 或返回错误由上下文策略决定）。
- 局部可用 `@wrap` 或 `@unchecked` 关闭（进入可审计路径，见 [12.2](#122-内存安全与-unsafe-边界)）。

#### 3.2.2 字符串与字节

- `String`：UTF-8 拥有型字符串（`owned`）。
- `str`：只读切片视图（通常 `shared` 或 `local`）。
- `Bytes`：字节序列（支持零拷贝切片）。

#### 3.2.3 原生向量/张量类型（面向 SIMD 与 AI/科学计算）

ZULON 内置向量类型族：

- `vNf32`、`vNi32` 等（N 为 2/4/8/16…，由 target-feature 决定可用集合）。

ZULON 内置张量：

- `tensor<T, const Rank: u32>`：秩固定。
- `shape<const R: u32>`：维度元组类型，允许部分动态（例如 `shape<(?, 768)>`）。

编译器承诺：

- 对 `vN*` 的算术映射到 SIMD 指令集（SSE/AVX/NEON/RVV），失败则回退标量。
- 对 `tensor` 的 element-wise 操作默认向量化；对 reduction 使用并行/向量化联合优化（见 [4.3](#43-规划性能与内存管理路径)）。

#### 3.2.4 ADT（代数数据类型）与模式匹配完备性

```zulon
enum Option<T> {
  Some(T),
  None,
}

fn len_or_zero(x: String?) -> usize {
  match x {
    case null => 0,
    case s => s.len(),
  }
}
```

完备性检查要求：

- `match` 必须覆盖所有变体；否则编译错误。
- `match` 的 guard 分支不计入完备性；必须提供兜底分支或补齐。

#### 3.2.5 显式可空性与安全解包

```zulon
fn zip(user: User?) -> String {
  return user?.address?.zip ?? "000000";
}
```

规则：

- `?.` 将 `T?` 链式传播为 `U?`。
- `??` 为 `T?` 提供默认值，结果为非空类型。

#### 3.2.6 预期错误类型：`T ! E`

`T ! E` 是 `Result<T, E>` 的语法糖：

```zulon
fn read_text(p: Path) -> String ! IoError
```

---

### 3.3 并发与所有权类型注记

#### 3.3.1 并发能力：`Send` / `Sync` / `Share`

- `Send`：值可跨 `task/actor` 边界移动（Move）。
- `Sync`：值可被多个任务并发读取且保持内存安全。
- `Share`：值可跨边界共享别名，且必须是深度不可变（冻结）。

能力推断：

- `owned T` 默认可 `Send`，前提是其内部递归满足 `Send`。
- `shared T` 默认 `Sync + Share`，前提是其内部深度不可变。
- `local T` 永不 `Send/Share`。

跨边界规则（P0）：

- Move：参数必须满足 `Send`。
- Share：必须是 `Share` 且不可变。
- Region 绑定：`local` 值不可跨作用域，不可存入全局，不可被 escape。

术语澄清（避免歧义）：

- `shared T`：权限修饰符，表示“冻结后的共享不可变值”。
- `Share`：并发能力 trait，表示“允许跨并发边界共享别名”。
- `share(x)`：显式“冻结 + 晋升到 shared”的构造（要求 `x` 深度不可变或可冻结）。

闭包/跨任务捕获规则（P0）：

- 同一 task 内闭包捕获：允许以借用方式捕获，捕获的借用视为 `local` 并受 region/scope 约束。
- 跨 task 捕获（`task::spawn`/actor send）：禁止捕获 `local` 借用；只能捕获满足 `Send` 的 move 值，或 `Share` 的冻结值。

```zulon
fn demo(buf: Bytes) {
  task::scope {
    let frozen = share(buf);
    let h = task::spawn { use_bytes(frozen) };
    h.await;
  };
}
```

#### 3.3.2 权限修饰符的语义

- `owned`：唯一所有者；被 move 后原绑定失效。
- `local`：绑定到 region/task scope；其地址不可逃逸出作用域。
- `shared`：冻结后的共享快照；不可通过任何路径获得可变别名。

---

### 3.4 特征系统（Trait）：多态与约束

#### 3.4.1 单态化泛型 + where 约束

```zulon
fn add<T>(a: T, b: T) -> T
where T: Add<Output = T> {
  return a + b;
}
```

#### 3.4.2 关联类型与常量泛型

```zulon
trait Matrix<T> {
  const ROWS: usize;
  const COLS: usize;
  type Output;
  fn mul(self, other: Self) -> Self::Output;
}
```

#### 3.4.3 动态分发：`dyn Trait`

- `impl Trait`：静态分发（单态化）。
- `dyn Trait`：动态分发（vtable），仅用于边界处减少代码膨胀。

#### 3.4.4 孤儿规则与可控豁免

默认孤儿规则：禁止在外部包为外部类型实现外部 Trait。

可控豁免：

- 允许 `impl` 放在“适配包”内，并要求显式 `use impls::...` 导入生效，避免全局污染。

---

### 3.5 代数效应与渐进式类型

#### 3.5.1 效应声明与触发

```zulon
effect FileSystem {
  fn read(path: Path) -> Bytes ! IoError;
}

fn load_config() -> Config ! ConfigError performs FileSystem {
  let data = do FileSystem::read("config.json")?;
  return config::parse(data)?;
}
```

#### 3.5.2 处理效应（Handler）

```zulon
fn main() ! AppError {
  handle load_config() {
    case FileSystem::read(p) => resume(vfs::read(p)),
  }
}
```

#### 3.5.3 局部 `dynamic`

```zulon
fn from_json(x: dynamic) -> User ! JsonError {
  let id: i64 = cast<i64>(x.id)?;
  let name: String = cast<String>(x.name)?;
  return User { id, name };
}
```

#### 3.5.4 Effect 多态（P0 形态）

P0 允许“把效应当作类型参数传递”的表达方式，用于高阶函数复用：

```zulon
fn map<T, U, E>(xs: List<T>, f: fn(T) -> U performs E) -> List<U> performs E {
  let mut out = List::new();
  for x in xs { out.push(f(x)); }
  return out;
}
```

约束：

- `E` 在 P0 等价于 effects set（例如 `IO + Net`）；编译器必须能做集合并集/子集判定。
- `yan vet` 可对“过宽的 E”给出警告（建议在边界收敛 handler）。

---

### 3.6 类型检查与错误诊断（含学习模式）

错误信息必须同时满足：

- 可定位：指向具体 token/表达式。
- 可修复：提供可执行的修复建议（move/share/actor 化/缩小作用域）。
- 可教学：学习模式解释触发规则背后的安全目标与最佳实践。

统一结构（编译器与 `yan vet` 同构）：

```text
E-SEND: value of type `local Buffer` cannot cross task boundary
 --> src/net.zu:42:17
  |
42|   task::spawn { send(buf) }
  |                 ^^^ buf is `local` and not `Send`
  |
Rule: Values captured by child tasks must be `Send` or `shared`.
Help:
  - move ownership: `task::spawn { send(move buf) }`
  - or freeze: `let buf = share(buf)`
  - or restructure: keep buffer inside an actor
Learn:
  ZULON forbids sharing mutable state across tasks to prevent data races.
  `local` values are tied to a scope; sending them could outlive that scope.
```

---

## 4. 核心安全与性能机制（基于类型系统的实现）

### 4.1 统一内存与并发架构（UMCA）

UMCA 由两部分组成：

- UMMA：统一内存管理（栈/区域/共享冻结/可选托管堆）。
- UCMA：统一并发管理（结构化并发/actor/共享可变回退）。

### 4.2 内存分层：L1/L2/L3/L4

| 层级 | 权限视图 | 机制 | 跨并发语义 | 典型场景 |
|---|---|---|---|---|
| L1 | `owned` | 栈/标量替换 | Move | 局部与临时 |
| L2 | `local` | region bump 分配 | 作用域绑定 | 请求上下文、帧数据 |
| L3 | `shared` | 引用计数 + 冻结 | 可共享只读 | 配置、缓存、跨任务共享 |
| L4 | `managed` | 可选 GC | 托管 | 脚本模式、复杂图结构 |

region 语法（P0）：

```zulon
fn handle_req(req: Request) -> Response ! Error {
  region req_scope {
    let parsed = parse(req);
    let resp = route(parsed)?;
    return resp;
  }
}
```

#### 4.2.1 冻结（freeze / `share(x)`）语义（P0）

- `share(x)` 将 `owned T` 冻结并提升为 `shared T`，前提：`T` 可被冻结。
- 冻结是**深度操作**：`T` 内部所有可变字段必须被冻结或拒绝。
- 冻结后：任何路径都不得得到可变别名；否则编译错误 `E-SHARE`。

```zulon
let cfg: shared Config = share(parse_config(bytes)?);
```

分期落地：

- P0：L1/L2/L3
- P2+：L4（与 wasm-gc 等目标的兼容策略见 [9.7](#97-可选托管堆与-wasmgc-兼容策略p2)）

### 4.3 规划性能与内存管理路径

性能基准（以 `yan bench` 验收）：

- `await` ping-pong：单核 ≥ 1,000,000 次切换/秒。
- channel/actor ping-pong：本机 p99 < 1ms。
- Hello World（native, release）：二进制 < 500KB（可按平台调整）。

#### 4.3.1 逃逸分析（EA）与自动推导

推导目标：

1. 不逃逸：优先 L1（栈或标量替换）。
2. 作用域内逃逸：进入 L2（region）。
3. 跨 task/actor：提升至 L3（冻结共享）或 move 的 owned 值。
4. 复杂循环引用：P2+ 可选 L4。

实现策略：

- 前端：基于 MIR 的流敏感 EA（跨闭包捕获、跨 async 状态机字段、跨消息发送点）。
- 中端：基于 SSA 的逃逸点标注与 heapification 决策；必要时启用“动态 heapification”保护推断失效场景的正确性[^optimistic-stack]。

#### 4.3.2 L3 的并发引用计数实现建议

L3 选择引用计数作为 P0 共享机制，但要求：

- 冻结后的对象深度不可变。
- 仅在跨边界时引入 RC；边界内尽量消除 RC 计数更新（通过静态分析/借用区域化）。

在高度并发数据结构场景，可参考“并发即时引用计数”将 SMR 与 RC 结合，以降低内存增长与更新负载下的开销[^circ]。

### 4.4 零成本抽象（编译期消除）

- 泛型：单态化 + 内联 + 去虚函数化。
- `Result`：采用 ABI 友好布局，正常路径保持寄存器返回。
- `shared`：冻结后读路径不加锁。
- effect：handler 低成本跳转；可在 P0 先限制为“浅 handler + 明确 resumable”以简化实现。

---

## 5. 并发模型：无锁优先与结构化并发

### 5.1 并发哲学

- 默认通过任务与消息传递构建并发。
- 共享可变为回退，仅允许在 `unsafe` 或 FFI 边界。

### 5.2 协程与调度：M:N 运行时

P0 运行时目标：

- 栈管理：默认 stackless（async 状态机），对特定场景可选 stackful（P2+）。
- 调度：work-stealing + 协作式抢占点（safe point）插入（`await`、循环回边、I/O 边界、显式 `task::yield`）。
- 取消：协作式检查点（`task::check_cancelled()`）与结构化传播。

### 5.3 结构化并发：语法与语义

#### 5.3.1 `task::scope`

```zulon
fn parallel_map(xs: List<i32>) -> List<i32> ! Error {
  return task::scope {
    let h1 = task::spawn { work(xs[0..xs.len()/2]) };
    let h2 = task::spawn { work(xs[xs.len()/2..]) };
    let a = h1.await?;
    let b = h2.await?;
    return a + b;
  };
}
```

语义（P0）：

- 子任务不允许逃逸出 scope。
- scope 退出前必须 join 所有子任务。
- `?` 在 scope 内传播时触发取消传播：未完成子任务收到取消信号。

#### 5.3.2 取消与资源释放

- 取消是可观察副作用：可被 handler 捕获或向上传播。
- 析构保证：无论正常返回、`?` 提前返回、取消、`panic`，L1/L2 资源均确定性释放。

#### 5.3.3 非结构化任务（显式 `spawn_detached`）

为避免“野任务”与不可控泄漏，非结构化任务必须显式：

```zulon
let h = task::spawn_detached { background_work() };
```

规则（P0）：

- 默认不提供隐式 detached；`task::spawn` 在 `task::scope` 内创建结构化子任务。
- `spawn_detached` 必须返回 `TaskHandle`，且必须被消费（`await`/`detach`/`cancel`）；否则 `yan vet` 产生警告 `W-TASK-LEAK`。

### 5.4 通道与 select

P0 提供：

- `chan::bounded<T>(n)`：背压。
- `chan::unbounded<T>()`：无界。
- `select`：多路复用，保证公平性策略可配置（随机/轮询）。

### 5.5 Actor：隔离可变状态

```zulon
actor Counter {
  state n: i64

  fn inc(self, by: i64) -> i64 {
    self.n = self.n + by;
    return self.n;
  }
}
```

约束：

- actor 内部可变状态不泄漏引用。
- 仅允许消息传递（Move 或 Share 的不可变快照）。

### 5.6 监督与 `panic` 隔离（P0）

目标：把“不可靠任务”变成可控故障域，避免崩溃扩散为全局不确定性。

语义：

- `panic` 只终止当前 task；在 `task::scope` 内，`panic` 将触发 scope 退出并取消其余子任务（等价于“失败即取消”策略）。
- scope 外（根任务）默认策略为：打印诊断并退出进程；可由运行时配置为“监督树”策略（重启、回退、熔断）。
- `actor` 可配置监督策略：当 actor 的处理 loop 发生 `panic`，由其监督者决定重启 actor、停止 actor 或升级错误到上层。

---

## 6. 错误处理与代数效应：统一异常/异步/注入

### 6.1 三类“非正常流”

| 类别 | 机制 | 语义 | 典型场景 |
|---|---|---|---|
| 可恢复错误 | `T ! E` / `Result<T,E>` | 显式处理或 `?` 传播 | I/O、解析 |
| 不可恢复错误 | `panic` | 终止当前 task，由监督策略决定传播/重启 | 越界、断言失败 |
| 可组合控制流 | `effect/handle` | 可恢复非本地控制流 | 重试、注入、生成器 |

### 6.2 `?` 与 `try` 块

```zulon
fn load() -> Config ! Error {
  try {
    let a = read_a()?;
    let b = read_b()?;
    return merge(a, b);
  }
}
```

错误联合（P0）：

- `E1 | E2` 表示错误联合类型（可用于 `T ! (E1 | E2)` 的 `E` 部分）。
- `?` 传播时，若当前函数的 `E` 不能覆盖被传播的错误类型，则要求显式转换（例如 `map_err`/`from`）。

### 6.3 effect 与 async 的统一

原则：

- `await` 是挂起点；effect 是“请求能力”的语义点。
- `await` 与 effect 的组合必须保持可推导：`async fn f() -> T ! E performs IO` 的结果类型显式包含效应与错误。

### 6.4 效应多态

基础路线：

- P0：采用 effects set（例如 `performs IO + Net`）并限制 handler 形态，优先保证可推导与可实现。
- P1：演进到行多态/实例化机制；可借鉴“关联效应”让 trait 实例携带效应签名[^associated-effects]。

### 6.5 示例：重试策略作为 Handler

```zulon
effect Retry {
  fn should_retry(attempt: i32, err: Error) -> bool;
}

fn fetch(u: Url) -> Bytes ! NetError performs Net + Retry {
  let mut attempt = 0;
  loop {
    attempt += 1;
    match do Net::get(u) {
      case Ok(b) => return Ok(b),
      case Err(e) => {
        if do Retry::should_retry(attempt, e.into()) { continue; }
        return Err(e);
      }
    }
  }
}

fn main() ! AppError {
  return handle fetch(url) {
    case Retry::should_retry(a, err) => resume(a < 3),
  };
}
```

实现约束（P0）：

- handler 可实现为显式 continuation/状态机（选择性 CPS）。
- 正常路径不构造回溯栈；只在触发 effect 点进行最小化上下文切换。

---

## 7. 开发体验与诊断系统（含学习模式）

### 7.1 量化开发体验验收指标

- 冷启动编译（中型项目）：< 2s（基线机器）。
- 增量编译：< 100ms（单文件编辑）。
- LSP：补全/跳转/hover < 50ms（缓存命中）。
- `yan test`：并发测试可复现（固定随机种子与调度记录）。
- `yan bench`：输出可机读报告（JSON）与对比回归阈值。

### 7.2 诊断输出规范

- 错误码：稳定且可文档化（`E-SEND/E-SHARE/E-REGION-ESCAPE/E-ACTOR-LEAK/...`）。
- 输出结构：标题 + 位置 + 规则 + 修复建议 +（可选）学习段。

### 7.3 学习模式（Learning Mode）

触发方式：

- `yan build --learn`
- IDE 请求（LSP 扩展）

学习段必须回答：

1. 触发规则背后的安全目标是什么；
2. 为什么默认规则这样设计；
3. 推荐的工程模式是什么；
4. 在性能/可维护性上的权衡。

### 7.4 增量编译与查询式编译器（Query-based）

编译器推荐采用“查询系统（query-based）”组织：

- 解析、名称解析、类型推断、trait 求解、能力/借用检查、effect 推断、IR lowering 都是可缓存查询。
- cache key 以“语义哈希 + 依赖边”表示，支持细粒度增量。

### 7.5 静态分析与审计

- `yan vet`：并发边界（`Send/Share/region escape`）、`unsafe` 审计、`dynamic` 扩散、panic 逃逸等。
- `yan bench`：吞吐/延迟/取消压力的可机读基准报告。

---

## 8. 语法体系与高级特性（函数式、泛型、反射、元编程）

### 8.1 泛型系统：单态化与约束

```zulon
fn map<T, U>(xs: List<T>, f: fn(T) -> U) -> List<U> {
  let mut out = List::new();
  for x in xs { out.push(f(x)); }
  return out;
}
```

### 8.2 函数式能力

- 函数一等：`fn(...) => ...`
- 管道：`|>`
- 列表表达式：`[x*x for x in 1..10 if x%2==0]`
- `@pure`：声明纯函数，供编译器进行更激进的 CSE/向量化（不作为安全边界）。

### 8.3 语法糖（借鉴主流语言但保持正交）

- 属性存取：`obj.field`
- 可空链：`?.` / `??`
- 自动解构：`let (a,b) = f()`
- 尾随闭包：`task::spawn { ... }`

### 8.4 编译期反射与元编程（安全可控）

ZULON 选择“编译期反射 + 限域代码生成”的路线：

- 反射仅在 `comptime` 运行。
- 反射 API 只暴露结构信息（字段、变体、属性、可见性）。
- 生成代码必须通过 AST 构造器 API，禁止拼接字符串注入语法树。

反射 API（草案）：

```zulon
comptime fn derive_json<T>() -> Ast {
  let ty = reflect::type_of<T>();
  let fields = reflect::fields(ty);
  return json::derive_encoder(ty, fields);
}
```

---

## 9. 标准库架构：Core/Std/Ext

### 9.1 分层原则

- `core`：无 OS 依赖；基础类型、内存/并发原语、`Result/Option`。
- `std`：OS 抽象（文件、网络、时间、任务）；跨平台后端适配。
- `ext`：扩展库（HTTP/JSON/加密/数据库等）。

### 9.2 I/O：基于效应的全异步模型

`std::io` 以 effect 表达 I/O 能力，并由运行时 reactor 提供实现：

```zulon
effect Net {
  fn read(fd: Fd, n: usize) -> Bytes ! IoError;
  fn write(fd: Fd, b: Bytes) -> usize ! IoError;
}
```

后端：Linux 优先 io_uring，回退 epoll；macOS 使用 kqueue；Windows 使用 IOCP。

### 9.3 并发工具包：去锁化优先

- `task`：scope/spawn/join/cancel。
- `chan`：bounded/unbounded/select。
- `actor`：邮箱、监督策略、可观测性。

### 9.4 集合：内存分层感知

- `Vec<T>`：small-vector 优化（小容量内联）。
- `Map<K,V>`：为 `shared` 值提供不可变持久化结构（P1）。
- region-aware 容器：可在 `region` 中批量分配并在 scope 退出释放。

### 9.5 集合补充（P0→P1）

- `SmallVec<T, const N>`：小容量内联，超出转堆/region。
- region allocator 注入：容器可显式绑定 allocator（默认由编译器/运行时选择）。

### 9.6 数值与张量（P1→P2）

- `simd`：向量类型族与算子（目标特性驱动）。
- `tensor`：布局、广播、融合与并行策略；P2 支持自动微分与（可选）GPU lower。

### 9.7 可选托管堆与 wasm-gc 兼容策略（P2+）

- `managed` 仅在脚本模式或显式开启时可用。
- Wasm 目标：优先 wasm32-wasi；P2+ 可对接 wasm-gc 与组件模型。
- 跨语言互操作：参考 RichWasm 对“细粒度共享内存互操作”的类型化中间层思路[^richwasm]。

---

## 10. 多目标编译与工具链：`yan` 统一入口

### 10.1 设计目标

- 一次配置，多目标输出。
- 统一依赖与构建缓存。
- 统一测试/基准/诊断口径。

### 10.2 包管理

`yan` 提供：

- `yan init`：初始化项目（生成 `yan.toml` 与目录骨架）。
- `yan add <pkg>`：添加依赖与锁定版本。
- `yan remove <pkg>`：移除依赖。
- `yan update`：更新锁定文件（按策略升级）。
- `yan search <term>`：搜索包。
- `yan publish`：发布包。
- `yan vendor`：依赖落盘与离线构建。
- `yan audit`：依赖漏洞扫描与许可检查。

### 10.3 编译与运行

```bash
yan build [--release] [--target native|wasm|js|jvm|rust]
yan run [path.zu|pkg]
```

### 10.4 测试与基准

```bash
yan test
yan bench
yan vet
yan fmt
yan doc
yan clean
```

### 10.5 分层 IR（多目标核心）

为支撑多目标，编译器采用分层 IR：

1. HIR：语法降糖后、保留类型与 effect 信息。
2. MIR：显式控制流、显式 drop 点、所有权/借用约束清晰。
3. AIR（Abstract IR）：与后端无关的“能力化 IR”，明确 region/share/task/actor 边界。

后端映射：

- Native：AIR → LLVM/或 Cranelift。
- Wasm：AIR → Wasm dialect/或 LLVM wasm 后端。
- JS：AIR → JS（ES2020+，使用 promise/async 映射）。
- JVM：AIR → JVM bytecode（invokedynamic 仅用于必要处）。
- Rust：AIR → Rust AST（尽量映射到 Rust 的 ownership/Result/async）。

### 10.6 入口分流：`start` 与 `main` 的编译语义

- `start`：允许链接解释/JIT/可选 GC；产物可为脚本运行单元或轻量 bundle。
- `main`：必须可在 `no_runtime` 下通过；产物为单一可执行文件或静态链接产物（平台允许时）。

### 10.7 多目标示例

```bash
yan build --release --target native
yan build --release --target wasm
yan build --release --target js
yan build --release --target jvm
yan build --release --target rust
```

### 10.8 测试与基准（P0）

```bash
yan test
yan bench
```

P0 基准至少覆盖（可机读 JSON 输出）：

- `await` ping-pong（task switch ops/s）
- channel ping-pong（吞吐 + p99 延迟）
- actor mailbox 吞吐（含调度/队列）
- effect perform/resume 开销（6.5 类 handler）

---

## 11. 六大领域能力：特性与标准库 API 草案

### 11.1 系统编程

- 语言特性：可控布局、可预测内存、`no_std`。
- API：`core::ptr`、`std::os::*`、`std::ffi`。

### 11.2 Web 开发

- 语言特性：全异步 I/O、effect 中间件、结构化并发请求域。
- API：`std::net::http`、`std::sql`、`ext::json`。

### 11.3 移动与 GUI

- 语言特性：与 Swift/Kotlin/ObjC/Java 互操作的 FFI 绑定生成。
- API：`ext::ui`（跨平台渲染树草案）。

### 11.4 游戏开发

- 语言特性：ECS 友好布局、SIMD 数学、资源热重载（P1）。
- API：`ext::game::ecs`、`ext::game::math`。

### 11.5 嵌入式

- 语言特性：编译期资源计算、MMIO 安全封装。
- API：`core::embedded`。

### 11.6 AI / 科学计算

- 语言特性：原生张量、算子融合、自动微分（P2）。
- API：`ext::ml::tensor`、`ext::ml::autograd`。

---

## 12. 默认控安全：类型安全 + 内存安全 + 访问控制

### 12.1 类型安全（默认空安全）

- `T` 默认非空；`T?` 才可空。
- 禁止隐式类型转换；必须显式 `as` 或 `cast`。
- `match` 必须完备，防止遗漏分支造成隐藏错误。

### 12.2 内存安全与 `unsafe` 边界

默认保证：

- 所有权/借用检查杜绝悬垂引用与 double-free。
- 数组访问默认边界检查。

`unsafe` 的语义：

- `unsafe` 仅用于绕过某些检查，但必须显式标注，并可被 `yan vet` 审计。
- 标准库若暴露 `unsafe` API，必须给出安全前置条件与推荐封装（以文档与 lint 规则形式落地）。

### 12.3 访问控制安全

默认规则：

- 模块成员默认 `private`，需显式 `pub` 才可导出。
- 变量绑定默认不可变；可变需显式 `mut`。
- 跨包可见性需显式声明（`pub(crate)`/`pub(package)` 等级草案）。

能力式访问（Capabilities by Effects）：

- effect handler 是“能力授予点”：没有 `IO` handler，就无法执行 I/O。
- 这使得 sandbox、测试注入、最小权限成为自然模式。

```zulon
fn pure_compute(x: i32) -> i32 { x*x }

fn side_effect() performs IO {
  do IO::write("a.txt", "hi".bytes())?;
}
```

---

## 13. 编译器工程规格：IR、Pass、后端与路线图

### 13.1 前端管线

1. 解析（parser）→ AST  
2. 名称解析（resolver）→ 绑定与可见性图  
3. 类型推断与检查（typeck）→ HIR（带类型与 effect）  
4. 借用/能力检查（capck）→ `owned/local/shared` 与 `Send/Share/Sync` 约束  
5. 降糖（desugar）→ async 状态机、`?`、`|>`、列表表达式  

### 13.2 中端与优化

- MIR：显式 CFG、显式 drop、显式 region 作用域。
- 关键 pass：
  - 逃逸分析（EA）与 heapification 决策
  - 借用/能力检查分期：P0 使用 CFG 数据流（NLL 风格）保证 sound；P1/P2 引入更精细的关系分析框架（Polonius 路线）提升精度
  - 借用范围收缩与 RC 消除
  - effect 内联与 handler 规约
  - 向量化与张量算子融合

### 13.3 后端与多目标

- Native：LLVM/Cranelift
- Wasm：wasi +（P2+）component model
- JS：ES module 输出 + source map
- JVM：class/jar 输出 + 调试信息
- Rust：源码输出（互操作桥与语义验证工具）

### 13.4 MLIR 多层 IR 与数值后端（P2+）

为数值/张量与多目标优化，推荐采用“多层 IR + dialect”架构，并将 `tensor`/并行/向量化在更高层表达后再 lower。该路线与近年的张量语言验证/编译研究趋势一致，可作为 P2+ 的工程方向[^pldi24-papers]。

### 13.5 工具化验收

- `yan bench`：性能基线与回归阈值。
- `yan vet`：并发边界与 unsafe 审计（锁顺序、潜在死锁、跨边界能力违规）。
- `yan fuzz`（P1）：针对 parser/MIR/后端的差分与变异测试。

---

## 14. 附录：语义权衡与前沿参考

### 14.1 关键权衡记录（摘要）

- `shared` 选择 RC（P0）而非 GC：换取可预测延迟与可控资源；P2+ 以 `managed` 兜底。
- effect 系统优先“可实现/可推导”：P0 限制能力集合，P1+ 再扩展到更细粒度与实例化。
- 多目标以 AIR 为核心：避免前端语义分叉；将差异下沉到后端 lowering。
- `local` 永不跨 task/actor：换取简单一致的 region 心智模型；跨并发需求通过 move 或 `share(x)` 显式晋升解决。

### 14.2 前沿参考（含开源/论文）

- Koka：行多态效应类型与可推导的效应系统设计[^koka]。
- Associated Effects（PLDI 2024）：把效应签名作为 type class 的一部分以提升可组合性[^associated-effects]。
- RichWasm（PLDI 2024 / arXiv）：为 Wasm 生态提供“细粒度共享内存互操作”的类型化 IL 思路[^richwasm]。
- Optimistic Stack Allocation & Dynamic Heapification（PLDI 2024）：静态逃逸分析与运行时保护结合的工程路线[^optimistic-stack]。
- Concurrent Immediate Reference Counting（PLDI 2024 / Microsoft Research）：并发下将 SMR 与 RC 结合以改善内存增长与更新性能[^circ]。

### 14.3 为什么选择 effect 而不是传统异常（摘要）

- 传统异常隐藏控制流，降低可推理性并影响优化（尤其是跨 `await`/取消/注入的组合）。
- effect 让“请求能力/恢复策略”显式化，可组合，并能把 I/O、取消、重试、依赖注入统一到同一机制。

[^koka]: Daan Leijen, “Koka: Programming with Row Polymorphic Effect Types”, arXiv:1406.2061. https://arxiv.org/abs/1406.2061
[^associated-effects]: PLDI 2024, “Associated Effects”. https://pldi24.sigplan.org/details/pldi-2024-papers/17/Associated-Effects
[^richwasm]: Zoe Paraskevopoulou et al., “RichWasm: Bringing Safe, Fine-Grained, Shared-Memory Interoperability Down to WebAssembly”, arXiv:2401.08287. https://arxiv.org/abs/2401.08287
[^optimistic-stack]: “Optimistic Stack Allocation and Dynamic Heapification for Managed Runtimes”, PACMPL (PLDI 2024). https://dl.acm.org/doi/10.1145/3656389
[^circ]: Jaehwang Jung et al., “Concurrent Immediate Reference Counting”, Microsoft Research (PACMPL PLDI 2024). https://www.microsoft.com/en-us/research/publication/concurrent-immediate-reference-counting/
[^pldi24-papers]: PLDI 2024 accepted papers list (for related compiler/IR trends context). https://pldi24.sigplan.org/track/pldi-2024-papers
