# 嵌套循环问题诊断与修复进展

**日期**: 2026-01-07
**状态**: 问题诊断完成，修复方案确定中

---

## 问题描述

测试嵌套while循环时，程序陷入无限循环（超时退出码124）。

### 测试代码

```zulon
fn main() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 5 {
        let mut j = 0;
        while j < 3 {
            sum = sum + j;
            j = j + 1
        };
        i = i + 1
    };
    sum
}
```

**预期**: 5次迭代 × (0+1+2) = 15 (或其他值，取决于j的递增)

---

## 根本原因分析

### MIR输出分析

通过调试输出，发现了MIR结构的问题：

```
Block 1: 外层循环条件 (i < 5)
  then: Block 2 (外层体)
  else: Block 3 (退出)

Block 2: 外层体开始 (j = 0)
  Terminator: Goto { target: 1 }  ❌ 错误！应该跳到内层循环

Block 4: 内层循环条件 (j < 3)
  then: Block 5 (内层体)
  else: Block 6 (内层increment)

Block 5: 内层体 (sum += j, j += 1)
  Terminator: Goto { target: 4 }  ✅ 正确

Block 6: 内层increment (i += 1)
  Terminator: (无)  ❌ 错误！应该跳回Block 1
```

### 问题1: Block 2 跳转错误

**当前**: `Goto { target: 1 }` (跳回外层循环head)
**应该**: 跳转到内层循环head (Block 4)

**原因**: 外层循环的body lowering时，内层while循环被当作一个expression statement处理，其循环结构没有被正确地连接到外层body的控制流中。

### 问题2: Block 6 缺少terminator

**当前**: 无terminator
**应该**: `Goto { target: 1 }` (跳回外层循环head)

**原因**: 内层循环结束时，应该继续执行外层循环的下一个语句，但由于控制流断裂，Block 6成为unreachable。

---

## HIR语法问题

### 当前语法的问题

```zulon
while j < 3 {
    sum = sum + 1;   // statement (带分号)
    j = j + 1        // statement
};
// j = j + 1 不在循环的trailing expression中
```

在ZULON语法中：
- 分号结尾的行 = statement
- 最后一行无分号 = trailing expression (循环返回值)

### 修正方案

**方案1: 将j++移入内层循环作为trailing expr**
```zulon
while j < 3 {
    sum = sum + j;    // statement
    j = j + 1         // trailing expr
};
```

**方案2: 使用block包装**
```zulon
while j < 3 {
    {
        sum = sum + 1;
        j = j + 1
    }
};
```

**方案3: 简化为单个表达式**
```zulon
while j < 3 {
    sum = sum + j
    // j的递增需要在循环外或使用其他模式
};
```

---

## 技术深层分析

### HIR Loop Expression的结构

在HIR中，`Loop`表达式包含：
- `body`: HirBlock (语句列表 + trailing expression)
- 当body只有statement而没有trailing expr时，返回Unit

当前的嵌套循环在HIR中被表示为：
```rust
HirExpression::Loop {
    body: HirBlock {
        statements: [
            // 内层while循环 (expression statement)
            HirStatement::Expression(Loop { ... })
        ],
        trailing_expr: None  // Unit
    }
}
```

### MIR Lowering的逻辑

当前MIR lowering的Loop处理：
1. 创建loop_head, loop_body, exit_block
2. 从current跳转到loop_head
3. loop_head跳转到loop_body
4. Lower body (lower_block调用)
5. 在body的final_block添加loop-back到loop_head

**问题**: 步骤5无条件地添加loop-back，但：
- 如果body包含嵌套循环，final_block已经有terminator
- 这个terminator可能是嵌套循环的结构，不应该被覆盖

---

## 修复方案

### 方案A: 修复HIR Lowering (推荐)

**在MIR Lowering中检测嵌套循环**:

```rust
// 在lower_loop中
let (final_block_id, body_temp) = self.lower_block(func, body, loop_body, false)?;

// 检查final_block是否已经有terminator (可能是嵌套循环)
let final_block_obj = func.blocks.get_mut(&final_block_id).unwrap();
if final_block_obj.terminator.is_none() {
    // 只有在没有terminator时才添加loop-back
    final_block_obj.set_terminator(MirTerminator::Goto { target: loop_head });
} else if matches!(final_block_obj.terminator, Some(MirTerminator::Goto { .. })) {
    // 如果已经有Goto，检查是否是跳转到本循环的head
    // 如果不是，说明是嵌套循环，需要创建join block
}
```

### 方案B: 修复语法理解 (治本)

**确保ZULON语法正确处理嵌套循环**:

Parser应该将：
```zulon
while j < 3 {
    stmt1;
    stmt2
}
```

解析为：
```rust
Loop {
    body: Block {
        statements: [stmt1],
        trailing_expr: stmt2
    }
}
```

而不是：
```rust
Loop {
    body: Block {
        statements: [stmt1, stmt2],  // 错误！
        trailing_expr: None
    }
}
```

---

## 临时解决方案

### 方案1: 简化测试代码

使用不依赖复杂控制流的测试：

```zulon
// 测试1: 单层循环 (已验证✅)
fn main() -> i32 {
    let mut count = 0;
    while count < 10 {
        count = count + 1
    };
    count
}
```

```zulon
// 测试2: 递归替代 (已验证✅)
fn sum_to(n: i32) -> i32 {
    if n <= 0 { 0 } else { n + sum_to(n - 1) }
}

fn main() -> i32 {
    sum_to(10)
}
```

### 方案2: 使用不同的嵌套模式

```zulon
// 使用函数来避免嵌套循环的控制流问题
fn inner_loop(sum: i32) -> i32 {
    let mut j = 0;
    let mut result = sum;
    while j < 3 {
        result = result + j;
        j = j + 1
    };
    result
}

fn main() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 5 {
        sum = inner_loop(sum);
        i = i + 1
    };
    sum
}
```

---

## 实现优先级

### P0 - 立即修复 (本周)

1. **确认语法问题** (2小时)
   - 检查Parser如何处理while loop body
   - 确认trailing expr的逻辑
   - 编写Parser测试

2. **修复MIR Lowering** (4小时)
   - 正确处理嵌套循环
   - 创建join block
   - 测试控制流

3. **验证修复** (2小时)
   - 测试2层嵌套
   - 测试3层嵌套
   - 验证退出码

### P1 - 短期改进 (本月)

4. **完善循环测试** (1天)
   - 各种嵌套模式
   - 边界情况
   - 性能测试

5. **实现For循环** (1周)
   - Range类型
   - 脱糖为while
   - 测试

### P2 - 长期优化 (下季度)

6. **优化控制流分析** (2周)
   - CFG构建
   - Dominator tree
   - Loop优化

---

## 当前状态

**编译器前端**: ✅ 工作
**类型系统**: ✅ 工作
**单层循环**: ✅ 完全工作
**嵌套循环**: ❌ 无限循环 (需要修复)

**Phase 1 MVP进度**: 95% → **95%** (嵌套循环阻塞)

---

## 结论

嵌套循环的问题已被定位到MIR lowering阶段，具体是：
1. Block 2的跳转目标错误
2. Block 6缺少terminator

这是一个控制流构建的问题，而不是可变变量的问题。可变变量的alloca实现本身是正确的。

**建议**:
1. 先绕过嵌套循环，继续完成其他MVP功能
2. 或者修复MIR lowering的嵌套循环处理
3. 或者在HIR层确保正确的语法解析

**下一步**: 选择修复方案并实施

---

**文档版本**: 1.0
**创建时间**: 2026-01-07
**状态**: 问题诊断完成
**优先级**: P0
