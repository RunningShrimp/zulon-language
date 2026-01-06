# ZULON 编程语言白皮书
**版本 1.0**  
**2026年1月**  
*在不牺牲系统级性能的前提下，将内存安全与并发正确性作为默认路径*

---

## 执行摘要

ZULON 是一门面向系统编程、Web 服务、科学计算与嵌入式领域的通用编程语言。其核心创新在于 **将内存安全、并发安全与可预期错误处理作为默认路径** ，而非可选特性。通过统一的权限修饰符（`owned`/`local`/`shared`）、代数效应（Algebraic Effects）与结构化并发模型，ZULON 在编译期消除数据竞争、悬垂指针与空指针异常，同时保持与 C/Rust 相当的性能。

**关键承诺**：
- **默认空安全**：`T` 永不包含空值，`T?` 显式可空。
- **默认内存安全**：无悬垂指针、无 use-after-free。
- **默认并发安全**：跨并发边界的能力由类型系统静态验证。
- **统一错误处理**：`T ! E` 语法与 `?` 传播消除异常与控制流混淆。
- **多目标编译**：同一语法可编译为 Native/Wasm/JS/JVM/Rust 源码。

---

## 1. 语言哲学与设计原则

### 1.1 五大默认（The Five Defaults）

ZULON 将核心叙事收敛为五个默认行为，降低开发者认知负担：

1. **默认独占所有权**：值语义为 `owned`，移动后失效。
2. **默认结构化任务**：所有异步运行于 `task::scope`，取消与错误自动传播。
3. **默认不可变共享**：共享可变状态禁止，可变状态封装于 `actor` 内部。
4. **默认效应显式化**：I/O、取消、注入均以类型化副作用显式声明。
5. **默认工具闭环**：`yan` 统一编译、测试、诊断与部署。

### 1.2 设计权衡（Design Trade-offs）

| 设计点 | 选择 | 理由 | 放弃替代方案 |
|---|---|---|---|
| 内存管理 | 栈/区域/ARC，非默认 GC | 确定性、可预测延迟 | 全 GC（性能不可控） |
| 并发模型 | 结构化并发 + Actor，无锁优先 | 无数据竞争、易推理 | 共享内存锁（死锁风险） |
| 泛型 | 单态化 + where 约束 | 零成本抽象 | 动态分发（性能损失） |
| 效应系统 | Set-based + Handler | 可实现、可推导 | Row poly（实现复杂） |
| FFI | C 优先，双向互操作 | 生态广泛 | 仅 C（限制互操作） |
| 语法糖 | 最小化、正交 | 降低语言复杂度 | 无限语法糖（认知负担） |

### 1.3 非目标（Non-Goals）

- **不以"全能 GC 语言"为默认**：P0 仅要求确定性内存（栈/区域/共享冻结）。
- **不把"锁"作为抽象**：锁仅在底层库或 `unsafe` 边界使用。
- **不引入隐式异步**：所有异步必须通过 `task::` 或 effect 显式启动。
- **不追求无限语法糖**：核心语法小而正交，高级能力通过库与宏实现。

---

## 2. 核心语言特性

### 2.1 类型系统架构

#### 2.1.1 基础类型
```zulon
// 数值类型（默认溢出检查）
let a: i32 = 42;
let b: u8 = 0xFF;
let c: f32 = 1.23e-4;
let d: c64 = 1.0 + 2.0i;  // 复数

// 字符串与字节
let s: str = "Hello, ZULON!";  // 默认 shared str
let b: Bytes = Bytes::from("binary");  // 零拷贝缓冲

// 数组与切片
let arr: [i32; 3] = [1, 2, 3];  // 定长数组
let slice: &[i32] = &arr[..2];  // 切片视图

// 元组与解构
let (x, y, z) = (1, "two", 3.0);
let first = (1, 2).0;
```

#### 2.1.2 权限修饰符（Ownership Markers）
```zulon
fn demo() {
    let owned_value: owned i32 = 42;      // 默认，可移动
    let local_buffer: local Bytes = read(); // 限制在作用域
    let shared_config: shared Config = share(parse()); // 冻结共享
    
    // 跨 task 规则
    task::spawn {
        // ❌ 错误：local 值不可 Send
        // process(local_buffer);
        
        // ✅ OK：owned 值移动
        process(owned_value);
        
        // ✅ OK：shared 值共享
        use_config(shared_config);
    };
}
```

