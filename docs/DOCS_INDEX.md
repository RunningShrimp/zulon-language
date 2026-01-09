# ZULON 文档索引

**版本**: v0.1.0
**更新日期**: 2026-01-08

欢迎使用ZULON编程语言文档！本索引帮助你快速找到所需的文档。

---

## 📚 快速导航

### 新用户入门
1. **[快速开始指南](QUICK_START_GUIDE.md)** ⭐ 必读
   - 5分钟运行第一个ZULON程序
   - 安装和构建指南
   - 基础语法介绍

### 学习语言
2. **[语言特性详解](LANGUAGE_FEATURES.md)**
   - 完整的类型系统说明
   - 函数和控制流
   - 错误处理和模式匹配
   - 高级特性

3. **[最佳实践指南](BEST_PRACTICES.md)**
   - 代码风格规范
   - 性能优化技巧
   - 安全性建议
   - 常见陷阱避免

### 示例代码
4. **[示例程序集合](../examples/README.md)**
   - 10个完整示例
   - 从Hello World到HTTP服务器
   - 涵盖所有核心特性

---

## 📖 按主题分类

### 入门教程

| 文档 | 描述 | 时间 | 难度 |
|------|------|------|------|
| [快速开始](QUICK_START_GUIDE.md) | 安装、构建、运行第一个程序 | 5分钟 | ⭐ |
| [Hello World示例](../examples/00_hello_world.zl) | 最简单的ZULON程序 | 2分钟 | ⭐ |
| [基础语法](../examples/01_basics.zl) | 变量、运算符、控制流 | 10分钟 | ⭐⭐ |
| [类型系统](../examples/02_types.zl) | 基本类型、集合、类型推导 | 15分钟 | ⭐⭐ |

### 核心概念

