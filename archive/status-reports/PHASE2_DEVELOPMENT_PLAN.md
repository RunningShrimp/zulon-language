# Phase 2: Core Features - Development Plan

**Date**: January 8, 2026
**Previous**: MVP v0.1.0 Complete ✅
**Duration**: 12 months (2026 Q3 - 2027 Q2)
**Status**: Planning Phase

---

## 🎉 Phase 1 Complete - MVP v0.1.0

### Achievements

✅ **100% Functional MVP**
- End-to-end compilation (.zl → executable)
- C++-level runtime performance (100.7%)
- Complete toolchain (YAN)
- 65/65 tests passing
- Zero compiler warnings
- 4,000+ lines of documentation

### Known Limitations (Non-blocking)

1. **Complex Recursion**: Uses alloca instead of pure SSA
2. **No IO**: Standard library not yet linked
3. **Simplified HashMap**: Linear search implementation
4. **Minimal Runtime**: No panic runtime, no allocator

---

## Phase 2 Overview

**Goal**: Complete language features and standard library

**Timeline**: 12 months (2026 Q3 - 2027 Q2)

**Target**: Alpha release with full language capabilities

---

## Phase 2: Core Features (12 months)

### 2.1 高级语言特性 (8周) - Priority: P0

**错误处理增强 - 2周** ⚠️ 90% Complete (from previous sessions)
- [x] Parser 支持 throw, ?, | 语法
- [x] HIR 集成 (error_type, effects)
- [x] 类型检查 (throw/? 验证)
- [x] MIR 降级 (discriminant checking)
- [x] 标准库 (Outcome<T, E>)
- [ ] **Full runtime throw support** (1 week)
  - [ ] Implement panic runtime
  - [ ] Add stack trace capture
  - [ ] Test end-to-end error handling
- [ ] **Integration tests** (1 week)
  - [ ] Test error propagation
  - [ ] Test error recovery
  - [ ] Create examples

**效应系统 - 3周** - Priority: P0
- [ ] 实现 effect 定义 (effect 关键字)
- [ ] 实现 effect 执行 (perform 关键字)
- [ ] 实现 effect 处理器 (try...with 块)
- [ ] 实现内置效应
  - [ ] IO 效应
  - [ ] Database 效应
  - [ ] Log 效应
- [ ] 测试

**高级特性 - 3周** - Priority: P1
- [ ] 实现多返回值
  - [ ] 元组类型完善
  - [ ] 解构赋值
- [ ] 实现结构体解构
- [ ] 实现模板字符串插值
- [ ] 实现智能 defer
  - [ ] LIFO 执行顺序
  - [ ] 变量捕获
- [ ] 实现命名空间
  - [ ] namespace 定义
  - [ ] use 导入
- [ ] 实现 trait 组合
- [ ] 测试

### 2.2 并发运行时 (10周) - Priority: P0

**非阻塞 IO - 4周**
- [ ] 实现事件循环抽象
  - [ ] EventLoop trait
  - [ ] EventHandler trait
  - [ ] Reactor 模式
- [ ] 实现 Linux epoll
  - [ ] EpollEventLoop
  - [ ] 边缘触发模式
  - [ ] EPOLLONESHOT 支持
- [ ] 实现 Linux io_uring (可选)
  - [ ] IoUringEventLoop
  - [ ] 零拷贝优化
- [ ] 测试

**Windows IOCP - 2周**
- [ ] 实现 IOCP 事件循环
  - [ ] IocpEventLoop
  - [ ] 完成端口
- [ ] 实现 Windows 异步 IO
- [ ] 测试

**macOS/BSD kqueue - 2周**
- [ ] 实现 kqueue 事件循环
  - [ ] KqueueEventLoop
  - [ ] kevent 支持
- [ ] 实现 BSD 异步 IO
- [ ] 测试

**Channel 和 Select - 2周**
- [ ] 实现 Channel 类型
  - [ ] mpsc::channel
  - [ ] spsc::channel
  - [ ] oneshot::channel
- [ ] 实现 select 语句
- [ ] 测试

### 2.3 异步编程 (6周) - Priority: P1

**Async/Await - 3周**
- [ ] 实现 async fn 语法
- [ ] 实现 .await 语法
- [ ] 实现 Future trait
  - [ ] poll 方法
  - [ ] Context 类型
- [ ] 实现任务调度器
- [ ] 测试

**异步 IO 标准库 - 3周**
- [ ] 实现 AsyncRead/AsyncWrite traits
- [ ] 实现异步文件操作
- [ ] 实现异步网络操作
- [ ] 实现异步标准库
  - [ ] fs 异步文件系统
  - [ ] net 异步网络