#### 2.1.3 并发能力（Send / Share / Sync）
- **`Send`**：值可跨 task/actor 移动（唯一所有权转移）。
- **`Share`**：值可跨边界共享别名，必须是深度不可变（冻结后）。
- **`Sync`**：值可被多个任务并发读取（隐式于 `Share`）。

**自动推导规则**：
- `i32`, `bool`, `shared T` 自动 `Send + Share`。
- `&mut T` 从不 `Send/Share`。
- `local T` 从不 `Send/Share`。

**显式实现**：
```zulon
struct MyStruct { data: i32 }

// 条件实现
impl<T> Send for MyStruct<T> where T: Send {}
impl<T> Share for MyStruct<T> where T: Share {}
```

### 2.2 错误处理模型

#### 2.2.1 `T ! E` 语法糖
```zulon
// T ! E 等价于 Result<T, E>
fn read_file(path: Path) -> Bytes ! IoError {
    // 内部返回 Result<Bytes, IoError>
}

// 使用 ? 传播
fn process() -> Data ! AppError {
    let bytes = read_file("data.json")?;  // IoError -> AppError
    parse(bytes)?
}

// 错误联合
fn complex() -> i32 ! IoError | NetError | ParseError {
    // ...
}
```

#### 2.2.2 `Error` Trait 与链
```zulon
trait Error {
    fn message(self) -> str;      // 人类可读
    fn source(self) -> Error?;    // 错误链
    fn backtrace(self) -> Backtrace?; // P1 可选
}

// 约定：enum XxxError 自动生成实现
enum IoError {
    NotFound,
    Permission(str),
}

// 自定义覆盖
impl Error for IoError {
    fn message(self) -> str {
        match self {
            NotFound => "文件不存在",
            Permission(p) => format!("权限不足: {p}"),
        }
    }
}
```

#### 2.2.3 `try` 块与错误映射
```zulon
fn load() -> Config ! ConfigError {
    try {
        let a = read_a()?;
        let b = read_b()?;
        merge(a, b)
    } catch IoError::NotFound => {
        // 特定错误处理
        default_config()
    } catch e => {
        // 其他错误转换
        Err(ConfigError::from(e))
    }
}
```

### 2.3 代数效应系统

#### 2.3.1 声明与触发
```zulon
effect IO {
    fn read(path: Path) -> Bytes ! IoError;
    fn write(path: Path, data: Bytes) -> usize ! IoError;
}

effect Net {
    fn get(url: Url) -> Bytes ! NetError;
}

fn fetch_home()
    -> Bytes
    ! AppError
    performs Net + IO
{
    let b = do Net::get("https://example.com")?;
    return do IO::write("/tmp/out", b)?;
}
```

#### 2.3.2 Handler 定义
```zulon
fn start() ! AppError {
    let bytes = handle fetch_home() {
        // 绑定具体实现
        case Net::get(u) => resume(net_runtime::get(u)),
        case IO::write(p, b) => resume(io_runtime::write(p, b)),
    }?;
    
    println("wrote {}", bytes);
}
```

**Handler 解析规则**：
1. **就近优先**：`do` 由最内层 lexically nearest 的 `handle` 处理。
2. **同层唯一**：同层多个 handler 编译错误 `E-HANDLER-AMBIGUOUS`。
3. **显式消歧**：通过 `use handler` 模块级默认绑定。

```zulon
// 模块级默认 handler（可审计）
use handler IO = std::io::posix;
use handler Net = std::net::native;

fn prod() ! AppError {
    // 隐式使用默认 handler
    fetch_home()?;
}

// 测试覆盖
fn test() ! AppError {
    handle prod() {
        case IO::read(_) => resume(Ok("fake".bytes())),
        case Net::get(_) => resume(Ok("mock".bytes())),
    }
}
```

