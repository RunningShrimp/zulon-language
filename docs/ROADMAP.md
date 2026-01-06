# ZULON 语言工程实现路线图：P0/P1/P2 分层规范

**版本**: 3.0-ROADMAP  
**日期**: 2026-01-06  
**状态**: 工程拆分与实现约束

> 本文档将 ZULON 语言设计拆分为三个明确约束的实现阶段，每个阶段有独立的验收标准与开发边界。P0 追求"最小可用"，P1 追求"生产就绪"，P2 追求"专业完整"。

---

## 执行摘要：分层哲学

| 阶段 | 核心目标 | 时间估算 | 团队规模 | 关键交付物 |
|---|---|---|---|---|
| **P0** | 核心语言可编译运行，基础并发安全，可测试 | 4-6 个月 | 5-8 人 | 编译器、运行时、core+std、yan 工具链 |
| **P1** | 生产环境可用，完整生态，性能工具 | 6-8 个月 | 10-15 人 | 完整标准库、FFI、REPL、性能调优 |
| **P2** | 高级特性，领域扩展，专业工具 | 8-12 个月 | 15-20 人 | AI/游戏库、托管堆、复杂调试、Wasm-GC |

---

## P0：核心可用版本（MVP）

### P0.1 范围与边界

**目标**：验证核心语义可实现，编译器与运行时闭环，支持编写并测试基础系统程序。

**明确不做**：
- 不实现宏系统（过程宏、声明宏）。
- 不实现高级 FFI（仅 C 基础调用）。
- 不实现 REPL 与热重载。
- 不实现托管堆（L4）。
- 不实现复杂 IDE 功能（仅基础 LSP 补全）。
- 不实现领域扩展库（AI/游戏/嵌入式高级特性）。

### P0.2 语言特性

#### P0.2.1 类型系统核心
- **基础类型**：`i32`, `i64`, `u8`..`u128`, `f32`, `f64`, `bool`, `char`, `byte`（`u8` 别名）。
- **字面量**：十进制、十六进制、二进制；下划线分隔符；浮点科学计数法。
- **字符串**：`str` 单一类型，UTF-8，字面量默认 `shared str`。
- **可空性**：`T` 非空默认，`T?` 显式可空；`?.` 链式调用，`??` 默认值。
- **强制解包**：`!` 操作符（触发 panic）。
- **数组**：`[T; N]` 定长数组，支持索引与子切片；**数组字面量仅在期望类型上下文中解析**。
- **切片**：`slice<T>`（语法糖 `T[]`），只读视图；`mut slice<T>` 可写视图。
- **元组**：`(T1, T2, ..)`，支持解构与位置访问 `.0`/`.1`。
- **多返回值**：`fn f() -> A, B` 等价于 `-> (A, B)`；`return Ok(x, y)` 自动组包。
- **权限修饰符**：仅保留 `owned`（默认）、`local`、`shared` 语法，**不强制完整实现 region 分配器，但保留编译期检查**。
  - `owned`：默认语义，move。
  - `local`：编译期检查不逃逸出 scope（静态分析实现）。
  - `shared`：**仅实现冻结语法**，运行时 ARC 简化实现（无竞争优化）。
- **并发能力**：`Send` 与 `Share` trait 定义，**自动推导基础场景**（如 `i32` 自动 `Send`），复杂场景可手动标注。
- **泛型**：单态化泛型，`where` 约束，**不支持关联类型与常量泛型**。
- **Trait**：基础 trait 定义与实现；`dyn Trait` 动态分发支持，**仅用于边界**（减少代码膨胀）；**孤儿规则严格禁止豁免**。
- **动态类型**：`dynamic` 类型支持，**仅允许在函数参数与返回**，需显式 `cast<T>` 转换。

#### P0.2.2 错误处理与效应（核心子集）
- **`Result<T, E>` 类型**：`T ! E` 语法糖，`?` 自动传播。
- **`Error` trait**：最小定义 `fn message(self) -> str`，**不要求 `source` 链**。
- **基础效应**：仅实现 `IO`、`Net` 两个内置 effect。
  - `effect IO { fn read(...) -> T ! IoError; }`。
  - `effect Net { fn get(...) -> Bytes ! NetError; }`。
