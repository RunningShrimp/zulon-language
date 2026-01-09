# ZULON Lexer 功能分析和改进计划

**分析日期**: 2026-01-07
**目标**: 验证和完善 Lexer 功能

---

## ✅ 已实现的功能

### 1. Token 类型定义 (100%)
- ✅ 所有关键字 (if, else, fn, struct, enum, trait, 等)
- ✅ 所有运算符 (算术、比较、逻辑、位运算)
- ✅ 所有分隔符 ((, ), {, }, [, ], 等)
- ✅ 字面量类型 (整数、浮点、字符串、字符)
- ✅ 特殊值 (true, false, null)
- ✅ Unicode 标识符支持

### 2. 核心词法分析 (90%)
- ✅ 标识符和关键字识别
- ✅ 数字字面量 (整数、浮点)
- ✅ 字符串字面量 (包括转义序列)
- ✅ 字符字面量
- ✅ 运算符识别
- ✅ 分隔符识别
- ✅ 位置跟踪 (行号、列号)
- ✅ 空白字符跳过

### 3. 高级功能 (60%)
- ⚠️ 模板字符串 (部分实现)
  - ✅ 反引号识别
  - ✅ 多行支持
  - ✅ 转义序列
  - ⚠️ 字符串插值识别 (`${`) 但未完全实现
  - ❌ 插值表达式收集 (TODO)

- ⚠️ 注释处理
  - ✅ 行注释 (`//`)
  - ✅ 块注释 (`/* */`)
  - ❓ 多行注释验证

- ✅ 错误处理
  - ✅ 未闭合字符串
  - ✅ 未闭合字符
  - ✅ 未闭合模板字符串
  - ✅ 非法字符
  - ⚠️ 错误恢复机制待验证

### 4. 转义序列 (100%)
- ✅ `\\` - 反斜杠
- ✅ `\n` - 换行
- ✅ `\r` - 回车
- ✅ `\t` - 制表符
- ✅ `\0` - null
- ✅ `\"` - 双引号
- ✅ `\'` - 单引号
- ✅ `\xHH` - 十六进制字符
- ✅ `\u{HHHH}` - Unicode 字符

---

## ⚠️ 需要改进的功能

### 1. 字符串插值 (优先级: P1)

**当前状态**: 框架已有，但未完整实现

**问题**:
```rust
// 第 341 行
// TODO: Collect interpolated expression
```

**需要实现**:
- [ ] 解析 `${` 后的表达式
- [ ] 嵌套插值支持
- [ ] 插值错误处理
- [ ] 测试用例

**实现方案**:
```rust
// 伪代码
fn lex_template_string(&mut self) -> TokenKind {
    let mut s = String::new();

    while let Some(&c) = self.chars.peek() {
        match c {
            '`' => { /* ... */ }
            '$' => {
                self.advance(); // consume '$'
                if let Some(&'{') = self.chars.peek() {
                    self.advance(); // consume '{'
                    s.push_str("${");

                    // 收集插值表达式
                    let mut depth = 1;
                    while depth > 0 {
                        if let Some(ch) = self.advance() {
                            s.push(ch);
                            if ch == '{' { depth += 1; }
                            else if ch == '}' { depth -= 1; }
                        } else {
                            // 错误: 未闭合的插值
                            self.errors.push(LexError {
                                kind: LexErrorKind::UnterminatedInterpolation,
                                position: self.token_start,
                            });
                            break;
                        }
                    }
                } else {
                    s.push('$');
                }
            }
            _ => { /* ... */ }
        }
    }

    TokenKind::TemplateString(s.into())
}
```

### 2. 注词法错误恢复 (优先级: P2)

**当前状态**: 有错误收集，但恢复机制待验证

**需要验证**:
- [ ] 错误后能否继续词法分析
- [ ] 错误位置是否准确
- [ ] 错误信息是否清晰

**测试用例**:
```rust
// 非法字符
fn main() { ☺ }

// 未闭合字符串
let s = "hello