#### 2.3.3 效应多态
```zulon
// 高阶函数携带效应
fn map<T, U, Eff>(
    xs: List<T>,
    f: fn(T) -> U performs Eff
) -> List<U> performs Eff {
    let mut out = List::new();
    for x in xs {
        out.push(f(x));
    }
    out
}

// 效应集合运算
fn compose<Eff1, Eff2>(f: fn() performs Eff1, g: fn() performs Eff2)
    -> fn() performs Eff1 + Eff2 {
    || { f(); g() }
}
```

#### 2.3.4 取消与超时作为效应
```zulon
effect Cancel {
    fn is_requested() -> bool;
    fn reason() -> CancelReason;
}

effect Timeout {
    fn deadline() -> Instant;
}

fn long_task() performs Cancel + Timeout {
    loop {
        if do Cancel::is_requested() {
            return cleanup();
        }
        
        if Instant::now() > do Timeout::deadline() {
            return Err(TimeoutError);
        }
        
        do_work();
    }
}
```

### 2.4 并发模型：结构化并发

#### 2.4.1 Task 与 Scope
```zulon
fn parallel_sum(xs: List<i32>) -> i32 {
    task::scope {
        let mid = xs.len() / 2;
        let left = task::spawn { sum(&xs[..mid]) };
        let right = task::spawn { sum(&xs[mid..]) };
        
        left.await? + right.await?
    }
}

// Scope 失败即取消
fn fetch_parallel() -> Result<Data, Error> {
    task::scope {
        let h1 = task::spawn { fetch_from_primary() };
        let h2 = task::spawn { fetch_from_backup() };
        
        // 任一成功返回，另一自动取消
        select {
            case Ok(d) = h1.await => Ok(d),
            case Ok(d) = h2.await => h1.cancel()?; Ok(d),
            case Err(e) = h1.await => Err(e),
        }
    }
}
```

#### 2.4.2 取消机制
- **协作式**：任务必须显式检查 `task::check_cancelled()?`。
- **结构化**：Scope 退出时向未完成任务发送取消信号。
- **资源释放**：取消触发 `defer` 块，保证资源释放。

```zulon
fn background_work() -> Result<(), Cancelled> {
    defer { cleanup_resources(); }
    
    loop {
        task::check_cancelled()?; // 检查点
        process_chunk();
    }
}
```

#### 2.4.3 Actor 模型
```zulon
actor Counter {
    state n: i64
    
    fn inc(self, by: i64) -> i64 {
        self.n += by;
        self.n
    }
    
    fn get(self) -> i64 {
        self.n
    }
}

// 使用
let counter = Counter::new(0);
let handle = counter.inc(5);  // 异步消息
let value = handle.await?;    // 等待结果
```

**Actor 约束**：
- 状态不泄漏引用。
- 仅允许消息传递（Move 或 Share 快照）。
- 监督策略配置：
```zulon
actor Worker {
    supervision: OneForOne
    max_restarts: 3
    restart_delay: Duration::seconds(1)
}
```

#### 2.4.4 通道与 Select
```zulon
let (tx, rx) = chan::bounded::<i32>(100);
let (tx2, rx2) = chan::unbounded::<str>();

// 多路复用
select {
    case msg = rx.recv() => handle_int(msg),
    case msg = rx2.recv() => handle_str(msg),
    default => handle_idle(), // 非阻塞
}

// 带超时
select {
    case msg = rx.recv() => handle(msg),
    case after Duration::seconds(5) => handle_timeout(),
}
```

---

## 3. 内存管理：统一内存与并发架构（UMCA）

### 3.1 内存分层 L1/L2/L3/L4

| 层级 | 权限视图 | 分配器 | 跨并发语义 | 典型场景 | 性能特征 |
|---|---|---|---|---|---|
| **L1** | `owned` | 栈/寄存器 | Move | 局部变量、临时值 | 零分配，零开销 |
| **L2** | `local` | Region bump | 作用域绑定 | 请求上下文、帧数据 | < 10ns 分配，批量释放 |
| **L3** | `shared` | ARC + 冻结 | 不可变共享 | 配置、缓存 | 原子操作，读无锁 |
| **L4** | `managed` | 可选 GC | 托管 | 脚本模式、复杂图 | 毫秒延迟，自动回收 |