- **效应触发**：`do IO::read(path)` 显式语法。
- **效应处理**：`handle expr { case IO::read(p) => resume(v) }` 基础形态。
  - **Handler 规则**：就近词法优先、同层唯一性检查、编译期报错 `E-HANDLER-AMBIGUOUS`。
  - **默认 Handler**：支持 `use handler IO = std::io::posix;` 语法，**实现简单模块级隐式包装**。
- **效应集合**：`| uses IO + Net` 语法，`performs` 作为别名。
  - **P0 限制**：效应集合仅支持编译期常量，不支持高阶传递（如 `fn map<T, Eff>` 暂不实现）。
- **取消机制**：`task::scope` 内 `?` 传播触发取消信号，**仅实现信号发送，不实现资源中断**。
- **Panic**：终止当前 task，**不实现监督策略**，scope 内 panic 直接向上传播并退出进程。

#### P0.2.3 并发模型（基础结构化）
- **Task**：`task::spawn` 创建子任务（结构化），**必须**在 `task::scope` 内。
- **Scope**：`task::scope { ... }` 保证退出前 join 所有子任务。
  - **P0 限制**：Scope 内子任务**不允许**返回不同类型，必须统一为同一 `Result<T, E>`。
- **Await**：`await` 挂起点，**仅支持在 async 函数内**（async 函数基于状态机去糖）。
- **Detach**：`task::spawn_detached` **必须显式调用**，返回 `TaskHandle`，**不实现 `TaskHandle` 的取消/超时**。
- **Channel**：`chan::bounded<T>(n)` 与 `chan::unbounded<T>()`。
  - 基础 `send`/`recv` 操作，**不支持 `select` 多路复用**。
- **Actor**：基础 actor 定义，`actor Counter { state n: i64 }`，**仅支持单状态字段**，消息传递仅允许 `i32`/`str` 等基础类型。
  - **P0 限制**：Actor **不实现监督策略**，panic 直接终止 actor。
- **取消检查**：`task::check_cancelled()` 返回 `bool`，**不实现 `Cancel` effect**。

#### P0.2.4 语法与开发体验
- **函数**：基础 `fn` 定义，参数类型推断（私有函数），返回类型**必须显式**。
- **变量**：`let`/`let mut`，默认不可变。
- **控制流**：`if`/`else`，`while`，`for ... in ...`（仅支持迭代器），`loop`/`break`/`continue`。
- **模式匹配**：`match` 基础变体匹配，**不支持守卫（guard）**，必须完备。
- **属性**：仅支持 `#[test]` 与 `#[derive(Debug, Clone)]`。
- **管道**：`|>` 操作符实现。
- **字符串插值**：**P0 不做**，字面量仅支持基础拼接。

### P0.3 标准库

#### P0.3.1 Core 层（`core::*`）
- `core::result::{Result, Ok, Err}`
- `core::option::{Option, Some, None}`
- `core::ptr`：**仅提供** `addr_of!`/`addr_of_mut!` 宏，**不提供 pointer arithmetic**。
- `core::mem`：`size_of`/`align_of`，**不提供 `forget`**。
- `core::slice`：基础切片操作 `len`/`is_empty`/`get`。
- `core::array`：**仅提供** `len()` 与 `as_slice()`，**无 `map`/`try_from_slice`**。
- `core::marker`：`Send`/`Share` trait 定义（空 trait）。
- `core::convert`：`From`/`Into` 基础实现（仅 `i32`↔`i64` 等安全转换）。
- `core::fmt`：**仅实现** `Debug` trait 的 `{:?}` 打印。

#### P0.3.2 Std 层（`std::*`）
- `std::io`：**仅实现** `File` 类型与同步读写，`async io` **P0 不做**。
  - `File::open(path)` → `Result<File, IoError>`。
  - `file.read_to_end(&mut buf)`。
- `std::net`：**仅实现** `TcpStream` 与 `TcpListener`，同步阻塞 API。
- `std::time`：`Duration` 与 `Instant`（仅支持 `std::time::now()`）。
- `std::task`：`task::scope`、`task::spawn`、`TaskHandle`（仅 `await`）。
- `std::chan`：`bounded`/`unbounded`，**不支持 select**。
- `std::actor`：**仅实现** 基础 actor 宏，**无监督**。
- `std::thread`：**P0 不做**，所有并发基于 task。

#### P0.3.3 Ext 层（`ext::*`）
- **P0 不实现任何 ext 库**，所有领域扩展延后到 P1。

### P0.4 工具链（`yan`）

