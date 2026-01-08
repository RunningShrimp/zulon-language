# ZULON 语言示例代码

欢迎使用 ZULON 编程语言示例代码集合！这些示例将帮助您快速了解和学习 ZULON 语言的各种特性。

## 目录

- [快速开始](#快速开始)
- [示例列表](#示例列表)
- [学习路径](#学习路径)
- [运行示例](#运行示例)
- [常见问题](#常见问题)

## 快速开始

### 前置要求

1. 安装 ZULON 工具链 `yan`
2. 确保 `yan` 在系统 PATH 中
3. 推荐使用支持 ZULON 的代码编辑器（如 VS Code + ZULON 扩展）

### 安装工具链

```bash
# macOS/Linux
curl -sSL https://get.zulon-lang.sh | sh

# 或使用包管理器
cargo install zulon-lang
```

### 运行第一个示例

```bash
# 进入示例目录
cd examples

# 运行 Hello World
yan run 00_hello_world.zl

# 或编译后运行
yan build 00_hello_world.zl -o hello
./hello
```

## 示例列表

### 00 - Hello World (00_hello_world.zl)

**难度**: ⭐
**时间**: 2 分钟

第一个 ZULON 程序，了解基本语法结构。

**包含内容**:
- `main()` 函数定义
- `println!()` 宏输出
- 函数调用
- 中文字符串支持

**运行**:
```bash
yan run 00_hello_world.zl
```

**学习要点**:
- ZULON 程序从 `main()` 函数开始
- 使用 `println!` 输出文本（`!` 表示宏）
- 字符串使用 UTF-8 编码，原生支持中文

---

### 01 - 基础语法 (01_basics.zl)

**难度**: ⭐⭐
**时间**: 10 分钟

演示 ZULON 的变量、类型、运算符和控制流。

**包含内容**:
- 变量和常量声明
- 基本数据类型（整数、浮点、布尔、字符、字符串）
- 算术、比较、逻辑、位运算符
- if-else 条件语句
- loop、while、for 循环
- 数组和迭代

**运行**:
```bash
zc run 01_basics.zl
```

**学习要点**:
- `let` 声明变量，`let mut` 声明可变变量
- `const` 声明编译时常量
- 类型注解可选（如 `let x: i32 = 42`）
- `loop` 无限循环需要 `break` 退出
- `for..in` 用于范围迭代和集合遍历

---

### 02 - 类型系统 (02_types.zl)

**难度**: ⭐⭐⭐
**时间**: 15 分钟

展示 ZULON 的类型系统：结构体、枚举、trait 和泛型。

**包含内容**:
- 结构体（struct）定义和方法
- 枚举（enum）与数据关联
- Trait 定义和实现
- 泛型函数和结构体
- 模式匹配（match）
- Option 类型处理空值

**运行**:
```bash
zc run 02_types.zl
```

**学习要点**:
- 结构体使用 `{}` 初始化，字段名简写（`{name, age}`）
- `impl` 块为类型添加方法
- `&self` 表示借用 self，`&mut self` 表示可变借用
- 枚举可以携带数据（类似 Rust 的 ADT）
- Trait 定义共享行为，支持多态
- 泛型使用 `<T>` 语法，可用 trait bounds 约束

**代码示例**:
```go
struct Person {
    name: str,
    age: i32,
}

impl Person {
    fn new(name: str, age: i32) -> Person {
        return Person { name, age }
    }

    fn greet(&self) {
        println!("你好，我是 {}", self.name)
    }
}
```

---

### 03 - 错误处理 (03_error_handling.zl)

**难度**: ⭐⭐⭐⭐
**时间**: 20 分钟

演示 ZULON 的 Result 类型、throw 关键字和代数效应。

**包含内容**:
- 自定义错误类型（自动实现 Error trait）
- Result<T, E> 类型
- `?` 运算符传播错误
- `throw` 关键字抛出错误
- 效应（Effect）定义
- `perform` 关键字执行效应
- `|` 分隔符标记返回值、错误和效应
- `try...with` 处理效应

**运行**:
```bash
zc run 03_error_handling.zl
```

**学习要点**:
- 所有 `error` 类型自动实现 Error trait
- 使用 `|` 分隔符同时标注返回值、错误和效应
- `throw` 替代 `return Err(...)`
- `perform` 调用效应操作
- 效应处理器用 `try...with` 块实现

**代码示例**:
```go
// 自定义错误
error MathError {
    DivisionByZero,
    NegativeNumber,
}

// 使用 | 标记错误
fn divide(a: i32, b: i32) -> i32 | MathError {
    if b == 0 {
        throw MathError::DivisionByZero
    }
    return a / b
}

// 效应定义
effect IO {
    fn read_line() -> str
    fn print_line(line: str)
}

// 使用效应
fn greet() -> str | IO {
    perform print_line("请输入名字:")
    let name = perform read_line()
    return format!("你好，{}！", name)
}

// 处理效应
try {
    greet()
} with IO {
    fn read_line() -> str {
        return "Alice"  // 模拟输入
    }
    fn print_line(line: str) {
        println!("{}", line)
    }
}
```

---

### 04 - 高级特性 (04_advanced_features.zl)

**难度**: ⭐⭐⭐⭐
**时间**: 25 分钟

展示 ZULON 的高级语言特性。

**包含内容**:
- 多返回值（Go 风格）
- 结构体解构赋值（JavaScript 风格）
- 多行模板字符串（JavaScript 风格）
- Defer 语句（Go/Swift 风格）
- 命名空间（C++/Python 风格）
- Trait 组合继承（Go 风格）

**运行**:
```bash
zc run 04_advanced_features.zl
```

**学习要点**:
- 多返回值使用元组 `(T1, T2, ...)`
- 解构使用 `Struct { field1, field2 }` 语法
- 模板字符串用反引号 `` ` `` 和 `${}` 插值
- `defer` 语句在函数返回前执行（LIFO 顺序）
- `namespace` 组织代码，支持嵌套
- Trait 组合用 `type Trait = A + B + C`

**代码示例**:
```go
// 多返回值
fn divide_and_remainder(a: i32, b: i32) -> (i32, i32) {
    return (a / b, a % b)
}

let (quotient, remainder) = divide_and_remainder(10, 3)

// 结构体解构
struct Point { x: f64, y: f64 }
let Point { x, y } = point

// 模板字符串
let message = `Hello, ${name}! You are ${age} years old.`

// Defer
fn process_file() {
    let file = open()?
    defer {
        file.close()  // 函数返回前自动执行
    }
    // 处理文件...
}

// 命名空间
namespace math {
    pub fn square(x: f64) -> f64 {
        return x * x
    }
}

use math::square
println!("5² = {}", square(5.0))
```

---

### 05 - 并发编程 (05_concurrency.zl)

**难度**: ⭐⭐⭐⭐⭐
**时间**: 30 分钟

演示 ZULON 的并发和并行编程能力。

**包含内容**:
- EPVS 无锁数据结构（POPL 2025）
- 结构化并发（Go 风格任务组）
- Actor 模型（消息传递并发）
- Async/Await 异步编程
- 并行迭代器
- 无锁队列和哈希表

**运行**:
```bash
zc run 05_concurrency.zl
```

**学习要点**:
- EPVS 实现无锁、无等待的数据结构
- `TaskGroup` 管理并发任务的生命周期
- `spawn_actor` 创建 Actor，使用消息通信
- `async/await` 用于非阻塞 I/O
- `par_iter()` 实现数据并行处理

**代码示例**:
```go
// EPVS 无锁向量
use std::sync::epvs::{EpochGuard, EpsVector}

let vector = EpsVector::<i32>::new()
let guard = EpochGuard::new()

spawn {
    for i in 0..100 {
        vector.push(i, &guard)
    }
}

// 结构化并发
let group = TaskGroup::new()
for url in urls {
    group.spawn {
        download(url)
    }
}
for task in group {
    let result = task.await
}

// Actor
actor Counter {
    count: i32,
    on message => {
        match message {
            CounterMessage::Increment => {
                self.count = self.count + 1
            },
            // ...
        }
    }
}

// Async/Await
async fn fetch_user(id: i32) -> Result<User> {
    let response = http_get(&url).await?
    return Ok(parse_user(&response))
}

// 并行迭代器
let result: Vec<i32> = numbers
    .par_iter()
    .map(|x| x * 2)
    .collect()
```

---

### 06 - HTTP 服务器 (06_http_server.zl)

**难度**: ⭐⭐⭐⭐⭐
**时间**: 30 分钟

构建一个生产级 RESTful API 服务器。

**包含内容**:
- TCP 网络编程
- HTTP 协议解析
- 路由系统
- JSON 序列化/反序列化
- 异步请求处理
- 错误处理和响应

**运行**:
```bash
zc run 06_http_server.zl

# 测试 API
curl http://localhost:8080/health
curl http://localhost:8080/api/users
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'
```

**学习要点**:
- 使用 `TcpListener` 创建 TCP 服务器
- 手动解析 HTTP 请求格式
- 路由匹配和处理器分发
- 使用 `serde_json` 处理 JSON
- `Arc<Mutex<T>>` 实现线程安全的共享状态

**代码示例**:
```go
struct Server {
    address: str,
    router: Router,
}

impl Server {
    fn new(address: str) -> Server {
        return Server {
            address,
            router: Router::new(),
        }
    }

    fn get(&mut self, path: str, handler: Handler) {
        self.router.get(path, handler)
    }

    fn run(&self) | IoError {
        let listener = TcpListener::bind(self.address)?
        for stream in listener.incoming() {
            self.handle_connection(stream?)
        }
        return Ok(())
    }
}

// 使用
let mut server = Server::new("0.0.0.0:8080")
server.get("/api/users", handle_get_users)
server.post("/api/users", handle_create_user)
server.run()
```

---

### 07 - CLI 工具 (07_cli_tool.zl)

**难度**: ⭐⭐⭐⭐
**时间**: 25 分钟

实现一个实用的命令行文件处理工具。

**包含内容**:
- 命令行参数解析
- 文件读写操作
- 文本搜索（grep）
- 排序和去重
- 文本替换
- 文件分析
- 格式转换

**运行**:
```bash
yan run 07_cli_tool.zl count examples/00_hello_world.zl
yan run 07_cli_tool.zl grep "println" examples/01_basics.zl -n
yan run 07_cli_tool.zl sort data.txt -o sorted.txt
yan run 07_cli_tool.zl analyze examples/02_types.zl
yan run 07_cli_tool.zl replace "old" "new" input.txt -o output.txt
```

**学习要点**:
- 使用 `std::env::args()` 获取命令行参数
- `HashMap` 存储选项和标志
- `File` 和 `BufReader` 高效读取文件
- 字符串处理和模式匹配
- 错误处理和用户友好的错误消息

**代码示例**:
```go
struct CliArgs {
    command: Command,
    input_file: str,
    output_file: Option<str>,
    options: HashMap<str, str>,
}

fn parse_args() -> Result<CliArgs, CliError> {
    let argv = args().collect::<Vec<String>>()
    // 解析逻辑...
}

fn main() {
    match parse_args() {
        Ok(args) => {
            match &args.command {
                Command::Count => handle_count(&args),
                Command::Grep { .. } => handle_grep(&args),
                // ...
            }
        },
        Err(e) => {
            eprintln!("错误: {:?}", e)
            print_usage()
        },
    }
}
```

---

### 08 - EFPL 和测试 (08_efpl_and_test.zl)

**难度**: ⭐⭐⭐
**时间**: 20 分钟

演示 ZULON 的 EFPL 交互环境和测试框架。

**包含内容**:
- EFPL 交互式执行环境
- 测试框架使用
- 参数化测试
- 异步测试
- 超时测试
- 非阻塞 IO 示例
- Channel 和并发

**运行**:
```bash
# 运行程序
yan run 08_efpl_and_test.zl

# 运行测试
yan test 08_efpl_and_test.zl

# 显示测试覆盖率
yan test --coverage 08_efpl_and_test.zl

# 启动 EFPL 环境
yan efpl

# 在 EFPL 中加载本文件
yan efpl -i 08_efpl_and_test.zl

# 执行单个表达式
yan efpl -e "factorial(5)"
```

**EFPL 交互示例**:
```bash
$ yan efpl
ZULON EFPL v1.0

>>> 1 + 2
3

>>> fn square(n: i32) -> i32 { n * n }
fn square(i32) -> i32

>>> square(5)
25

>>> :type square
fn(i32) -> i32

>>> :quit
```

**测试示例**:
```go
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5)
}

#[test]
async fn test_async_operation() {
    let result = async_add(2, 3).await
    assert_eq!(result, 5)
}

#[test]
#[timeout(1000)]
fn test_slow_operation() {
    // 1秒超时
}
```

**学习要点**:
- EFPL 提供类似 Python REPL 的交互体验
- 支持表达式求值、函数定义、模块导入
- 测试框架使用 `#[test]` 宏
- 支持异步测试和参数化测试
- 非阻塞 IO 在 Linux/io_uring、Windows/IOCP、macOS/kqueue 下自动选择最优实现

---

### 07 - CLI 工具 (07_cli_tool.zl)

**难度**: ⭐⭐⭐⭐
**时间**: 25 分钟

实现一个实用的命令行文件处理工具。

**包含内容**:
- 命令行参数解析
- 文件读写操作
- 文本搜索（grep）
- 排序和去重
- 文本替换
- 文件分析
- 格式转换

**运行**:
```bash
zc run 07_cli_tool.zl count examples/00_hello_world.zl
zc run 07_cli_tool.zl grep "println" examples/01_basics.zl -n
zc run 07_cli_tool.zl sort data.txt -o sorted.txt
zc run 07_cli_tool.zl analyze examples/02_types.zl
zc run 07_cli_tool.zl replace "old" "new" input.txt -o output.txt
```

**学习要点**:
- 使用 `std::env::args()` 获取命令行参数
- `HashMap` 存储选项和标志
- `File` 和 `BufReader` 高效读取文件
- 字符串处理和模式匹配
- 错误处理和用户友好的错误消息

**代码示例**:
```go
struct CliArgs {
    command: Command,
    input_file: str,
    output_file: Option<str>,
    options: HashMap<str, str>,
}

fn parse_args() -> Result<CliArgs, CliError> {
    let argv = args().collect::<Vec<String>>()
    // 解析逻辑...
}

fn main() {
    match parse_args() {
        Ok(args) => {
            match &args.command {
                Command::Count => handle_count(&args),
                Command::Grep { .. } => handle_grep(&args),
                // ...
            }
        },
        Err(e) => {
            eprintln!("错误: {:?}", e)
            print_usage()
        },
    }
}
```

## 学习路径

### 初级（编程新手）

1. **00_hello_world.zl** - 了解基本结构
2. **01_basics.zl** - 学习变量、类型和控制流
3. **02_types.zl** - 理解结构体和枚举

**预计时间**: 30-45 分钟

### 中级（有编程经验）

1. 完成"初级"所有示例
2. **03_error_handling.zl** - 掌握错误处理
3. **04_advanced_features.zl** - 学习高级特性
4. **07_cli_tool.zl** - 实践项目开发

**预计时间**: 2-3 小时

### 高级（系统程序员）

1. 完成"中级"所有示例
2. **05_concurrency.zl** - 深入并发编程
3. **06_http_server.zl** - 网络编程实践
4. 阅读 [ZULON 语言设计文档](../docs/ZULON_LANGUAGE_INTEGRATED_DESIGN.md)

**预计时间**: 4-6 小时

## 运行示例

### 方式 1: 直接运行（推荐用于学习）

```bash
yan run <example-file>
```

**优点**:
- 快速迭代
- 自动编译和运行
- 适合调试

### 方式 2: 编译后运行（推荐用于生产）

```bash
yan build <example-file> -o <output-name>
./<output-name>
```

**优点**:
- 启动更快
- 可以分发二进制文件
- 性能更好

### 方式 3: 发布模式（最高性能）

```bash
yan build <example-file> -o <output-name> --release
./<output-name>
```

**优点**:
- 启用所有优化
- 最小化二进制大小
- 适合性能测试

### 方式 4: EFPL 交互式执行

```bash
# 启动交互环境
yan efpl

# 执行单个表达式
yan efpl -e "println!(2 + 2)"

# 加载文件后进入交互模式
yan efpl -i 00_hello_world.zl
```

**优点**:
- 即时反馈
- 适合学习和实验
- 支持表达式求值

### 方式 5: 运行测试

```bash
# 运行所有测试
yan test

# 运行特定测试文件
yan test tests/test_math.zl

# 显示测试覆盖率
yan test --coverage

# 并行运行测试
yan test --parallel
```

## 常见问题

### Q: 编译时报错 "undefined reference to LLVM"

**A**: 确保安装了 LLVM 15.0 或更高版本：

```bash
# macOS
brew install llvm@15

# Ubuntu/Debian
sudo apt-get install llvm-15-dev

# Arch Linux
sudo pacman -S llvm
```

### Q: 示例运行时卡住

**A**: 某些示例（如 HTTP 服务器、并发示例）会持续运行。按 `Ctrl+C` 终止。

### Q: 中文显示乱码

**A**: 确保终端使用 UTF-8 编码：

```bash
# macOS/Linux
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Windows PowerShell
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
```

### Q: 如何调试 ZULON 程序？

**A**: 使用 `--debug` 标志编译调试版本：

```bash
yan build <example-file> -o <output-name> --debug
gdb ./<output-name>
# 或
lldb ./<output-name>
```

### Q: YAN 工具链支持哪些非阻塞 IO 机制？

**A**: YAN 默认使用平台最优的非阻塞 IO 机制：
- **Linux**: io_uring (Linux 5.1+) 或 epoll
- **Windows**: IOCP (I/O Completion Ports)
- **macOS/BSD**: kqueue

这些机制对用户透明，所有异步操作自动使用最优实现。

### Q: 如何使用 EFPL 进行交互式开发？

**A**: EFPL 提供类似 Python REPL 的交互环境：

```bash
# 启动 EFPL
yan efpl

# 执行表达式
>>> 1 + 2
3

# 定义函数
>>> fn square(n: i32) -> i32 { n * n }
fn square(i32) -> i32

# 调用函数
>>> square(5)
25

# 查看类型
>>> :type square
fn(i32) -> i32

# 查看文档
>>> :doc Vec
Vector - 动态数组类型

# 导入模块
>>> :import std::math

# 退出
>>> :quit
```

### Q: 如何编写和运行测试？

**A**: 使用 `#[test]` 宏标记测试函数：

```go
#[test]
fn test_addition() {
    assert_eq!(1 + 2, 3)
}

#[test]
async fn test_async() {
    let result = fetch_data().await
    assert!(result.len() > 0)
}
```

运行测试：
```bash
# 运行所有测试
yan test

# 运行特定测试
yan test --test "test_addition"

# 显示覆盖率
yan test --coverage

# 并行运行
yan test --parallel
```

### Q: 示例中的 `?` 运算符是什么？

**A**: `?` 是错误传播运算符，相当于：

```go
// 使用 ?
let result = some_function()?

// 等价于
let result = match some_function() {
    Ok(value) => value,
    Err(e) => return Err(e),
}
```

### Q: `perform` 和 `throw` 有什么区别？

**A**:
- `throw` 用于抛出错误（提前返回）
- `perform` 用于执行效应（可被处理器拦截）

```go
// throw: 立即返回错误
fn validate(x: i32) -> i32 | Error {
    if x < 0 {
        throw Error::Invalid  // 返回 Err
    }
    return x
}

// perform: 可被处理器捕获
effect IO {
    fn print(s: str)
}

fn main() {
    try {
        perform print("Hello")  // 被 try...with 捕获
    } with IO {
        fn print(s: str) {
            println!("实际输出: {}", s)
        }
    }
}
```

## 进阶资源

### 官方文档

- [ZULON 语言设计文档](../docs/ZULON_LANGUAGE_INTEGRATED_DESIGN.md) - 完整语言规范
- [技术详细设计](../docs/TECHNICAL_DESIGN.md) - 编译器实现细节
- [架构设计](../docs/ARCHITECTURE.md) - 系统架构
- [技术选型](../docs/TECHNOLOGY_SELECTION.md) - 技术决策

### 白皮书

- [ZULON 语言白皮书](../docs/ZULON_WHITEPAPER.md) - 技术理念和愿景

### 社区

- GitHub: https://github.com/zulon-lang/zulon
- Discord: https://discord.gg/zulon-lang
- 论坛: https://forum.zulon-lang.org

### 贡献

欢迎贡献示例代码！请查看 [CONTRIBUTING.md](../CONTRIBUTING.md)

## 许可证

所有示例代码采用 MIT 许可证。详见 [LICENSE](../LICENSE)

---

**开始你的 ZULON 之旅吧！** 🚀

如有问题，欢迎在 [GitHub Issues](https://github.com/zulon-lang/zulon/issues) 提问。