### 3.2 Region 分配器（P1）
```zulon
fn handle_request(req: Request) -> Response {
    region req_scope {
        let parser = Parser::new(&req.body);
        let data = parser.parse()?;
        let result = compute(&data)?; // 所有分配在 req_scope
        
        // scope 退出时批量释放，无碎片
        result
    }
}
```

**实现**：
```rust
pub struct Region {
    buffer: Vec<u8>,
    offset: Cell<usize>,
}

impl Region {
    pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        let current = self.offset.get();
        let aligned = align_up(current, align);
        
        if aligned + size <= self.buffer.len() {
            self.offset.set(aligned + size);
            &self.buffer[aligned] as *const u8 as *mut u8
        } else {
            panic!("Region overflow")
        }
    }
}
```

### 3.3 冻结（Freeze）与共享
```zulon
let config = Config { host: "localhost", port: 8080 };
let shared_config: shared Config = share(config); // 深度冻结

// 冻结后任何路径不得可变访问
// ❌ shared_config.port = 9000; // 编译错误
```

**冻结规则**：
- 递归冻结所有字段。
- 冻结后类型变为 `shared T`，内部字段不可变。
- 冻结操作 `O(n)`，在构造时一次性完成。

### 3.4 逃逸分析与自动提升
```rust
// P1 编译器自动分析
fn create_buffer() -> Bytes {
    let buf = Bytes::with_capacity(1024); // L1 栈分配
    
    // 若未逃逸，保持 L1
    if is_local_use(&buf) {
        return buf; // 提升为 L2 region
    }
    
    // 若跨 task，提升为 L3 shared
    share(buf)
}
```

### 3.5 可选托管堆（P2）
```zulon
#![feature(managed_heap)]

#[managed]
struct ComplexGraph { nodes: List<Node> } // 允许循环引用

fn script_mode() {
    let graph = ComplexGraph::new(); // L4 GC 管理
    // 无需手动释放
}
```

---

## 4. 工具链：`yan` 统一入口

### 4.1 命令概述
```bash
# 项目管理
yan new <name>          # 新建项目
yan add <crate>         # 添加依赖
yan remove <crate>      # 移除依赖
yan update              # 更新依赖

# 构建与运行
yan build [--release] [--target native|wasm|js|jvm]
yan run [path.zl]       # 编译并运行
yan check               # 快速类型检查

# 测试与质量
yan test                # 运行测试
yan bench               # 运行基准
yan vet [--rules=all]   # 静态审计
yan fmt                 # 格式化代码
yan doc                 # 生成文档

# 开发工具
yan repl                # 交互式环境
yan run --watch         # 热重载模式
yan profile             # 性能分析
yan clean               # 清理构建产物
```

### 4.2 包管理（yan.toml）
```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2026"
authors = ["ZULON Team"]
license = "MIT"

[targets]
main = { entry = "src/main.zl", type = "executable" }
lib = { entry = "src/lib.zl", type = "library" }

[dependencies]
std = "^1.0"
serde = { version = "^2.0", features = ["json"] }

[dev-dependencies]
test = "^1.0"

[profile.release]
opt-level = "speed"
lto = true

[workspace]
members = ["crates/*"]
resolver = "2"

[registries.company]
index = "https://crates.company.com/index"
```

### 4.3 工作区（Workspace）支持
```
my-project/
├── yan.toml          # 工作区根
├── crates/
│   ├── core/         # 核心库
│   ├── std/          # 标准库
│   ├── http/         # HTTP 库
│   └── cli/          # CLI 应用
└── examples/
```

### 4.4 构建缓存与增量
```bash
# 编译器查询缓存（salsa）
yan build --incremental  # 默认开启
# 缓存存储于 target/.yan-cache/

# 缓存命中率监控
yan build --verbose
# 输出: [incremental] 95% cache hit, 3 files recompiled
```

### 4.5 交叉编译
```bash
yan build --target x86_64-unknown-linux-gnu
yan build --target aarch64-apple-darwin
yan build --target wasm32-wasi
yan build --target riscv32imc-unknown-none-elf
```

