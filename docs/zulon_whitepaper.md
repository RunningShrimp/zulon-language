# ZULON 语言白皮书（Whitepaper）

**白皮书版本**: 1.0  
**基于设计规范**: ZULON 语言集成式设计与增强规划（工程可执行版）v2.2  
**日期**: 2026-01-05  
**作者**: Zulon Language Design Team  

> 本白皮书面向技术决策者、语言使用者与生态贡献者：用更易读的方式阐述 ZULON 的设计目标、核心语义与工程落地路线。
> 
> 说明：本白皮书不引入新设定；所有关键承诺以 v2.2 设计规范为准。

---

## 摘要

ZULON 是一门以“默认安全、可预期并发、可组合副作用”为核心目标的现代系统语言。它希望在不牺牲系统级性能的前提下，把以下痛点变成**默认路径**：

- **空值安全**：空值不属于 `T`，仅属于 `T?`。
- **内存安全**：默认杜绝悬垂指针、double-free、use-after-free。
- **并发安全**：跨并发边界的移动/共享由类型系统与编译器验证。
- **可组合控制流**：错误、取消、I/O、重试、依赖注入以“代数效应（effects）”统一表达。
- **可交付工具链**：`yan` 统一依赖、构建、运行、测试、基准、静态审计。

ZULON 采用“统一内存与并发架构（UMCA）”将内存管理分层为 L1/L2/L3/L4：默认以确定性释放（栈/区域/冻结共享）为主，脚本场景可选托管堆（P2+）。并发采用结构化并发与 actor 隔离，避免共享可变状态成为默认；非结构化并发必须显式并受 lint 约束。

---

## 1. 背景与动机

现代软件系统普遍同时面对三类复杂性：

1. **资源复杂性**：低延迟、可预测释放、跨平台多目标（native/wasm/js/jvm 等）。
2. **并发复杂性**：多核并行、异步 I/O、取消/超时、故障隔离与可观测。
3. **控制流复杂性**：错误传播、重试策略、依赖注入与测试替身、可控的“非本地返回”。

传统路径往往在这些维度之间做割裂式取舍：

- 高性能系统语言常把安全与并发正确性交给开发者手工维护。
- 高层运行时语言通过 GC/异常/动态性提升开发效率，但付出可预测性与跨边界正确性的代价。

ZULON 的设计目标是：**把“正确并发 + 内存安全 + 可预期错误处理”变成默认心智模型**，同时提供脚本式迭代入口与多目标生态连接。

---

## 2. 设计目标与非目标

### 2.1 可量化的工程目标

ZULON 的核心承诺以工程验收口径表达（白皮书列出关键项，完整清单见设计规范）：

- 默认空安全：`T` 非空、`T?` 可空。
- 默认内存安全：无悬垂指针、无 double-free、无 use-after-free。
- 默认并发安全：跨 `task/actor` 边界必须满足 `Send/Sync/Share` 约束。
- 多目标可行：同一前端语义可 lower 到 Native/Wasm/JS/JVM/Rust 源码。

### 2.2 五大统一（The Power of 5）

ZULON 将语言核心叙事收敛为五个统一概念：

1. **统一所有权**：默认 `owned`；逃逸/并发驱动推导为 `local` 或 `shared`。
2. **统一任务**：所有异步在 `task` 上运行，取消/超时/错误传播有明确语义。
3. **统一隔离**：共享可变默认禁止；可变状态推荐封装在 `actor` 内。
4. **统一副作用**：错误、取消、I/O、注入统一为类型化副作用（effects）。
5. **统一工具链**：`yan` 一体化管理依赖、构建、测试、基准与诊断。

### 2.3 非目标（P0/P1 约束）

为保证可交付性，P0/P1 明确不做：

- 不以“全能 GC 语言”作为默认路线：P0 仅要求确定性内存（栈/区域/共享冻结）。
- 不把“锁”作为默认并发模型：锁仅在底层库/FFI/`unsafe` 边界内允许。
- 不引入“无限语法糖”：核心语法保持小而正交，高级能力优先通过库与编译期反射实现。

---

## 3. 语言入口与工程形态

ZULON 用同一语言支持脚本式与系统式编程：

- **脚本模式入口**：`start`（允许依赖运行时能力，例如可选 GC、动态模块加载、热重载）。
- **系统模式入口**：`main`（要求在 `no_runtime` 配置下可用，产物偏向单一可执行文件/静态链接）。

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

## 4. 类型系统：安全与并发的基石

### 4.1 推断先行与 API 可读性

- 局部变量、闭包参数、私有函数参数允许推断。
- 公共 API（`pub`）默认要求显式类型，保证稳定与可读。

### 4.2 默认空安全：`T` 与 `T?`