#### P0.4.1 构建与包管理
- `yan new <project>`：创建 `yan.toml` 与 `src/main.zl` 模板。
- `yan build`：编译当前包，**仅支持 `native` 目标**。
  - 输出：可执行文件或静态库。
  - 配置：**仅支持 `[profile.debug/release]` 中 `opt-level` 0/1/2**。
- `yan check`：快速类型检查，不生成代码（基于查询缓存）。
- `yan run`：编译并运行 `start` 或 `main`。
  - **P0 限制**：不支持 `--watch` 或热重载。
- `yan test`：运行 `#[test]` 函数，**并行度固定为 1**（无并发测试）。
- `yan clean`：删除 `target/` 目录。
- `yan.toml` 解析：**仅支持** `package.name/version/authors` 与 `dependencies` 数组。

#### P0.4.2 诊断与审计
- `yan vet`：**仅实现** `borrowck`、`send-sync`、`unsafe` 三条规则。
  - `unsafe` 审计：**仅统计** `unsafe` 块数量与行数。
- 错误码：**实现** `E-SEND`、`E-SHARE`、`E-REGION-ESCAPE`、`E-HANDLER-AMBIGUOUS`。
- 学习模式：`--learn` 标志，**为上述 4 个错误提供基础解释**（无外部链接）。
- LSP：**仅支持** 文本同步、诊断推送、补全（基于作用域变量）。
  - **不实现**：跳转、悬停、重构。

### P0.5 编译器工程

#### P0.5.1 IR 与 Pass
- **AST 解析**：手写递归下降，**不支持增量解析**（全文件重解析）。
- **HIR**：保留类型与 effect 签名，**不实现泛型单态化**（泛型函数直接报错 `E-GENERIC-P0`）。
- **MIR**：显式 CFG，**借用检查采用简单作用域模型**（非 NLL），`local` 检查仅基于 AST 逃逸分析。
- **AIR**：定义结构，**P0 后端直接跳过 AIR，从 MIR 翻译到 LLVM IR**。
- **后端**：**仅支持 LLVM 13+**，输出 `native` 可执行文件。
  - **不实现**：Cranelift、Wasm、JS、JVM 后端。

#### P0.5.2 性能与体积
- **逃逸分析**：基础分析，仅识别不逃逸到函数外的变量。
- **优化**：**仅开启** LLVM `-O1`，内联阈值 100。
- **二进制体积**：Hello World **< 1MB**（P0 放宽要求）。
- `await` 切换：**> 100,000 次/秒**（P0 降低要求）。

### P0.6 验收标准

#### P0.6.1 功能正确性
- 能通过 `yan build` 编译并运行 `start` 入口。
- 能编写 100 行以内的并发程序（task scope + channel + actor）。
- 能使用 `handle` 注入 mock IO/Net 并通过测试。
- `yan vet` 能检出 `local` 值逃逸并发边界。

#### P0.6.2 性能基准
- 编译时间：中型项目（100 文件）`< 5s`。
- 增量编译：单文件修改 `< 200ms`。
- LSP 补全延迟 `< 100ms`。
- Task 切换 `> 100k ops/s`。
- 内存分配：Region 模拟分配 `< 10ns/op`。

#### P0.6.3 文档与测试
- 标准库 API 文档覆盖率 `80%`（仅 `core` 与 `std`）。
- P0 阶段测试用例 `> 500` 个，覆盖 `> 70%` 代码路径。
- 提供 3 个示例程序（hello, echo-server, concurrent-crawler）。

---

## P1：生产就绪版本

### P1.1 范围与边界

**目标**：ZULON 可用于生产环境，具备完整工具链、标准库、FFI 与性能调优能力。

**新增特性**：
- 宏系统、编译期反射、完整 FFI。
- REPL、热重载、性能分析工具。
- 完整 effect 多态、Actor 监督、取消机制。
- SIMD、张量基础、领域库雏形。

**明确不做**：
- 不实现托管堆 L4（P2）。
- 不实现复杂 GPU 加速（P2）。
- 不实现形式化验证集成（P2+）。

### P1.2 语言特性增强

#### P1.2.1 类型系统扩展
- **常量泛型**：`[T; N]` 中 `N` 可为泛型参数，`fn foo<const N: usize>()`.
- **关联类型**：`trait Matrix { type Elem; }`.
- **泛型特化**：`impl<T> Display for T where T: Debug` 默认实现，特定类型覆盖。
- **孤儿规则豁免**：`use impls::...` 导入，适配包模式。
- **SIMD 类型**：`v4f32`, `v8i32` 等，编译期目标特性检测。
- **张量类型**：`tensor<T, const R: usize>`，秩为编译期常量。
  - **P1 限制**：仅支持 CPU 计算，**无自动微分**。
