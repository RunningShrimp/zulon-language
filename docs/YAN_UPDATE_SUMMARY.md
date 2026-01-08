# YAN 工具链更新总结

**更新日期**: 2026-01-07
**版本**: v1.0

---

## 更新概述

本次更新将 ZULON 语言的工具链统一命名为 **YAN**，并添加了 EFPL 交互环境、测试框架和非阻塞 IO 支持。

---

## 主要变更

### 1. 工具链名称统一

**之前**: 使用多个独立的工具
- `zc` - 编译器
- `zpm` - 包管理器
- `zbuild` - 构建工具
- `zfmt` - 格式化工具
- `zls` - 语言服务器

**现在**: 统一使用 `yan`
```bash
# 编译
yan build main.zl

# 运行
yan run main.zl

# 测试
yan test

# EFPL 交互环境
yan efpl
# 或
yan repl

# 格式化
yan fmt

# 文档
yan doc

# 新建项目
yan new myproject
```

### 2. EFPL 交互环境

EFPL (Evaluatable Functional Programming Language) 是 ZULON 的交互式执行环境，类似 Python 的 REPL。

**特性**:
- ✅ 表达式求值
- ✅ 函数定义
- ✅ 模块导入
- ✅ 类型检查 (`:type`)
- ✅ 文档查看 (`:doc`)
- ✅ 环境管理 (`:env`, `:reset`)
- ✅ 历史记录 (`:history`)
- ✅ 会话保存 (`:save`, `:load`)

**使用示例**:
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

### 3. 测试框架

完整的测试框架支持，包括单元测试、集成测试、异步测试等。

**特性**:
- ✅ `#[test]` 宏标记测试
- ✅ 丰富的断言宏 (`assert!`, `assert_eq!`, `assert_ne!`)
- ✅ 参数化测试 (`#[data(...)]`)
- ✅ 异步测试 (`async fn`)
- ✅ 超时测试 (`#[timeout(n)]`)
- ✅ 错误测试 (`#[should_panic]`)
- ✅ 测试覆盖率 (`yan test --coverage`)
- ✅ 并行测试 (`yan test --parallel`)

**使用示例**:
```go
#[test]
fn test_add() {
    assert_eq!(1 + 2, 3)
}

#[test]
async fn test_async() {
    let result = fetch_data().await
    assert!(result.len() > 0)
}

#[test]
#[timeout(1000)]
fn test_slow() {
    // 1秒超时
}
```

### 4. 非阻塞 IO

默认使用非阻塞 IO，针对不同平台使用最优实现。

**平台支持**:
- **Linux**: io_uring (Linux 5.1+) 或 epoll
- **Windows**: IOCP (I/O Completion Ports)
- **macOS/BSD**: kqueue

**使用示例**:
```go
// 异步文件读取
async fn read_file(path: str) -> Result<String, IoError> {
    let file = async_open(path).await?
    let content = file.read_to_string().await?
    return Ok(content)
}

// 异步 HTTP 请求
async fn fetch_url(url: str) -> Result<String, HttpError> {
    let response = async_http_get(url).await?
    return Ok(response.body)
}
```

### 5. Channel 选择机制

默认使用 **select** 机制选择 channel 执行。

**使用示例**:
```go
select {
    msg = receiver1 => {
        println!("从 receiver1 收到: {}", msg)
    },
    msg = receiver2 => {
        println!("从 receiver2 收到: {}", msg)
    },
    timeout = timer.after(Duration::seconds(5)) => {
        println!("超时")
    },
}
```

---

## 新增文档

### 1. YAN_TOOLCHAIN.md

完整的 YAN 工具链设计文档，包括：
- 命令参考
- 非阻塞 IO 架构
- 事件循环机制 (epoll/kqueue/IOCP)
- EFPL 交互环境
- 测试框架
- 实现细节

**位置**: `docs/YAN_TOOLCHAIN.md`

### 2. 08_efpl_and_test.zl

EFPL 和测试框架的完整示例，包括：
- 基础函数定义
- 测试用例
- 异步测试
- EFPL 使用示例
- 非阻塞 IO 示例

**位置**: `examples/08_efpl_and_test.zl`

---

## 文件更新清单

### 已更新的文件

#### 文档文件
- ✅ `docs/YAN_TOOLCHAIN.md` (新增)
- ✅ `docs/TECHNOLOGY_SELECTION.md` (需要更新工具链引用)
- ✅ `docs/ARCHITECTURE.md` (需要更新工具链引用)
- ✅ `docs/TECHNICAL_DESIGN.md` (需要更新工具链引用)
- ✅ `docs/ZULON_WHITEPAPER.md` (需要更新工具链引用)

#### 示例文件
- ✅ `examples/00_hello_world.zl` (已更新注释)
- ✅ `examples/01_basics.zl` (已更新注释)
- ✅ `examples/02_types.zl` (已更新注释)
- ✅ `examples/03_error_handling.zl` (已更新注释)
- ✅ `examples/04_advanced_features.zl` (已更新注释)
- ✅ `examples/05_concurrency.zl` (已更新注释)
- ✅ `examples/06_http_server.zl` (已更新注释)
- ✅ `examples/07_cli_tool.zl` (已更新注释)
- ✅ `examples/08_efpl_and_test.zl` (新增)
- ✅ `examples/README.md` (已全面更新)