- `T` 默认非空。
- `T?` 显式可空。
- `null` 仅属于 `T?`。

配套语法：

- `?.` 可空链：将 `T?` 链式传播为 `U?`。
- `??` 默认值：为 `T?` 提供兜底，结果为非空类型。

```zulon
fn zip(user: User?) -> String {
  return user?.address?.zip ?? "000000";
}
```

### 4.3 预期错误类型：`T ! E`

`T ! E` 是 `Result<T, E>` 的语法糖，用于显式表达可恢复错误：

```zulon
fn read_text(p: Path) -> String ! IoError
```

同时支持错误联合（P0）：`E1 | E2`。

### 4.4 ADT 与模式匹配

ZULON 以 ADT（代数数据类型）和完备性检查降低遗漏分支的风险：

```zulon
enum Option<T> { Some(T), None }

fn len_or_zero(x: String?) -> usize {
  match x {
    case null => 0,
    case s => s.len(),
  }
}
```

### 4.5 渐进式动态：`dynamic`

ZULON 允许局部 `dynamic`，但必须是边界化的：

- `dynamic` 只能出现在显式标注位置。
- 从 `dynamic` 回到静态类型必须通过 `cast` 或模式匹配。
- `yan vet` 对 `dynamic` 传播出包边界给出告警（可配置为错误）。

```zulon
fn from_json(x: dynamic) -> User ! JsonError {
  let id: i64 = cast<i64>(x.id)?;
  let name: String = cast<String>(x.name)?;
  return User { id, name };
}
```

### 4.6 Trait、多态与泛型

- 泛型默认单态化（零成本抽象路径）。
- `where` 约束、关联类型、常量泛型用于表达复杂约束。
- 边界处可用 `dyn Trait` 做动态分发以控制代码膨胀。

```zulon
fn add<T>(a: T, b: T) -> T
where T: Add<Output = T> {
  return a + b;
}
```

---

## 5. 并发与所有权：`owned/local/shared` + `Send/Sync/Share`

### 5.1 权限修饰符（语义视图）

- `owned`：默认；唯一所有者，move 后原绑定失效。
- `local`：区域/作用域绑定；不可逃逸、不可跨并发边界。
- `shared`：冻结后的共享快照；深度不可变，可跨并发边界共享。

### 5.2 并发能力（类型约束）

- `Send`：值可跨 `task/actor` 边界移动。
- `Sync`：值可被多个任务并发读取且保持内存安全。
- `Share`：值可跨边界共享别名，且必须深度不可变（冻结）。

闭包捕获与跨任务规则（P0 摘要）：

- 同一 task 内闭包捕获：允许借用捕获（视为 `local`，受 region/scope 约束）。
- 跨 task 捕获：禁止捕获 `local` 借用；只能捕获满足 `Send` 的 move 值，或 `Share` 的冻结值。

```zulon
fn demo(buf: Bytes) {
  task::scope {
    let frozen = share(buf);
    let h = task::spawn { use_bytes(frozen) };
    h.await;
  };
}
```

---

## 6. UMCA：统一内存与并发架构

UMCA 由两部分组成：

- **UMMA**（统一内存管理）：栈/区域/共享冻结/可选托管堆。
- **UCMA**（统一并发管理）：结构化并发/actor/共享可变回退。

### 6.1 内存分层（L1/L2/L3/L4）

| 层级 | 权限视图 | 机制 | 跨并发语义 | 典型场景 |
|---|---|---|---|---|
| L1 | `owned` | 栈/标量替换 | Move | 局部与临时 |
| L2 | `local` | region bump 分配 | 作用域绑定 | 请求上下文、帧数据 |
| L3 | `shared` | 引用计数 + 冻结 | 可共享只读 | 配置、缓存、跨任务共享 |
| L4 | `managed` | 可选 GC（P2+） | 托管 | 脚本模式、复杂图结构 |

### 6.2 region（L2）

region 是 P0 的核心语义结构：将一批分配绑定到作用域，并在退出时确定性释放。

```zulon
fn handle_req(req: Request) -> Response ! Error {
  region req_scope {
    let parsed = parse(req);
    let resp = route(parsed)?;
    return resp;
  }
}
```

### 6.3 冻结与 `share(x)`（L3，P0）

- `share(x)` 将 `owned T` 冻结并提升为 `shared T`，前提是 `T` 可被冻结。
- 冻结是**深度操作**：若内部存在不可冻结可变字段，必须拒绝并报错。
- 冻结后任何路径都不得获得可变别名。

```zulon
let cfg: shared Config = share(parse_config(bytes)?);
```

### 6.4 逃逸分析（EA）驱动的自动推导

