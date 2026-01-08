# ZULON 架构设计文档

**版本**: 1.0
**日期**: 2026-01-07
**作者**: ZULON Language Team
**状态**: 设计阶段

---

## 目录

1. [系统架构概览](#系统架构概览)
2. [编译器架构](#编译器架构)
3. [运行时架构](#运行时架构)
4. [标准库架构](#标准库架构)
5. [工具链架构](#工具链架构)
6. [模块间通信](#模块间通信)
7. [扩展性设计](#扩展性设计)
8. [部署架构](#部署架构)

---

## 系统架构概览

### 整体架构图

```
┌────────────────────────────────────────────────────────────────┐
│                        ZULON 生态系统                          │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │  IDE 插件     │  │  LSP 服务器   │  │  调试器       │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│           │                  │                  │              │
│           └──────────────────┼──────────────────┘              │
│                              ▼                                 │
│  ┌──────────────────────────────────────────────────┐        │
│  │              YAN 统一工具链                       │        │
│  ├──────────────────────────────────────────────────┤        │
│  │  ┌─────────────────────────────────────────┐    │        │
│  │  │           yan build/run/test/efpl       │    │        │
│  │  │          编译/运行/测试/交互             │    │        │
│  │  └─────────────────────────────────────────┘    │        │
│  └──────────────────────────────────────────────────┘        │
│                              │                                 │
│                              ▼                                 │
│  ┌──────────────────────────────────────────────────┐        │
│  │              ZULON 标准库                         │        │
│  ├──────────────────────────────────────────────────┤        │
│  │  core  collections  sync  io  async  testing    │        │
│  └──────────────────────────────────────────────────┘        │
│                              │                                 │
│                              ▼                                 │
│  ┌──────────────────────────────────────────────────┐        │
│  │              ZULON 运行时                         │        │
│  ├──────────────────────────────────────────────────┤        │
│  │  内存管理  线程调度  异步IO  效应处理          │        │
│  └──────────────────────────────────────────────────┘        │
│                              │                                 │
│                              ▼                                 │
│  ┌──────────────────────────────────────────────────┐        │
│  │              平台层                               │        │
│  ├──────────────────────────────────────────────────┤        │
│  │  Linux  macOS  Windows  WebAssembly  Android   │        │
│  └──────────────────────────────────────────────────┘        │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### 分层架构

```
┌─────────────────────────────────────────────────┐
│           应用层 (Application Layer)            │
│     用户程序、库、工具                           │
├─────────────────────────────────────────────────┤
│         语言层 (Language Layer)                 │
│  语法、语义、类型系统                            │
├─────────────────────────────────────────────────┤
│        编译器层 (Compiler Layer)                │
│  前端、中端、后端                                │
├─────────────────────────────────────────────────┤
│       运行时层 (Runtime Layer)                   │
│  内存管理、线程调度、GC                          │
├─────────────────────────────────────────────────┤
│        平台层 (Platform Layer)                   │
│  操作系统、硬件抽象                              │
└─────────────────────────────────────────────────┘
```

### 核心设计原则

1. **模块化**: 每个组件职责单一，高内聚低耦合
2. **可扩展**: 易于添加新特性和平台支持
3. **可测试**: 每个模块都可独立测试
4. **性能导向**: 关键路径优化
5. **用户友好**: 清晰的API和错误消息

---

## 编译器架构

### 编译器模块划分

```
compiler/
├── frontend/           // 前端
│   ├── lexer/          // 词法分析器
│   ├── parser/         // 语法分析器
│   ├── ast/            // AST 定义
│   └── macros/         // 宏展开
├── middle/             // 中端
│   ├── hir/            // 高级中间表示
│   ├── mir/            // 中级中间表示
│   ├── typeck/         // 类型检查
│   ├── borrowck/       // 借用检查
│   ├── effectck/       // 效应检查
│   └── resolve/        // 名称解析
├── backend/            // 后端
│   ├── llvm/           // LLVM 后端
│   ├── cranelift/      // Cranelift 后端 (未来)
│   └── wasm/           // WebAssembly 后端
├── optimizer/          // 优化器
│   ├── passes/         // 各种优化 Pass
│   └── analysis/       // 数据流分析
└── codegen/            // 代码生成
    ├── llvm_gen/       // LLVM IR 生成
    └── machine/        // 机器代码生成
```

### 模块接口设计

#### 前端接口

```rust
// Lexer 接口
pub trait Lexer {
    fn new(input: &str) -> Self;
    fn next_token(&mut self) -> Result<Token, LexerError>;
    fn peek_token(&self) -> &Token;
    fn location(&self) -> Location;
}

// Parser 接口
pub trait Parser {
    fn new(lexer: Box<dyn Lexer>) -> Self;
    fn parse(&mut self) -> Result<AST, ParseError>;
}

// Macro Expander 接口
pub trait MacroExpander {
    fn expand_macro(&mut self, mac: &Macro) -> Result<AST, MacroError>;
}
```

#### 中端接口

```rust
// AST to HIR
pub trait ASTLowerer {
    fn lower_ast(&mut self, ast: &AST) -> Result<HIR, LowerError>;
}

// Type Checker 接口
pub trait TypeChecker {
    fn check(&mut self, hir: &mut HIR) -> Result<TyEnv, TypeError>;
}

// Borrow Checker 接口
pub trait BorrowChecker {
    fn check(&mut self, mir: &MIR) -> Result<(), BorrowError>;
}

// Effect Checker 接口
pub trait EffectChecker {
    fn check(&mut self, mir: &MIR) -> Result<(), EffectError>;
}
```

#### 后端接口

```rust
// Code Generator 接口
pub trait CodeGenerator {
    fn compile(&mut self, mir: &MIR) -> Result<CompiledArtifact, CodeGenError>;
    fn optimize(&mut self, level: OptimizationLevel);
}

// LLVM Backend
pub struct LLVMBackend {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
}

impl CodeGenerator for LLVMBackend {
    fn compile(&mut self, mir: &MIR) -> Result<CompiledArtifact, CodeGenError> {
        // 生成 LLVM IR
    }
}
```

### 数据流

```
源代码 (.zl)
    │
    ▼
┌─────────┐
│ Lexer   │ → Token 流
└─────────┘
    │
    ▼
┌─────────┐
│ Parser  │ → AST
└─────────┘
    │
    ▼
┌────────────┐
│ Macro      │ → 展开后的 AST
│ Expander   │
└────────────┘
    │
    ▼
┌────────────┐
│ AST        │
│ Lower      │ → HIR
└────────────┘
    │
    ├─────────────────┐
    ▼                 ▼
┌──────────┐    ┌──────────┐
│ Type      │    │ Name     │
│ Checker   │    │ Resolver │
└──────────┘    └──────────┘
    │                 │
    └─────────────────┘
    │
    ▼
┌────────────┐
│ Borrow     │
│ Checker    │
└────────────┘
    │
    ▼
┌────────────┐
│ Effect     │
│ Checker    │
└────────────┘
    │
    ▼
┌────────────┐
│ MIR        │
│ Lower      │ → MIR
└────────────┘
    │
    ▼
┌────────────┐
│ Optimizer  │ → 优化后的 MIR
└────────────┘
    │
    ▼
┌────────────┐
│ Code       │
│ Generator  │ → LLVM IR / 机器码
└────────────┘
    │
    ▼
可执行文件
```

---

## 运行时架构

### 运行时组件

```
runtime/
├── memory/           // 内存管理
│   ├── gc/           // GC (可选)
│   ├── arc/          // 引用计数
│   └── arena/        // Arena 分配器
├── thread/           // 线程管理
│   ├── scheduler/    // 任务调度器
│   ├── pool/         // 线程池
│   └── coroutine/    // 协程
├── io/               // IO 运行时
│   ├── async/        // 异步IO
│   ├── event_loop/   // 事件循环
│   └── sync/         // 同步IO
├── effect/           // 效应处理运行时
│   └── handler/      // 效应处理器
└── sync/             // 同步原语
    ├── mutex/        // 互斥锁
    ├── rwlock/       // 读写锁
    └── lockfree/     // 无锁数据结构
```

**非阻塞 IO 和事件循环详细实现**: 请参阅 [RUNTIME_IO.md](RUNTIME_IO.md) 获取完整的非阻塞 IO 架构、事件循环实现（epoll/io_uring/kqueue/IOCP）和性能优化策略。

### 内存管理架构

```
┌────────────────────────────────────────────────┐
│              内存管理子系统                    │
├────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  栈分配   │  │  堆分配   │  │  静态区   │    │
│  └──────────┘  └──────────┘  └──────────┘    │
│       │             │              │          │
│       └─────────────┴──────────────┘          │
│                     │                         │
│                     ▼                         │
│           ┌──────────────────┐                │
│           │   ARC 引用计数   │                │
│           │   (自动管理)      │                │
│           └──────────────────┘                │
│                     │                         │
│                     ▼                         │
│           ┌──────────────────┐                │
│           │   逃逸分析优化    │                │
│           │   (编译期决定)    │                │
│           └──────────────────┘                │
│                                                  │
└────────────────────────────────────────────────┘
```

**内存分配策略**:

```rust
pub enum AllocationStrategy {
    // 栈分配（编译期确定）
    Stack {
        size: usize,
        offset: usize,
    },

    // 堆分配（ARC）
    Heap {
        ref_count: Arc<usize>,
    },

    // 静态分配
    Static {
        address: usize,
    },
}
```

### 线程调度架构

```
┌────────────────────────────────────────────────┐
│               线程调度系统                      │
├────────────────────────────────────────────────┤
│                                                  │
│  ┌─────────────────────────────────────────┐  │
│  │          工作窃取调度器                 │  │
│  │  (Work-Stealing Scheduler)              │  │
│  └─────────────────────────────────────────┘  │
│                      │                         │
│        ┌─────────────┼─────────────┐          │
│        │             │             │          │
│        ▼             ▼             ▼          │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐      │
│  │ Worker1 │  │ Worker2 │  │ WorkerN │      │
│  │ Thread  │  │ Thread  │  │ Thread  │      │
│  └─────────┘  └─────────┘  └─────────┘      │
│        │             │             │          │
│        └─────────────┴─────────────┘          │
│                      │                         │
│                      ▼                         │
│  ┌─────────────────────────────────────────┐  │
│  │          全局任务队列                   │  │
│  │    (Global Task Queue)                  │  │
│  └─────────────────────────────────────────┘  │
│                                                  │
│  ┌─────────────────────────────────────────┐  │
│  │          线程池                          │  │
│  │    (Thread Pool)                        │  │
│  └─────────────────────────────────────────┘  │
│                                                  │
└────────────────────────────────────────────────┘
```

**调度器实现**:

```rust
pub struct ThreadPool {
    pub workers: Vec<Worker>,
    pub task_queue: Arc<Queue<Task>>,
    pub steal_queues: Vec<Arc<StealQueue<Task>>>,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<JoinHandle<()>>,
    pub local_queue: Arc<StealQueue<Task>>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let mut workers = Vec::new();
        let mut steal_queues = Vec::new();

        for id in 0..num_threads {
            let local_queue = Arc::new(StealQueue::new());
            steal_queues.push(local_queue.clone());

            let worker = Worker {
                id,
                thread: Some(spawn_worker(id, local_queue)),
                local_queue,
            };

            workers.push(worker);
        }

        ThreadPool {
            workers,
            task_queue: Arc::new(Queue::new()),
            steal_queues,
        }
    }
}
```

### 异步IO架构

```
┌────────────────────────────────────────────────┐
│            异步IO运行时                         │
├────────────────────────────────────────────────┤
│                                                  │
│  ┌─────────────────────────────────────────┐  │
│  │          事件驱动器                      │  │
│  │  (Event Loop / Reactor)                 │  │
│  └─────────────────────────────────────────┘  │
│                      │                         │
│                      ▼                         │
│  ┌─────────────────────────────────────────┐  │
│  │       IO 事件多路复用                   │  │
│  │  (epoll / kqueue / IOCP)                │  │
│  └─────────────────────────────────────────┘  │
│                      │                         │
│        ┌─────────────┼─────────────┐          │
│        │             │             │          │
│        ▼             ▼             ▼          │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐      │
│  │  网络IO  │  │  文件IO  │  │  定时器  │      │
│  └─────────┘  └─────────┘  └─────────┘      │
│                                                  │
└────────────────────────────────────────────────┘
```

---

## 标准库架构

### 标准库组织

```
std/
├── core/              // 核心类型和trait
│   ├── mod.rs
│   ├── clone.rs
│   ├── copy.rs
│   ├── fmt.rs
│   ├── iter.rs
│   ├── option.rs
│   ├── result.rs
│   └── marker.rs
├── collections/        // 集合类型
│   ├── mod.rs
│   ├── vec.rs
│   ├── hashmap.rs
│   ├── hashset.rs
│   ├── btreemap.rs
│   └── btreeset.rs
├── sync/              // 并发原语
│   ├── mod.rs
│   ├── mutex.rs
│   ├── rwlock.rs
│   ├── condvar.rs
│   ├── barrier.rs
│   ├── atomic.rs
│   ├── arc.rs
│   └── lockfree/
│       ├── queue.rs
│       ├── stack.rs
│       └── hashmap.rs
├── io/                // 输入输出
│   ├── mod.rs
│   ├── fs.rs
│   ├── net/
│   │   ├── tcp.rs
│   │   ├── udp.rs
│   │   └── dns.rs
│   ├── process.rs
│   └── stdio.rs
├── async/             // 异步运行时
│   ├── mod.rs
│   ├── task.rs
│   ├── future.rs
│   ├── stream.rs
│   └── await.rs
├── os/                // 操作系统接口
│   ├── mod.rs
│   ├── linux.rs
│   ├── macos.rs
│   ├── windows.rs
│   └── unix.rs
└── testing/           // 测试框架
    ├── mod.rs
    ├── assert.rs
    ├── bench.rs
    └── mock.rs
```

### 核心Trait层级

```
                ┌──────────┐
                │  Object  │
                └──────────┘
                     │
        ┌────────────┼────────────┐
        │            │            │
   ┌─────┴─────┐ ┌──┴───┐  ┌────┴────┐
   │   Clone   │ │ Copy │  │ Display │
   └───────────┘ └──────┘  └─────────┘
        │
   ┌────┴────────────┐
   │                 │
┌──┴──────┐    ┌─────┴─────┐
│  PartialEq │   │  Eq       │
└───────────┘    └───────────┘
```

### 模块依赖图

```
                    core
                     │
         ┌───────────┼───────────┐
         │           │           │
    collections   sync        io
         │           │           │
         └───────────┼───────────┘
                     │
                  async
                     │
                testing
```

---

## 工具链架构

### YAN 统一工具链

```
toolchain/
├── yan/                 // YAN 统一工具链
│   └── src/
│       ├── main.rs
│       ├── build.rs      // 构建命令
│       ├── run.rs        // 运行命令
│       ├── test.rs       // 测试命令
│       ├── repl.rs       // EFPL/REPL
│       ├── new.rs        // 项目创建
│       ├── fmt.rs        // 格式化
│       ├── doc.rs        // 文档生成
│       └── clean.rs      // 清理工具
├── compiler/            // 编译器核心
│   └── src/
│       ├── driver.rs     // 编译器驱动
│       ├── parser.rs     // 解析器
│       ├── typeck.rs     // 类型检查
│       ├── borrowck.rs   // 借用检查
│       └── codegen.rs    // 代码生成
├── runtime/             // 运行时支持
│   ├── memory/          // 内存管理
│   ├── thread/          // 线程调度
│   ├── io/              // 非阻塞IO
│   └── effect/          // 效应处理
└── lsp/                 // LSP 服务器
    └── src/
        ├── server.rs     // LSP 服务器
        ├── diagnostics.rs // 诊断信息
        ├── completion.rs // 代码补全
        └── hover.rs      // 悬停提示
```

**详细文档**: 请参阅 [YAN_TOOLCHAIN.md](YAN_TOOLCHAIN.md) 获取完整工具链使用指南。

### 编译器驱动架构

```
┌────────────────────────────────────────────────┐
│              编译器驱动 (Driver)                 │
├────────────────────────────────────────────────┤
│                                                  │
│  1. 解析命令行参数                               │
│  2. 加载配置文件                                 │
│  3. 初始化编译器组件                             │
│  4. 执行编译Pipeline                            │
│  5. 调用链接器                                  │
│  6. 输出结果                                     │
│                                                  │
└────────────────────────────────────────────────┘
```

**编译Pipeline**:

```rust
pub struct CompilerDriver {
    pub config: CompilerConfig,
    pub session: CompilerSession,
}

impl CompilerDriver {
    pub fn compile(&mut self, input: &Input) -> Result<Output, CompileError> {
        // 1. 词法分析
        let tokens = self.lex(input)?;

        // 2. 语法分析
        let ast = self.parse(&tokens)?;

        // 3. 宏展开
        let ast = self.expand_macros(&ast)?;

        // 4. AST 到 HIR
        let hir = self.lower_ast(&ast)?;

        // 5. 类型检查
        self.type_check(&mut hir)?;

        // 6. 借用检查
        let mir = self.lower_to_mir(&hir)?;
        self.borrow_check(&mir)?;

        // 7. 效应检查
        self.effect_check(&mir)?;

        // 8. 优化
        let mir = self.optimize(&mir)?;

        // 9. 代码生成
        let artifact = self.codegen(&mir)?;

        // 10. 链接
        let output = self.link(&artifact)?;

        Ok(output)
    }
}
```

### LSP 服务器架构

```
┌────────────────────────────────────────────────┐
│            LSP 服务器架构                       │
├────────────────────────────────────────────────┤
│                                                  │
│  ┌─────────────────────────────────────────┐  │
│  │         LSP 协议处理层                   │  │
│  │  (Protocol Handler)                      │  │
│  └─────────────────────────────────────────┘  │
│                     │                         │
│        ┌────────────┼────────────┐           │
│        │            │            │           │
│        ▼            ▼            ▼           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐      │
│  │ 诊断    │ │ 补全    │ │ 悬停    │      │
│  └─────────┘ └─────────┘ └─────────┘      │
│        │            │            │           │
│        └────────────┼────────────┘           │
│                     ▼                         │
│  ┌─────────────────────────────────────────┐  │
│  │          编译器接口层                   │  │
│  │   (Compiler Interface)                  │  │
│  └─────────────────────────────────────────┘  │
│                     │                         │
│                     ▼                         │
│  ┌─────────────────────────────────────────┐  │
│  │         文件缓存和索引                  │  │
│  │   (File Cache & Index)                 │  │
│  └─────────────────────────────────────────┘  │
│                                                  │
└────────────────────────────────────────────────┘
```

**LSP 服务器实现**:

```rust
pub struct LSPServer {
    pub compiler: Compiler,
    pub document_cache: HashMap<PathBuf, Document>,
    pub symbol_index: SymbolIndex,
}

impl LSPServer {
    pub fn handle_request(&mut self, request: Request) -> Result<Response, Error> {
        match request.method {
            "textDocument/didOpen" => self.handle_did_open(request.params),
            "textDocument/didChange" => self.handle_did_change(request.params),
            "textDocument/completion" => self.handle_completion(request.params),
            "textDocument/hover" => self.handle_hover(request.params),
            "textDocument/definition" => self.handle_definition(request.params),
            "textDocument/references" => self.handle_references(request.params),
            _ => Ok(Response::error("Unknown method")),
        }
    }

    pub fn handle_completion(&self, params: CompletionParams) -> Result<CompletionList, Error> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let document = self.document_cache.get(&uri)
            .ok_or(Error::DocumentNotFound)?;

        // 获取光标位置的上下文
        let context = self.get_completion_context(&document, pos)?;

        // 查询符号索引
        let completions = self.symbol_index.query(&context)?;

        Ok(CompletionList { items: completions })
    }
}
```

---

## 模块间通信

### 编译器内部通信

```rust
// 消息传递机制
pub enum CompilerMessage {
    LexResult(Result<TokenStream, LexerError>),
    ParseResult(Result<AST, ParseError>),
    TypeCheckResult(Result<TyEnv, TypeError>),
    BorrowCheckResult(Result<(), BorrowError>),
}

// 通道通信
pub struct CompilerChannel {
    pub sender: Sender<CompilerMessage>,
    pub receiver: Receiver<CompilerMessage>,
}

impl CompilerChannel {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        CompilerChannel { sender, receiver }
    }
}
```

### LSP 与编译器通信

```rust
// LSP 请求
pub enum LSPRequest {
    CheckFile(PathBuf),
    GetCompletions(PathBuf, Position),
    GetHover(PathBuf, Position),
    GetDefinition(PathBuf, Position),
}

// 编译器响应
pub enum CompilerResponse {
    Diagnostics(Vec<Diagnostic>),
    Completions(Vec<CompletionItem>),
    Hover(HoverInfo),
    Definition(Location),
}

// 双向通信
pub struct CompilerBridge {
    pub lsp_to_compiler: Sender<LSPRequest>,
    pub compiler_to_lsp: Receiver<CompilerResponse>,
}
```

---

## 扩展性设计

### 插件系统

```rust
// 插件接口
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn on_load(&mut self, context: &PluginContext) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;

    fn register_passes(&self, registry: &mut PassRegistry);
    fn register_attributes(&self, registry: &mut AttrRegistry);
}

// 插件上下文
pub struct PluginContext {
    pub config: Config,
    pub compiler: Compiler,
}

// 插件管理器
pub struct PluginManager {
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, path: &Path) -> Result<(), PluginError> {
        let lib = unsafe { Lib::new(path) };
        let create: Symbol<fn() -> Box<dyn Plugin>> = unsafe {
            lib.get(b"create_plugin")?
        };

        let mut plugin = create()?;
        plugin.on_load(&self.context)?;
        self.plugins.push(plugin);

        Ok(())
    }
}
```

### 自定义优化Pass

```rust
// 优化Pass接口
pub trait OptimizationPass: Any {
    fn name(&self) -> &str;
    fn run(&mut self, mir: &mut MIR) -> PassResult;
}

// Pass注册表
pub struct PassRegistry {
    pub passes: HashMap<String, Box<dyn OptimizationPass>>,
}

impl PassRegistry {
    pub fn register<P: OptimizationPass + 'static>(&mut self, pass: P) {
        self.passes.insert(pass.name().to_string(), Box::new(pass));
    }

    pub fn get_pass(&self, name: &str) -> Option<&dyn OptimizationPass> {
        self.passes.get(name).map(|p| p.as_ref())
    }
}

// 示例：内联Pass
pub struct InlinePass {
    pub inline_threshold: usize,
}

impl OptimizationPass for InlinePass {
    fn name(&self) -> &str { "inline" }

    fn run(&mut self, mir: &mut MIR) -> PassResult {
        // 内联小函数
        Ok(PassResult::Changed)
    }
}
```

### 自定义属性

```rust
// 属性接口
pub trait Attribute {
    fn name(&self) -> &str;

    fn on_struct(&self, strukt: &Struct) -> Result<(), AttributeError>;
    fn on_function(&self, func: &Function) -> Result<(), AttributeError>;
    fn on_statement(&self, stmt: &mut Statement) -> Result<(), AttributeError>;
}

// 属性注册
pub struct AttrRegistry {
    pub attrs: HashMap<String, Box<dyn Attribute>>,
}

impl AttrRegistry {
    pub fn register<A: Attribute + 'static>(&mut self, attr: A) {
        self.attrs.insert(attr.name().to_string(), Box::new(attr));
    }
}

// 示例：derive 属性
pub struct DeriveAttribute;

impl Attribute for DeriveAttribute {
    fn name(&self) -> &str { "derive" }

    fn on_struct(&self, strukt: &Struct) -> Result<(), AttributeError> {
        // 为结构体自动实现 trait
        Ok(())
    }
}
```

---

## 部署架构

### 编译器部署

```
┌────────────────────────────────────────────────┐
│            编译器部署方式                       │
├────────────────────────────────────────────────┤
│                                                  │
│  1. 二进制分发                                   │
│     ├── 预编译二进制                             │
│     └── 系统包管理器 (apt, brew, chocolatey)   │
│                                                  │
│  2. 源码编译                                     │
│     ├── 从源码构建                               │
│     └── 跨平台编译                               │
│                                                  │
│  3. 容器化部署                                   │
│     ├── Docker 镜像                             │
│     └── OCI 兼容格式                             │
│                                                  │
│  4. 云端服务                                     │
│     ├── WASM 编译服务                           │
│     └── Playground                              │
│                                                  │
└────────────────────────────────────────────────┘
```

### CI/CD 集成

```
┌────────────────────────────────────────────────┐
│          CI/CD Pipeline 架构                    │
├────────────────────────────────────────────────┤
│                                                  │
│  1. 代码提交                                     │
│     └──> Git Push                               │
│                                                  │
│  2. 触发 CI                                     │
│     └──> GitHub Actions / GitLab CI             │
│                                                  │
│  3. 构建和测试                                   │
│     ├── 编译器自举编译                           │
│     ├── 标准库编译                               │
│     └── 测试套件运行                             │
│                                                  │
│  4. 构建产物                                     │
│     ├── 编译器二进制                             │
│     ├── 标准库文档                               │
│     └── 性能基准测试结果                         │
│                                                  │
│  5. 部署                                         │
│     ├── 发布到 GitHub Releases                  │
│     ├── 推送到包管理器                           │
│     └── 更新 Playground                         │
│                                                  │
└────────────────────────────────────────────────┘
```

**GitHub Actions 配置**:

```yaml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]

    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y llvm-dev clang

      - name: Build compiler
        run: |
          cargo build --release

      - name: Run tests
        run: |
          cargo test --all

      - name: Build stdlib
        run: |
          ./zc --build-stdlib

      - name: Run benchmarks
        run: |
          cargo bench

  wasm:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Build WASM
        run: |
          cargo build --target wasm32-unknown-unknown

      - name: Deploy to playground
        run: |
          ./deploy-playground.sh
```

---

**文档版本**: 1.0
**最后更新**: 2026-01-07
**维护者**: ZULON Language Team
