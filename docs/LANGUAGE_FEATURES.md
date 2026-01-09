# ZULON 语言特性详解

**版本**: v0.1.0
**更新日期**: 2026-01-08

本文档详细说明ZULON编程语言的核心特性和使用方法。

---

## 📋 目录

1. [类型系统](#类型系统)
2. [变量绑定](#变量绑定)
3. [函数](#函数)
4. [控制流](#控制流)
5. [错误处理](#错误处理)
6. [模式匹配](#模式匹配)
7. [集合类型](#集合类型)
8. [结构体和枚举](#结构体和枚举)
9. [Trait系统](#trait系统)
10. [高级特性](#高级特性)

---

## 类型系统

ZULON使用静态类型系统，支持类型推导和Hindley-Milner类型推断。

### 基本类型

```zulon
// 整数类型
let small: i8 = 100
let medium: i32 = 100000
let big: i64 = 10000000000

let unsigned: u32 = 42
let size: usize = 1024

// 浮点类型
let float: f32 = 3.14
let double: f64 = 3.14159265359

// 布尔类型
let is_true: bool = true
let is_false: bool = false

// 字符类型
let letter: char = 'A'
let emoji: char = '🎉'

// 字符串类型
let greeting: String = "Hello, ZULON!"
let multiline: String = "
    This is a
    multiline string
"
```

### 类型推导

ZULON编译器可以自动推导变量类型：

```zulon
// 编译器自动推导类型
let number = 42        // 推导为 i32
let pi = 3.14          // 推导为 f64
let name = "ZULON"     // 推导为 String
let flag = true        // 推导为 bool
```

### 类型转换

```zulon
// 显式类型转换
let x: i32 = 42
let y: i64 = x as i64

let float_val = 3.14
let int_val = float_val as i32  // int_val = 3
```

---

## 变量绑定

### 不可变变量（默认）

```zulon
let x = 5
// x = 10  // 编译错误：不能重新赋值不可变变量
```

### 可变变量

```zulon
let mut x = 5
x = 10  // OK: 可变变量可以重新赋值
```

### 变量遮蔽（Shadowing）

```zulon
let x = 5
let x = x + 1  // 创建新的变量x，值为6
let x = "hello"  // 可以改变类型

// 与可变变量的区别
let mut x = 5
x = 10  // 重新赋值，类型不能改变
// x = "hello"  // 编译错误：类型不匹配
```

### 解构赋值

```zulon
// 元组解构
let (x, y) = (1, 2)

// 结构体解构
struct Point {
    x: i32,
    y: i32,
}
let p = Point { x: 10, y: 20 }
let Point { x: px, y: py } = p
// 或者简写
let Point { x, y } = p
```

---

## 函数

### 函数定义

```zulon
// 基本函数
fn greet(name: String) -> String {
    return "Hello, " + name + "!"
}

// 无返回值
fn say_hello() {
    println!("Hello!")
}

// 表达式作为返回值（省略return）
fn add(a: i32, b: i32) -> i32 {
    a + b  // 最后一个表达式自动返回
}
```

### 多返回值

```zulon
fn divide(a: i32, b: i32) -> (i32, Optional<String>) {
    if b == 0 {
        return (0, Optional::Some("Division by zero".to_string()))
    }
    return (a / b, Optional::None)
}

// 使用
let (result, error) = divide(10, 2)
match error {
    Optional::Some(msg) => println!("Error: {}", msg),
    Optional::None => println!("Result: {}", result),
}
```

### 高阶函数

```zulon
// 函数作为参数
fn apply_function(x: i32, f: fn(i32) -> i32) -> i32 {
    f(x)
}

fn double(x: i32) -> i32 {
    x * 2
}

let result = apply_function(5, double)  // result = 10

// 闭包（未来版本）
// let multiply = |x: i32| x * n
```

### 方法调用

```zulon
struct Rectangle {
    width: i32,
    height: i32,
}

impl Rectangle {
    // 构造函数
    fn new(width: i32, height: i32) -> Rectangle {
        Rectangle { width, height }
    }

    // 方法
    fn area(&self) -> i32 {
        self.width * self.height
    }
}

let rect = Rectangle::new(10, 20)
println!("Area: {}", rect.area())
```

---

## 控制流

### if表达式

```zulon
// 基本if
let number = 5
if number < 10 {
    println!("less than 10")
} else if number > 10 {
    println!("greater than 10")
} else {
    println!("equal to 10")
}

// if作为表达式
let condition = true
let number = if condition { 5 } else { 6 }
```

### 循环

```zulon
// loop无限循环
let mut count = 0
loop {
    count = count + 1
    if count == 10 {
        break
    }
}

// while循环
let mut number = 3
while number != 0 {
    println!("{}!", number)
    number = number - 1
}

// for循环（未来版本）
// let numbers = [1, 2, 3, 4, 5]
// for n in numbers {
//     println!("{}", n)
// }
```

---

## 错误处理

ZULON使用`Outcome<T, E>`类型进行错误处理，而不是异常。

### Outcome类型

```zulon
enum Outcome<T, E> {
    Ok(T),
    Err(E),
}
```

### 基本错误处理

```zulon
fn parse_number(s: String) -> Outcome<i32, String> {
    if s == "" {
        return Outcome::Err("Empty string".to_string())
    }
    // 假设有parse方法
    match s.parse::<i32>() {
        Ok(n) => Outcome::Ok(n),
        Err(_) => Outcome::Err("Invalid number".to_string()),
    }
}

// 使用
let result = parse_number("42")
match result {
    Outcome::Ok(n) => println!("Number: {}", n),
    Outcome::Err(e) => println!("Error: {}", e),
}
```

### ?运算符（错误传播）

```zulon
fn process(s: String) -> Outcome<i32, String> {
    // ?运算符自动传播错误
    let n = parse_number(s)?
    let doubled = n * 2
    return Outcome::Ok(doubled)
}

// 等价于
fn process_manual(s: String) -> Outcome<i32, String> {
    let n = match parse_number(s) {
        Outcome::Ok(v) => v,
        Outcome::Err(e) => return Outcome::Err(e),
    }
    let doubled = n * 2
    return Outcome::Ok(doubled)
}
```

### throw表达式

```zulon
fn divide(a: i32, b: i32) -> Outcome<i32, String> {
    if b == 0 {
        throw "Division by zero".to_string()
    }
    return Outcome::Ok(a / b)
}

// throw可以在表达式中使用
let result = if b == 0 {
    throw "Error".to_string()
} else {
    Outcome::Ok(a / b)
}
```

### 组合错误处理

```zulon
fn validate_and_process(input: String) -> Outcome<i32, String> {
    // 多个可能失败的操作
    let trimmed = input.trim()?        // trim可能失败
    let parsed = trimmed.parse::<i32>()?  // parse可能失败
    let checked = if parsed < 0 {
        throw "Negative number".to_string()
    } else {
        parsed
    }
    return Outcome::Ok(checked * 2)
}
```

---

## 模式匹配

### match表达式

```zulon
let number = 3

match number {
    1 => println!("One"),
    2 => println!("Two"),
    3 => println!("Three"),
    _ => println!("Other"),  // 默认情况
}

// match作为表达式
let result = match number {
    1 => "one",
    2 => "two",
    _ => "other",
}
```

### 解构匹配

```zulon
// Optional匹配
let maybe_value = Optional::Some(5)
match maybe_value {
    Optional::Some(v) => println!("Value: {}", v),
    Optional::None => println!("No value"),
}

// 元组匹配
let pair = (2, -2)
match pair {
    (0, y) => println!("First is zero, second is {}", y),
    (x, 0) => println!("First is {}, second is zero", x),
    _ => println!("No zeros"),
}

// 结构体匹配
struct Point {
    x: i32,
    y: i32,
}
let p = Point { x: 0, y: 7 }
match p {
    Point { x: 0, y } => println!("On y-axis at {}", y),
    Point { x, y: 0 } => println!("On x-axis at {}", x),
    Point { x, y } => println!("At ({}, {})", x, y),
}
```

### 守卫（Guards）

```zulon
let number = 4
match number {
    n if n % 2 == 0 => println!("Even"),
    n if n % 2 == 1 => println!("Odd"),
    _ => println!("Other"),
}
```

---

## 集合类型

### Vec<T>（动态数组）

```zulon
// 创建Vec
let mut numbers = Vec::new()
numbers.push(1)
numbers.push(2)
numbers.push(3)

// 访问元素
let first = numbers.get(0)  // Optional<i32>
match first {
    Optional::Some(v) => println!("First: {}", v),
    Optional::None => println!("No element"),
}

// 遍历
for i in 0..numbers.len() {
    println!("{}", numbers.get(i))
}

// 修改
numbers.set(0, 10)
let popped = numbers.pop()
```

### HashMap<K, V>

```zulon
// 创建HashMap
let mut scores = HashMap::new()
scores.insert("Alice", 100)
scores.insert("Bob", 200)

// 访问
match scores.get("Alice") {
    Optional::Some(score) => println!("Alice's score: {}", score),
    Optional::None => println!("Alice not found"),
}

// 更新
scores.insert("Alice", 150)  // 覆盖
scores.remove("Bob")
```

### HashSet<T>

```zulon
// 创建HashSet
let mut set = HashSet::new()
set.insert(1)
set.insert(2)
set.insert(2)  // 重复值被忽略

// 检查包含
if set.contains(1) {
    println!("Set contains 1")
}

// 移除
set.remove(1)
```

---

## 结构体和枚举

### 结构体

```zulon
// 定义结构体
struct Person {
    name: String,
    age: i32,
}

// 创建实例
let person = Person {
    name: "Alice".to_string(),
    age: 30,
}

// 访问字段
println!("Name: {}", person.name)
println!("Age: {}", person.age)

// 元组结构体
struct Color(i32, i32, i32)
let black = Color(0, 0, 0)

// 单元结构体
struct UnitStruct
let instance = UnitStruct
```

### 枚举

```zulon
// 定义枚举
enum Option<T> {
    Some(T),
    None,
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

// 使用枚举
let msg = Message::Move { x: 10, y: 20 }

match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(s) => println!("Write: {}", s),
    Message::ChangeColor(r, g, b) => println!("Color: ({}, {}, {})", r, g, b),
}
```

### 枚举带数据

```zulon
enum IpAddr {
    V4(String),
    V6(String),
}

let home = IpAddr::V4("127.0.0.1".to_string())
let loopback = IpAddr::V6("::1".to_string())

// 模式匹配
match home {
    IpAddr::V4(addr) => println!("IPv4: {}", addr),
    IpAddr::V6(addr) => println!("IPv6: {}", addr),
}
```

---

## Trait系统

Trait定义共享的行为。

### 定义Trait

```zulon
trait Summary {
    fn summarize(&self) -> String;
}

// 实现Trait
struct Article {
    title: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        self.title.clone() + ": " + &self.content[..10] + "..."
    }
}
```

### 默认实现

```zulon
trait Summary {
    fn summarize(&self) -> String {
        "(Default summary)".to_string()
    }
}

// 可以使用默认实现
struct Post {}
impl Summary for Post {}  // 使用默认summarize
```

### Trait作为参数

```zulon
fn notify(item: impl Summary) {
    println!("Breaking news! {}", item.summarize())
}

// 或者使用trait bound
fn notify_generic<T: Summary>(item: T) {
    println!("Breaking news! {}", item.summarize());
}
```

### 标准库Traits

```zulon
// Clone trait
let x = vec![1, 2, 3]
let y = x.clone()

// Copy trait（基本类型）
let x = 5
let y = x  // x仍然有效，因为i32实现了Copy

// PartialEq
if x == y {
    println!("Equal")
}

// PartialOrd
if x < y {
    println!("x is less than y")
}
```

---

## 高级特性

### 字符串插值

```zulon
let name = "Alice"
let age = 30
let message = "Hello, ${name}! You are ${age} years old."
println!(message)  // 输出: Hello, Alice! You are 30 years old.
```

### 模板字符串

```zulon
let template = `
    <html>
        <body>
            <h1>Hello, ${name}!</h1>
        </body>
    </html>
`
```

### defer语句（延迟执行）

```zulon
fn process_file() {
    let file = File::open("data.txt")
    defer {
        file.close()  // 函数返回时自动执行
    }

    // 处理文件...
    // 无论发生什么，file.close()都会被调用
}
```

### 多返回值

```zulon
fn compute(x: i32, y: i32) -> (i32, i32, i32) {
    return (x + y, x - y, x * y)
}

let (sum, diff, product) = compute(10, 5)
println!("Sum: {}, Diff: {}, Product: {}", sum, diff, product)
```

### 方法链调用

```zulon
struct Builder {
    data: String,
}

impl Builder {
    fn new() -> Builder {
        Builder { data: "".to_string() }
    }

    fn add(&mut self, s: String) -> &mut Builder {
        self.data = self.data + &s
        self
    }

    fn build(&self) -> String {
        self.data.clone()
    }
}

let result = Builder::new()
    .add("Hello".to_string())
    .add(" ".to_string())
    .add("World".to_string())
    .build()
```

---

## 内存管理

### Arc<T>（原子引用计数）

```zulon
use std::memory::Arc

// 创建Arc
let data = Arc::new(5)

// 克隆引用（不复制数据）
let reference1 = data.clone()
let reference2 = data.clone()

// 所有引用指向同一数据
println!("{}", data)        // 5
println!("{}", reference1)  // 5
println!("{}", reference2)  // 5

// 引用计数管理
// strong_count = 3 (data + reference1 + reference2)
```

### Weak<T>（弱引用）

```zulon
use std::memory::{Arc, Weak}

// 创建循环引用时使用Weak避免内存泄漏
struct Node {
    value: i32,
    parent: Optional<Weak<Node>>,
    children: Vec<Arc<Node>>,
}
```

### 所有权规则

1. 每个值有一个所有者
2. 同一时间只能有一个所有者
3. 所有者离开作用域时值被丢弃
4. 可以通过移动或克隆转移所有权

```zulon
// 移动
let s1 = String::from("hello")
let s2 = s1  // s1被移动，不再有效
// println!("{}", s1)  // 编译错误

// 克隆
let s3 = s2.clone()  // 深拷贝，两者都有效
println!("{}", s2)
println!("{}", s3)
```

---

## 并发（基础）

ZULON v0.1.0的并发功能仍在开发中，但提供了基础支持：

### 线程安全

```zulon
// Arc提供线程安全的引用计数
use std::memory::Arc
use std::sync::Mutex

let data = Arc::new(Mutex::new(0))
let data_clone = data.clone()

// 在不同线程间共享数据
// (具体线程API在开发中)
```

---

## 最佳实践

### 1. 使用类型推导

```zulon
// 好
let name = "Alice"

// 不必要
let name: String = "Alice"
```

### 2. 优先使用Outcome而非异常

```zulon
// 好
fn divide(a: i32, b: i32) -> Outcome<i32, String> {
    if b == 0 {
        return Outcome::Err("Division by zero".to_string())
    }
    return Outcome::Ok(a / b)
}

// 避免（如果可能）
fn divide_bad(a: i32, b: i32) -> i32 {
    if b == 0 {
        throw "Error"
    }
    return a / b
}
```

### 3. 使用模式匹配处理所有情况

```zulon
// 好
match result {
    Optional::Some(v) => println!("Value: {}", v),
    Optional::None => println!("No value"),
}

// 避免
let value = result.get()  // 可能忽略None情况
```

### 4. 利用不可变性

```zulon
// 好
let x = 5

// 只在必要时使用可变
let mut x = 5
x = 6
```

### 5. 使用Arc共享数据

```zulon
// 好：共享不复制
let data = Arc::new(large_vector)
let ref1 = data.clone()
let ref2 = data.clone()

// 避免：不必要的克隆
let data2 = data.clone()  // 如果不需要所有权转移
```

---

## 总结

ZULON v0.1.0提供了现代编程语言的核心特性：

- ✅ **静态类型系统**：类型推导、类型安全
- ✅ **强大的错误处理**：Outcome类型、?运算符、throw表达式
- ✅ **模式匹配**：解构、守卫、穷尽检查
- ✅ **Trait系统**：行为共享、默认实现
- ✅ **内存安全**：Arc/Weak智能指针、所有权系统
- ✅ **集合类型**：Vec、HashMap、HashSet
- ✅ **高级特性**：字符串插值、模板字符串、defer、多返回值

**下一步**:
- 查看 [快速开始指南](QUICK_START_GUIDE.md)
- 运行 [示例程序](../examples/README.md)
- 阅读 [API文档](../api)

---

**语言特性文档 v1.0** | **ZULON Language Team** | **2026-01-08**