推导目标（摘要）：

1. 不逃逸：优先 L1（栈/标量替换）。
2. 作用域内逃逸：进入 L2（region）。
3. 跨 task/actor：提升至 L3（冻结共享）或 move 的 owned 值。
4. 复杂循环引用：P2+ 可选 L4。

---

## 7. 并发模型：结构化并发 + Actor 隔离

### 7.1 结构化并发：`task::scope`

`task::scope` 将子任务生命周期绑定到作用域，确保退出前 join，失败触发取消传播。

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

语义要点（P0）：

- 子任务不允许逃逸出 scope。
- scope 退出前必须 join 所有子任务。
- `?` 在 scope 内提前返回触发取消传播。

### 7.2 非结构化任务：显式 `spawn_detached`

为了避免“野任务”与资源泄漏：

- `task::spawn` 默认仅用于结构化子任务。
- `spawn_detached` 必须显式调用，并返回必须被消费的 `TaskHandle`；否则 `yan vet` 产生 `W-TASK-LEAK`。

```zulon
let h = task::spawn_detached { background_work() };
```

### 7.3 通道与 `select`

P0 提供：

- `chan::bounded<T>(n)`：背压。
- `chan::unbounded<T>()`：无界。
- `select`：多路复用，公平策略可配置。

### 7.4 Actor：封装共享可变

ZULON 推荐把可变状态放入 actor，外界只能通过消息传递交互（Move 或冻结后的 Share）。

```zulon
actor Counter {
  state n: i64

  fn inc(self, by: i64) -> i64 {
    self.n = self.n + by;
    return self.n;
  }
}
```

### 7.5 `panic` 与故障隔离（P0）

- `panic` 只终止当前 task。
- 在 `task::scope` 内，`panic` 触发 scope 退出并取消其余子任务（失败即取消）。
- 根任务默认打印诊断并退出；运行时可配置监督策略（重启/熔断等，设计给出方向）。

---

## 8. 错误处理与代数效应：统一异常/异步/注入

### 8.1 三类非正常流

| 类别 | 机制 | 语义 | 场景 |
|---|---|---|---|
| 可恢复错误 | `T ! E` / `Result<T,E>` | 显式处理或 `?` 传播 | I/O、解析 |
| 不可恢复错误 | `panic` | 终止当前 task，由监督策略决定 | 断言失败、越界 |
| 可组合控制流 | `effect/handle` | 可恢复非本地控制流 | 重试、注入、生成器 |

### 8.2 effect 的基本形态

```zulon
effect FileSystem {
  fn read(path: Path) -> Bytes ! IoError;
}

fn load_config() -> Config ! ConfigError performs FileSystem {
  let data = do FileSystem::read("config.json")?;
  return config::parse(data)?;
}

fn main() ! AppError {
  handle load_config() {
    case FileSystem::read(p) => resume(vfs::read(p)),
  }
}
```

### 8.3 P0 的 effect 多态（把效应当作类型参数）

P0 允许把效应当作高阶函数的参数，使复用不必丢失效应信息：

```zulon
fn map<T, U, E>(xs: List<T>, f: fn(T) -> U performs E) -> List<U> performs E {
  let mut out = List::new();
  for x in xs { out.push(f(x)); }
  return out;
}
```

约束（P0 摘要）：

- `E` 等价于 effects set，编译器需支持并集/子集判定。
- `yan vet` 可对过宽的 `E` 给出建议（在边界收敛 handler）。

### 8.4 示例：重试策略作为 handler

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

实现约束（P0 摘要）：

- handler 可实现为显式 continuation/状态机。
- 正常路径不构造回溯栈；只在触发 effect 点进行最小化上下文切换。

---

## 9. 标准库与生态分层

ZULON 标准库分层：

- `core`：无 OS 依赖；基础类型、并发与内存原语、`Result/Option`。
- `std`：OS 抽象（文件、网络、时间、任务）；跨平台后端适配。
- `ext`：扩展库（HTTP/JSON/加密/数据库等）。

I/O 以 effect 表达能力，运行时提供实现（Linux io_uring 优先，macOS kqueue，Windows IOCP）。

---

## 10. 工具链：`yan` 统一入口

ZULON 以 `yan` 统一工程入口：

- 项目/依赖：`yan init`、`yan add/remove/update/search/publish/vendor/audit`
- 构建/运行：`yan build`、`yan run`
- 工程质量：`yan test`、`yan bench`、`yan vet`、`yan fmt`、`yan doc`

测试与基准（P0）至少覆盖：

- `await` ping-pong（task switch ops/s）
- channel ping-pong（吞吐 + p99 延迟）
- actor mailbox 吞吐
- effect perform/resume 开销

---