- **`dynamic` 增强**：支持 `comptime` 反射 RTTI，调试模式类型信息。
- **字符串插值**：`"Hello, {name}!"`，编译期检查变量。

#### P1.2.2 效应系统完整化
- **效应多态**：`fn map<T, Eff>` 完全实现，编译期效应集合并集/子集判定。
- **高阶效应**：支持 `fn with_retry<T, Eff>(f: fn() -> T performs Eff) -> T performs Eff + Retry`.
- **取消效应**：`effect Cancel { fn is_requested() -> bool; }`，集成到 `task::scope`.
- **超时效应**：`effect Timeout { fn deadline() -> Instant; }`，实现 `task::timeout`.
- **日志效应**：`effect Log { fn info/warn/error(msg: str); }`，标准库提供默认 handler.
- **监督效应**：`effect Supervision { fn on_child_fail(id: TaskId); }`，Actor 监督基础。

#### P1.2.3 并发模型增强
- **Select**：`select { case msg = rx.recv() => ..., default => ... }`，支持超时。
- **Actor 监督**：`actor Worker { supervision: OneForOne; max_restarts: 3; }`。
- **TaskHandle 完整**：`cancel()`/`detach()`/`is_finished()` 实现。
- **线程安全原子操作**：`std::sync::atomic::{AtomicI32, ...}`。
- **无锁数据结构**：`CMap<K, V>` 并发哈希表实现。
- **Stackful 协程**：`task::spawn_stackful`（P1 可选，用于 FFI 回调）。

#### P1.2.4 语法与开发体验
- **模式匹配守卫**：`case n if n > 0 => ...`。
- **`try` 块**：`try { ... } catch E => ...` 完整实现。
- **`defer`**：`defer { ... }` 作用域退出时执行（LIFO）。
- **属性扩展**：`#[inline(always/never)]`、`#[deprecated]`、`#[wasm_bindgen]`。
- **`?` 错误映射**：自动 `From<E>` 转换实现。

### P1.3 标准库扩展

#### P1.3.1 Core 层增强
- `core::ptr`：补充 `add`/`sub`/`offset`（`unsafe`）。
- `core::mem`：补充 `forget`/`replace`/`swap`。
- `core::array`：`map`/`try_from_slice`/`from_fn`。
- `core::simd`：`v2`/`v4`/`v8`/`v16` 类型族定义。
- `core::convert`：`TryFrom`/`TryInto` trait。

#### P1.3.2 Std 层完整化
- `std::io`：**完整异步**基于 io_uring/epoll/kqueue，提供 `AsyncRead`/`AsyncWrite` trait。
- `std::net`：`TcpStream`/`UdpSocket` 异步实现，`std::net::http` 基础 HTTP 客户端。
- `std::fs`：异步文件操作。
- `std::time`：`sleep`/`interval` 异步定时器。
- `std::task`：完整取消、超时、TaskHandle。
- `std::chan`：`select` 多路复用。
- `std::actor`：监督树、重启策略、邮箱监控。
- `std::sync`：`Barrier`/`Semaphore`/`RwLock`（基于 Actor）。

#### P1.3.3 Ext 层雏形
- `ext::serde`：基础 `Serialize`/`Deserialize` trait，JSON 格式支持。
- `ext::regex`：正则表达式引擎（基于有限自动机）。
- `ext::crypto`：Hash（SHA256）、HMAC、基础密码学。
- `ext::test`：丰富断言（`assert_near`/`assert_err_contains`）、基准宏。
- `ext::http`：HTTP 服务器框架（中间件基于 effect）。

### P1.4 工具链增强

#### P1.4.1 REPL 与热重载
- `yan repl`：交互式环境，支持 `:t`/`:doc`/`:load`，**启动时间 < 500ms**。
- 热重载：`yan run --watch --hot-reload`，**状态保持限于 `#[persist]` actor**。
  - **P1 限制**：类型定义更改需重启，函数级热重载支持。

#### P1.4.2 性能与调试工具
- `yan bench`：集成统计（均值/p99/吞吐量）、JSON 输出。
- `yan profile`：采样分析器，生成火焰图。
- `yan run --sanitize={address,thread,leak}`：Sanitizer 集成。
- LLDB/GDB 扩展：**支持 `zulon task backtrace` 与 `zulon actor inspect`**。