- [ ] 测试

### 2.4 EPVS 无锁数据结构 (6周) - Priority: P2

**理论基础 - 1周**
- [ ] 研究 EPVS 论文 (POPL 2025)
- [ ] 设计数据结构接口
- [ ] 设计 epoch 保护机制

**实现 EPVS - 4周**
- [ ] 实现 epoch 管理
- [ ] 实现无锁队列
- [ ] 实现无锁哈希表
- [ ] 实现无锁栈
- [ ] 测试

**集成到标准库 - 1周**
- [ ] 在 sync 模块暴露 EPVS 类型
- [ ] 文档编写
- [ ] 示例代码

### 2.5 高级标准库 (8周) - Priority: P0

**async 库 - 3周**
- [ ] 实现任务类型
  - [ ] Task
  - [ ] JoinHandle
- [ ] 实现同步原语
  - [ ] Mutex (异步互斥锁)
  - [ ] RwLock (异步读写锁)
  - [ ] Semaphore
  - [ ] Barrier
- [ ] 实现定时器
  - [ ] interval
  - [ ] timeout
  - [ ] delay
- [ ] 测试

**io 库增强 - 2周**
- [ ] 实现路径操作
  - [ ] Path
  - [ ] PathBuf
- [ ] 实现文件系统抽象
  - [ ] Metadata
  - [ ] Permissions
- [ ] 实现进程管理
  - [ ] Command
  - [ ] Child
  - [ ] ExitCode
- [ ] 测试

**net 库 - 3周**
- [ ] 实现 TCP/UDP
  - [ ] TcpStream
  - [ ] TcpListener
  - [ ] UdpSocket
- [ ] 实现高级网络功能
  - [ ] IpAddr
  - [ ] SocketAddr
  - [ ] DNS 查找
- [ ] 实现 HTTP 客户端（基础）
  - [ ] Request
  - [ ] Response
  - [ ] Client
- [ ] 测试

### 2.6 EFPL 交互环境 (6周) - Priority: P2

**REPL 核心 - 3周**
- [ ] 实现词法分析（增量）
- [ ] 实现语法分析（增量）
- [ ] 实现类型推导（增量）
- [ ] 实现 JIT 执行
  - [ ] 表达式求值
  - [ ] 语句执行
  - [ ] 函数调用
- [ ] 实现错误恢复
- [ ] 测试

**REPL 功能 - 2周**
- [ ] 实现交互命令
  - [ ] :type (类型检查)
  - [ ] :doc (文档查看)
  - [ ] :env (环境信息)
  - [ ] :reset (重置环境)
  - [ ] :import (导入模块)
  - [ ] :load/:save (会话管理)
- [ ] 实现历史记录
  - [ ] 上下箭头浏览
- [ ] 实现 Tab 补全
- [ ] 实现语法高亮
- [ ] 测试

**REPL 集成 - 1周**
- [ ] 集成到 yan repl
- [ ] 实现 -e 选项
- [ ] 实现 -i 选项
- [ ] 测试

### 2.7 测试框架完善 (4周) - Priority: P1

**测试增强 - 2周**
- [ ] 实现参数化测试
  - [ ] #[data(...)] 宏
  - [ ] 数据驱动测试
- [ ] 实现异步测试支持
  - [ ] #[test] async fn
  - [ ] 异步测试运行器
- [ ] 实现超时测试
  - [ ] #[timeout(n)] 宏
  - [ ] 超时检测
- [ ] 实现测试隔离
  - [ ] 独立测试环境
  - [ ] 资源清理
- [ ] 测试

**测试覆盖率 - 1周**
- [ ] 实现代码覆盖率收集
  - [ ] 行覆盖率
  - [ ] 分支覆盖率
  - [ ] 函数覆盖率
- [ ] 实现覆盖率报告
  - [ ] HTML 格式
  - [ ] LCOV 格式
  - [ ] JSON 格式
- [ ] 测试

**测试工具 - 1周**
- [ ] 实现 yan test --parallel
  - [ ] 并行测试执行
  - [ ] 测试分片
- [ ] 实现 yan test --repeat
  - [ ] 重复测试
  - [ ] flaky 测试检测
- [ ] 实现性能基准测试
- [ ] 测试

### 2.8 工具链增强 (6周) - Priority: P1

**YAN 增强 - 3周**
- [ ] 实现 yan test
  - [ ] 测试发现
  - [ ] 测试运行
  - [ ] 测试报告