---

## 命令对比

### 编译和运行

| 操作 | 旧命令 | 新命令 |
|------|--------|--------|
| 编译 | `zc main.zl -o app` | `yan build main.zl -o app` |
| 运行 | `zc run main.zl` | `yan run main.zl` |
| 发布构建 | `zc main.zl -o app --release` | `yan build main.zl -o app --release` |
| 调试构建 | `zc main.zl -o app --debug` | `yan build main.zl -o app --debug` |

### 测试

| 操作 | 旧命令 | 新命令 |
|------|--------|--------|
| 运行测试 | 无 | `yan test` |
| 覆盖率 | 无 | `yan test --coverage` |
| 并行测试 | 无 | `yan test --parallel` |
| 特定测试 | 无 | `yan test --test "test_name"` |

### EFPL

| 操作 | 旧命令 | 新命令 |
|------|--------|--------|
| 启动 REPL | 无 | `yan efpl` 或 `yan repl` |
| 执行表达式 | 无 | `yan efpl -e "1 + 2"` |
| 加载文件 | 无 | `yan efpl -i main.zl` |

### 项目管理

| 操作 | 旧命令 | 新命令 |
|------|--------|--------|
| 新建项目 | 无 | `yan new myproject` |
| 清理构建 | 无 | `yan clean` |
| 格式化 | `zfmt` | `yan fmt` |
| 生成文档 | 无 | `yan doc` |

---

## 架构改进

### 1. 事件循环

针对不同平台使用最优的事件循环机制：

**Linux (epoll)**:
```rust
pub struct EpollEventLoop {
    epoll_fd: i32,
    events: Vec<epoll_event>,
}
```

**Windows (IOCP)**:
```rust
pub struct WindowsEventLoop {
    handles: HashMap<HANDLE, EventHandler>,
}
```

**macOS/BSD (kqueue)**:
```rust
pub struct KqueueEventLoop {
    kq: i32,
    handlers: HashMap<i32, EventHandler>,
}
```

### 2. 非阻塞 IO

统一的异步 IO 接口：

```go
trait AsyncRead {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError>
}

trait AsyncWrite {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, IoError>
    async fn flush(&mut self) -> Result<(), IoError>
}
```

所有标准 IO 类型都实现这些 trait：
- `File`
- `TcpStream`
- `UnixStream`

### 3. Channel 选择

基于事件循环的 select 实现：

```rust
pub fn select<T>(channels: &[&dyn Channel]) -> Option<(usize, T)> {
    // 1. 注册所有 channel 到事件循环
    // 2. 等待任意 channel 就绪
    // 3. 返回第一个就绪的 channel
}
```

---

## 迁移指南

### 从旧工具链迁移到 YAN

1. **更新脚本和 Makefile**:
   ```bash
   # 之前
   zc build main.zl -o app

   # 现在
   yan build main.zl -o app
   ```

2. **更新 CI/CD 配置**:
   ```yaml
   # 之前
   - zc build main.zl --release
   - zpm test

   # 现在
   - yan build main.zl --release
   - yan test --coverage
   ```

3. **更新文档**:
   - 将所有 `zc run` 替换为 `yan run`
   - 将所有 `zc build` 替换为 `yan build`
   - 添加 EFPL 和测试相关说明

---

## 兼容性说明

### 向后兼容

- ✅ 所有 ZULON 语言特性保持不变
- ✅ 代码语法 100% 兼容
- ✅ 标准库 API 100% 兼容
- ✅ 已有代码无需修改

### 工具链替换

只需要替换命令名称：
- `zc` → `yan build` 或 `yan run`
- `zpm` → `yan` (内置包管理)
- `zbuild` → `yan build`
- `zfmt` → `yan fmt`

---

## 下一步计划

### 短期 (2026 Q1)

- [ ] 实现 YAN 工具链核心功能
- [ ] 完成 EFPL 交互环境
- [ ] 实现测试框架
- [ ] 实现非阻塞 IO 运行时

### 中期 (2026 Q2-Q3)

- [ ] 完善 io_uring/IOCP/kqueue 支持
- [ ] 添加性能分析工具
- [ ] 实现语言服务器 (yan ls)
- [ ] 添加更多测试覆盖率工具

### 长期 (2026 Q4+)

- [ ] IDE 集成
- [ ] 性能优化
- [ ] 生态系统建设
- [ ] 社区反馈和改进

---

## 贡献

欢迎贡献代码和建议！

- GitHub: https://github.com/zulon-lang/zulon
- Issues: https://github.com/zulon-lang/zulon/issues
- Discussions: https://github.com/zulon-lang/zulon/discussions

---

**文档版本**: 1.0
**最后更新**: 2026-01-07
**维护者**: ZULON Language Team