#### P1.4.3 LSP 完整
- **跳转定义**：跨模块、跨 crate、FFI。
- **悬停提示**：类型、文档、错误信息。
- **重构**：重命名、提取函数、内联变量。
- **代码操作**：快速修复（Quick Fix）自动应用。
- **LSP 延迟**：补全/跳转 `< 50ms`。

#### P1.4.4 包管理进化
- **工作区**：`[workspace]` 多 crate 支持，统一依赖解析。
- **语义化版本**：`^`, `~`, `=` 版本约束。
- **依赖覆盖**：`[patch]` 与 `[replace]`。
- **私有注册表**：`[registries]` 配置，支持认证。
- **构建脚本**：`build.zl` 编译期代码生成。

### P1.5 编译器工程

#### P1.5.1 IR 优化
- **HIR 增量**：基于查询缓存，单文件编辑不重新解析依赖文件。
- **MIR 优化**：NLL 借用检查、region 生命周期分析。
- **AIR 实现**：正式引入，从 MIR 降维，明确 task/actor 边界。
- **LIR**：引入寄存器分配前优化，指令选择框架。

#### P1.5.2 后端扩展
- **后端切换**：除 LLVM 外，**支持 Cranelift**（用于调试构建，提升编译速度）。
- **Wasm 后端**：**AIR -> Wasm (WASI)** 基础实现，输出 `.wasm` 可运行。
- **JS 后端**：**AIR -> ES2020** 源码，**P1 限制**：仅支持同步子集，异步映射为 Promise。
- **交叉编译**：支持 `x86_64`/`aarch64`/`riscv64` Linux/macOS/Windows。

#### P1.5.3 性能目标
- **逃逸分析**：识别 `local` 提升 `L2`，`shared` 提升 `L3`。
- **ARC 消除**：静态分析消除冗余 retain/release。
- **Effect 内联**：Handler 跳转去虚拟化。
- **二进制体积**：Hello World **< 500KB**（release, stripped）。
- **Task 切换**：`> 1M ops/s`。
- **编译速度**：中型项目 `< 2s`，增量 `< 100ms`。

### P1.6 验收标准

#### P1.6.1 功能可用性
- 实现并发布 3 个真实示例（Web 服务器、CLI 工具、数据处理器）。
- 标准库 API 文档覆盖率 `95%`。
- 通过 `ext::serde` 实现 JSON 序列化/反序列化。
- 支持 C/Rust 基础 FFI 调用与回调。

#### P1.6.2 性能与工具
- `yan repl` 启动 `< 500ms`，求值 `< 50ms`。
- 热重载状态保持 `< 200ms` 延迟。
- LSP 所有操作 `< 50ms`。
- 基准测试工具可检测 `5%` 性能回归。
- Sanitizer 可检出 `90%` 内存安全 bug。

#### P1.6.3 社区与生态
- 发布包注册表 `crates.zulon-lang.org`（公开测试版）。
- 提供 VS Code 官方插件。
- 发布语言规范文档（中英文）。
- 实现 RFC 流程，接受社区提案。

---

## P2：高级完整版本

### P2.1 范围与边界

**目标**：ZULON 成为专业领域首选语言，具备高级元编程、领域扩展与前沿技术集成。

**新增特性**：
- 托管堆 L4 与 Wasm-GC 集成。
- 自动微分、GPU 代码生成。
- 形式化验证插件、模型检查。
- 分布式 actor、跨节点消息传递。
- 高级 IDE：语义搜索、依赖可视化、AI 辅助。

### P2.2 高级语言特性

#### P2.2.1 类型系统前沿
- **泛型常量泛型**：`fn foo<const N: usize, const M: usize>()`，支持常量表达式。
- **泛型特化**：完整特化优先级与重叠检查。
- **Type Families**：类型级函数，用于张量 shape 计算。
- **子结构类型**：线性类型子集，`@must_consume` 标记。
- **类型状态**：编译期状态机验证（如文件句柄的 Open/Close）。
- **依赖类型（P2+）**：条件性实现，用于形式化验证边界。