| 文档 | 描述 | 时间 | 难度 |
|------|------|------|------|
| [类型系统](LANGUAGE_FEATURES.md#类型系统) | 静态类型、类型推导、类型转换 | - | ⭐⭐ |
| [错误处理](LANGUAGE_FEATURES.md#错误处理) | Outcome类型、?运算符、throw | - | ⭐⭐⭐ |
| [模式匹配](LANGUAGE_FEATURES.md#模式匹配) | match表达式、解构、守卫 | - | ⭐⭐⭐ |
| [Trait系统](LANGUAGE_FEATURES.md#trait系统) | Trait定义、实现、trait bound | - | ⭐⭐⭐⭐ |

### 高级特性

| 文档 | 描述 | 时间 | 难度 |
|------|------|------|------|
| [高级特性](LANGUAGE_FEATURES.md#高级特性) | 字符串插值、defer、多返回值 | - | ⭐⭐⭐ |
| [内存管理](LANGUAGE_FEATURES.md#内存管理) | Arc、Weak、所有权 | - | ⭐⭐⭐⭐ |
| [高级示例](../examples/04_advanced_features.zl) | 模板字符串、defer、方法链 | 20分钟 | ⭐⭐⭐⭐ |
| [错误处理示例](../examples/03_error_handling.zl) | 完整的错误处理演示 | 15分钟 | ⭐⭐⭐ |

### 实战项目

| 文档 | 描述 | 时间 | 难度 |
|------|------|------|------|
| [HTTP服务器](../examples/06_http_server.zl) | 构建简单的HTTP服务器 | 30分钟 | ⭐⭐⭐⭐ |
| [CLI工具](../examples/07_cli_tool.zl) | 命令行工具开发 | 25分钟 | ⭐⭐⭐⭐ |
| [并发模式](../examples/05_concurrency.zl) | 并发编程模式 | 30分钟 | ⭐⭐⭐⭐⭐ |

### 代码质量

| 文档 | 描述 | 适用对象 |
|------|------|----------|
| [最佳实践](BEST_PRACTICES.md) | 编码规范、性能优化、安全 | 所有开发者 |
| [代码风格](BEST_PRACTICES.md#代码风格) | 命名规范、格式化、注释 | 所有开发者 |
| [性能优化](BEST_PRACTICES.md#性能优化) | 避免常见性能陷阱 | 中高级开发者 |
| [安全性](BEST_PRACTICES.md#安全性) | 输入验证、类型安全 | 所有开发者 |

---

## 🎯 学习路径

### 路径1: 快速上手（1小时）

适合想快速了解ZULON的开发者：

1. 阅读 [快速开始指南](QUICK_START_GUIDE.md) - **10分钟**
2. 运行 [Hello World](../examples/00_hello_world.zl) - **5分钟**
3. 学习 [基础语法](../examples/01_basics.zl) - **15分钟**
4. 了解 [类型系统](../examples/02_types.zl) - **15分钟**
5. 阅读 [最佳实践](BEST_PRACTICES.md) - **15分钟**

**完成目标**: 能够编写简单的ZULON程序

### 路径2: 系统学习（1天）

适合想要深入学习的开发者：

**上午** (4小时):
- 快速开始指南 - **30分钟**
- 语言特性详解（类型、函数、控制流）- **1小时**
- 错误处理和模式匹配 - **1.5小时**
- 运行基础示例（00-03）- **1小时**

**下午** (4小时):
- Trait系统和高级特性 - **1.5小时**
- 内存管理（Arc、Weak）- **1小时**
- 运行高级示例（04-05）- **1小时**
- 最佳实践和代码质量 - **30分钟**

**完成目标**: 全面掌握ZULON v0.1.0特性

### 路径3: 项目实战（1周）

适合准备在项目中使用ZULON的开发者：

**Day 1-2**: 系统学习（路径2）
**Day 3**: 深入最佳实践
- 阅读完整 [最佳实践指南](BEST_PRACTICES.md)
- 理解性能优化策略
- 学习安全性考虑

**Day 4-5**: 实战项目
- HTTP服务器示例 ([06_http_server.zl](../examples/06_http_server.zl))
- CLI工具示例 ([07_cli_tool.zl](../examples/07_cli_tool.zl))
- 完整语言游览 ([complete_tour.zl](../examples/complete_tour.zl))

**Day 6-7**: 项目实践
- 构建自己的小项目
- 应用最佳实践
- 编写测试

**完成目标**: 能够独立使用ZULON开发项目

---

## 📋 文档清单

### 用户文档（v0.1.0）

✅ **已完成**:
- [快速开始指南](QUICK_START_GUIDE.md) - 6步快速上手
- [语言特性详解](LANGUAGE_FEATURES.md) - 完整特性说明
- [最佳实践指南](BEST_PRACTICES.md) - 编码规范和优化
- [示例程序集合](../examples/README.md) - 10个完整示例

⏳ **规划中**:
- API参考手册
- 调试指南
- 性能调优指南
- FFI（外部函数接口）文档

### 技术文档

✅ **已完成**:
- [架构设计文档](../ARCHITECTURE.md)
- [技术设计文档](../docs/TECHNICAL_DESIGN.md)
- [错误处理设计](../docs/ERROR_HANDLING_DESIGN.md)
- [HIR快速参考](../docs/HIR_QUICK_REFERENCE.md)
- [YAN工具链](../docs/YAN_TOOLCHAIN.md)

⏳ **规划中**:
- 编译器内部文档
- 运行时设计文档
- 标准库设计文档

---

## 🔍 按问题查找

### "我想开始使用ZULON"
→ [快速开始指南](QUICK_START_GUIDE.md)

### "如何处理错误？"
→ [语言特性 - 错误处理](LANGUAGE_FEATURES.md#错误处理)
→ [错误处理示例](../examples/03_error_handling.zl)
→ [最佳实践 - 错误处理](BEST_PRACTICES.md#错误处理)

### "如何管理内存？"
→ [语言特性 - 内存管理](LANGUAGE_FEATURES.md#内存管理)
→ [最佳实践 - 内存管理](BEST_PRACTICES.md#内存管理)

### "如何提高性能？"
→ [最佳实践 - 性能优化](BEST_PRACTICES.md#性能优化)
→ [优化框架文档](../OPTIMIZATION_FRAMEWORK.md)

### "如何编写安全的代码？"
→ [最佳实践 - 安全性](BEST_PRACTICES.md#安全性)
→ [语言特性 - 类型系统](LANGUAGE_FEATURES.md#类型系统)

### "有完整的示例吗？"
→ [示例程序集合](../examples/README.md)
→ [完整语言游览](../examples/complete_tour.zl)

### "代码风格规范是什么？"
→ [最佳实践 - 代码风格](BEST_PRACTICES.md#代码风格)

---

## 🛠️ 工具和资源

### 构建工具

- **YAN**: ZULON的包管理和构建工具
  - `yan build` - 构建项目
  - `yan run` - 运行程序
  - `yan new` - 创建新项目
  - `yan test` - 运行测试
  - 详见 [YAN工具链](YAN_TOOLCHAIN.md)

### 开发工具

- **代码格式化**: `yan fmt`
- **Linter**: `cargo clippy`
- **文档生成**: `yan doc --open`
- **测试运行**: `yan test`

### 示例代码

所有示例位于 `examples/` 目录：
- 00-08: 编号示例（从易到难）
- complete_tour.zl: 完整特性展示
- 各种专题示例（错误处理、并发等）

---

## 📊 文档统计

### 用户文档

| 类型 | 数量 | 总页数 |
|------|------|--------|
| 入门指南 | 1 | ~15页 |
| 语言参考 | 1 | ~40页 |
| 最佳实践 | 1 | ~30页 |
| 示例程序 | 10+ | ~100页 |
| **总计** | **13+** | **~185页** |

### 技术文档

| 类型 | 数量 |
|------|------|
| 架构设计 | 5+ |
| 实现文档 | 10+ |
| 迭代总结 | 20+ |
| **总计** | **35+** |

---

## 🔗 外部资源

### 官方资源
- **网站**: https://www.zulon-lang.org
- **GitHub**: https://github.com/zulon-lang/zulon
- **文档**: https://docs.zulon-lang.org
- **API文档**: https://docs.zulon-lang.org/api

### 社区
- **Discord**: https://discord.gg/zulon
- **Twitter**: @zulon_lang
- **博客**: https://blog.zulon-lang.org

### 相关项目
- **ZULON标准库**: https://github.com/zulon-lang/std
- **ZULON VSCode扩展**: https://marketplace.visualstudio.com/items?itemName=zulon-lang.vscode

---

## 📝 文档更新记录

### v0.1.0 (2026-01-08)

**新增**:
- ✅ 快速开始指南
- ✅ 语言特性详解
- ✅ 最佳实践指南
- ✅ 示例程序集合
- ✅ 文档索引

**更新**:
- ✅ 所有示例添加详细注释
- ✅ 创建README索引

---

## 🤝 贡献文档

发现文档问题或有改进建议？欢迎贡献！

### 如何贡献

1. Fork项目仓库
2. 编辑文档（markdown格式）
3. 提交Pull Request

### 文档规范

- 使用清晰的标题层次
- 提供代码示例
- 说明预期输出
- 标注难度和时间
- 使用友好的语言

---

## 📧 获取帮助

遇到问题？

1. **查看文档** - 本索引和相关文档
2. **搜索示例** - examples/ 目录
3. **查看FAQ** - [常见问题](QUICK_START_GUIDE.md#常见问题)
4. **社区求助** - Discord或GitHub Issues
5. **提交Bug** - GitHub Issues

---

## 🎓 推荐阅读顺序

### 第一次接触ZULON？

```
1. 快速开始指南
   ↓
2. Hello World示例
   ↓
3. 基础语法示例
   ↓
4. 类型系统示例
   ↓
5. 最佳实践（前几章）
```

### 有Rust/其他语言经验？

```
1. 快速开始指南（快速浏览）
   ↓
2. 语言特性详解（重点看差异）
   ↓
3. 错误处理和模式匹配
   ↓
4. 高级示例
   ↓
5. 最佳实践
```

### 准备深入使用？

```
1. 完整阅读所有用户文档
   ↓
2. 运行所有示例
   ↓
3. 阅读技术文档（架构、设计）
   ↓
4. 构建自己的项目
   ↓
5. 贡献代码或文档
```

---

**祝你学习愉快！** 🚀

有问题？查阅 [FAQ](QUICK_START_GUIDE.md#常见问题) 或加入 [Discord社区](https://discord.gg/zulon)

---

**文档索引 v1.0** | **ZULON Language Team** | **2026-01-08**