## 11. 多目标编译：分层 IR 与后端映射

为支撑多目标，编译器采用分层 IR：

1. **HIR**：降糖后，保留类型与 effect 信息。
2. **MIR**：显式控制流、显式 drop 点、所有权/借用约束清晰。
3. **AIR**（Abstract IR）：与后端无关的“能力化 IR”，明确 region/share/task/actor 边界。

后端映射（摘要）：

- Native：AIR → LLVM 或 Cranelift。
- Wasm：AIR → Wasm 路径（wasi 优先）。
- JS：AIR → ES2020+（async/promise 映射）。
- JVM：AIR → JVM bytecode。
- Rust：AIR → Rust AST（映射到 Rust 的 ownership/Result/async）。

---

## 12. 默认控安全：类型安全 + 内存安全 + 访问控制

### 12.1 `unsafe` 边界

- 默认提供内存安全与数组边界检查。
- `unsafe` 用于绕过检查但必须显式标注，并可被 `yan vet` 审计。
- 标准库暴露 `unsafe` API 时必须给出安全前置条件与推荐封装策略（以文档与 lint 落地）。

### 12.2 能力式访问控制（Capabilities by Effects）

ZULON 把 handler 视作“能力授予点”：没有 `IO` handler，就无法执行 I/O。

这使得 sandbox、测试注入与最小权限模型成为自然工程模式。

---

## 13. 性能与可预测性

ZULON 的性能策略强调“零成本抽象 + 边界付费”：

- 泛型单态化与内联。
- `Result` 使用 ABI 友好布局，正常路径保持低开销。
- `shared` 冻结后读路径无锁。
- effect 的成本集中在触发点；P0 限制 handler 形态以保证可实现与低开销。

设计规范给出 P0 的基准门槛示例（以 `yan bench` 验收）：

- `await` ping-pong：单核 ≥ 1,000,000 次切换/秒。
- channel/actor ping-pong：本机 p99 < 1ms。
- Hello World（native, release）：二进制 < 500KB（可按平台调整）。

---

## 14. 路线图（P0 → P2+）

### P0（可交付内核）

- 默认空安全、`T ! E`、ADT + match 完备性。
- `owned/local/shared` 与 `Send/Sync/Share`。
- L1/L2/L3：栈/region/冻结共享 + RC。
- 结构化并发 `task::scope`、显式 `spawn_detached` 与泄漏 lint。
- effect/handle/resume（受限形态）与 effects set 推导。
- `yan` 基础命令 + `test/bench/vet` 验收骨架。
- 多目标 IR 分层（HIR/MIR/AIR）与至少一个后端落地（native 优先）。

### P1（生态与精度）

- 容器与 allocator 注入增强，更多 lint/诊断与学习模式完善。
- 持久化不可变集合（shared 友好结构）等。

### P2+（拓展能力）

- L4 `managed`（可选 GC）与 wasm-gc 兼容策略。
- 张量/算子融合更完整路线、自动微分与（可选）GPU lowering。
- 可能引入更强的效应多态（行多态/关联效应）、更精细借用分析框架。

---

## 15. 结语

ZULON 的设计不是“拼盘式堆特性”，而是围绕三条主线做系统性统一：

- **资源与内存**：以 UMCA 分层确保可预测释放与跨边界正确性。
- **并发与故障**：以结构化并发与 actor 隔离，把生命周期与取消变成可推理语义。
- **控制流与能力**：以代数效应统一错误、I/O、取消、注入，形成可组合的能力模型。

如果你希望参与生态建设，优先方向通常是：运行时与调度、`yan` 工具链、标准库 I/O 抽象、多目标后端与诊断体验。

---

## 参考

- Koka：行多态效应类型与可推导的效应系统设计。Daan Leijen, “Koka: Programming with Row Polymorphic Effect Types”, arXiv:1406.2061. https://arxiv.org/abs/1406.2061
- Associated Effects（PLDI 2024）：“Associated Effects”. https://pldi24.sigplan.org/details/pldi-2024-papers/17/Associated-Effects
- RichWasm（arXiv:2401.08287）：“RichWasm: Bringing Safe, Fine-Grained, Shared-Memory Interoperability Down to WebAssembly”. https://arxiv.org/abs/2401.08287
- Optimistic Stack Allocation & Dynamic Heapification（PLDI 2024）. https://dl.acm.org/doi/10.1145/3656389
- Concurrent Immediate Reference Counting（Microsoft Research / PLDI 2024）. https://www.microsoft.com/en-us/research/publication/concurrent-immediate-reference-counting/
- PLDI 2024 papers list（多层 IR / 数值后端趋势背景）。https://pldi24.sigplan.org/track/pldi-2024-papers