#### P2.2.2 效应系统高级
- **效应多态行变体**：类似 Koka 的 row polymorphism，支持 `Eff - {IO}` 移除效应。
- **效应实例化**：`effect Async<T> { fn await(f: Future<T>) -> T; }`。
- **资源效应**：`effect Resource<T> { fn acquire() -> T; fn release(t: T); }`，编译析构保证。
- **事务效应**：`effect Transaction { fn commit(); fn rollback(); }`，集成到 actor 状态机。

#### P2.2.3 并发与分布式
- **分布式 Actor**：`actor DistCounter { location: Distributed; }`，跨节点透明消息传递。
- **CRDT 库**：内置 `GCounter`、`PNCounter`、`GSet`，支持最终一致性。
- **分布式调度**：跨 work-stealing，全局任务队列。
- **硬实时任务**：`task::spawn_realtime<F>(priority: u8)`，EDF 调度。
- **内存序模型**：`std::sync::atomic` 补充 `SeqCst`/`Acquire`/`Release` 详细控制。

#### P2.2.4 编译期元编程完整
- **过程宏稳定**：`proc_macro` crate 稳定 API，支持属性/函数/派生宏。
- **编译期虚拟机**：`comptime` 函数图灵完备，支持循环与条件。
- **AST 操作**：完整 `quote!`/`syn` 风格 AST 构造与模式匹配。
- **编译期 I/O**：`comptime` 限制文件读取（仅 `include_str!`/`include_bytes!`）。
- **泛型反射**：`reflect::type_of::<T>()` 在单态化后返回完整类型信息。

### P2.3 高级标准库

#### P2.3.1 AI / 科学计算库（`ext::ml::*`）
- **张量运算**：自动微分（Reverse-mode AD），`tensor::grad()`。
- **GPU 后端**：CUDA、Metal、Vulkan 代码生成（基于 AIR -> GPU IR）。
- **神经网络层**：`Linear`/`Conv2d`/`Transformer`，编译期 shape 检查。
- **数据加载**：`Dataset<T>` 并行预处理，`DataLoader` 支持 actor 化。

#### P2.3.2 游戏开发库（`ext::game::*`）
- **ECS 框架**：`World`/`Query`/`System`，Archetype 存储，编译期组件反射。
- **渲染图**：`RenderGraph`，节点自动排序与资源生命周期管理。
- **热重载资源**：Shader、纹理、模型，LRU 缓存。
- **物理集成**：FFI 绑定 Box2D/PhysX。

#### P2.3.3 嵌入式库（`ext::embedded::*`）
- **MMIO 安全**：`#[mmio(base = 0x4000_0000)]` 自动生成 volatile 读写。
- **中断处理**：`#[interrupt]` 属性，自动保存/恢复上下文。
- **裸机运行时**：`#![no_std]` 完整支持，`core` 作为唯一依赖。
- **编译期资源**：`const` heapless `Vec`/`Map`。

#### P2.3.4 移动与 GUI（`ext::ui::*`）
- **声明式 UI**：`view! { Button { on_click: || ... } }`，编译期检查事件处理器。
- **跨平台后端**：SwiftUI（macOS/iOS）、Jetpack Compose（Android）、Qt（Linux/Windows）。
- **FFI 代码生成**：自动从 ZULON 生成 Swift/Kotlin/ObjC 绑定。

### P2.4 前沿工具链

#### P2.4.1 托管堆与垃圾回收
- **L4 层级**：`managed T` 类型标记，**可选 GC 区域**。
  - GC 算法：分代 GC（年轻代/老年代），STW 时间 `< 10ms`。
  - 与 L1/L2/L3 互通：`managed` 值可持有 `shared` 引用，反之禁止。
- **Wasm-GC 集成**：AIR -> Wasm-GC 指令，支持 `struct`/`array` Wasm 类型。
- **跨语言 GC**：与 JavaScript GC 集成（宿主环境共享堆）。

#### P2.4.2 高级调试与分析
- **Time-Travel Debugging**：记录任务调度与内存变更，可回放。
- **并发缺陷检测**：基于模型检查的竞态检测（`yan vet --model-check`）。
- **内存可视化**：`yan heapviz`，显示 region/ARC/GC 布局。
- **性能预测**：`yan predict --input workload.json`，基于静态分析预测性能瓶颈。

#### P2.4.3 AI 辅助开发
- **GitHub Copilot 集成**：ZULON 特定提示工程。
- **代码自动生成**：基于注释生成 effect handler mock 实现。
- **智能修复**：利用 LLM 解释复杂错误（如生命周期冲突）。
- **依赖安全**：AI 检测恶意 crate，自动审计。