// 未闭合插值
`Hello ${name`
```

### 3. 数字字面量增强 (优先级: P2)

**当前功能**: 基础数字支持

**需要验证/添加**:
- [ ] 十六进制 (`0xFF`)
- [ ] 八进制 (`0o755`)
- [ ] 二进制 (`0b1010`)
- [ ] 下划线分隔 (`1_000_000`)
- [ ] 类型后缀 (`42u8`, `3.14f32`)

### 4. 标识符增强 (优先级: P3)

**当前功能**: Unicode 标识符

**需要验证**:
- [ ] 完整的 Unicode 标识符支持 (使用 `unicode-ident` crate)
- [ ] 标识符不能以数字开头
- [ ] 原始标识符 (r#ident)

---

## 📋 改进计划

### Week 1: 字符串插值完整实现

**Day 1-2: 实现基础插值**
- [ ] 实现 `${...}` 表达式收集
- [ ] 处理嵌套大括号
- [ ] 错误处理 (未闭合插值)

**Day 3-4: 测试和验证**
- [ ] 单元测试
- [ ] 集成测试
- [ ] 边界情况

**Day 5: 文档和示例**
- [ ] 添加文档注释
- [ ] 创建示例程序
- [ ] 更新 README

### Week 2: 错误处理增强

**Day 1-2: 错误恢复**
- [ ] 验证错误恢复机制
- [ ] 改进错误消息
- [ ] 添加错误测试

**Day 3-4: 数字字面量**
- [ ] 添加十六进制
- [ ] 添加八进制
- [ ] 添加二进制
- [ ] 验证下划线分隔

**Day 5: 集成测试**
- [ ] 端到端测试
- [ ] 性能测试

---

## 🧪 测试计划

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interpolation() {
        let source = r#"Hello ${name}!"#;
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(!errors.is_empty()); // 当前应该有未实现的错误
    }

    #[test]
    fn test_template_string() {
        let source = "`Hello`";
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert_eq!(tokens[0].kind, TokenKind::TemplateString("Hello".into()));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_escape_sequences() {
        let source = r#"\n\t\r\\\"\'\x41\u{0041}"#;
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.lex_all();

        assert!(errors.is_empty());
    }
}
```

### 集成测试
```rust
#[test]
fn test_complex_template() {
    let source = r#"
        `Hello ${user.name}, you have ${count} messages.
         Your balance is $${amount}.`
    "#;

    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex_all();

    // 验证 token 序列
    assert!(errors.is_empty());
}
```

---

## 🎯 验收标准

### Phase 1: 字符串插值
- [ ] 能够解析 `${...}` 表达式
- [ ] 正确处理嵌套大括号
- [ ] 未闭合插值报错
- [ ] 所有测试通过

### Phase 2: 错误处理
- [ ] 错误后继续分析
- [ ] 错误位置准确
- [ ] 错误信息清晰

### Phase 3: 完整性
- [ ] 所有 Token 类型测试
- [ ] 边界情况覆盖
- [ ] 性能可接受

---

## 📊 当前进度评估

### Lexer 完成度: **85%**

**已完成**:
- ✅ Token 类型定义 (100%)
- ✅ 核心词法分析 (90%)
- ✅ 基础字面量 (100%)
- ✅ 运算符 (100%)
- ✅ 转义序列 (100%)
- ✅ 位置跟踪 (100%)

**待完成**:
- ⚠️ 字符串插值 (60% - 框架已有，需完整实现)
- ⚠️ 错误恢复 (待验证)
- ⚠️ 数字格式增强 (待验证)

**预计完成时间**: 1-2周

---

## 🚀 下一步行动

### 立即开始 (本周)
1. **实现字符串插值** (P1)
   - 完成 `${...}` 表达式收集
   - 添加错误处理
   - 编写测试

2. **验证现有功能**
   - 运行现有测试
   - 修复发现的 bug
   - 补充测试用例

### 短期 (2周内)
3. **完善错误处理**
   - 错误恢复验证
   - 改进错误消息

4. **增强数字字面量**
   - 添加更多格式支持
   - 验证类型后缀

### 中期 (1个月内)
5. **Parser 集成**
   - 确保 Lexer 和 Parser 配合良好
   - 端到端测试
   - 性能优化

---

**生成时间**: 2026-01-07
**维护者**: ZULON Language Team
