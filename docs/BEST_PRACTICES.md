# ZULON 最佳实践指南

**版本**: v0.1.0
**更新日期**: 2026-01-08

本指南提供ZULON编程的最佳实践，帮助您编写更安全、更高效、更易维护的代码。

---

## 📋 目录

1. [代码风格](#代码风格)
2. [错误处理](#错误处理)
3. [内存管理](#内存管理)
4. [类型系统](#类型系统)
5. [性能优化](#性能优化)
6. [代码组织](#代码组织)
7. [测试](#测试)
8. [安全性](#安全性)
9. [并发](#并发)
10. [工具使用](#工具使用)

---

## 代码风格

### 命名规范

```zulon
// 变量和函数：snake_case
let user_name = "Alice"
fn calculate_sum() {}

// 常量：SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS = 100
const DEFAULT_TIMEOUT = 30

// 类型：PascalCase
struct UserInfo {}
enum ColorType {}
trait Drawable {}

// 关联常量：PascalCase或SCREAMING_SNAKE_CASE
impl MyStruct {
    const MAX_SIZE: usize = 100
}
```

### 缩进和格式

```zulon
// 使用4空格缩进（推荐）
fn main() {
    let x = 1
    if x > 0 {
        println!("positive")
    }
}

// 函数参数：每个参数一行（如果太长）
fn complex_function(
    param1: String,
    param2: i32,
    param3: f64,
) -> Outcome<i32, String> {
    // ...
}
```

### 注释规范

```zulon
// 单行注释：解释为什么，而不是是什么

/// 文档注释：用于函数、结构体、公共API
///
/// # Examples
/// ```
/// let result = add(1, 2)
/// assert!(result == 3)
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

//! 模块级注释
//! 这个模块提供了数学运算功能
```

### 行宽

- 推荐最大行宽：100字符
- 硬性限制：120字符（超出需换行）

```zulon
// 好：行宽合理
let result = some_function(param1, param2, param3)

// 如果太长，换行
let result = some_function(
    long_parameter_name_1,
    long_parameter_name_2,
    long_parameter_name_3,
)
```

---

## 错误处理

### 优先使用 Outcome 而非 throw

```zulon
// ✅ 好：显式错误处理
fn divide(a: i32, b: i32) -> Outcome<i32, String> {
    if b == 0 {
        return Outcome::Err("Division by zero".to_string())
    }
    return Outcome::Ok(a / b)
}

// ❌ 避免：过度使用throw
fn divide_bad(a: i32, b: i32) -> Outcome<i32, String> {
    if b == 0 {
        throw "Division by zero".to_string()
    }
    return Outcome::Ok(a / b)
}
```

### 提供有意义的错误信息

```zulon
// ✅ 好：具体的错误信息
fn parse_age(s: String) -> Outcome<i32, String> {
    match s.parse::<i32>() {
        Ok(n) if n >= 0 && n <= 150 => Outcome::Ok(n),
        Ok(n) => Outcome::Err(format!("Age {} out of range (0-150)", n)),
        Err(_) => Outcome::Err(format!("Invalid age: '{}'", s)),
    }
}

// ❌ 避免：模糊的错误
fn parse_age_bad(s: String) -> Outcome<i32, String> {
    match s.parse::<i32>() {
        Ok(n) => Outcome::Ok(n),
        Err(_) => Outcome::Err("Error".to_string()),
    }
}
```

### 使用 ? 运算符简化错误传播

```zulon
// ✅ 好：使用?运算符
fn process_user(id: String) -> Outcome<User, String> {
    let user = fetch_user(id)?
    let validated = validate_user(user)?
    return Outcome::Ok(validated)
}

// ❌ 避免：显式match每个错误
fn process_user_bad(id: String) -> Outcome<User, String> {
    let user = match fetch_user(id) {
        Outcome::Ok(u) => u,
        Outcome::Err(e) => return Outcome::Err(e),
    }
    let validated = match validate_user(user) {
        Outcome::Ok(u) => u,
        Outcome::Err(e) => return Outcome::Err(e),
    }
    return Outcome::Ok(validated)
}
```

### 不要忽略 Outcome

```zulon
// ✅ 好：总是处理Outcome
fn process() {
    let result = may_fail()
    match result {
        Outcome::Ok(v) => println!("Success: {}", v),
        Outcome::Err(e) => println!("Error: {}", e),
    }
}

// ❌ 避免：忽略错误
fn process_bad() {
    let result = may_fail()
    // 忘记处理result
}
```

---

## 内存管理

### 优先使用不可变引用

```zulon
// ✅ 好：使用不可变引用
fn print_length(s: &String) {
    println!("Length: {}", s.len())
}

// ❌ 避免：不必要的克隆
fn print_length_bad(s: String) {
    println!("Length: {}", s.len())
}  // s被drop，可能很昂贵
```

### 使用 Arc 共享大对象

```zulon
// ✅ 好：共享大对象
use std::memory::Arc

let large_data = Arc::new(vec![1, 2, 3, 4, 5])
let ref1 = large_data.clone()
let ref2 = large_data.clone()
// 只克隆引用，不克隆数据

// ❌ 避免：不必要地克隆大对象
let large_data = vec![1, 2, 3, 4, 5]
let copy1 = large_data.clone()  // 昂贵的深拷贝
let copy2 = large_data.clone()
```

### 避免循环引用

```zulon
// ✅ 好：使用Weak打破循环
use std::memory::{Arc, Weak}

struct Node {
    value: i32,
    parent: Optional<Weak<Node>>,
    children: Vec<Arc<Node>>,
}

// ❌ 避免：强引用循环（内存泄漏）
struct NodeBad {
    value: i32,
    parent: Optional<Arc<NodeBad>>,
    children: Vec<Arc<NodeBad>>,
}
```

### 及时释放资源

```zulon
// ✅ 好：使用defer确保资源释放
fn process_file() -> Outcome<(), String> {
    let file = File::open("data.txt")?
    defer {
        file.close()
    }

    // 处理文件...
    return Outcome::Ok(())
}  // file自动被关闭

// ❌ 避免：手动管理资源
fn process_file_bad() -> Outcome<(), String> {
    let file = File::open("data.txt")?

    // 处理...
    if error {
        // 忘记关闭file
        return Outcome::Err("Error".to_string())
    }

    file.close()
    return Outcome::Ok(())
}
```

---

## 类型系统

### 利用类型推导

```zulon
// ✅ 好：让编译器推导类型
let name = "Alice"
let count = 42
let items = vec![1, 2, 3]

// ❌ 避免：不必要的类型标注
let name: String = "Alice"
let count: i32 = 42
let items: Vec<i32> = vec![1, 2, 3]
```

### 但在公开API中明确类型

```zulon
// ✅ 好：公开API明确类型
pub fn calculate(
    width: i32,
    height: i32,
) -> f64 {
    // ...
}

// ❌ 避免：公开API依赖推导
pub fn calculate(width, height) {
    // 类型不清楚
}
```

### 使用类型别名提高可读性

```zulon
// ✅ 好：类型别名
type UserId = i64
type UserName = String
type Result<T> = Outcome<T, String>

fn fetch_user(id: UserId) -> Result<User> {
    // ...
}

// ❌ 避免：重复的复杂类型
fn fetch_user_bad(id: i64) -> Outcome<User, String> {
    // ...
}
```

### 使用枚举代替魔法值

```zulon
// ✅ 好：使用枚举
enum Status {
    Pending,
    InProgress,
    Completed,
    Failed,
}

fn update_status(status: Status) {
    // ...
}

// ❌ 避免：魔法数字/字符串
fn update_status_bad(status: i32) {
    // 0 = Pending? 1 = InProgress? 难以记忆
}
```

---

## 性能优化

### 避免不必要的分配

```zulon
// ✅ 好：重用缓冲区
let mut buffer = String::new()
for item in items {
    buffer.clear()
    buffer.push_str(item.to_string())
    process(&buffer)
}

// ❌ 避免：循环中重复分配
for item in items {
    let buffer = item.to_string()  // 每次循环都分配
    process(&buffer)
}
```

### 使用引用避免复制

```zulon
// ✅ 好：传递引用
fn sum(numbers: &Vec<i32>) -> i32 {
    let mut total = 0
    for n in numbers {
        total = total + n
    }
    return total
}

// ❌ 避免：不必要的所有权转移
fn sum_bad(numbers: Vec<i32>) -> i32 {
    // numbers被move，调用方不能再用
    let mut total = 0
    for n in numbers {
        total = total + n
    }
    return total
}
```

### 预分配容量

```zulon
// ✅ 好：预分配已知容量
let mut vec = Vec::with_capacity(100)
for i in 0..100 {
    vec.push(i)
}

// ❌ 避免：多次重新分配
let mut vec = Vec::new()
for i in 0..100 {
    vec.push(i)  // 可能触发多次重新分配
}
```

### 使用适当的集合类型

```zulon
// ✅ 好：根据场景选择
let mut vec = Vec::new()        // 需要索引访问
let mut set = HashSet::new()    // 需要去重
let mut map = HashMap::new()    // 需要键值查找

// ❌ 避免：总是使用Vec
// 如果你需要频繁查找，Vec效率低
let items = vec![1, 2,3, 4, 5]
if items.contains(&value) {  // O(n)查找
    // ...
}

// 应该用HashSet
// O(1)查找
let items = HashSet::from([1, 2, 3, 4, 5])
if items.contains(&value) {
    // ...
}
```

---

## 代码组织

### 模块化

```zulon
// ✅ 好：清晰的模块结构
mod models {
    pub struct User {}
    pub struct Post {}
}

mod services {
    pub fn fetch_user() {}
    pub fn save_post() {}
}

fn main() {
    services::fetch_user()
}
```

### 保持函数简短

```zulon
// ✅ 好：每个函数做一件事
fn validate_input(input: String) -> Outcome<(), String> {
    // 验证逻辑
    return Outcome::Ok(())
}

fn process_input(input: String) -> Outcome<Result, String> {
    validate_input(input)?
    // 处理逻辑
    return Outcome::Ok(result)
}

fn save_result(result: Result) -> Outcome<(), String> {
    // 保存逻辑
    return Outcome::Ok(())
}

// ❌ 避免：一个函数做多件事
fn do_everything(input: String) -> Outcome<(), String> {
    // 验证
    // 处理
    // 保存
    // 100+ 行代码...
}
```

### 使用trait定义行为

```zulon
// ✅ 好：使用trait
trait Display {
    fn display(&self) -> String
}

impl Display for User {
    fn display(&self) -> String {
        self.name.clone()
    }
}

fn print_info<T: Display>(item: T) {
    println!("{}", item.display())
}
```

---

## 测试

### 编写测试

```zulon
// ✅ 好：全面的测试
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5)
}

#[test]
fn test_add_negative() {
    assert_eq!(add(-2, -3), -5)
}

#[test]
fn test_divide_by_zero() {
    let result = divide(10, 0)
    match result {
        Outcome::Err(_) => {},
        Outcome::Ok(_) => panic!("Should error"),
    }
}
```

### 测试边界情况

```zulon
#[test]
fn test_empty_input() {
    let result = process("".to_string())
    assert!(result.is_err())
}

#[test]
fn test_maximum_value() {
    let result = calculate(i32::MAX)
    assert!(result.is_ok())
}

#[test]
fn test_minimum_value() {
    let result = calculate(i32::MIN)
    assert!(result.is_ok())
}
```

---

## 安全性

### 验证输入

```zulon
// ✅ 好：验证所有输入
fn parse_age(s: String) -> Outcome<i32, String> {
    let age = s.parse::<i32>()?

    if age < 0 || age > 150 {
        return Outcome::Err("Age out of range".to_string())
    }

    return Outcome::Ok(age)
}

// ❌ 避免：信任输入
fn parse_age_bad(s: String) -> Outcome<i32, String> {
    return s.parse::<i32>()
}
```

### 防止整数溢出

```zulon
// ✅ 好：检查溢出
fn multiply(a: i32, b: i32) -> Outcome<i32, String> {
    let result = a.checked_mul(b)
    match result {
        Some(v) => Outcome::Ok(v),
        None => Outcome::Err("Multiplication overflow".to_string()),
    }
}

// ❌ 避免：可能溢出
fn multiply_bad(a: i32, b: i32) -> i32 {
    a * b  // 在debug模式下会panic
}
```

### 使用类型安全

```zulon
// ✅ 好：使用新类型包装
struct UserId(i64)
struct Temperature(f64)

fn set_user_temp(id: UserId, temp: Temperature) {
    // 类型系统防止混淆
}

// ❌ 避免：原始类型容易混淆
fn set_user_temp_bad(id: i64, temp: f64) {
    // 容易传错参数顺序
}
```

---

## 并发

### 使用Arc共享线程间数据

```zulon
// ✅ 好：线程安全的共享
use std::memory::Arc
use std::sync::Mutex

let counter = Arc::new(Mutex::new(0))
let counter1 = counter.clone()
let counter2 = counter.clone()

// 在不同线程中使用counter1和counter2
```

### 避免数据竞争

```zulon
// ✅ 好：使用Mutex保护共享状态
let shared_data = Arc::new(Mutex::new(vec![1, 2, 3]))

// 访问前必须lock
let mut data = shared_data.lock().unwrap()
data.push(4)

// ❌ 避免：未保护的共享可变状态
let shared_data = Arc::new(vec![1, 2, 3])
// 如果多个线程同时修改，会导致数据竞争
```

---

## 工具使用

### 使用YAN构建

```bash
# 开发构建（快速）
yan build

# 发布构建（优化）
yan build --release

# 运行测试
yan test

# 格式化代码
yan fmt

# 检查代码（不构建）
yan check
```

### 使用Clippy检查代码质量

```bash
# 运行linter
cargo clippy

# 自动修复可修复的问题
cargo clippy --fix
```

### 文档生成

```bash
# 生成并打开文档
yan doc --open

# 只为公开API生成文档
yan doc --document-private-items
```

---

## 常见陷阱

### 1. 忘记处理Outcome

```zulon
// ❌ 忘记处理错误
let file = File::open("data.txt")

// ✅ 正确处理
let file = match File::open("data.txt") {
    Ok(f) => f,
    Err(e) => return Outcome::Err(e.to_string()),
}
```

### 2. 不必要的克隆

```zulon
// ❌ 昂贵的克隆
fn process(s: String) {
    println!("{}", s)
}
let data = large_string.clone()
process(data)

// ✅ 使用引用
fn process(s: &String) {
    println!("{}", s)
}
process(&large_string)
```

### 3. 循环中的重复分配

```zulon
// ❌ 每次循环都分配
for i in 0..1000 {
    let temp = vec![1, 2, 3]
    process(temp)
}

// ✅ 重用
let mut temp = vec![1, 2, 3]
for i in 0..1000 {
    temp.clear()
    // 填充temp
    process(&temp)
}
```

### 4. 忽略编译器警告

```zulon
// 不要忽略警告！
// warning: unused variable: x
let x = 42

// 要么使用它，要么用_前缀
let _x = 42  // 明确表示不使用
```

---

## 检查清单

在提交代码前，确保：

- [ ] 所有测试通过（`yan test`）
- [ ] 无编译警告（`yan build --warnings`）
- [ ] 无Clippy警告（`cargo clippy`）
- [ ] 代码已格式化（`yan fmt`）
- [ ] 所有公开API有文档注释
- [ ] 错误处理完整（没有忽略Outcome）
- [ ] 没有明显的性能问题（避免不必要的克隆/分配）
- [ ] 输入验证完整
- [ ] 适当的测试覆盖（边界情况、错误路径）

---

## 总结

遵循这些最佳实践将帮助你：

- ✅ 编写更安全的代码
- ✅ 提高性能
- ✅ 提高可维护性
- ✅ 改善代码可读性
- ✅ 减少bug

**记住**:
- 代码被阅读的次数多于被编写的次数
- 清晰的代码胜过聪明的代码
- 测试是代码质量保证的重要组成部分
- 工具是你的朋友（fmt, clippy, doc）

---

**最佳实践指南 v1.0** | **ZULON Language Team** | **2026-01-08**

**相关文档**:
- [快速开始指南](QUICK_START_GUIDE.md)
- [语言特性详解](LANGUAGE_FEATURES.md)
- [API文档](../api)