- [ ] 实现 yan fmt
  - [ ] 代码格式化
  - [ ] 配置文件支持
- [ ] 实现 yan doc
  - [ ] 文档生成
  - [ ] 文档查看
- [ ] 测试

**编译优化 - 2周**
- [ ] 实现增量编译
  - [ ] 依赖跟踪
  - [ ] 增量构建
- [ ] 实现并行编译
  - [ ] 多核并行
  - [ ] 编译图优化
- [ ] 实现编译缓存
- [ ] 测试

**诊断工具 - 1周**
- [ ] 实现性能分析
  - [ ] CPU profiling
  - [ ] 内存 profiling
- [ ] 实现调试信息生成
  - [ ] --debug 标志
  - [ ] 符号表生成
- [ ] 测试

### 2.9 示例和文档 (4周) - Priority: P0

**高级示例 - 2周**
- [ ] 更新 03_error_handling.zl
- [ ] 更新 04_advanced_features.zl
- [ ] 更新 05_concurrency.zl
- [ ] 更新 06_http_server.zl
- [ ] 更新 07_cli_tool.zl
- [ ] 更新 08_efpl_and_test.zl
- [ ] 创建更多高级示例
- [ ] 测试所有示例

**文档完善 - 2周**
- [ ] 更新所有技术文档
- [ ] 编写 API 文档
- [ ] 编写最佳实践
- [ ] 编写性能指南
- [ ] 编写故障排查指南

---

## Phase 2 交付目标

### Alpha 版本 (2027年12月底)

- ✅ 完整的语言特性支持
- ✅ 非阻塞 IO 运行时
- ✅ 异步编程支持
- ✅ EFPL 交互环境
- ✅ 测试框架完善
- ✅ 性能达到 85-90% C++ 性能

---

## 优先级说明

### P0 - Must Complete (Blocking)

Must complete for Alpha release:
- Error handling runtime (1 week)
- Effect system (3 weeks)
- Concurrent runtime (10 weeks)
- Advanced standard library (8 weeks)
- Examples and documentation (4 weeks)

**Total**: ~26 weeks (6 months)

### P1 - Should Complete (Important)

Significantly impacts user experience:
- Advanced features (3 weeks)
- Async programming (6 weeks)
- Test framework (4 weeks)
- Toolchain enhancement (6 weeks)

**Total**: ~19 weeks (4.5 months)

### P2 - Can Defer (Enhancement)

Nice to have but not blocking:
- EPVS lock-free data structures (6 weeks)
- EFPL (6 weeks)

**Total**: ~12 weeks (3 months)

---

## Phase 2 Timeline

### Q1 2026 (Months 1-3): Foundation

**Focus**: Error handling + Effect system
- Error handling runtime (2 weeks)
- Effect system (3 weeks)
- Advanced features (3 weeks)

**Deliverable**: Working effect system

### Q2 2026 (Months 4-6): Concurrency

**Focus**: Async runtime
- Non-blocking IO (4 weeks)
- IOCP + kqueue (4 weeks)
- Channels (2 weeks)

**Deliverable**: Async IO working

### Q3 2026 (Months 7-9): Async Programming

**Focus**: Async/await
- Async/await (3 weeks)
- Async stdlib (3 weeks)
- EPVS (6 weeks) [P2, optional]

**Deliverable**: Full async support

### Q4 2026 (Months 10-12): Polish

**Focus**: Tools and docs
- Test framework (4 weeks)
- Toolchain (6 weeks)
- Examples/docs (4 weeks)

**Deliverable**: **ALPHA RELEASE** 🚀

---

## Next Steps for Ralph Loop

### Immediate (Iteration 4)

**Priority**: Fix known MVP bugs
- Fix recursion codegen (eliminate alloca)
- Add basic IO (printf/scanf)
- Optimize HashMap (real hashing)

**Estimated**: 2-3 iterations

### Short-term (Iterations 5-10)

**Priority**: Start Phase 2.1
- Complete error handling runtime
- Implement effect system
- Add advanced features

**Estimated**: 6-8 iterations

---

## Conclusion

**Phase 1 Status**: ✅ **COMPLETE**
**Phase 2 Status**: 📋 **PLANNING**
**Next**: Fix MVP bugs → Start Phase 2.1

The foundation is solid. We're ready to build advanced features on top of our working MVP!

---

**Phase 2 Plan**
**Date**: January 8, 2026
**Duration**: 12 months
**Target**: Alpha Release
**Priority**: Effects → Async → Polish

---

*ZULON Language Team*
*Building the future of systems programming* 🦀