---

## 5. 跨语言互操作（FFI）

### 5.1 C 互操作
```zulon
extern "C" {
    fn printf(format: *const c_char, ...);
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    
    struct CPoint {
        x: c_int,
        y: c_int,
    }
    
    fn c_add_point(a: CPoint, b: CPoint) -> CPoint;
}

// 导出 ZULON 函数
#[no_mangle]
pub extern "C" fn zulon_add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Safe 包装**：
```zulon
struct CStr {
    ptr: *const c_char,
}

impl CStr {
    fn new(s: &str) -> Result<CStr, NulError> {
        // 检查并添加 NUL
    }
    
    fn as_ptr(&self) -> *const c_char {
        self.ptr
    }
}
```

### 5.2 Rust 互操作
```zulon
// 使用 Rust 类型
use rust::std::collections::HashMap;

fn use_rust_map() {
    let map = HashMap::<str, i32>::rust_new();
    map.rust_insert("key", 42);
}

// 导出到 Rust
#[export_to_rust]
async fn zulon_async() -> i32 {
    compute().await
}
```

### 5.3 JavaScript / Wasm
```zulon
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn console_log(s: &str);
    
    type HTMLElement;
    fn appendChild(this: &HTMLElement, child: &HTMLElement);
}
```

### 5.4 FFI 安全准则
- **边界清晰**：FFI 调用必须 `unsafe` 块包装。
- **内存管理**：ZULON 所有权不延伸到 FFI，手动管理。
- **错误转换**：C 错误码必须转为 `Result`。
- **线程安全**：跨语言共享需 `Send/Sync` 标记。

---

## 6. 性能特征与优化

### 6.1 零成本抽象
- **泛型**：单态化 + 内联，无运行时开销。
- **Effect Handler**：P0 限制为浅 handler，正常路径无跳转。
- `**Result `：ABI 优化，`Ok` 路径寄存器返回，`Err` 路径分支预测。
- **shared**：冻结后读路径无原子操作。

### 6.2 性能基准（P1）
```bash
$ yan bench
-------------------------------------------------------------
Benchmark                    Time (ns)    Throughput   p99   
-------------------------------------------------------------
await_ping_pong              950          1.05M ops/s  1.2μs
channel_bounded_100          1,200        833K ops/s   1.5μs
actor_message_1kb            2,100        476K ops/s   2.5μs
effect_perform_resume        45           22M ops/s    50ns
region_alloc_1kb             8            125M ops/s   10ns
array_index_check            12           83M ops/s    15ns
-------------------------------------------------------------
```

### 6.3 优化手段
- **编译期**：
  - 逃逸分析提升 L1/L2。
  - 借用范围收缩。
  - ARC 消除（静态计数为 1）。
  - Effect 内联（handler 确定时）。
- **链接期**：
  - LTO（Link-Time Optimization）。
  - PGO（Profile-Guided Optimization）。
  - BOLT（Binary Optimization）。

### 6.4 二进制体积
```bash
# P1 release build
$ yan build --release --opt-level=size
$ strip target/release/hello
$ ls -lh target/release/hello
-rwxr-xr-x  1 user  group   180K Jan  6 12:00 hello

# 体积构成
- 运行时内核: 40K
- 标准库: 80K
- 用户代码: 60K
```

---

## 7. 开发体验

### 7.1 诊断系统
```zulon
// 错误示例
fn send_local() {
    let buf: local Bytes = read();
    
    task::spawn {
        process(buf); // ❌ 错误
    };
}
```

```
E-REGION-ESCAPE: `local` value `buf` may outlive its region
 --> src/net.zl:42:17
  |
42|   task::spawn { process(buf) }
  |                 ^^^ `buf` is `local` to region `req_scope`
  |
Rule: `local` values cannot cross `task::spawn` boundaries.
Help:
  - Option 1: Move ownership: `task::spawn { process(move buf) }`
  - Option 2: Freeze and share: `let buf = share(buf)`
  - Option 3: Wrap in actor: `actor Processor { ... }`
