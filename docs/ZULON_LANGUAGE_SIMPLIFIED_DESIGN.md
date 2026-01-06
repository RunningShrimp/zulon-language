# ZULON 编程语言 - 简化设计文档 v1.0

**版本**: v1.0 Simplified
**日期**: 2026-01-07
**设计理念**: 简单性、安全性、性能的平衡
**目标**: 让系统编程像写脚本一样简单

---

## 📋 目录

1. [设计哲学](#1-设计哲学)
2. [快速开始](#2-快速开始)
3. [核心语法](#3-核心语法)
4. [类型系统](#4-类型系统)
5. [内存模型](#5-内存模型)
6. [并发编程](#6-并发编程)
7. [错误处理](#7-错误处理)
8. [模块系统](#8-模块系统)
9. [高级特性](#9-高级特性)
10. [实战案例](#10-实战案例)

---

## 1. 设计哲学

### 1.1 核心原则

ZULON 的设计遵循以下三个核心原则：

#### **简单性优先**

```go
// ❌ Rust: 复杂的生命周期标注
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len { x } else { y }
}

// ✅ ZULON: 编译器自动推断
fn longest(x: str, y: str) -> str {
    if x.len() > y.len { x } else { y }
}
```

**为什么这样设计？**
- 95% 的情况下，编译器可以自动推断
- 减少认知负荷：不需要理解复杂的生命周期规则
- 学习曲线更平缓

#### **安全性保证**

```go
// ✅ 编译期空安全
fn greet(name: str?) {  // str? 表示可能为空
    // 编译错误：必须处理空值
    print(name.to_uppercase())
}

// ✅ 正确做法
fn greet(name: str?) {
    if let Some(n) = name {
        print(n.to_uppercase())
    } else {
        print("Hello, Guest")
    }
}
```

**安全性特性：**
- 默认非空引用（str 不会是 null）
- 可空类型显式标记（str? 可能是 null）
- 编译期数据竞争检测
- 自动边界检查

#### **性能不妥协**

```go
// ✅ 零成本抽象
fn sum<T: Add>(items: &[T]) -> T {
    let mut total = T::default()
    for item in items {
        total = total + item
    }
    return total
}

// 编译后等价于手写的优化代码
// 没有运行时开销
```

**性能目标：**
- 接近 C++ 的运行时性能
- 零成本抽象（泛型单态化）
- 内联优化和逃逸分析
- SIMD 向量化

### 1.2 语言定位

```
简单性 ←──────────────→ 性能
Python        ZULON        C++
Go            ●
              |
              └─ 在简单性和性能间取得最佳平衡

安全性 ←──────────────→ 灵活性
Java          ZULON        C++
Rust          ●
              |
              └─ 默认安全，允许底层控制
```

**目标用户：**
- 系统程序员：需要性能和控制力
- 后端开发者：需要并发和类型安全
- 全栈工程师：需要学习曲线平缓
- 学生和新手：入门友好的系统语言

---

## 2. 快速开始

### 2.1 Hello World

```go
// hello.zl
fn main() {
    println("Hello, World!")
}
```

**运行：**
```bash
$ yan run hello.zl
Hello, World!
```

**编译：**
```bash
$ yan build hello.zl
./hello
Hello, World!
```

### 2.2 变量与类型

```go
fn main() {
    // 类型推断（推荐）
    let name = "ZULON"
    let year = 2026
    let version = 1.0

    // 显式类型标注（可选）
    let count: u32 = 100
    let price: f64 = 99.99

    // 可变变量
    let mut sum = 0
    sum = sum + 1

    // 常量
    const MAX_SIZE = 1000

    println("Language: {}, Year: {}", name, year)
    println("Version: {}, Sum: {}", version, sum)
}
```

### 2.3 函数定义

```go
// 简单函数
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

// 多返回值
fn divmod(a: i32, b: i32) -> (i32, i32) {
    return (a / b, a % b)
}

fn main() {
    let result = add(10, 20)
    let (quotient, remainder) = divmod(17, 5)

    println("10 + 20 = {}", result)
    println("17 / 5 = {} remainder {}", quotient, remainder)
}
```

### 2.4 基础控制流

```go
fn main() {
    // if-else
    let score = 85

    if score >= 90 {
        println("优秀")
    } else if score >= 60 {
        println("及格")
    } else {
        println("不及格")
    }

    // 循环
    for i in 0..5 {
        println!("i = {}", i)
    }

    // while 循环
    let mut count = 0
    while count < 3 {
        println!("count = {}", count)
        count = count + 1
    }

    // match 模式匹配
    let day = "Monday"
    match day {
        "Monday" | "Tuesday" => println("工作日"),
        "Saturday" | "Sunday" => println("周末"),
        _ => println("其他"),
    }
}
```

---

## 3. 核心语法

### 3.1 变量声明

```go
fn variables() {
    // let: 不可变变量（默认）
    let name = "Alice"
    name = "Bob"  // ❌ 编译错误

    // let mut: 可变变量
    let mut age = 25
    age = 26  // ✅ 可以修改

    // 类型标注（可选）
    let height: f64 = 1.75
    let mut weight: f32 = 70.0

    // 同时声明多个变量
    let (x, y, z) = (1, 2, 3)

    // 解构赋值
    let point = (10, 20)
    let (x, y) = point
}
```

**设计理念：**
- 默认不可变 → 减少意外修改
- 类型自动推断 → 减少冗余标注
- 显式可变标记 → 清晰表达意图

### 3.2 基本类型

```go
fn basic_types() {
    // 布尔类型
    let is_active: bool = true
    let is_valid: bool = false

    // 字符类型（Unicode）
    let letter: char = 'A'
    let emoji: char = '😀'
    let chinese: char = '中'

    // 整数类型
    let small: i8 = 100
    let medium: i32 = 100000
    let big: i64 = 10000000000
    let unsigned: u32 = 4000000000

    // 浮点类型
    let pi: f32 = 3.14
    let e: f64 = 2.718281828459045

    // 字符串类型
    let text: str = "Hello"
    let mut buffer: String = String::new()
    buffer.push_str("World")

    // 数组（固定大小）
    let primes: [i32; 5] = [2, 3, 5, 7, 11]

    // 向量（动态大小）
    let mut numbers: Vec<i32> = vec![1, 2, 3]
    numbers.push(4)

    // 元组
    let person: (str, i32) = ("Alice", 30)
    let (name, age) = person
}
```

### 3.3 类型推断

```go
// 强大的类型推断
fn type_inference() {
    // 编译器自动推断类型
    let integer = 42        // i32
    let float = 3.14        // f64
    let text = "hello"      // &str
    let list = vec![1, 2, 3] // Vec<i32>

    // 函数返回值也能推断
    fn add(a: i32, b: i32) {
        return a + b  // 推断返回 i32
    }

    // 复杂表达式也能推断
    let numbers = vec![1, 2, 3, 4, 5]
    let doubled: Vec<i32> = numbers.iter()
        .map(|x| x * 2)
        .collect()
}
```

### 3.4 运算符

```go
fn operators() {
    // 算术运算符
    let sum = 10 + 20
    let diff = 50 - 15
    let product = 6 * 7
    let quotient = 100 / 4
    let remainder = 17 % 5

    // 比较运算符
    let equal = 10 == 10
    let not_equal = 5 != 3
    let less = 3 < 5
    let greater = 10 > 8
    let less_equal = 5 <= 5
    let greater_equal = 10 >= 10

    // 逻辑运算符
    let and = true && false
    let or = true || false
    let not = !true

    // 位运算符
    let bit_and = 0b1010 & 0b1100  // 0b1000
    let bit_or = 0b1010 | 0b1100   // 0b1110
    let bit_xor = 0b1010 ^ 0b1100  // 0b0110
    let bit_shift_left = 0b0001 << 2  // 0b0100
    let bit_shift_right = 0b1000 >> 2 // 0b0010
}
```

---

## 4. 类型系统

### 4.1 空安全设计

```go
// 默认非空（安全）
struct User {
    name: str,      // 不能为 null
    email: str,     // 不能为 null
    age: u32,       // 不能为 null
}

fn create_user(name: str, email: str) -> User {
    return User {
        name,
        email,
        age: 0,
    }
}

// 使用时的安全性
fn greet_user(user: User) {
    // ✅ 安全：不会 panic
    println("Hello, {}", user.name)
}

// 可空类型（显式标记）
struct UserProfile {
    user: User,
    nickname: str?,  // 可能为 null
    bio: str?,       // 可能为 null
}

fn get_nickname(profile: UserProfile) -> str {
    // ✅ 必须处理空值
    if let Some(nick) = profile.nickname {
        return nick
    } else {
        return profile.user.name
    }
}

// ? 运算符：简化空值处理
fn get_bio(profile: UserProfile) -> str {
    // 如果 bio 为 null，返回默认值
    return profile.bio ? "No bio available"
}

// ? 链式调用
fn get_user_email(profile: UserProfile?) -> str? {
    // 如果 profile 为 null 或 email 为 null，返回 null
    return profile?.user.email
}
```

**空安全的好处：**
- 编译期保证：不会有空指针异常
- 显式标记：`?` 清楚表达可能为空
- 强制处理：编译器要求处理空值

### 4.2 错误处理

```go
// Result 类型：成功或失败
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// 使用 ! 表示可能抛出错误
fn divide(a: f64, b: f64) -> f64 ! DivideError {
    if b == 0.0 {
        return DivideError::DivisionByZero
    }
    return a / b
}

// ? 运算符：自动传播错误
fn calculate() -> f64 ! Error {
    let a = read_number()?
    let b = read_number()?
    return divide(a, b)?
}

// 显式处理错误
fn handle_calculation() {
    match calculate() {
        Ok(result) => println!("Result: {}", result),
        Err(Error::IoError(e)) =>
            println!("IO Error: {}", e),
        Err(Error::DivideError) =>
            println!("Division by zero!"),
    }
}

// 简化版：unwrap_or
fn safe_calculate() -> f64 {
    return calculate().unwrap_or(0.0)
}
```

### 4.3 枚举与模式匹配

```go
// 定义枚举
enum Option<T> {
    Some(T),
    None,
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(str),
    ChangeColor(i32, i32, i32),
}

// 模式匹配
fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quit")
        },
        Message::Move { x, y } => {
            println!("Move to ({}, {})", x, y)
        },
        Message::Write(text) => {
            println!("Write: {}", text)
        },
        Message::ChangeColor(r, g, b) => {
            println!("Color: {}, {}, {}", r, g, b)
        },
    }
}

// if let 简化单分支匹配
fn maybe_print(opt: Option<i32>) {
    if let Some(value) = opt {
        println!("Value: {}", value)
    } else {
        println!("No value")
    }
}

// while let 简化循环匹配
fn process_list(list: List<i32>) {
    let mut current = list
    while let List::Cons(value, next) = current {
        println!("{}", value)
        current = next
    }
}
```

### 4.4 结构体与方法

```go
// 定义结构体
struct Point {
    x: f64,
    y: f64,
}

// 实现方法
impl Point {
    // 构造函数
    fn new(x: f64, y: f64) -> Point {
        return Point { x, y }
    }

    // 实例方法
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x
        let dy = self.y - other.y
        return (dx * dx + dy * dy).sqrt()
    }

    // 可变方法
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x = self.x + dx
        self.y = self.y + dy
    }
}

// 使用
fn geometry() {
    let p1 = Point::new(0.0, 0.0)
    let p2 = Point::new(3.0, 4.0)

    println!("Distance: {}", p1.distance(&p2))

    let mut p = Point::new(1.0, 2.0)
    p.translate(10.0, 20.0)
    println!("New position: ({}, {})", p.x, p.y)
}
```

### 4.5 泛型

```go
// 泛型函数
fn identity<T>(value: T) -> T {
    return value
}

fn max<T: Comparable>(a: T, b: T) -> T {
    if a > b {
        return a
    } else {
        return b
    }
}

// 泛型结构体
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Pair<T, U> {
        return Pair { first, second }
    }

    fn swap(self) -> Pair<U, T> {
        return Pair {
            first: self.second,
            second: self.first,
        }
    }
}

// 使用
fn generics() {
    let x = identity(42)        // T = i32
    let y = identity("hello")   // T = &str

    let pair = Pair::new(1, "first")
    let swapped = pair.swap()   // Pair<str, i32>
}
```

### 4.6 Trait（特性）

```go
// 定义 Trait
trait Printable {
    fn format(&self) -> str;
}

// 为类型实现 Trait
impl Printable for Point {
    fn format(&self) -> str {
        return format!("Point({}, {})", self.x, self.y)
    }
}

// Trait 约束
fn print_item<T: Printable>(item: T) {
    println!("{}", item.format())
}

// 多重 Trait 约束
fn clone_and_hash<T: Clone + Hash>(item: T) -> u64 {
    let cloned = item.clone()
    return cloned.hash()
}

// 使用
fn traits() {
    let p = Point::new(1.0, 2.0)
    print_item(p)  // 输出: Point(1.0, 2.0)
}
```

---

## 5. 内存模型

### 5.1 设计理念：简化而非复杂

ZULON 的内存模型设计基于以下观察：

**问题：Rust 的所有权太复杂**
```rust
// ❌ Rust: 需要理解借用检查器
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len { x } else { y }
}

// ❌ 需要理解生命周期参数
struct Context<'a> {
    data: &'a Vec<i32>,
}
```

**解决：ZULON 的智能区域系统**
```go
// ✅ ZULON: 编译器自动处理
fn longest(x: str, y: str) -> str {
    if x.len() > y.len { x } else { y }
}

// ✅ 简单直观
struct Context {
    data: Vec<i32>,
}
```

### 5.2 内存区域

ZULON 使用**区域内存管理**（Region-based Memory Management）：

```go
// 三种内存区域

// 1. 栈区域（Stack）：自动管理
fn stack_example() {
    let x = 42          // 在栈上分配
    let y = 3.14        // 在栈上分配
    // 函数结束时自动释放
}

// 2. 堆区域（Heap）：自动引用计数
fn heap_example() {
    let data = vec![1, 2, 3]  // 堆分配 + ARC
    let cloned = data.clone()   // 引用计数 +1
    // 引用计数归零时自动释放
}

// 3. 静态区域（Static）：编译期确定
const MAX_SIZE: usize = 1000  // 静态数据
static GLOBAL_CONFIG: Config = Config {
    timeout: 30,
    retries: 3,
}
```

**为什么这样设计？**
- **栈分配**：快速、自动、无需思考
- **ARC（自动引用计数）**：类似 Swift/iOS，开发者熟悉
- **逃逸分析**：编译器自动优化，95% 的情况无需手动管理

### 5.3 值类型 vs 引用类型

```go
// 值类型：栈分配，复制语义
struct Point {
    x: f64,
    y: f64,
}

fn value_types() {
    let p1 = Point { x: 1.0, y: 2.0 }
    let p2 = p1  // 复制整个结构体

    p2.x = 10.0
    println!("p1.x = {}", p1.x)  // 1.0（不受影响）
}

// 引用类型：堆分配，共享语义
class Node {
    data: i32,
    next: Box<Node?>,
}

fn reference_types() {
    let n1 = box Node {
        data: 1,
        next: None,
    }
    let n2 = n1  // 共享同一个对象

    n2.data = 10
    println!("n1.data = {}", n1.data)  // 10（共享修改）
}
```

**类型分类：**
- **值类型**：基本类型（i32, f64, bool）、结构体、枚举
- **引用类型**：类（class）、Box、Vec、String

### 5.4 自动内存管理

```go
// 编译器自动优化

// 示例1：逃逸分析
fn create() -> Vec<i32> {
    let mut v = vec![1, 2, 3]
    v.push(4)
    return v
}
// 编译器分析：v 逃逸到函数外，使用堆分配

// 示例2：栈分配优化
fn consume() {
    let v = vec![1, 2, 3]
    process(v)
    // v 在此之后不再使用，编译器可以优化为栈分配
}

// 示例3：内联优化
fn small() -> Point {
    return Point { x: 1.0, y: 2.0 }
}
// 编译器可能完全内联，不分配任何内存
```

**编译器优化策略：**
1. **逃逸分析**：确定变量是否逃逸函数
2. **栈分配优先**：不逃逸的变量优先栈分配
3. **内联优化**：小对象可能完全内联
4. **ARC 优化**：消除不必要的引用计数操作

### 5.5 借用规则（简化版）

```go
// ZULON 的借用规则（比 Rust 简单）

// 规则1：默认不可变借用
fn read_data(data: &Vec<i32>) {
    println!("{}", data.len())
}

// 规则2：需要修改时显式标记 mut
fn modify_data(data: &mut Vec<i32>) {
    data.push(42)
}

// 规则3：可以同时有多个不可变借用
fn multiple_borrows() {
    let data = vec![1, 2, 3]

    let r1 = &data  // ✅ 可行
    let r2 = &data  // ✅ 可行

    println!("{} {}", r1.len(), r2.len())
}

// 规则4：可变借用时独占访问
fn mutable_borrow() {
    let mut data = vec![1, 2, 3]

    let r = &mut data
    r.push(4)  // ✅ 可行

    // ❌ 编译错误：不能有其他借用
    // let r2 = &data;
}
```

**关键简化：**
- **无需显式生命周期**：编译器自动推断（95%+ 情况）
- **借用检查更宽松**：类似 Swift，而非 Rust
- **运行时检查辅助**：必要时使用运行时检查

### 5.6 智能指针

```go
// Box<T>：堆分配
fn box_example() {
    let b = box 42  // 在堆上分配
    println!("{}", *b)  // 解引用
}

// Rc<T>：引用计数（单线程）
fn rc_example() {
    use std::rc::Rc

    let data = Rc::new(vec![1, 2, 3])
    let rc1 = Rc::clone(&data)  // 引用计数 +1
    let rc2 = Rc::clone(&data)  // 引用计数 +1

    println!("Count: {}", Rc::strong_count(&data))  // 3
}

// Arc<T>：原子引用计数（多线程）
fn arc_example() {
    use std::sync::Arc
    use std::thread

    let data = Arc::new(vec![1, 2, 3])
    let mut handles = vec![]

    for _ in 0..3 {
        let data = Arc::clone(&data)
        handles.push(thread::spawn(move || {
            println!("{:?}", *data)
        }))
    }

    for handle in handles {
        handle.join().unwrap()
    }
}
```

---

## 6. 并发编程

### 6.1 设计理念：Go 式简洁性

ZULON 的并发模型灵感来自 Go，但更安全：

```go
// ❌ Rust: 复杂的 async/await
async fn fetch_data() -> Result<Data, Error> {
    let response = reqwest::get(url).await?;
    return response.json().await?;
}

// ✅ ZULON: 简单直观
fn fetch_data() -> Data ! Error {
    let response = http_get(url)?
    return parse_json(response)?
}
```

### 6.2 Goroutine 风格的轻量线程

```go
// spawn: 创建轻量级线程
fn spawn_example() {
    // 创建新线程
    spawn(|| {
        for i in 0..5 {
            println!("Worker: {}", i)
            thread::sleep(100ms)
        }
    })

    // 主线程继续执行
    for i in 0..3 {
        println!("Main: {}", i)
        thread::sleep(100ms)
    }

    // 等待所有线程完成
    thread::join_all()
}
```

### 6.3 Channel：消息传递

```go
// 创建 channel
fn channel_example() {
    // 创建无缓冲 channel
    let (tx, rx) = channel::<i32>()

    // 发送线程
    spawn(move || {
        for i in 0..5 {
            tx.send(i)
            println!("Sent: {}", i)
        }
    })

    // 接收线程
    spawn(move || {
        for _ in 0..5 {
            let value = rx.recv()
            println!("Received: {}", value)
        }
    })

    thread::join_all()
}

// 带缓冲的 channel
fn buffered_channel() {
    let (tx, rx) = channel::<i32>(10)  // 缓冲区大小 10

    spawn(move || {
        for i in 0..20 {
            tx.send(i)  // 可以发送多个而不阻塞
        }
    })

    spawn(move || {
        for _ in 0..20 {
            let value = rx.recv()
            println!("Got: {}", value)
        }
    })
}
```

### 6.4 结构化并发

```go
// task::scope: 作用域内并发
fn structured_concurrency() {
    task::scope(|scope| {
        // 创建多个并发任务
        scope.spawn(|| {
            println!("Task 1")
        })

        scope.spawn(|| {
            println!("Task 2")
        })

        scope.spawn(|| {
            println!("Task 3")
        })

        // 所有任务在 scope 结束前自动等待
    })
    // 这里所有任务都已完成
}
```

### 6.5 Actor 模型

```go
// 定义 Actor
actor Calculator {
    // Actor 状态
    value: i32,

    // 初始化
    fn new(initial: i32) -> Calculator {
        return Calculator {
            value: initial,
        }
    }

    // 处理消息
    fn receive(&mut self, msg: Message) {
        match msg {
            Message::Add(x) => {
                self.value = self.value + x
            },
            Message::Multiply(x) => {
                self.value = self.value * x
            },
            Message::GetValue(reply_channel) => {
                reply_channel.send(self.value)
            },
        }
    }
}

// 使用 Actor
fn actor_example() {
    let calc = Calculator::new(0)

    // 发送消息
    calc.send(Message::Add(10))
    calc.send(Message::Multiply(2))

    // 请求值
    let (tx, rx) = channel()
    calc.send(Message::GetValue(tx))
    let result = rx.recv()

    println!("Result: {}", result)  // 20
}
```

### 6.6 共享状态（线程安全）

```go
// Mutex：互斥锁
fn mutex_example() {
    use std::sync::{Arc, Mutex}

    let counter = Arc::new(Mutex::new(0))
    let mut handles = vec![]

    for _ in 0..10 {
        let counter = Arc::clone(&counter)
        handles.push(spawn(move || {
            let mut data = counter.lock().unwrap()
            *data = *data + 1
        }))
    }

    for handle in handles {
        handle.join()
    }

    println!("Counter: {}", *counter.lock().unwrap())
}

// RwLock：读写锁
fn rwlock_example() {
    use std::sync::{Arc, RwLock}

    let data = Arc::new(RwLock::new(vec![1, 2, 3]))

    // 读线程
    spawn(|| {
        let r = data.read().unwrap()
        println!("Read: {:?}", *r)
    })

    // 写线程
    spawn(|| {
        let mut w = data.write().unwrap()
        w.push(4)
    })
}
```

---

## 7. 错误处理

### 7.1 Result 类型

```go
// Result<T, E>：表示可能失败的操作
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// 使用 ! 表示可能返回错误
fn divide(a: f64, b: f64) -> f64 ! DivideError {
    if b == 0.0 {
        return DivideError::DivisionByZero
    }
    return a / b
}

// 定义错误类型
enum DivideError {
    DivisionByZero,
    InvalidInput,
}

fn safe_divide(a: f64, b: f64) -> Result<f64, DivideError> {
    if b == 0.0 {
        return Result::Err(DivideError::DivisionByZero)
    }
    return Result::Ok(a / b)
}
```

### 7.2 ? 运算符

```go
// ? 运算符：自动传播错误
fn load_config() -> Config ! Error {
    // 读取文件
    let content = read_file("config.json")?

    // 解析 JSON
    let config: Config = parse_json(content)?

    return Ok(config)
}

// 等价于：
fn load_config_explicit() -> Result<Config, Error> {
    let content = read_file("config.json")
    let content = match content {
        Ok(c) => c,
        Err(e) => return Err(e),
    }

    let config = parse_json(content)
    let config = match config {
        Ok(c) => c,
        Err(e) => return Err(e),
    }

    return Ok(config)
}
```

### 7.3 错误转换

```go
// 使用 map_err 转换错误
fn load_user_config() -> Config ! MyError {
    let content = fs::read_to_string("config.json")
        .map_err(|e| MyError::IoError {
            path: "config.json",
            source: e,
        })?

    let config: Config = serde_json::from_str(&content)
        .map_err(|e| MyError::ParseError {
            message: e.to_string(),
        })?

    return Ok(config)
}

// 使用 ? 自定义错误消息
fn parse_config(path: str) -> Config ! MyError {
    let content = fs::read_to_string(path)
        .map_err(|e| MyError::IoFailed(
            format!("无法读取配置文件 {}: {}", path, e)
        ))?

    // ...
}
```

### 7.4 自定义错误

```go
// 定义错误类型
enum AppError {
    IoError { message: str },
    ParseError { line: usize, column: usize },
    ValidationError { field: str, reason: str },
}

// 实现 Display trait
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AppError::IoError { message } => {
                write!(f, "IO 错误: {}", message)
            },
            AppError::ParseError { line, column } => {
                write!(f, "解析错误: 行 {}, 列 {}", line, column)
            },
            AppError::ValidationError { field, reason } => {
                write!(f, "验证错误 {}: {}", field, reason)
            },
        }
    }
}

// 使用自定义错误
fn process_data(input: str) -> Result<Data, AppError> {
    if input.is_empty() {
        return Err(AppError::ValidationError {
            field: "input",
            reason: "不能为空",
        })
    }

    // 处理数据...
    return Ok(data)
}
```

---

## 8. 模块系统

### 8.1 模块定义

```go
// math.zl
mod math {
    // 公开函数（pub）
    pub fn add(a: i32, b: i32) -> i32 {
        return a + b
    }

    // 私有函数
    fn helper() {
        // ...
    }

    // 公开常量
    pub const PI: f64 = 3.14159265359

    // 公开类型
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }

    impl Point {
        pub fn new(x: f64, y: f64) -> Point {
            return Point { x, y }
        }

        pub fn distance(&self, other: &Point) -> f64 {
            let dx = self.x - other.x
            let dy = self.y - other.y
            return (dx * dx + dy * dy).sqrt()
        }
    }
}
```

### 8.2 使用模块

```go
// main.zl
// 导入模块
use math::{add, Point, PI}

fn main() {
    // 使用导入的函数
    let sum = add(10, 20)
    println!("Sum: {}", sum)

    // 使用导入的类型
    let p1 = Point::new(0.0, 0.0)
    let p2 = Point::new(3.0, 4.0)
    println!("Distance: {}", p1.distance(&p2))

    // 使用导入的常量
    println!("PI: {}", PI)
}
```

### 8.3 包管理

```go
// yan.toml
[package]
name = "myapp"
version = "1.0.0"
edition = "2026"

[dependencies]
http = "1.0"
json = "2.0"
database = "3.0"

[dev-dependencies]
testing = "1.0"
```

```go
// 使用外部包
use http::Client
use json::parse

fn fetch_data(url: str) -> Data ! Error {
    let client = Client::new()
    let response = client.get(url)?
    return parse(response.body())?
}
```

---

## 9. 高级特性

### 9.1 宏（Macros）

```go
// 声明式宏
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new()
            $( temp_vec.push($x); )*
            temp_vec
        }
    };
}

// 使用宏
fn main() {
    let v = vec![1, 2, 3, 4, 5]
    println!("{:?}", v)
}

// 派生宏（自动实现 trait）
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

// 等价于手动实现
impl Debug for Point { /* ... */ }
impl Clone for Point { /* ... */ }
impl PartialEq for Point { /* ... */ }
```

### 9.2 属性（Attributes）

```go
// 测试属性
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5)
}

// 条件编译
#[cfg(target = "windows")]
fn get_path() -> str {
    return "C:\\Users\\..."
}

#[cfg(target = "linux")]
fn get_path() -> str {
    return "/home/user/..."
}

// 内联提示
#[inline(always)]
fn small_function() -> i32 {
    return 42
}

// 不内联
#[inline(never)]
fn large_function() {
    // 大量代码...
}

// 弃用警告
#[deprecated(since = "1.0.0", note = "使用 new_function 代替")]
fn old_function() {
    // ...
}
```

### 9.3 反射（Reflection）

```go
// 获取类型信息
fn print_type<T>(value: T) {
    let type_info = type_of(value)
    println!("Type: {}", type_info.name())
    println!("Size: {} bytes", type_info.size())
}

// 动态调用
fn dynamic_call(obj: &dyn Any, method: str, args: &[&dyn Any]) {
    if let Some(method) = obj.get_method(method) {
        let result = method.invoke(obj, args)
        println!("Result: {:?}", result)
    }
}
```

### 9.4 异步 I/O

```go
// 异步文件操作
fn async_read_file(path: str) -> Vec<u8> ! Error {
    let file = async_open(path).await?
    let content = file.read_all().await?
    return Ok(content)
}

// 异步 HTTP 请求
fn async_fetch(url: str) -> Response ! Error {
    let response = http_get(url).await?
    return Ok(response)
}

// 并发异步任务
fn async_concurrent() {
    task::scope(|scope| {
        scope.spawn(async || {
            let data = async_fetch("https://api.example.com/1").await
            println!("Data 1: {:?}", data)
        })

        scope.spawn(async || {
            let data = async_fetch("https://api.example.com/2").await
            println!("Data 2: {:?}", data)
        })
    })
}
```

---

## 10. 实战案例

### 10.1 HTTP 服务器

```go
use http::{Server, Request, Response};
use json::{Json, Value};

fn main() {
    let server = Server::new("127.0.0.1:8080")

    server.get("/api/users", handle_get_users)
    server.post("/api/users", handle_create_user)
    server.get("/api/users/:id", handle_get_user)

    server.start()
}

fn handle_get_users(req: Request) -> Response {
    let users = vec![
        User { id: 1, name: "Alice" },
        User { id: 2, name: "Bob" },
    ]

    return Response::json()
        .status(200)
        .body(json!(users))
}

fn handle_create_user(req: Request) -> Response {
    let user: User = req.body_json().unwrap()

    // 保存到数据库...
    let saved = db::save_user(&user).unwrap()

    return Response::json()
        .status(201)
        .body(json!(saved))
}

fn handle_get_user(req: Request) -> Response {
    let id = req.param("id").parse::<i32>().unwrap()

    let user = db::get_user(id).unwrap()

    match user {
        Some(u) => Response::json()
            .status(200)
            .body(json!(u)),
        None => Response::json()
            .status(404)
            .body(json!({ "error": "User not found" })),
    }
}
```

### 10.2 数据库操作

```go
use database::{Connection, Query};
use sql::SELECT;

fn main() ! Error {
    // 连接数据库
    let conn = Connection::connect("postgres://localhost/mydb")?

    // 创建表
    conn.execute(
        "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE
        )"
    )?

    // 插入数据
    conn.execute(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        &["Alice", "alice@example.com"]
    )?

    // 查询数据
    let users: Vec<User> = conn.query(
        "SELECT * FROM users WHERE name = $1",
        &["Alice"]
    )?

    for user in users {
        println!("User: {} ({})", user.name, user.email)
    }

    return Ok(())
}
```

### 10.3 文件处理

```go
use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};

fn main() ! Error {
    // 读取文件
    let content = read_to_string("input.txt")?
    println!("Content:\n{}", content)

    // 逐行读取
    let file = File::open("input.txt")?
    let reader = BufReader::new(file)

    for line in reader.lines() {
        let line = line?
        println!("Line: {}", line)
    }

    // 写入文件
    let output = "Hello, World!\n"
    std::fs::write("output.txt", output)?

    // 复制文件
    std::fs::copy("input.txt", "backup.txt")?

    return Ok(())
}
```

### 10.4 命令行工具

```go
use std::env;
use std::process;

fn main() ! Error {
    let args = env::args()

    if args.len() < 2 {
        println!("Usage: {} <command> [args]", args[0])
        process::exit(1)
    }

    let command = &args[1]

    match command.as_str() {
        "build" => {
            println!("Building...")
            build_project()
        },
        "test" => {
            println!("Running tests...")
            run_tests()
        },
        "deploy" => {
            println!("Deploying...")
            deploy_project()
        },
        _ => {
            println!("Unknown command: {}", command)
            process::exit(1)
        },
    }

    return Ok(())
}
```

---

## 附录

### A. 关键字列表

ZULON 只有 **25 个关键字**（与 Go 相当）：

```
// 声明
fn, let, mut, const, struct, enum, trait, impl, type, mod

// 控制流
if, else, match, for, while, in, return, break, continue

// 其他
true, false, null, where, pub, use, spawn, async, await
```

### B. 运算符优先级

```
优先级（从高到低）：
1. 路径、方法调用       :: .
2. 单目运算符          ! - * &
3. 乘除模              * / %
4. 加减                + -
5. 移位                << >>
6. 比较                == != < > <= >=
7. 逻辑与              &&
8. 逻辑或              ||
9. 赋值                = += -= *= /= 等
```

### C. 标准库组织

```
std::
├── core       // 核心类型（Option, Result）
├── collections // 集合（Vec, HashMap, HashSet）
├── io         // 输入输出
├── fs         // 文件系统
├── net        // 网络编程
├── sync       // 同步原语（Mutex, RwLock, Arc）
├── thread     // 线程
├── time       // 时间处理
└── math       // 数学函数
```

### D. 常见问题

**Q: ZULON vs Rust？**
- ZULON 更简单：无需理解复杂的所有权
- ZULON 更安全：ARC + 运行时检查辅助
- ZULON 性能接近：零成本抽象 + 编译器优化

**Q: ZULON vs Go？**
- ZULON 更安全：空安全 + 错误类型
- ZULON 更强大：泛型 + trait 系统
- ZULON 性能相当：都是高性能编译语言

**Q: 什么时候选择 ZULON？**
- 需要系统编程（操作系统、数据库、游戏引擎）
- 需要高性能（Web 服务、数据处理）
- 需要类型安全（金融、医疗、航空航天）

**Q: 学习曲线？**
- 有编程经验：1-2 周掌握基础
- 从 Go/Python 迁移：2-4 周完全适应
- 从 Rust 迁移：更简单，快速上手

---

## 总结

ZULON 的设计理念是**简单性、安全性、性能的平衡**：

✅ **简单性**
- 清晰的语法，类似 Go/Python
- 强大的类型推断，减少标注
- 无需理解复杂的生命周期

✅ **安全性**
- 编译期空安全
- 错误类型系统
- 自动内存管理

✅ **性能**
- 零成本抽象
- 编译器优化（内联、逃逸分析）
- 接近 C++ 的性能

**开始使用 ZULON，让系统编程像写脚本一样简单！**

---

**参考资料：**
- [Swift ARC - Automatic Reference Counting](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/automaticreferencecounting/)
- [Go Concurrency Patterns](https://compositecode.blog/2025/06/22/concurrency-patterns-in-go-a-short-deep-dive-series/)
- [Region-Based Memory Management](https://en.wikipedia.org/wiki/Region-based_memory_management)
- [Memory Safe Languages](https://medium.com/@QuarkAndCode/memory-safe-languages-in-practice-rust-gc-and-a-roadmap-for-c-c-teams-6b8f81814449)