#### P2.4.4 形式化验证
- **验证插件**：`yan verify --prover=z3`，验证函数契约。
- **Effect 契约**：`effect IO { fn read(p: Path) -> Bytes ! IoError; } @ensures |b| > 0;`
- **Actor 模型检查**：验证 actor 无死锁、无消息丢失。

### P2.5 编译器工程

#### P2.5.1 MLIR 集成
- **多层 dialect**：`zulon_tensor`, `zulon_actor`, `zulon_effect`。
- **Lowering 路径**：HIR -> MLIR -> LLVM/GPU/Wasm。
- **算子融合**：在 MLIR 层自动融合张量算子，生成融合内核。

#### P2.5.2 后端完整
- **JVM 后端**：AIR -> JVM bytecode，支持 `invokedynamic` 与 `MethodHandle`。
- **Rust 后端**：AIR -> Rust 源码，用于语义验证与 FFI 桥接。
- **Native AOT**：支持链接时优化（LTO）、PGO、BOLT 后优化。
- **二进制体积**：Hello World **< 200KB**（release, LTO, stripped）。

### P2.6 验收标准

#### P2.6.1 领域能力
- 实现一个可训练 MNIST 的 NN 库（`ext::ml`）。
- 实现一个 3D 旋转立方体 demo（`ext::game`）。
- 在裸机 RISC-V 上运行 RTOS 示例（`ext::embedded`）。
- 实现一个简单的跨平台 Todo App（`ext::ui`）。

#### P2.6.2 工具链专业度
- 时间旅行调试支持 10 万事件回放，延迟 `< 2x` 实时。
- 模型检查检测出 90% 以上并发 bug。
- Wasm-GC 模块体积比 Wasm MVP 减少 40%。
- AI 辅助修复建议采纳率 `> 60%`。

#### P2.6.3 社区生态
- 公开 crate 数量 `> 1000`。
- 企业采用案例 `> 5` 家。
- 举办年度 ZULON 大会。
- 通过 ISO 语言标准化提案。

---

## 附录：P0/P1/P2 快速参考表

| 特性 | P0 (MVP) | P1 (生产) | P2 (专业) |
|---|---|---|---|
| **类型系统** | 基础类型 + 泛型（无关联） | 关联类型、常量泛型、SIMD | 子结构、类型状态、依赖类型 |
| **内存管理** | L1/L2/L3，基础 ARC | Region 优化、ARC 消除 | L4 托管堆、Wasm-GC |
| **并发** | Task/Scope/Channel/Actor 基础 | Select/Cancel/Supervision | 分布式 Actor、CRDT、硬实时 |
| **效应** | IO/Net 基础 | 多态、Timeout/Log | Row poly、Resource/Transaction |
| **标准库** | core + 简化 std | 完整 std + ext 雏形 | AI/Game/Embedded 领域库 |
| **工具链** | yan build/test/vet，基础 LSP | REPL/热重载/调试器/分析器 | 时间旅行/模型检查/AI 辅助 |
| **FFI** | 仅 C 基础调用 | C/Rust/JS 完整 | 自动生成绑定、跨语言 GC |
| **宏** | 无 | 声明宏 + 过程宏 | 稳定 API、编译期 VM |
| **后端** | LLVM native | +Cranelift/Wasm/JS | +JVM/Rust/GPU |
| **性能** | 基础逃逸分析 | EA + LTO + PGO | MLIR 融合、BOLT 优化 |
| **示例** | 3 个基础示例 | 真实 Web/CLI 项目 | MNIST/3D 游戏/RTOS |

**工程建议**：
- P0 团队：编译器 3-4 人，运行时 2 人，工具 2 人。
- P1 团队：编译器 5-6 人（优化），标准库 4-5 人，工具 3-4 人。
- P2 团队：领域专家 3-4 人（AI/游戏/嵌入式），前沿研究 3-4 人（验证/GC/GPU）。

---

**实现路径**：
1. **P0**：3 个月开发核心编译器，2 个月开发运行时与工具链，1 个月集成测试与文档。
2. **P1**：P0 发布后并行开发，4 个月增强语言特性，3 个月完善工具链与标准库。
3. **P2**：P1 稳定后，6 个月领域库开发，4 个月前沿特性研究与落地。

本路线图确保每个阶段都有独立交付价值，同时避免过度设计。P0 验证核心语义可行性，P1 构建生产生态，P2 探索专业领域与前沿技术。