Learn:
  Values tied to a region have a scoped lifetime. Sending them to
  another task could cause use-after-free. ZULON requires explicit
  ownership transfer or immutable sharing for cross-task communication.
  See: https://docs.zulon-lang.org/ownership/regions
```

### 7.2 LSP 功能
- **实时诊断**：增量编译错误 `< 50ms`。
- **智能补全**：基于类型与作用域。
- **跳转定义**：跨模块、跨 crate、FFI。
- **悬停提示**：类型、文档、错误解释。
- **重构**：重命名、提取函数、内联变量。
- **快速修复**：一键应用建议。

### 7.3 REPL 与热重载
```bash
$ yan repl
ZULON REPL 1.0.0
> let x = 42
x: i32 = 42
> x * 2
84
> :t x
i32
> :doc Vec
pub struct Vec<T>
A growable array type ...
> :load script.zl
```

**热重载**：
```bash
$ yan run --watch --hot-reload
[2026-01-06 12:00:00] File src/lib.zl changed, rebuilding...
[2026-01-06 12:00:02] Hot reload successful, state preserved.
```

状态保持机制：
- 标记为 `#[persist]` 的 actor 状态序列化到内存。
- 热重载后恢复状态并调用 `on_reload` 生命周期钩子。
- 类型定义更改需完全重启。

### 7.4 学习模式
```bash
$ yan build --learn
Analyzing error E-SEND...

📚 ZULON Ownership & Concurrency Primer

This error occurs because you're trying to send a `local` value across a task boundary.
In ZULON, `local` values are tied to a specific region/scope and cannot outlive it.

🔍 Why this rule exists:
   - Prevents use-after-free bugs
   - Makes data race conditions impossible by construction
   - Enables compiler optimizations based on region lifetimes

✅ Recommended patterns:
   1. Move ownership: `task::spawn { process(move data) }`
   2. Freeze and share: `let shared_data = share(data)`
   3. Actor encapsulation: Keep mutable state inside actors

⚖️ Trade-offs:
   - Moving: Zero-cost, but loses access in current scope
   - Sharing: Allows concurrent reads, requires immutability
   - Actor: Best for long-lived mutable state, adds message passing overhead

🔗 Further reading: https://docs.zulon-lang.org/guide/ownership
```

---

## 8. 应用场景与领域库

### 8.1 系统编程（Systems Programming）
```zulon
#![no_std]
#![no_runtime]

fn main() {
    // 直接内存操作
    unsafe {
        let ptr = 0x4000_0000 as *mut u32;
        ptr.write(0xDEADBEEF);
    }
}
```

**API**：
- `core::ptr`: `addr_of!`, `read_volatile`, `write_volatile`
- `core::mem`: `size_of`, `align_of`, `transmute`

### 8.2 Web 开发（Web Development）
```zulon
use std::net::http;

async fn handle_request(req: Request) -> Response {
    let data = req.json::<Data>()?;
    let result = process(data).await?;
    Response::json(result)
}

fn main() {
    let server = http::Server::new("127.0.0.1:8080");
    server.route("/", handle_request).run();
}
```

**API**：
- `std::net::http`: Server, Request, Response, Middleware
- `ext::serde`: JSON, CBOR, MessagePack
- `ext::sql`: Async 数据库驱动

### 8.3 AI / 科学计算（P2）
```zulon
use ext::ml::tensor;

fn train(model: &Model, data: Dataset) -> f32 {
    for epoch in 0..100 {
        let loss = model.forward(data)?;
        model.backward(loss)?;
        model.update();
    }
    model.evaluate()
}

// 自动微分
let x = tensor::ones([1024, 768]);
let y = x.matmul(&w).relu();
let grad = y.backward();
```

**API**：
- `ext::ml::tensor`: 多维张量，自动微分
- `ext::ml::nn`: 神经网络层
- `ext::ml::optim`: 优化器

### 8.4 游戏开发（P2）
```zulon
use ext::game::ecs;

struct Position { x: f32, y: f32 }
struct Velocity { vx: f32, vy: f32 }

fn physics_system(world: &mut World) {
    for (pos, vel) in world.query::<(&mut Position, &Velocity)>() {
        pos.x += vel.vx;
        pos.y += vel.vy;
    }
}
```

