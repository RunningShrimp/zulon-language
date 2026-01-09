# ZULON 快速开始指南

**欢迎来到ZULON编程语言！** 🎉

本指南将帮助你在5分钟内运行第一个ZULON程序，并了解语言的核心特性。

---

## 📦 第一步：安装

### 前置要求

在开始之前，请确保你的系统已安装：

- **Rust** 1.92.0 或更高版本
- **LLVM** 工具链（clang, lld）
- **C 编译器**（gcc 或 clang）

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/zulon-lang/zulon
cd zulon

# 构建发布版本
cargo build --release

# YAN工具现在可用
./target/release/yan --version
```

**预期输出**:
```
yan 0.1.0
```

---

## 🚀 第二步：创建你的第一个项目

ZULON提供了 `yan new` 命令来创建新项目：

```bash
# 创建新项目
yan new my_first_zulon

# 进入项目目录
cd my_first_zulon

# 查看项目结构
ls -la
```

**项目结构**:
```
my_first_zulon/
├── src/
│   └── main.zl          # 主程序源文件
├── Cargo.toml           # 项目配置
└── README.md            # 项目说明
```

---

## 💻 第三步：编写你的第一个程序

打开 `src/main.zl`，你会看到：

```zulon
// ZULON程序的入口点

fn main() {
    println!("Hello, ZULON!")
}
```

让我们修改它，添加更多功能：

```zulon
// my_first_zulon/src/main.zl

fn main() {
    // 打印欢迎消息
    println!("🎉 欢迎来到ZULON！")

    // 调用自定义函数
    greet_user("ZULON开发者")

    // 演示变量绑定
    let x = 42
    println!("数字: {}", x)

    // 演示Optional类型
    let maybe_value = Optional::Some(100)
    match maybe_value {
        Optional::Some(v) => println!("值: {}", v),
        Optional::None => println!("无值"),
    }
}

// 自定义问候函数
fn greet_user(name: String) -> String {
    let greeting = "你好, ".to_string() + name + "!"
    println!("{}", greeting)
    greeting
}
```

---

## 🔨 第四步：构建和运行

### 构建项目

```bash
# 使用YAN构建项目
yan build
```

**预期输出**:
```
🔨 Building project...
   Compiling...
✅ Build successful!
```

### 运行程序

```bash
# 运行编译后的程序
yan run
```

**预期输出**:
```
🎉 欢迎来到ZULON！
你好, ZULON开发者!
数字: 42
值: 100
```

---

## 📚 第五步：学习核心特性

### 1. 变量和类型

ZULON支持类型推导，你可以显式指定或让编译器推导：

```zulon
// 显式类型
let count: i32 = 42

// 类型推导
let name = "ZULON"  // 推导为 String
let pi = 3.14159     // 推导为 f64
```

### 2. 集合类型

ZULON标准库提供了丰富的集合：

```zulon
// 使用Vec（动态数组）
let numbers = Vec::new()
numbers.push(1)
numbers.push(2)
numbers.push(3)
println!("数组: {:?}", numbers)

// 使用HashMap
let mut map = HashMap::new()
map.insert("key", "value")
println!("映射: {:?}", map)
```

### 3. 函数和返回值

```zulon
// 带返回类型的函数
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

// 多返回值（使用元组）
fn divide(a: i32, b: i32) -> (i32, Optional<String>) {
    if b == 0 {
        return (0, Optional::Some("除零错误".to_string()))
    }
    return (a / b, Optional::None)
}
```

### 4. 错误处理

ZULON使用 `Outcome<T, E>` 类型进行错误处理：

```zulon
// 可能失败的函数
fn parse_number(s: String) -> Outcome<i32, String> {
    // 尝试解析
    match s.parse::<i32>() {
        Ok(n) => Outcome::Ok(n),
        Err(_) => Outcome::Err("无法解析数字".to_string()),
    }
}

// 使用?
fn process(s: String) -> Outcome<i32, String> {
    let n = parse_number(s)?  // 自动错误传播
    return Outcome::Ok(n * 2)
}
```

### 5. 结构体和枚举

```zulon
// 定义结构体
struct Point {
    x: f64,
    y: f64,
}

// 定义枚举
enum Option<T> {
    Some(T),
    None,
}

// 使用
let p = Point { x: 1.0, y: 2.0 }
println!("Point: ({}, {})", p.x, p.y)
```

---

## 🎯 第六步：探索更多

### 示例程序

ZULON包含了丰富的示例程序来展示各种特性：

```bash
# 在ZULON仓库中
cd examples

# 查看所有示例
ls *.zl

# 尝试运行基础示例
cd ../
yan build --example basics
```

**可用示例**:
- `00_hello_world.zl` - Hello World
- `01_basics.zl` - 基础语法
- `02_types.zl` - 类型系统
- `03_error_handling.zl` - 错误处理
- `04_advanced_features.zl` - 高级特性
- `05_concurrency.zl` - 并发模式
- `06_http_server.zl` - HTTP服务器
- `07_cli_tool.zl` - CLI工具

### 文档资源

- **完整文档**: https://docs.zulon-lang.org
- **API参考**: https://docs.zulon-lang.org/api
- **教程**: https://docs.zulon-lang.org/tutorials
- **社区**: https://discord.gg/zulon

---

## 🔧 高级构建选项

### Release模式（优化）

```bash
# 使用优化编译
yan build --release
```

这会启用 `-O2` 优化，性能提升约46%。

### 并行编译

```bash
# 使用8个并行任务
yan build --jobs 8
```

### 清理构建产物

```bash
# 清理所有构建产物
yan clean
```

---

## 🐛 常见问题

### Q: 编译时出现LLVM错误

**A**: 确保已安装LLVM工具：
```bash
# macOS
brew install llvm

# Ubuntu/Debian
sudo apt-get install llvm-dev clang

# 验证安装
llvm-as --version
llc --version
```

### Q: 找不到标准库类型

**A**: 确保在文件顶部导入了需要的模块：
```zulon
use std::collections::Vec
use std::collections::HashMap
```

### Q: 如何调试程序

**A**: 使用 `yan build --verbose` 查看详细编译信息：
```bash
yan build --verbose
```

---

## 📖 下一步

恭喜！你已经成功运行了第一个ZULON程序！

**推荐学习路径**:

1. **基础语法** → 阅读 `01_basics.zl`
2. **类型系统** → 阅读 `02_types.zl`
3. **错误处理** → 阅读 `03_error_handling.zl`
4. **高级特性** → 阅读 `04_advanced_features.zl`
5. **实战项目** → 尝试 `06_http_server.zl`

**参与社区**:
- 🌟 GitHub: https://github.com/zulon-lang/zulon
- 💬 Discord: https://discord.gg/zulon
- 🐦 Twitter: @zulon_lang
- 📧 Email: hello@zulon-lang.org

---

## 🎊 享受ZULON！

你现在已经准备好开始你的ZULON之旅了！

**记住**:
- ✅ ZULON是安全的（内存安全，类型安全）
- ✅ ZULON是快速的（接近C++性能）
- ✅ ZULON是现代的（清晰的语法，强大的工具）
- ✅ ZULON正在快速发展（活跃的社区）

**祝你编码愉快！** 🚀

---

**快速入门指南 v1.0** | **ZULON Language Team** | **2026-01-08**
