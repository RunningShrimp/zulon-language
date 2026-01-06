# ZULON 语言集成式设计规范（深度优化版 4.0）

**版本**: 4.0（深度优化版）  
**日期**: 2026-01-06  
**作者**: Zulon Language Design Team + AI Optimization  
**状态**: 权威设计规范与工程实施蓝图  
**基于**: ZULON_LANGUAGE_INTEGRATED_DESIGN.md 3.0 修订版

> 本文档面向编译器、运行时与标准库工程落地：把愿景转化为可实现的语义、IR 约束、验收口径与 API 草案。本深度优化版整合前沿技术研究成果（PLDI 2024、结构化并发、WebAssembly 集成等），重构类型系统、并发模型与工具链设计，产出具备技术前瞻性、跨平台、多目标编译可行性的权威方案。

---

## 目录

1. [愿景演进到可执行战略](#1-愿景演进到可执行战略)
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
14. [跨语言互操作与FFI](#14-跨语言互操作与ffi)
15. [宏系统与编译期代码生成](#15-宏系统与编译期代码生成)
16. [开发者工具链完整规格](#16-开发者工具链完整规格)
17. [附录：语义权衡与前沿参考](#17-附录语义权衡与前沿参考)

---

## 1. 愿景演进到可执行战略

### 1.1 愿景

ZULON 的目标是在不牺牲系统级性能的前提下，把"正确并发 + 内存安全 + 可预期错误处理"变成默认路径；同时提供脚本式迭代体验与跨平台生态连接能力。

**工程承诺以可量化验收口径表达**：

- **默认空安全**：空值不属于 `T`，仅属于 `T?`。
- **默认内存安全**：无悬垂指针、无 double-free、无 use-after-free。
- **默认并发安全**：跨并发边界的可传输/可共享能力由类型系统与编译器推断/验证。
- **多目标可行**：同一前端语义可落地到 Native/Wasm/JS/JVM/Rust 源码。
- **开发者冷启动到第一个可运行程序 < 30 秒**（含环境安装）。
- **从编程错误到可执行修复建议 < 200ms**（LSP 反馈延迟）。
- **编译期安全检查覆盖率 ≥ 95%**：运行时内存安全错误率 < 0.1%。

### 1.2 五大统一进阶为"五大默认"

原"五大统一"演进为更落地的 **"五大默认"** ，强调"默认即正确"：

1. **默认所有权**：值的默认语义为 `owned`；并发/逃逸驱动自动提升为 `local` 或 `shared`。
2. **默认结构化任务**：所有异步都运行于 `task`；取消、超时、错误传播有明确语义。
3. **默认不可变共享**：共享可变默认禁止；可变状态推荐封装在 `actor` 内部。
4. **默认效应显式**：错误、取消、I/O、注入都以类型化副作用（effects）显式呈现。
5. **默认工具闭环**：`yan` 一体化管理依赖、编译、运行、测试、基准、诊断、调试、部署。

### 1.3 非目标（P0/P1 约束）

为保证可交付性，P0/P1 明确不做：

- 不以"全能 GC 语言"作为默认路线：P0 仅要求确定性内存（栈/区域/共享冻结）。
- 不把"锁"作为默认并发模型：锁仅在底层库/FFI/unsafe 边界内允许。
- 不引入"无限语法糖"：核心语法保持小而正交；高级能力通过库与编译期反射实现。
- 不默认隐式异步运行时：所有异步必须通过 `task::` 或显式 handler 启动。
- 不支持隐式类型转换：所有类型转换必须显式表达。

### 1.4 核心设计洞察

基于对主流编程语言的深度分析和前沿技术趋势研究，ZULON设计遵循以下核心洞察：

#### 1.4.1 性能与安全的平衡

**问题**：C/C++提供极致性能但牺牲安全性；Java/C#提供GC但引入停顿；Rust提供内存安全但学习曲线陡峭。

**ZULON方案**：
- 采用**UMCA（统一内存与并发架构）**：四层内存模型（L1栈/L2区域/L3共享/L4托管）
- 编译期逃逸分析自动选择最优内存层级
- 所有权模型隐喻化：权限修饰符（`owned`/`shared`/`local`）而非Rust式生命周期标注

#### 1.4.2 并发模型的演进

**问题**：线程+锁模型复杂且易错；Go的goroutine缺乏结构化并发；Rust async/await与所有权结合复杂。

**ZULON方案**：
- **结构化并发**作为核心原语（类似Swift TaskGroup）
- Actor模型+消息传递作为默认并发抽象
- 编译期验证并发安全性（无数据竞争）

#### 1.4.3 类型系统的现代化

**问题**：Java类型擦除导致运行时信息丢失；C++模板错误信息不可读；Python动态类型维护成本高。

**ZULON方案**：
- 单态化泛型+类型推断（类似Rust但简化）
- 代数数据类型（ADT）+ 完备性检查
- 渐进式类型：`dynamic`关键字用于边界化动态性

#### 1.4.4 生态系统策略

**问题**：C++包管理碎片化；Rust生态质量参差不齐；npm依赖地狱。

**ZULON方案**：
- 官方主导的统一包管理器`yan`
- 三层标准库架构：Core/Std/Ext
- 严格的包质量审计机制

---

## 2. 语言总览与入口模型（脚本/系统统一）

### 2.1 文件、模块与包

- **文件扩展名**：`.zl`
- **包（package）** ：分发与版本单元；模块（module）是命名空间单元
- **yan.toml** 定义包名、版本、目标、依赖、特性开关

```toml
# yan.toml 示例（优化版）
[package]
name = "myapp"
version = "0.1.0"
edition = "2026"
authors = ["team@example.com"]

[targets]
main = { entry = "src/main.zl", type = "executable" }
lib = { entry = "src/lib.zl", type = "library" }
script = { entry = "src/script.zl", type = "script" }

[dependencies]
std = "^1.0"
serde = { version = "^2.1", features = ["json", "yaml"] }
async = "^1.2"

[dev-dependencies]
test = "^1.0"
mock = "^2.0"

[profile.release]
opt-level = "speed"
lto = true
strip = true

[profile.dev]
opt-level = "debug"
debug = true
```

### 2.2 两种入口：`start` 与 `main`

ZULON 同时支持脚本式与系统式编程，但用同一语言表达：

- **脚本模式入口**：`start`（适合快速运行、解释/JIT、或轻量 AOT）
- **系统模式入口**：`main`（生成可执行文件、无运行时依赖、支持 `--release` 优化）

**规则**：

- 一个包最多存在一个 `start` 与一个 `main`
- `start/main` 在成功路径上返回 `()`；但可以声明类型化错误通道（`! E` 或 `throws E`）
- 若 `start/main` 返回 `Err(e)`，运行时/宿主负责打印错误并执行系统调用 `exit(1)`；返回 `Ok(())` 则正常退出
- `start` 允许依赖运行时能力（例如可选 GC、动态模块加载、热重载）
- `main` 必须可在 `no_runtime` 配置下通过（除非显式开启）
- `start` 支持返回 `i32` 作为退出码，允许更精细的进程状态表达

**示例**：

```zulon
// src/main.zl
// 脚本模式：快速迭代
fn start() throws AppError {
  let cfg = config::load()?;
  println("Starting script mode: {cfg.name}");
  return Ok(());
}

// 系统模式：高性能部署
fn main() throws AppError {
  server::run()?;
  return Ok(());
}
```

### 2.3 REPL 与热重载支持

**REPL 设计**：
- `yan repl` 启动交互式环境
- 支持表达式求值、类型查询 `:t expr`、文档查询 `:doc name`
- REPL 运行在 `start` 模式，允许动态加载和实验效应 handler

**热重载（P1）**：
- `yan run --watch` 监听文件变化，智能重编译变更模块
- 状态保持：通过 `actor` 的持久化状态机制，在重载后恢复
- 效应 handler 可标记为 `hot-reloadable`，实现运行时热替换

---

## 3. 类型系统深度设计规范（安全与并发的基石）

### 3.1 设计哲学

#### 3.1.1 推断先行（Inference First）

- 局部变量、闭包参数、私有函数参数允许推断
- 公共 API（`pub`）默认要求显式类型，保证可读性与稳定性
- 类型别名推断 `type Bytes = List<byte>` 可在模块内传播

#### 3.1.2 默认空安全（Null-Safety by Default）

- 非空类型 `T` 为默认
- 可空类型显式写作 `T?`
- 无 `null` 或 `nil` 关键字，但保留为关键字，供后续扩展

#### 3.1.3 所有权注记隐喻化（Ownership as Capability）

ZULON 以权限修饰符表达能力边界，并由编译器尽量推断，但可以由开发者使用标记，编译器尽力而为进行优化：

- `owned`：独占所有权（默认）
- `shared`：共享不可变（冻结后可跨并发共享）
- `local`：作用域/区域绑定（不可逃逸）

#### 3.1.4 代数效应（Algebraic Effects as Unifier）

ZULON 用 **effect/handler** 把"会触发环境能力的操作"变成显式、可推导、可注入的语义点。

**核心目标**：

- **可读**：代码里看到 `do`，就知道这里会触发副作用请求；看到 `handle`，就知道这里在"授予能力/决定策略"
- **可测**：同一份业务逻辑可在测试中用 handler 注入 fake/mock（不需要全局单例或隐式上下文）
- **可推导**：P0 只做 effects set（集合）推导与检查，避免一上来引入过强的类型系统负担

**P0 正式语法修订（更友好）**：

```zulon
effect IO {
  fn read(path: Path) -> Bytes ! IoError;
  fn write(path: Path, data: Bytes) -> usize ! IoError;
}

// 紧凑写法（推荐用于短函数）
fn run() -> usize ! AppError performs IO + Net {
  let b = do Net::get(url)?;
  return do IO::write("/tmp/out", b)?;
}

// 可读写法（推荐用于对外 API）
fn run()
  -> usize
  ! AppError
  performs IO + Net
{
  // ...
}
```

**关键改进**：

1. **`!` 优先级高于 `performs`**：语法更清晰，`-> T ! E performs Eff` 读作"返回 T 或错误 E，需要能力 Eff"
2. **`do` 操作符可选括号**：当参数简单时允许 `do Net::get url`，提升可读性
3. **`performs` 作为 `uses` 的别名**：语义等价，但 `performs` 更强调"执行时所需"

#### 3.1.5 渐进式动态（Local Gradual Typing）

局部允许 `dynamic`，但动态性必须是边界化的：

- `dynamic` 只能出现在显式标注位置
- 从 `dynamic` 回到静态类型必须通过 `cast`（可失败）或模式匹配
- `yan vet` 对 `dynamic` 传播出包边界给出警告（可配置为错误）
- `dynamic` 值在调试模式下携带运行时类型信息（RTTI），支持类型反射

### 3.2 核心类型架构

#### 3.2.1 基础数值类型

**扩展：更精确的字面量类型推断**：

```zulon
let x = 42;          // 类型为 i32（默认整数类型）
let y: u8 = 255;     // 显式标注
let z = 0xFFu8;      // 后缀标注（类似 Rust）
let f = 3.14f32;     // f32 浮点
let c = 1.0 + 2.0i;  // c64 复数
```

**整数溢出规则增强**：

- 默认：溢出检查（在 debug 模式触发 panic，release 模式返回错误）
- 局部可用 `@wrap` 或 `@unchecked` 关闭（进入可审计路径）
- `@saturating` 饱和溢出（最大值/最小值截断）
- `@widening` 自动扩展类型（如 i32 + i32 -> i64）

#### 3.2.2 字符串、字节与切片

**字符串设计修订**：

- `str`：UTF-8 字符串（不可变）
- `strbuf`：可变字符串缓冲区（类似 Rust 的 `String`）
- 字面量 `"hi"` 默认类型为 `shared str`（可跨任务共享）
- 字符串插值 `"Hello, {name}!"`（编译期检查变量存在性）

```zulon
let name = "Zulon";
let greeting = "Hello, {name}!";  // "Hello, Zulon!"
```

**Bytes 与切片增强**：

- `Bytes`：不可变字节缓冲，支持零拷贝切片：`.slice(start, end)`
- `BytesMut`：可变字节缓冲（P0），支持 `.freeze()` 转为 `Bytes`
- 与 `slice<byte>` 的桥接：
  - `Bytes::as_slice() -> &[byte]`
  - `BytesMut::as_mut_slice() -> &mut [byte]`

**切片语法糖统一**：

- `&[T]`：只读切片（C 兼容）
- `&mut [T]`：可变切片
- `T[]`：作为 `slice<T>` 的别名，仅用于类型注释
- 取子切片：`xs[a..b]`、`xs[a..]`、`xs[..b]`、`xs[..]`

```zulon
fn process(data: &[byte]) -> i32 { /* ... */ }
```

#### 3.2.3 原生向量/张量类型（面向 SIMD 与 AI/科学计算）

**向量类型族增强**：

```zulon
type Vec4f = v4f32;
type Vec8i = v8i32;

fn simd_add(a: v4f32, b: v4f32) -> v4f32 {
  return a + b;  // 映射到 SIMD 指令
}
```

**张量类型设计**：

```zulon
type Tensor2D<T> = tensor<T, 2>;
type Tensor3D<T> = tensor<T, 3>;

let t: Tensor2D<f32> = tensor::from_shape([1024, 768]);
let slice: Tensor2D<f32> = t.slice([0..512, 0..512]);  // 零拷贝视图
```

**编译器承诺**：

- 对 `vN*` 的算术映射到 SIMD 指令集（SSE/AVX/NEON/RVV），失败则回退标量
- 对 `tensor` 的 element-wise 操作默认向量化；对 reduction 使用并行/向量化联合优化
- 张量算子自动融合（类似 XLA），通过 effect handler 配置融合策略

#### 3.2.4 ADT（代数数据类型）与模式匹配完备性

**模式匹配增强**：

```zulon
enum Result<T, E> {
  Ok(T),
  Err(E),
}

enum Option<T> {
  Some(T),
  None,
}

// 守卫模式（P0）
fn grade(score: i32) -> str {
  match score {
    case n if n >= 90 => "A",
    case n if n >= 80 => "B",
    case n if n >= 60 => "C",
    case _ => "F",
  }
}

// 嵌套解构
fn handle(result: Result<Option<i32>, Error>) {
  match result {
    case Ok(Some(x)) => println("Got {x}"),
    case Ok(None) => println("Empty"),
    case Err(e) => println("Error: {e}"),
  }
}
```

**完备性检查强化**：

- `match` 必须覆盖所有变体；否则编译错误
- 提供 `_` 通配符用于兜底，但 `yan vet` 会警告未显式处理的变体
- 对于 `bool` 和枚举类型，必须显式处理所有情况

#### 3.2.5 显式可空性与安全解包

```zulon
// 安全链式调用
fn get_zip(user: User?) -> str {
  return user?.address?.zip ?? "000000";
}

// 强制解包（风险操作）
fn risky_get(user: User?) -> Address {
  return user!.address;  // 如果 user 为 null，触发 panic
}
```

**空值传播操作符 `??>`**：

```zulon
fn process(data: i32?) -> Result<i32, Error> {
  let x = data ?? 0;  // 提供默认值
  let y = data ??> Error::Empty;  // null 时返回错误
  return Ok(x + y);
}
```

#### 3.2.6 预期错误类型：`T ! E`

```zulon
fn read_text(p: Path) -> str ! IoError {
  // ...
}

// 多错误联合
fn complex_op() -> i32 ! IoError | NetError | ParseError {
  // ...
}
```

**错误类型增强**：

```zulon
trait Error {
  fn message(self) -> str;
  fn source(self) -> Error?;
  fn backtrace(self) -> Backtrace?;  // P0 可选捕获
}

// 自动实现与自定义
enum MyError {
  Io(IoError),
  Custom(str),
}

// 自动实现 Error，若需要自定义：
impl Error for MyError {
  fn message(self) -> str {
    match self {
      case Io(e) => e.message(),
      case Custom(msg) => msg,
    }
  }
}
```

### 3.3 并发与所有权类型注记

#### 3.3.1 并发能力：`Send` / `Sync` / `Share`

**能力语义澄清**：

- `Send`：值可跨 `task/actor` 边界移动（Move）。等价于"所有权转移安全"
- `Sync`：值可被多个任务并发读取且保持内存安全。等价于"共享只读安全"
- `Share`：值可跨边界共享别名，且必须是深度不可变（冻结）。等价于"不可变共享安全"

**自动推导规则增强**：

```zulon
// 对于复合类型，递归检查字段
struct Point { x: i32, y: i32 }  // 自动 Send + Sync + Share

struct Config {
  data: shared Map<str, str>,  // 冻结后自动 Share
}

// 包含裸指针的类型默认不 Send/Sync
struct RawHandle { ptr: *const u8 }  // !Send + !Sync
```

**`MaybeSend` / `MaybeSync` 标记**：

用于条件能力，在泛型中表达"如果 T 是 Send，则此类型也是 Send"：

```zulon
struct Wrapper<T> {
  value: T,
}

// 条件实现
impl<T> Send for Wrapper<T> where T: Send {}
impl<T> Sync for Wrapper<T> where T: Sync {}
```

#### 3.3.2 权限修饰符的语义

**生命周期省略规则**：

```zulon
// 函数参数生命周期省略（类似 Rust）
fn foo(x: &str, y: &str) -> &str { /* 编译器自动分配生命周期 */ }

// 结构体需要显式生命周期（避免隐式复杂）
struct Parser<'a> { input: &'a str }
```

**`pinned` 修饰符**：

防止值被移动，用于自引用结构：

```zulon
struct SelfRef {
  data: [i32; 10],
  ptr: *const i32,  // 指向 data 内部
}

impl SelfRef {
  fn new() -> pinned SelfRef {
    // ...
  }
}
```

### 3.4 特征系统（Trait）：多态与约束

#### 3.4.1 单态化泛型 + where 约束

```zulon
fn add<T>(a: T, b: T) -> T
where T: Add<Output = T> + Copy {
  return a + b;
}

// 多约束
fn process<T>(x: T)
where T: Display + Debug + Serialize {
  // ...
}
```

#### 3.4.2 关联类型与常量泛型

```zulon
trait Matrix {
  const ROWS: usize;
  const COLS: usize;
  type Elem;
  
  fn get(self, row: usize, col: usize) -> Self::Elem;
  fn set(self, row: usize, col: usize, val: Self::Elem);
}

struct Mat3x3 {
  data: [f32; 9],
}

impl Matrix for Mat3x3 {
  const ROWS = 3;
  const COLS = 3;
  type Elem = f32;
  
  fn get(self, row: usize, col: usize) -> f32 {
    return self.data[row * 3 + col];
  }
}
```

#### 3.4.3 动态分发：`dyn Trait`

```zulon
trait Drawable {
  fn draw(self, ctx: &mut Graphics);
}

fn render(entities: &[dyn Drawable]) {
  for e in entities {
    e.draw(&mut ctx);
  }
}
```

**性能提示**：`yan vet` 会警告动态分发热点路径，建议单态化。

#### 3.4.4 孤儿规则与可控豁免

```zulon
// 默认孤儿规则：不能为外部类型实现外部 trait
// 允许在适配包内实现，需显式导入
use impls::serde_for_external::VecSerializer;
```

**`@fundamental` 属性**：

标记基础类型（如 Box、&T），允许更灵活的实现：

```zulon
@fundamental
struct Box<T> { /* ... */ }

// 现在可以为 Box<ExternalType> 实现 ExternalTrait
```

### 3.5 代数效应与渐进式类型

#### 3.5.1 效应声明与触发

```zulon
effect FileSystem {
  fn read(path: Path) -> Bytes ! IoError;
  fn write(path: Path, data: Bytes) -> usize ! IoError;
}

effect Log {
  fn info(msg: str);
  fn warn(msg: str);
  fn error(msg: str);
}
```

**效应组合**：

```zulon
fn complex_task()
  -> Result<Data, Error>
  performs FileSystem + Log + Net
{
  do Log::info("Starting");
  let data = do FileSystem::read("input.txt")?;
  // ...
}
```

#### 3.5.2 处理效应（Handler）

**嵌套 handler**：

```zulon
fn main() ! AppError {
  handle {
    handle {
      let result = do FileSystem::read("config.json")?;
    } {
      case FileSystem::read(p) => resume(mock_fs::read(p)),
    }
  } {
    case Log::info(msg) => println("[INFO] {msg}"),
    case Log::warn(msg) => println("[WARN] {msg}"),
  }
}
```

**effect 作为依赖注入**：

```zulon
trait Repository {
  fn get(self, id: i32) -> User ! DbError;
}

effect Database {
  fn repo() -> dyn Repository;
}

fn get_user(id: i32) -> User ! DbError performs Database {
  let repo = do Database::repo();
  return repo.get(id)?;
}
```

#### 3.5.3 局部 `dynamic`

```zulon
fn parse_json(raw: str) -> Result<User, ParseError> {
  let dyn_data: dynamic = json::parse(raw)?;
  
  let id: i64 = cast<i64>(dyn_data["id"])?;
  let name: str = cast<str>(dyn_data["name"])?;
  
  return Ok(User { id, name });
}
```

**运行时类型检查**：

```zulon
if let Ok(num) = cast<i32>(value) {
  // 类型匹配
}
```

#### 3.5.4 Effect 多态（P0 形态）

```zulon
fn map<T, U, Eff>(xs: List<T>, f: fn(T) -> U performs Eff) -> List<U> performs Eff {
  let mut out = List::new();
  for x in xs {
    out.push(f(x));
  }
  return out;
}
```

**效应推断**：

```zulon
// 编译器自动推断 Eff = Database + Log
let users = map(ids, |id| get_user(id)?);
```

### 3.6 类型检查与错误诊断（含学习模式）

**错误信息结构**：

```text
E-REGION-ESCAPE: `local` value `buf` may outlive its region
 --> src/net.zl:42:17
  |
42|   task::spawn { send(buf) }
  |                 ^^^ `buf` is `local` to region `req_scope`
  |
Rule: `local` values cannot cross `task::spawn` boundaries.
Help:
  - Option 1: Move ownership: `task::spawn { send(move buf) }`
  - Option 2: Freeze and share: `let buf = share(buf)`
  - Option 3: Wrap in actor: `actor BufSender { ... }`
Learn:
  Values tied to a region have a scoped lifetime. Sending them to
  another task could cause use-after-free. ZULON requires explicit
  ownership transfer or immutable sharing for cross-task communication.
  See: https://docs.zulon-lang.org/ownership/regions
```

**诊断工具链**：

- `yan check`：快速语法和类型检查（不生成代码）
- `yan explain E-XXX`：显示错误详细解释和示例
- IDE 集成：LSP 提供实时错误提示和自动修复
- **学习模式**：编译器错误附带"Learn"段落，解释语言设计原理和最佳实践

---

## 4. 核心安全与性能机制（基于类型系统的实现）

### 4.1 统一内存与并发架构（UMCA）

UMCA 由两部分组成：

- **UMMA**：统一内存管理（栈/区域/共享冻结/可选托管堆）
- **UCMA**：统一并发管理（结构化并发/actor/共享可变回退）

### 4.2 内存分层：L1/L2/L3/L4

| 层级 | 权限视图 | 机制 | 跨并发语义 | 典型场景 | 性能特征 |
|:---|:---|:---|:---|:---|:---|
| **L1** | `owned` | 栈/标量替换 | Move | 局部与临时 | 零分配，寄存器级 |
| **L2** | `local` | region bump 分配 | 作用域绑定 | 请求上下文、帧数据 | 亚微秒分配，连续内存 |
| **L3** | `shared` | ARC + 冻结 | 可共享只读 | 配置、缓存、跨任务共享 | 原子操作，读无锁 |
| **L4** | `managed` | 可选 GC | 托管 | 脚本模式、复杂图结构 | 毫秒级延迟，适合原型 |

**Region 语法增强**：

```zulon
fn handle_req(req: Request) -> Response ! Error {
  region req_scope {
    let parser = Parser::new(&req.body);
    let data = parser.parse()?;
    
    // 所有分配在 req_scope 内
    let result = compute(&data)?;
    
    // scope 退出时批量释放
    return result;
  }
}
```

**Region 作为类型参数**：

```zulon
fn process_in_region<R>(data: &R Data) performs Alloc<R> {
  // 在指定 region 内分配
}
```

### 4.3 规划性能与内存管理路径

**性能基准（以 `yan bench` 验收）**：

- `await` ping-pong：单核 ≥ 1,000,000 次切换/秒
- channel/actor ping-pong：本机 p99 < 1ms
- Hello World（native, release）：二进制 < 500KB（可按平台调整）
- 内存分配器压力测试：region 分配吞吐量 > 10M ops/s/thread
- effect 处理延迟：单次 perform/resume < 50ns

### 4.4 零成本抽象（编译期消除）

- 泛型：单态化 + 内联 + 去虚函数化
- `Result`：采用 ABI 友好布局，正常路径保持寄存器返回
- `shared`：冻结后读路径不加锁
- effect：handler 低成本跳转；P0 限制为"浅 handler + 明确 resumable"
- `@inline(always/never)` 属性，结合 PGO 自动优化

### 4.5 内存安全实现机制

#### 4.5.1 所有权系统（Ownership System）

ZULON的所有权系统是其内存安全保证的核心，但相比Rust做了简化：

**核心规则**：
1. 每个值都有一个所有者（owner）
2. 一个值在任一时刻只能有一个所有者
3. 当所有者离开作用域，值被自动释放

**与Rust的区别**：
- 无显式生命周期标注（`'a`）
- 编译器自动推断所有权转移
- 支持"借用检查器"但错误信息更友好

**示例**：

```zulon
fn ownership_demo() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1的所有权转移给s2
    
    // println!("{s1}"); // ❌ 编译错误：s1已失效
    println!("{s2}");   // ✅ 正确：s2是新的所有者
}
```

#### 4.5.2 借用规则（Borrowing Rules）

- 在任意给定时间，要么只能有一个可变引用，要么只能有多个不可变引用
- 引用必须总是有效的

```zulon
fn borrowing_demo() {
    let mut s = String::from("hello");
    
    let r1 = &s;     // 不可变引用
    let r2 = &s;     // 另一个不可变引用
    println!("{r1} and {r2}");
    
    let r3 = &mut s; // 可变引用
    r3.push_str(" world");
}
```

#### 4.5.3 切片类型（Slice Types）

切片是对集合中部分元素的引用，而非所有权：

```zulon
fn slice_demo() {
    let s = String::from("hello world");
    let hello = &s[0..5];   // 字符串切片
    let world = &s[6..11];  // 字符串切片
    
    let a = [1, 2, 3, 4, 5];
    let slice = &a[1..3];   // 数组切片
}
```

---

## 5. 并发模型：无锁优先与结构化并发

### 5.1 并发哲学

- 默认通过任务与消息传递构建并发
- 共享可变为回退，仅允许在 `unsafe` 或 FFI 边界
- 提供 `atomic` 模块用于无锁算法（基于 `core::atomic`）

### 5.2 协程与调度：M:N 运行时

**P0 运行时目标**：

- `task` 是协程抽象：`await` 是挂起点；`task::spawn` 创建可并发运行的协程任务；调度采用 M:N
- 栈管理：默认 stackless（async 状态机），对特定场景可选 stackful（P2+）
- 调度：work-stealing + 协作式抢占点（safe point）插入
- 取消：协作式检查点（`task::check_cancelled()`）与结构化传播
- 调度器可配置：`yan.toml` 中可指定调度策略（轮转、优先级、EDF 实时）

### 5.3 结构化并发：语法与语义

#### 5.3.1 `task::scope`

```zulon
fn parallel_map(xs: List<i32>) -> List<i32> ! Error {
  return task::scope {
    let mid = xs.len() / 2;
    let h1 = task::spawn { work(&xs[0..mid]) };
    let h2 = task::spawn { work(&xs[mid..]) };
    
    let a = h1.await?;
    let b = h2.await?;
    
    return a + b;
  };
}
```

**Scope 错误传播策略**：

```zulon
// 任一任务失败，所有兄弟任务取消
task::scope {
  let h1 = task::spawn { may_fail()? };
  let h2 = task::spawn { long_running() };
  
  // 如果 h1 失败，h2 收到取消信号
  try {
    return h1.await? + h2.await?;
  } catch Error::Cancelled {
    // 清理逻辑
  }
}
```

#### 5.3.2 取消与资源释放

**取消信号传递**：

```zulon
fn background_task() {
  loop {
    task::check_cancelled()?;  // 返回 Err(Cancelled) 或 ()
    do_work();
  }
}
```

**超时机制**：

```zulon
fn with_timeout<T>(duration: Duration, f: fn() -> T) -> Result<T, TimeoutError> {
  return task::timeout(duration, f);
}
```

#### 5.3.3 非结构化任务（显式 `spawn_detached`）

```zulon
let handle = task::spawn_detached { background_work() };

// 必须显式处理
handle.cancel()?;  // 请求取消
handle.detach();   // 完全分离（警告：可能泄漏）
let result = handle.await?;  // 等待完成
```

**TaskHandle API**：

```zulon
trait TaskHandle<T> {
  fn await(self) -> Result<T, Cancelled>;
  fn cancel(self) -> Result<(), CancelError>;
  fn detach(self);
  fn is_finished(&self) -> bool;
}
```

### 5.4 通道与 select

**通道创建**：

```zulon
let (tx, rx) = chan::bounded::<i32>(100);  // 背压
let (tx2, rx2) = chan::unbounded::<str>();  // 无界
```

**Select 语法（修订）**：

```zulon
select {
  case msg = rx.recv() => handle_msg(msg),
  case msg2 = rx2.recv() => handle_str(msg2),
  default => handle_idle(),  // 非阻塞
}

// 带超时
select {
  case msg = rx.recv() => handle_msg(msg),
  case after Duration::seconds(5) => handle_timeout(),
}
```

**Select 公平性**：

- 默认随机选择（避免饥饿）
- 可配置：`select::fair()` 轮询，`select::priority()` 按顺序

### 5.5 Actor：隔离可变状态

**Actor 定义增强**：

```zulon
actor Counter {
  state n: i64
  state history: List<i64>  // 多个状态字段

  fn inc(self, by: i64) -> i64 {
    self.n += by;
    self.history.push(self.n);
    return self.n;
  }
  
  fn get(self) -> i64 {
    return self.n;
  }
}

// 使用
let counter = Counter::new(0);
let handle = counter.inc(5);  // 异步消息
let value = handle.await?;    // 等待结果
```

**Actor 监督策略**：

```zulon
actor Worker {
  supervision: OneForOne  // 一个子 actor 失败只重启它
  max_restarts: 3
  restart_delay: Duration::seconds(1)
  
  // ...
}
```

### 5.6 监督与 panic 隔离（P0）

**Panic 隔离层级**：

- Task panic：仅终止当前 task，scope 内其他 tasks 收到取消信号
- Actor panic：由监督策略决定（重启/停止/升级）
- 全局 panic：根任务默认打印诊断并退出；可配置为"监督树"模式

**Panic 作为效应**：

```zulon
effect Panic {
  fn panic(msg: str) -> !;  // ! 表示发散
}

// 可拦截 panic 用于测试
fn test_panic() {
  handle {
    may_panic();
  } {
    case Panic::panic(msg) => {
      assert_eq(msg, "Expected error");
      resume(unreachable!());  // 或特殊返回值
    }
  }
}
```

---

## 6. 错误处理与代数效应：统一异常/异步/注入

### 6.1 三类"非正常流"

| 类别 | 机制 | 语义 | 典型场景 | 恢复性 |
|:---|:---|:---|:---|:---|
| 可恢复错误 | `T ! E` / `Result<T,E>` | 显式处理或 `?` 传播 | I/O、解析 | 可恢复 |
| 不可恢复错误 | `panic` | 终止当前 task | 越界、断言失败 | 不可恢复 |
| 可组合控制流 | `effect/handle` | 可恢复非本地控制流 | 重试、注入、生成器 | 可恢复 |
| **取消** | **`cancel`** | 协作式取消 | 超时、用户中断 | 可恢复 |

### 6.2 `?` 与 `try` 块

**Try 块作为表达式**：

```zulon
let result = try {
  let a = read_file("a.txt")?;
  let b = read_file("b.txt")?;
  Ok(merge(a, b))
} catch IoError::NotFound => {
  Ok(default_data())
} catch e => {
  Err(e)
};
```

**错误映射简化**：

```zulon
// 自动 From 转换
fn foo() -> Result<T, Error> ! ParseError {
  let data = bar()?;  // ParseError 自动转为 Error（实现 From trait）
}
```

### 6.3 effect 与 async 的统一

**Async 作为语法糖**：

```zulon
// 以下两种等价
async fn fetch_data() -> Result<Data, Error> {
  let resp = await http::get(url)?;
  return resp.json();
}

// 等价于：
fn fetch_data() -> Result<Data, Error> performs Async {
  let resp = do Async::await(http::get(url))?;
  return resp.json();
}
```

**await 作为 effect 操作**：

```zulon
effect Async {
  fn await<T>(future: Future<T>) -> T;
  fn spawn<T>(task: fn() -> T) -> Future<T>;
}
```

### 6.4 错误诊断与调试

### 6.5 Result/Option与操作符的等价性

ZULON保证`Result<T, E>`和`Option<T>`类型与操作符之间的语义等价性：

#### 6.5.1 Result与?操作符

```zulon
// 以下两种形式等价：
fn read_file(path: Path) -> Result<String, IoError> {
  let file = File::open(path)?;           // ?操作符自动转换
  let content = read_to_string(file)?;
  Ok(content)
}

fn read_file_explicit(path: Path) -> Result<String, IoError> {
  let file = match File::open(path) {
    Ok(f) => f,
    Err(e) => return Err(e.into()),      // 显式转换
  };
  let content = match read_to_string(file) {
    Ok(c) => c,
    Err(e) => return Err(e.into()),
  };
  Ok(content)
}
```

**?操作符规则**：
- `expr?`等价于`match expr { Ok(v) => v, Err(e) => return Err(e.into()) }`
- 要求函数返回类型为`Result<T, E>`
- 自动调用`From::from`进行错误类型转换

#### 6.5.2 Option与?操作符

```zulon
// 以下两种形式等价：
fn get_zip(user: User?) -> str? {
  user?.address?.zip
}

fn get_zip_explicit(user: User?) -> str? {
  match user {
    Some(u) => match u.address {
      Some(addr) => addr.zip,
      None => None,
    },
    None => None,
  }
}
```

**?操作符规则**：
- `expr?.field`等价于`match expr { Some(v) => v.field, None => None }`
- 支持链式调用：`a?.b?.c`

#### 6.5.3 ??操作符

```zulon
// 以下两种形式等价：
let name = user?.name ?? "Unknown";

let name = match user?.name {
  Some(n) => n,
  None => "Unknown",
};
```

#### 6.5.4 !操作符（强制解包）

```zulon
// 以下两种形式等价但风险不同：
let value = option_value!;              // 触发panic

let value = match option_value {
  Some(v) => v,
  None => panic!("Unexpected null value"),
};
```

**安全建议**：
- 优先使用`?`和`??`而非`!`
- `!`仅用于确实不可能为null的场景
- 在代码审查中标记所有`!`使用

#### 6.5.5 T ! E语法糖

```zulon
// 以下两种形式等价：
fn read_file() -> str ! IoError {
  // ...
}

fn read_file_desugar() -> Result<str, IoError> {
  // ...
}
```

**转换规则**：
- `T ! E`是`Result<T, E>`的语法糖
- `-> T ! E`等价于`-> Result<T, E>`
- `!`优先级高于`performs`：`-> T ! E performs Eff`

#### 6.5.6 操作符优先级

| 操作符 | 优先级 | 结合性 | 说明 |
|:---|:---:|:---:|:---|
| `!` | 高 | 右结合 | 错误类型标记 |
| `?` | 中 | 左结合 | 可选链/错误传播 |
| `??` | 低 | 右结合 | 空值合并 |
| `??>` | 低 | 右结合 | 空值传播到错误 |


**编译期错误检查**：

- 未处理的 `Result` 或 `! E` 类型在编译期报错
- `panic` 在 debug 模式下打印完整堆栈，release 模式下可选禁用

**运行时错误处理**：

- 结构化并发中的错误传播：子任务失败 -> 父任务取消 -> 错误向上传播
- Actor 监督：根据策略决定重启、停止或升级

---

## 7. 开发体验与诊断系统（含学习模式）

### 7.1 量化开发体验验收指标

| 指标 | 目标值 | 测量方式 |
|:---|:---|:---|
| 冷启动编译时间 | < 30 秒 | `yan build` 首次执行 |
| 增量编译时间 | < 200ms | 修改单行代码后重新编译 |
| LSP 响应延迟 | < 100ms | VS Code 插件测量 |
| 错误诊断准确率 | > 95% | 测试集验证 |
| 学习模式覆盖率 | > 90% 的错误类型 | 文档统计 |
| 包安装时间 | < 10 秒 | `yan add` 执行时间 |

### 7.2 编译器诊断系统

**错误信息设计原则**：

1. **结构化**：错误代码 + 位置 + 原因 + 解决方案 + 学习链接
2. **可操作性**：提供具体的修复建议代码
3. **教育性**：解释背后的设计原理

**示例诊断**：

```text
E-BORROW-INVALID: cannot borrow `data` as mutable because it is also borrowed as immutable
 --> src/main.zl:15:5
   |
13 | let r1 = &data;
   |          ----- immutable borrow occurs here
14 | println!("{}", r1);
15 | let r2 = &mut data;
   |     ^^^^ mutable borrow occurs here
   |
Note: immutable borrow later used here
Help: consider cloning the data before the mutable borrow:
  let cloned = data.clone();
  let r2 = &mut cloned;
Learn: ZULON's borrowing rules ensure memory safety at compile time.
See: https://docs.zulon-lang.org/ownership/borrowing
```

### 7.3 学习模式（Learning Mode）

**学习模式特性**：

- 启用学习模式：`yan config set learning-mode true`
- 错误信息附带详细解释和设计原理
- 提供交互式修复建议
- 推荐相关的学习资源

**学习模式示例**：

```text
E-LIFETIME-MISSING: missing lifetime specifier
 --> src/main.zl:8:23
   |
8 | fn longest(x: &str, y: &str) -> &str {
   |                       ^ expected named lifetime parameter
   |
Help: this function's return type contains a borrowed value, but the 
      signature does not say whether it is borrowed from `x` or `y`.
      Consider adding explicit lifetime parameters:
  fn longest<'a>(x: &'a str, y: &'a str) -> &'a str
Learn: Lifetimes ensure that references are always valid. In this case,
       we need to tell the compiler how the return value relates to 
       the input parameters. See: https://docs.zulon-lang.org/lifetimes
```

### 7.4 IDE 集成

**LSP 服务器功能**：

- 实时错误提示
- 智能代码补全
- 重构支持（重命名、提取函数等）
- 跳转到定义/引用
- 悬停显示类型信息

**VS Code 扩展**：

- 语法高亮
- 代码片段
- 集成终端
- 调试支持

---

## 8. 语法体系与高级特性（函数式、泛型、反射、元编程）

### 8.1 泛型系统

#### 8.1.1 单态化泛型

```zulon
// 泛型函数
fn identity<T>(x: T) -> T {
  return x;
}

// 泛型结构体
struct Point<T> {
  x: T,
  y: T,
}

// 泛型枚举
enum Option<T> {
  Some(T),
  None,
}
```

#### 8.1.2 where 子句约束

```zulon
fn sort<T>(items: &mut [T])
where T: Ord + Clone {
  // ...
}
```

#### 8.1.3 常量泛型

```zulon
struct Array<T, const N: usize> {
  data: [T; N],
}

fn create_array<T, const N: usize>() -> Array<T, N> {
  Array { data: [T::default(); N] }
}
```

### 8.2 函数式编程

#### 8.2.1 一等函数与闭包

```zulon
let add = |x, y| x + y;
let result = add(1, 2);  // 3

// 捕获环境的闭包
let multiplier = |x| x * factor;
```

#### 8.2.2 管道操作符

```zulon
let result = data
  |> filter(|x| x > 0)
  |> map(|x| x * 2)
  |> sum();
```

#### 8.2.3 列表表达式

```zulon
let squares = [x * x for x in 1..10 if x % 2 == 0];
```

#### 8.2.4 模式匹配

```zulon
match value {
  case 0 => "zero",
  case n if n > 0 => "positive",
  case _ => "negative",
}
```

#### 8.2.5 不可变数据结构

```zulon
// 持久化列表
let list1 = List::empty();
let list2 = list1.push(1);
let list3 = list2.push(2);

// list1 仍然可用
```

### 8.3 编译期反射与元编程

#### 8.3.1 编译期函数求值（Comptime）

借鉴Zig的设计：

```zulon
fn fibonacci(comptime n: usize) -> usize {
  if n <= 1 { return n; }
  return fibonacci(n - 1) + fibonacci(n - 2);
}

const FIB_10 = fibonacci(10);  // 编译期计算
```

#### 8.3.2 类型反射

```zulon
// 获取类型信息
let type_info = type_of(value);
let fields = type_info.fields();
let methods = type_info.methods();
```

#### 8.3.3 代码生成

```zulon
// 派生宏
#[derive(Debug, Serialize)]
struct User {
  id: i64,
  name: str,
}
```

### 8.4 现代语法糖

#### 8.4.1 属性访问

```zulon
// 自动 getter/setter
struct User {
  @get @set
  id: i64,
  
  @get
  name: str,
}
```

#### 8.4.2 空安全操作符

```zulon
// 安全导航
let zip = user?.address?.zip;

// 空值合并
let name = user?.name ?? "Unknown";

// 空值传播到错误
let value = maybe_null ??> Error::Empty;
```

#### 8.4.3 解构赋值

```zulon
let Point { x, y } = point;
let (name, age) = get_user_info();
```

#### 8.4.4 尾随闭包

```zulon
// 函数最后一个参数是闭包时可尾随
with_file("data.txt", |file| {
  let content = file.read_all();
  process(content);
});
```

---

## 9. 标准库架构：Core/Std/Ext

### 9.1 设计哲学

标准库展示语言"无锁并发、分层内存、代数效应"特性，分为三层：

- **Core**：最小化、无分配、无并发依赖
- **Std**：完整标准库，依赖Core
- **Ext**：扩展库，可选特性

### 9.2 Core 库

```zulon
// 基础类型
pub type Result<T, E> = Ok(T) | Err(E);
pub type Option<T> = Some(T) | None;

// 基础trait
pub trait Clone { fn clone(self) -> Self; }
pub trait Copy: Clone {}
pub trait Debug { fn fmt(&self) -> str; }
pub trait Display { fn fmt(&self) -> str; }
```

### 9.3 Std 库

#### 9.3.1 I/O系统

基于代数效应的全异步模型：

```zulon
effect IO {
  fn read(path: Path) -> Bytes ! IoError;
  fn write(path: Path, data: Bytes) -> usize ! IoError;
  fn create_dir(path: Path) -> Result<(), IoError>;
}

// 使用
async fn copy_file(src: Path, dst: Path) -> Result<(), IoError> {
  let data = do IO::read(src)?;
  do IO::write(dst, data)?;
  Ok(())
}
```

#### 9.3.2 集合库

内存分层感知的容器：

```zulon
// 支持区域分配的Vector
struct Vector<T> {
  data: *mut T,
  len: usize,
  capacity: usize,
  allocator: &Allocator,  // 显式分配器
}

// 不可变列表（持久化）
struct List<T> {
  head: Option<(T, List<T>)>,
}

// 哈希表
struct HashMap<K, V> {
  // 实现细节
}
```

#### 9.3.3 并发工具包

彻底去锁化的并发工具：

```zulon
// 消息通道
let (tx, rx) = chan::bounded::<T>(capacity);

// Actor 框架
actor MyActor {
  state data: T
  
  fn process(self, msg: Message) -> Response {
    // ...
  }
}

// 任务编排
task::scope {
  let h1 = task::spawn { work1() };
  let h2 = task::spawn { work2() };
  
  let result = h1.await? + h2.await?;
  Ok(result)
}
```

### 9.4 Ext 库

可选的扩展功能：

- `serde`：序列化/反序列化
- `regex`：正则表达式
- `crypto`：加密算法
- `net`：网络协议
- `async`：异步运行时

---

## 10. 多目标编译与工具链：`yan` 统一入口

### 10.1 `yan` 工具设计哲学

`yan`作为统一命令行工具，集成：

- 包管理（依赖解析、版本锁定）
- 编译（多目标后端）
- 测试（单元测试、集成测试、基准测试）
- 运行（脚本模式、系统模式）
- 诊断（错误解释、学习模式）

### 10.2 多目标编译

#### 10.2.1 编译目标

| 目标 | 输出格式 | 适用场景 |
|:---|:---|:---|
| Native | 机器码（ELF/Mach-O/PE） | 系统编程、高性能服务 |
| WebAssembly | .wasm | 浏览器、边缘计算 |
| JavaScript | .js | Web前端、Node.js |
| JVM | .class | Java生态集成 |
| Rust源码 | .rs | Rust生态互操作 |

#### 10.2.2 编译流程

```
ZULON源码 (.zl)
    ↓
前端解析
    ↓
AST (抽象语法树)
    ↓
类型检查
    ↓
UMCA分析
    ↓
IR (中间表示)
    ↓
优化管道
    ↓
目标代码生成
    ↓
Native / Wasm / JS / JVM / Rust
```

### 10.3 包管理

#### 10.3.1 依赖管理

```toml
[dependencies]
std = "^1.0"                    # 标准库
serde = { version = "^2.1", features = ["json"] }
async = { git = "https://github.com/zulon/async", tag = "v1.0" }
local = { path = "./libs/local" }  # 本地路径依赖
```

#### 10.3.2 语义化版本

遵循SemVer规范：

- `^1.2.3`：兼容1.2.3及以上，但<2.0.0
- `~1.2.3`：兼容1.2.3到1.2.x
- `1.2.3`：精确版本

### 10.4 脚本模式 vs 系统模式

#### 10.4.1 脚本模式

```zulon
// script.zl
// 使用 start 函数
fn start() {
  println("Hello from script!");
  let result = compute();
  println("Result: {result}");
}

// 运行: yan run script.zl
```

#### 10.4.2 系统模式

```zulon
// main.zl
// 使用 main 函数
fn main() -> Result<(), Error> {
  let config = load_config()?;
  let server = Server::new(config);
  server.run()?;
  Ok(())
}

// 编译: yan build --release
// 运行: ./myapp
```

---

## 11. 六大领域能力：特性与标准库 API 草案

### 11.1 系统编程

**目标**：零成本抽象、内存安全、C互操作

**特性**：
- 裸指针操作（unsafe）
- FFI（C/C++互操作）
- 内联汇编
- 编译期计算（comptime）

```zulon
// C函数调用
extern "C" {
  fn printf(fmt: *const u8, ...) -> i32;
}

// 内联汇编
let result = asm! {
  "mov {0}, 1",
  "add {0}, 2",
  out(reg) value,
};
```

### 11.2 Web开发

**目标**：高吞吐异步I/O、快速启动、类型安全

**特性**：
- HTTP服务器/客户端
- WebSocket支持
- 模板引擎
- 中间件框架

```zulon
// Web框架示例
use web::{get, post, App, HttpResponse};

async fn hello() -> HttpResponse {
  HttpResponse::ok().body("Hello, ZULON!")
}

fn main() {
  let app = App::new()
    .route("/", get(hello))
    .route("/users", post(create_user));
  
  app.run("127.0.0.1:8080").await;
}
```

### 11.3 移动应用

**目标**：跨平台UI、原生性能、小内存占用

**特性**：
- 声明式UI框架
- 响应式编程
- 平台通道

```zulon
// UI组件
component Button {
  state pressed: bool = false
  
  render() {
    div {
      class: if self.pressed { "pressed" } else { "normal" },
      onclick: { self.pressed = !self.pressed },
      text: "Click me!",
    }
  }
}
```

### 11.4 游戏开发

**目标**：稳定帧率、低延迟、高性能

**特性**：
- ECS（实体组件系统）框架
- 渲染引擎集成
- 物理引擎
- 音频系统

```zulon
// ECS示例
entity Player {
  position: Position,
  velocity: Velocity,
  sprite: Sprite,
}

system Movement {
  query(Position, Velocity)
  
  run(pos: &mut Position, vel: &Velocity) {
    pos.x += vel.x;
    pos.y += vel.y;
  }
}
```

### 11.5 嵌入式系统

**目标**：小内存占用、无运行时、裸机支持

**特性**：
- `no_std`模式
- 静态内存分配
- 中断处理
- 寄存器操作

```zulon
#![no_std]

// 裸机编程
@interrupt
fn timer_isr() {
  // 中断处理
}

@start
fn main() -> ! {
  loop {
    // 主循环
  }
}
```

### 11.6 AI/ML与科学计算

**目标**：高性能数值计算、自动微分、GPU加速

**特性**：
- 原生张量类型
- SIMD向量化
- GPU计算
- 自动微分

```zulon
// 张量操作
let a: Tensor<f32, 2> = tensor::zeros([100, 100]);
let b: Tensor<f32, 2> = tensor::ones([100, 100]);
let c = a @ b;  // 矩阵乘法

// 自动微分
fn loss(x: Tensor<f32>) -> Tensor<f32> {
  (x - target).pow(2).sum()
}

let grad = gradient(loss, x);
```

---

## 12. 默认控安全：类型安全 + 内存安全 + 访问控制

### 12.1 类型安全

- 默认强类型
- 类型推断减少冗余
- 类型擦除最小化（保留运行时类型信息）

### 12.2 内存安全

- 所有权系统保证无悬垂指针
- 借用检查器防止数据竞争
- 编译期消除常见内存错误

### 12.3 访问控制

```zulon
// 模块级访问控制
pub struct User {  // 公开
  pub id: i64,    // 公开
  name: str,      // 私有（当前模块内可见）
  pub(crate) email: str,  // crate内可见
}

// 文件级访问控制
file private fn helper() {  // 仅在当前文件可见
  // ...
}
```

### 12.4 安全默认值

| 维度 | 默认行为 | 可选覆盖 |
|:---|:---|:---|
| 空值 | 禁止null（Option<T>显式） | 强制解包（!） |
| 内存 | 所有权+借用检查 | unsafe块 |
| 并发 | 消息传递+结构化并发 | Arc<Mutex> |
| 类型转换 | 显式转换 | unsafe转换 |
| 数组访问 | 边界检查 | @unchecked |

---

## 13. 编译器工程规格：IR、Pass、后端与路线图

### 13.1 中间表示（IR）

#### 13.1.1 CPS IR（Continuation-Passing Style）

采用CPS IR而非传统SSA，天然支持挂起/恢复：

```
// 示例CPS转换
fn example(x: i32) -> i32 {
  let y = x + 1;
  let z = y * 2;
  z
}

// CPS形式
fn example_cps(x: i32, k: fn(i32) -> !) -> ! {
  let y = x + 1;
  let z = y * 2;
  k(z)
}
```

#### 13.1.2 IR层次

1. **HIR**（高级IR）：保留语法结构
2. **MIR**（中级IR）：类型化、所有权检查
3. **LIR**（低级IR）：接近机器码

### 13.2 优化管道

#### 13.2.1 优化Pass

- 内联
- 死代码消除
- 循环优化
- 向量化
- 逃逸分析

#### 13.2.2 Equality Saturation

采用Equality Saturation技术，在多个等价IR间搜索最优表达：

```
// 示例：自动优化
(x + y) - y  =>  x
```

### 13.3 后端

#### 13.3.1 LLVM后端

- 生成高效机器码
- 支持多平台
- 成熟优化支持

#### 13.3.2 自举后端

- 快速编译
- 简单实现
- 适合脚本模式

### 13.4 开发路线图

#### 13.4.1 P0（核心功能）

- [ ] 基础类型系统
- [ ] 所有权系统
- [ ] 结构化并发
- [ ] 代数效应
- [ ] 标准库Core
- [ ] Native后端

#### 13.4.2 P1（增强功能）

- [ ] 完整标准库
- [ ] WebAssembly后端
- [ ] JavaScript后端
- [ ] 包管理器
- [ ] IDE支持

#### 13.4.3 P2+（扩展功能）

- [ ] JVM后端
- [ ] GPU支持
- [ ] 形式化验证
- [ ] 增量编译

---

## 14. 跨语言互操作与FFI

### 14.1 C FFI

#### 14.1.1 外部函数声明

```zulon
extern "C" {
  fn printf(fmt: *const u8, ...) -> i32;
  fn malloc(size: usize) -> *mut u8;
  fn free(ptr: *mut u8);
}
```

#### 14.1.2 C ABI兼容

```zulon
@repr(C)
struct Point {
  x: f32,
  y: f32,
}

@no_mangle
pub extern "C" fn add_points(a: Point, b: Point) -> Point {
  Point { x: a.x + b.x, y: a.y + b.y }
}
```

### 14.2 Rust互操作

#### 14.2.1 Rust源码生成

ZULON可编译为Rust源码：

```zulon
// ZULON代码
fn add(a: i32, b: i32) -> i32 {
  a + b
}
```

生成Rust代码：

```rust
// 生成的Rust代码
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

#### 14.2.2 与Rust库互操作

```zulon
// 使用Rust库
use rust::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
  id: i64,
  name: str,
}
```

### 14.3 JavaScript互操作

#### 14.3.1 WebAssembly集成

```zulon
// ZULON编译为Wasm
@wasm_bindgen
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

JavaScript调用：

```javascript
import { greet } from './greet.js';

console.log(greet('ZULON'));  // "Hello, ZULON!"
```

#### 14.3.2 JavaScript后端

ZULON可直接编译为JavaScript：

```zulon
// ZULON代码
export fn calculate(x: i32, y: i32) -> i32 {
  x * y + 42
}
```

生成JavaScript：

```javascript
// 生成的JavaScript
export function calculate(x, y) {
    return x * y + 42;
}
```

### 14.4 JVM互操作

#### 14.4.1 JVM字节码生成

ZULON可编译为JVM字节码：

```zulon
// ZULON代码
class Calculator {
  fn add(a: i32, b: i32) -> i32 {
    a + b
  }
}
```

#### 14.4.2 与Java互操作

```zulon
// 使用Java类
use java::util::ArrayList;

let list = ArrayList::new();
list.add("Hello");
list.add("ZULON");
```

---

## 15. 宏系统与编译期代码生成

### 15.1 宏系统设计

#### 15.1.1 声明宏（Macro by Example）

```zulon
macro_rules! vec {
  ( $( $x:expr ),* ) => {{
    let mut temp_vec = Vec::new();
    $( temp_vec.push($x); )*
    temp_vec
  }};
}

// 使用
let v = vec![1, 2, 3];
```

#### 15.1.2 过程宏（Procedural Macros）

```zulon
// 派生宏
#[derive(Debug)]
struct Point {
  x: i32,
  y: i32,
}

// 属性宏
@route("/", GET)
fn index() -> Response {
  // ...
}
```

### 15.2 编译期代码生成

#### 15.2.1 Comptime函数

借鉴Zig：

```zulon
fn generate_lookup_table(comptime size: usize) -> [u8; size] {
  let mut table = [0; size];
  comptime for i in 0..size {
    table[i] = (i * i) as u8;
  }
  table
}

const LOOKUP = generate_lookup_table(256);
```

#### 15.2.2 类型生成

```zulon
fn make_pair_type(comptime T: type, comptime U: type) -> type {
  struct {
    first: T,
    second: U,
  }
}

type IntStrPair = make_pair_type(i32, str);
```

---

## 16. 开发者工具链完整规格

### 16.1 REPL

#### 16.1.1 基本功能

```bash
$ yan repl
ZULON REPL v1.0.0
> let x = 42
> :t x
x: i32
> :doc println
println formats and prints to stdout
Usage: println(fmt: str, args: ...)
> 
```

#### 16.1.2 高级功能

- 类型查询 `:t expr`
- 文档查询 `:doc name`
- 加载模块 `:load module`
- 保存会话 `:save file.zl`

### 16.2 调试器

#### 16.2.1 基本调试

```bash
$ yan debug main.zl
(gdb) break main
(gdb) run
(gdb) print x
(gdb) continue
```

#### 16.2.2 高级调试

- 条件断点
- 监视点
- 回溯
- 表达式求值

### 16.3 性能分析

#### 16.3.1 基准测试

```zulon
@bench
fn bench_sort(b: &mut Bencher) {
  let data = generate_random_data(1000);
  b.iter(|| sort(&data));
}
```

#### 16.3.2 性能分析

```bash
# CPU分析
$ yan profile --cpu main.zl

# 内存分析
$ yan profile --memory main.zl

# 并发分析
$ yan profile --concurrent main.zl
```

### 16.4 代码格式化与检查

#### 16.4.1 格式化

```bash
# 格式化当前目录
$ yan fmt

# 检查格式
$ yan fmt --check
```

#### 16.4.2 静态检查

```bash
# 运行clippy
$ yan lint

# 自动修复
$ yan lint --fix
```

---

## 17. 附录：语义权衡与前沿参考

### 17.1 设计权衡

#### 17.1.1 性能 vs 安全

ZULON选择：安全优先，性能通过零成本抽象保证

#### 17.1.2 简单 vs 表达力

ZULON选择：简单优先，表达力通过库和宏扩展

#### 17.1.3 编译时 vs 运行时

ZULON选择：编译时检查优先，运行时开销最小化

### 17.2 前沿技术参考

#### 17.2.1 PLDI 2024研究成果

- **抽象解释编译**：将抽象解释器转化为编译器pass
- **WebAssembly集成**：RichWasm、SpecTec
- **并发引用计数**：Concurrent Immediate Reference Counting

#### 17.2.2 结构化并发

- C++ Senders/Receivers框架（C++26）
- Swift TaskGroup
- Kotlin kotlinx-coroutines

#### 17.2.3 内存管理创新

- Rust所有权系统演进
- Swift ARC优化
- Zig comptime内存管理

### 17.3 相关语言对比

| 特性 | ZULON | Rust | Go | Zig |
|:---|:---|:---|:---|:---|
| 内存安全 | 所有权+借用 | 所有权+借用 | GC | 手动+检查 |
| 并发模型 | Actor+结构化 | async/await | goroutine | 手动 |
| 类型系统 | 静态+推断 | 静态+推断 | 静态 | 静态+comptime |
| 编译期计算 | 支持 | 有限 | 无 | 强大 |
| 学习曲线 | 中等 | 陡峭 | 平缓 | 中等 |

### 17.4 未来演进方向

1. **形式化验证**：集成SMT求解器
2. **量子计算**：预留量子抽象
3. **AI原生**：内置张量类型与自动微分
4. **云原生**：原生支持分布式追踪

---

**文档结束**

> 本文档为ZULON编程语言的深度优化设计规范，整合前沿技术研究成果，旨在指导编译器、运行时和标准库的工程实现。文档将持续演进，欢迎社区贡献和反馈。