**API**：
- `ext::game::ecs`: Entity-Component-System
- `ext::game::math`: SIMD 数学库
- `ext::game::render`: 渲染图

### 8.5 嵌入式（P2）
```zulon
#![no_std]
#![feature(mmio)]

#[mmio(base = 0x4000_0000)]
struct UART {
    dr: VolatileCell<u32>,      // 数据寄存器
    fr: VolatileCell<u32>,      // 标志寄存器
}

fn main() -> ! {
    let uart = UART::new();
    loop {
        let byte = uart.dr.read();
        uart.dr.write(byte);
    }
}
```

**API**：
- `ext::embedded::mmio`: MMIO 安全抽象
- `ext::embedded::hal`: 硬件抽象层
- `ext::embedded::rt`: 裸机运行时

---

## 9. 与其他语言比较

### 9.1 vs Rust
| 特性 | ZULON | Rust |
|---|---|---|
| **内存安全** | 权限修饰符 + ARC + Region | 所有权 + 生命周期 |
| **并发安全** | 结构化并发 + Actor 默认 | 手动 Sync/Send |
| **错误处理** | `T ! E` + Effect 统一 | Result + Panic |
| **学习曲线** | 渐进式，local/shared 简化 | 陡峭，生命周期复杂 |
| **编译速度** | Cranelift Debug 更快 | LLVM 为主 |
| **FFI** | 双向自动生成绑定 | 手动为主 |
| **GC** | 可选 L4 | 无 |
| **Async** | 统一 task/await | async/await + Future |

### 9.2 vs Go
| 特性 | ZULON | Go |
|---|---|---|
| **并发模型** | 结构化 + Actor，无锁优先 | Goroutine + Channel |
| **内存安全** | 编译期无竞争 | 运行时竞争检测 |
| **空安全** | 默认非空 | 可空（nil） |
| **泛型** | 单态化，零成本 | 接口，有开销 |
| **Error** | `T ! E` 类型安全 | 多返回值 error |
| **性能** | 与 Rust 相当 | GC 延迟 |
| **领域** | 系统/AI/嵌入式 | Web/云 |

### 9.3 vs Swift
| 特性 | ZULON | Swift |
|---|---|---|
| **并发安全** | Actor + compile-time | Actor + runtime |
| **内存管理** | ARC + Region + 可选 GC | ARC |
| **错误处理** | Effect 系统 | try/throw |
| **平台** | 多目标（Native/Wasm/JS） | Apple 生态为主 |
| **性能** | 系统级优化 | 应用级优化 |

### 9.4 vs TypeScript
| 特性 | ZULON | TypeScript |
|---|---|---|
| **类型安全** | 编译期保证 | gradual，运行时可选 |
| **空安全** | 默认非空 | 可选（strictNullChecks） |
| **并发** | Actor + task | 手动 Promise/async |
| **FFI** | 双向 C/Rust/JS | JS 单向 |
| **运行时** | 无（可 no_runtime） | V8 必需 |
| **性能** | Native 速度 | JS 解释 |

---

## 10. 未来路线图

### 10.1 P0（2026 Q1）：核心可用
- ✅ 编译器：parser, resolver, typeck, MIR, LLVM backend
- ✅ 运行时：单线程调度器, region allocator, ARC
- ✅ 标准库：core + 简化 std
- ✅ 工具链：yan build/test/check/vet, 基础 LSP
- ✅ 文档：The ZULON Book（基础）

**验收标准**：
- 可编译运行 1000 行并发程序。
- 无数据竞争、无内存泄漏。
- `yan vet` 检出所有违规。

### 10.2 P1（2026 Q4）：生产就绪
- 🔄 增量编译（salsa）
- 🔄 REPL + 热重载
- 🔄 完整 std + 扩展库（serde, regex, http）
- 🔄 LSP 完整（跳转、重构、快速修复）
- 🔄 Cranelift + Wasm 后端
- 🔄 性能工具（profile, sanitize）
- 🔄 完整 Actor 监督与取消

**验收标准**：
- 3 个真实应用（Web 服务器、CLI 工具、数据管道）。
- LSP 延迟 `< 50ms`。
- 二进制体积 `< 500KB`。

