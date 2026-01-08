# YAN 工具链使用指南

**版本**: 1.0
**日期**: 2026-01-07
**状态**: 使用指南

---

## 目录

1. [概述](#概述)
2. [快速开始](#快速开始)
3. [命令参考](#命令参考)
4. [配置文件](#配置文件)
5. [最佳实践](#最佳实践)
6. [常见问题](#常见问题)

---

## 概述

YAN 是 ZULON 语言的统一工具链，提供从开发到部署的完整工具支持。

### 核心功能

- **构建**: 编译 ZULON 代码为可执行文件
- **运行**: 直接运行 ZULON 文件
- **测试**: 运行单元测试和集成测试
- **EFPL**: 交互式执行环境（类似 Python REPL）
- **包管理**: 依赖管理和项目构建
- **格式化**: 代码格式化
- **文档**: 生成和查看文档

### 设计理念

- **统一入口**: 所有功能通过 `yan` 命令访问
- **开发友好**: 提供交互式开发和快速测试能力
- **性能优先**: 默认优化和增量编译

---

## 快速开始

### 安装

```bash
# macOS/Linux
curl -sSL https://get.zulon-lang.sh | sh

# 使用包管理器
cargo install zulon-lang

# 验证安装
yan --version
```

### 第一个程序

```bash
# 创建项目
yan new hello_world
cd hello_world

# 运行
yan run

# 测试
yan test

# 构建
yan build --release
```

---

## 命令参考

### yan build - 构建项目

编译 ZULON 代码为可执行文件。

```bash
# 编译单个文件
yan build main.zl

# 指定输出文件
yan build main.zl -o myapp

# 发布模式（优化编译）
yan build main.zl --release

# 调试模式（包含调试信息）
yan build main.zl --debug

# 增量编译
yan build main.zl --incremental

# 并行编译
yan build main.zl --parallel

# 显示编译时间
yan build main.zl --timings
```

**选项**:
- `-o, --output <file>`: 指定输出文件名
- `--release`: 发布模式，启用所有优化
- `--debug`: 调试模式，包含符号信息
- `--incremental`: 增量编译
- `--parallel <n>`: 并行编译（n=0表示使用所有核心）
- `--timings`: 显示每个编译阶段的时间
- `--target <triple>`: 目标平台三元组
- `--features <features>`: 启用特定功能特性
- `--no-default-features`: 禁用默认功能

**目标平台**:
```bash
# Linux x86_64
yan build main.zl --target x86_64-unknown-linux-gnu

# macOS ARM64
yan build main.zl --target aarch64-apple-darwin

# Windows x86_64
yan build main.zl --target x86_64-pc-windows-msvc

# WebAssembly
yan build main.zl --target wasm32-unknown-unknown
```

### yan run - 运行程序

直接运行 ZULON 文件。

```bash
# 运行 ZULON 文件
yan run main.zl

# 传递参数
yan run main.zl arg1 arg2

# 设置环境变量
yan run main.zl --env VAR=value

# 运行前检查
yan run main.zl --check

# 热重载（开发模式）
yan run main.zl --watch

# 性能分析
yan run main.zl --profile

# 内存分析
yan run main.zl --mem-profile
```

**选项**:
- `--env <key=value>`: 设置环境变量
- `--check`: 仅类型检查，不执行
- `--watch`: 监听文件变化自动重新运行
- `--profile`: 生成性能分析报告
- `--mem-profile`: 生成内存分析报告
- `--verbose`: 显示详细运行信息

### yan test - 运行测试

运行单元测试和集成测试。

```bash
# 运行所有测试
yan test

# 运行特定测试文件
yan test tests/test_math.zl

# 运行匹配模式的测试
yan test --pattern "test_*"

# 并行运行测试
yan test --parallel

# 显示测试输出
yan test --verbose

# 显示测试覆盖率
yan test --coverage

# 生成覆盖率报告
yan test --coverage --report

# 运行特定测试
yan test --test "test_addition"

# 失败时停止
yan test --fail-fast

# 重复运行测试
yan test --repeat 10
```

**选项**:
- `--pattern <pattern>`: 只运行匹配模式的测试
- `--parallel <n>`: 并行运行测试
- `-v, --verbose`: 显示详细测试输出
- `-c, --coverage`: 生成测试覆盖率
- `--report <format>`: 覆盖率报告格式（html/lcov/json）
- `--test <name>`: 运行特定测试
- `--fail-fast`: 第一个失败时停止
- `--repeat <n>`: 重复运行n次

**覆盖率报告**:
```bash
# 生成 HTML 覆盖率报告
yan test --coverage --report html

# 生成 LCOV 报告
yan test --coverage --report lcov

# 在浏览器中查看
yan test --coverage --report html --open
```

### yan repl / yan efpl - 交互式环境

启动 EFPL (Evaluatable Functional Programming Language) 交互环境。

```bash
# 启动交互环境
yan repl
# 或
yan efpl

# 执行表达式
yan repl -e "println!(2 + 2)"

# 执行文件后进入交互模式
yan repl --interactive main.zl

# 加载模块
yan repl --import std::math
yan repl --import mymodule

# 启用实验性功能
yan repl --experimental

# 设置提示符
yan repl --prompt ">>> "
```

**选项**:
- `-e, --eval <code>`: 执行表达式并退出
- `-i, --interactive <file>`: 执行文件后进入交互模式
- `--import <module>`: 启动时导入模块
- `--experimental`: 启用实验性功能
- `--prompt <prompt>`: 自定义提示符

**交互命令**:
```bash
>>> # 表达式求值
>>> 1 + 2
3

>>> # 变量定义
>>> let x = 10
10

>>> # 函数定义
>>> fn square(n: i32) -> i32 { n * n }
fn square(i32) -> i32

>>> # 调用函数
>>> square(5)
25

>>> # 类型检查
>>> :type square
fn(i32) -> i32

>>> # 查看文档
>>> :doc square
计算平方

>>> # 查看环境
>>> :env
Variables:
  x: i32 = 10

>>> # 导入模块
>>> :import std::math

>>> # 重置环境
>>> :reset

>>> # 退出
>>> :quit
```

### yan new - 创建新项目

创建新的 ZULON 项目。

```bash
# 创建新项目
yan new myproject

# 指定项目类型
yan new myproject --type binary
yan new myproject --type library

# 使用模板
yan new myproject --template http-server
yan new myproject --template cli-tool

# 初始化 Git
yan new myproject --git

# 创建示例代码
yan new myproject --example
```

**选项**:
- `--type <type>`: 项目类型（binary/library）
- `--template <name>`: 使用项目模板
- `--git`: 初始化 Git 仓库
- `--example`: 包含示例代码

**可用模板**:
- `binary` - 可执行程序（默认）
- `library` - 库项目
- `http-server` - HTTP 服务器
- `cli-tool` - 命令行工具
- `wasm-app` - WebAssembly 应用

### yan clean - 清理构建产物

清理构建目录和缓存。

```bash
# 清理构建目录
yan clean

# 清理依赖缓存
yan clean --deps

# 清理所有（包括目标文件）
yan clean --all

# 显示将要删除的文件
yan clean --dry-run
```

**选项**:
- `--deps`: 清理依赖缓存
- `--all`: 清理所有（包括目标文件）
- `--dry-run`: 显示将要删除的文件，不实际删除

### yan fmt - 格式化代码

格式化 ZULON 代码。

```bash
# 格式化当前目录
yan fmt

# 格式化指定文件
yan fmt src/main.zl

# 检查格式（不修改）
yan fmt --check

# 递归格式化
yan fmt --recursive

# 显示修改的文件
yan fmt --verbose
```

**选项**:
- `--check`: 仅检查格式，不修改文件
- `--recursive`: 递归格式化子目录
- `--verbose`: 显示修改的文件

**配置文件** (yan.toml):
```toml
[format]
max_width = 100
indent_width = 4
tab_spaces = 4
hard_tabs = false
trailing_comma = true
reorder_imports = true
```

### yan doc - 生成文档

生成和查看项目文档。

```bash
# 生成文档
yan doc

# 打开文档（默认浏览器）
yan doc --open

# 生成特定项目的文档
yan doc --project myproject

# 包含私有项
yan doc --private

# 指定输出格式
yan doc --format html
yan doc --format json
```

**选项**:
- `--open`: 生成后在浏览器中打开
- `--project <name>`: 指定项目名称
- `--private`: 包含私有项的文档
- `--format <format>`: 输出格式（html/json）

---

## 配置文件

### yan.toml

项目配置文件，位于项目根目录。

```toml
[package]
name = "myproject"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
description = "A short description"
license = "MIT"

[dependencies]
zulon-core = "0.1.0"
zulon-async = "0.1.0"

[dev-dependencies]
zulon-test = "0.1.0"

[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.release]
opt-level = 3
lto = true
strip = true
panic = "abort"

[profile.test]
opt-level = 0
debug = true

[format]
max_width = 100
indent_width = 4

[[bin]]
name = "myapp"
path = "src/main.zl"
```

### 配置选项

**[package]**
- `name`: 项目名称
- `version`: 版本号（语义化版本）
- `authors`: 作者列表
- `description`: 项目描述
- `license`: 许可证

**[dependencies]**
- 项目依赖包

**[dev-dependencies]**
- 开发依赖（仅用于测试）

**[profile.*]**
- 编译配置
- `opt-level`: 优化级别 (0-3)
- `debug`: 包含调试信息
- `lto`: 链接时优化
- `strip`: 移除符号表
- `incremental`: 增量编译

---

## 最佳实践

### 项目结构

推荐的项目目录结构：

```
myproject/
├── yan.toml              # 项目配置
├── README.md              # 项目说明
├── LICENSE                # 许可证
├── src/                   # 源代码
│   ├── main.zl
│   ├── lib.zl
│   └── utils/
├── tests/                 # 测试
│   ├── test_main.zl
│   └── integration/
├── examples/              # 示例
├── docs/                  # 文档
└── target/                # 构建输出（自动生成）
    ├── debug/
    └── release/
```

### 开发工作流

```bash
# 1. 创建项目
yan new myproject
cd myproject

# 2. 开发阶段
# 使用 yan run 快速迭代
yan run

# 使用 watch 模式自动重载
yan run --watch

# 3. 测试
# 运行所有测试
yan test

# 运行特定测试
yan test --test "test_function"

# 查看覆盖率
yan test --coverage --report html --open

# 4. 格式化代码
yan fmt

# 5. 构建
# 调试构建
yan build

# 发布构建
yan build --release

# 6. 文档
yan doc --open
```

### 性能优化

```bash
# 使用发布模式
yan build --release

# 启用 LTO（链接时优化）
# 在 yan.toml 中配置：
# [profile.release]
# lto = true

# PGO（Profile-Guided Optimization）
yan build --pgo

# 交叉编译到目标平台
yan build --target x86_64-unknown-linux-gnu --release
```

### CI/CD 集成

**GitHub Actions 示例**:
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v3

      - name: Install YAN
        run: curl -sSL https://get.zulon-lang.sh | sh

      - name: Build
        run: yan build --release

      - name: Test
        run: yan test --coverage

      - Name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## 常见问题

### Q: yan 和之前的 zc 有什么区别？

**A**: `yan` 是统一的工具链，整合了之前 `zc`（编译器）、`zpm`（包管理）、`zbuild`（构建）等所有功能。

### Q: 如何切换工具链版本？

**A**:
```bash
# 查看当前版本
yan --version

# 安装特定版本
curl -sSL https://get.zulon-lang.sh | sh -s -- --version 0.1.0

# 使用 rustup 安装多个版本
yan toolchain install 0.1.0
yan toolchain install 0.2.0

# 切换版本
yan default 0.2.0
```

### Q: 如何设置环境变量？

**A**:
```bash
# 方式1: 命令行
yan run main.zl --env VAR=value

# 方式2: .env 文件
# 创建 .env 文件：
VAR=value
ANOTHER_VAR=another_value

# yan 会自动加载 .env 文件
```

### Q: 如何调试编译错误？

**A**:
```bash
# 显示详细编译信息
yan build main.zl --verbose

# 生成中间文件用于调试
yan build main.zl --debug --emit-ir

# 使用 EFPL 测试片段
yan efpl -e "your_code_here"
```

### Q: 如何加速编译？

**A**:
```bash
# 增量编译
yan build --incremental

# 并行编译
yan build --parallel

# 使用缓存
yan build --cache

# 减少依赖
# 在 yan.toml 中只添加必要的依赖
```

### Q: 如何贡献代码？

**A**: 请访问 [GitHub](https://github.com/zulon-lang/zulon) 查看 CONTRIBUTING.md。

### Q: 获取更多帮助？

**A**:
```bash
# 查看命令帮助
yan --help
yan build --help
yan test --help

# 访问文档
yan doc --open

# 社区支持
- GitHub: https://github.com/zulon-lang/zulon
- Discord: https://discord.gg/zulon-lang
- 论坛: https://forum.zulon-lang.org
```

---

## 技术文档

对于工具链的内部实现和技术架构，请参考：

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 系统架构和非阻塞IO设计
- **[TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md)** - 编译器和运行时实现
- **[TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md)** - 技术选型

---

**文档版本**: 1.0
**最后更新**: 2026-01-07
**维护者**: ZULON Language Team