### 10.3 P2（2027 H2）：专业完整
- ⏳ MLIR 集成，算子融合
- ⏳ GPU 代码生成（CUDA/Metal/Vulkan）
- ⏳ L4 托管堆与 Wasm-GC
- ⏳ AI/游戏/嵌入式领域库
- ⏳ 时间旅行调试
- ⏳ 形式化验证插件
- ⏳ 分布式 Actor

**验收标准**：
- MNIST 训练速度媲美 PyTorch。
- 3D 游戏 demo 60 FPS。
- 在 RISC-V 裸机运行 RTOS。

---

## 11. 总结

ZULON 语言通过**统一的权限修饰符、代数效应与结构化并发**，在编译期构建起强大的安全保证。它既保留了 C/Rust 的系统级性能，又提供了 Go/Python 的开发体验，同时通过 effect 系统实现了前所未有的可测试性与可组合性。

我们相信，ZULON 将成为下一代系统编程语言的基石，为构建可靠、高效、可维护的软件提供全新范式。

---

## 附录

### A. 语法摘要
```zulon
// 变量与绑定
let x = 42;
let mut y = 0;
let z: i32 = 100;

// 函数
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 多返回值
fn split(s: &str) -> &str, &str {
    let mid = s.len() / 2;
    (&s[..mid], &s[mid..])
}

// 泛型
fn id<T>(x: T) -> T { x }

// Trait
trait Display {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>;
}

// 实现
impl Display for i32 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_int(*self)
    }
}

// 错误
fn div(a: i32, b: i32) -> i32 ! DivError {
    if b == 0 {
        Err(DivError::Zero)?
    }
    Ok(a / b)
}

// Effect
effect IO {
    fn read() -> Bytes ! IoError;
}

fn read_config() -> Config ! IoError performs IO {
    let bytes = do IO::read("config.json")?;
    parse(bytes)?
}

// 匹配
match opt {
    Some(x) => x,
    None => 0,
}

// 循环
for i in 0..10 {
    println(i);
}

while condition {
    // ...
}

// 闭包
let add = |a, b| a + b;
let closure = move |x| x + captured_val;

// 异步
async fn fetch() -> Data ! NetError performs Net {
    await net::get(url)?
}

// 模块
mod utils {
    pub fn helper() { ... }
}

// 导入
use std::io::File;
use std::task::{spawn, scope};
```

### B. 内置 Trait
```zulon
trait Copy { }              // 可拷贝
trait Clone { fn clone(self) -> Self; }
trait Drop { fn drop(&mut self); }  // P2
trait Send { }              // 可跨 task 移动
trait Share { }             // 可跨 task 共享
trait Sync { }              // 可并发读取
trait Error { fn message(self) -> str; fn source(self) -> Error?; }
trait Display { fn fmt(&self, f: &mut Formatter) -> Result<(), Error>; }

// 运算符 Trait
trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }
trait Sub<Rhs = Self> { type Output; fn sub(self, rhs: Rhs) -> Self::Output; }
// ... Mul, Div, Rem, BitAnd, BitOr, ...
```

### C. 词汇表
- **Effect**：类型化副作用，可处理、可注入。
- **Handler**：效应的实现与策略授予点。
- **Region**：作用域内存分配器，批量释放。
- **Task**：轻量级协程，M:N 调度。
- **Actor**：隔离可变状态的并发原语。
- **Send/Share**：并发能力 trait。
- **Freeze**：深度不可变转换，提升为 `shared`。
- **L1/L2/L3/L4**：内存管理层级。

### D. 参考实现
- **编译器**：`https://github.com/zulon-lang/zulonc`
- **运行时**：`https://github.com/zulon-lang/zulonrt`
- **标准库**：`https://github.com/zulon-lang/zulonstd`
- **工具链**：`https://github.com/zulon-lang/yan`

---

**文档版本**: 1.0  
**发布日期**: 2026-01-06  
**语言版本**: 0.1.0 (P0)  
**许可证**: CC BY-SA 4.0 (文档), Apache-2.0 (代码)