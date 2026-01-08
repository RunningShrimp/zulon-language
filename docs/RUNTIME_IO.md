# ZULON 运行时实现：非阻塞 IO 和事件循环

**版本**: 1.0
**日期**: 2026-01-07
**作者**: ZULON Language Team
**状态**: 技术设计

---

## 目录

1. [概述](#概述)
2. [非阻塞 IO 架构](#非阻塞-io-架构)
3. [事件循环实现](#事件循环实现)
4. [平台特定实现](#平台特定实现)
5. [Channel 选择机制](#channel-选择机制)
6. [性能优化](#性能优化)

---

## 概述

ZULON 运行时默认使用**非阻塞 IO**模型，针对不同平台使用最优的实现机制。

### 设计目标

1. **零成本抽象**: 同步代码风格，异步执行
2. **平台最优**: 自动选择平台最优的 IO 机制
3. **可扩展**: 支持百万级并发连接
4. **易用性**: 简单的 API，无需复杂的回调

### 核心组件

```
┌─────────────────────────────────────────────────┐
│          ZULON Async Runtime                    │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─────────────────────────────────────────┐   │
│  │        Async/Await Engine               │   │
│  │  (Future, Stream, Task)                 │   │
│  └──────────────┬──────────────────────────┘   │
│                 │                               │
│                 ▼                               │
│  ┌─────────────────────────────────────────┐   │
│  │         Event Loop                      │   │
│  │  (epoll / IOCP / kqueue / io_uring)    │   │
│  └──────────────┬──────────────────────────┘   │
│                 │                               │
│                 ▼                               │
│  ┌─────────────────────────────────────────┐   │
│  │      Non-blocking I/O Layer            │   │
│  │  (File, Socket, Pipe, Timer)           │   │
│  └──────────────┬──────────────────────────┘   │
│                 │                               │
│                 ▼                               │
│  ┌─────────────────────────────────────────┐   │
│  │        Platform Abstraction            │   │
│  │  (Linux / macOS / Windows / BSD)       │   │
│  └─────────────────────────────────────────┘   │
│                                                  │
└─────────────────────────────────────────────────┘
```

---

## 非阻塞 IO 架构

### 异步 IO Trait

```rust
// 异步读取 trait
pub trait AsyncRead {
    /// 异步读取数据到缓冲区
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError>;

    /// 异步读取确切数量
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), IoError>;
}

// 异步写入 trait
pub trait AsyncWrite {
    /// 异步写入数据
    async fn write(&mut self, buf: &[u8]) -> Result<usize, IoError>;

    /// 异步刷新缓冲区
    async fn flush(&mut self) -> Result<(), IoError>;

    /// 异步写入所有数据
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), IoError>;
}
```

### 标准类型实现

所有标准 IO 类型都实现异步 traits：

```rust
// 文件
impl AsyncRead for File;
impl AsyncWrite for File;

// TCP 流
impl AsyncRead for TcpStream;
impl AsyncWrite for TcpStream;

// Unix 流
impl AsyncRead for UnixStream;
impl AsyncWrite for UnixStream;

// 管道
impl AsyncRead for Pipe;
impl AsyncWrite for Pipe;

// 标准输入/输出
impl AsyncRead for Stdin;
impl AsyncWrite for Stdout;
impl AsyncWrite for Stderr;
```

### 使用示例

```go
// 异步文件读取
use std::io::AsyncRead

async fn read_file(path: str) -> Result<String, IoError> {
    let file = async_open(path).await?
    let mut buf = vec![0u8; 4096]
    let n = file.read(&mut buf).await?
    return Ok(String::from_utf8_lossy(&buf[0..n]))
}

// 异步 HTTP 请求
async fn fetch_url(url: str) -> Result<Response, HttpError> {
    let stream = TcpStream::connect(url.host(), 80).await?

    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", url.path(), url.host())
    stream.write_all(request.as_bytes()).await?

    let mut response = vec![]
    stream.read_to_end(&mut response).await?

    return Ok(parse_response(response))
}

// 异步 TCP 服务器
async fn run_server() -> Result<(), IoError> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?

    loop {
        select {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        spawn handle_client(stream)
                    },
                    Err(e) => {
                        eprintln!("Accept error: {:?}", e)
                    },
                }
            },
            _ = timer.after(Duration::seconds(60)) => {
                println!("Server running...")
            },
        }
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut buf = vec![0u8; 1024]
    let n = stream.read(&mut buf).await?

    let response = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, World!"
    stream.write_all(response.as_bytes()).await?
    stream.flush().await?
}
```

---

## 事件循环实现

### 抽象接口

```rust
// 事件循环 trait
pub trait EventLoop {
    /// 注册 IO 事件
    fn register(&mut self, fd: RawFd, interest: Interest) -> Result<(), IoError>;

    /// 注销 IO 事件
    fn unregister(&mut self, fd: RawFd) -> Result<(), IoError>;

    /// 修改事件兴趣
    fn reregister(&mut self, fd: RawFd, interest: Interest) -> Result<(), IoError>;

    /// 运行事件循环
    fn run(&mut self) -> Result<(), IoError>;

    /// 运行一次迭代
    fn tick(&mut self) -> Result<bool, IoError>;
}

// 事件兴趣
pub enum Interest {
    Readable,
    Writable,
    Both,
}

// 事件处理器
pub trait EventHandler {
    fn handle_event(&mut self, event: Event) -> Result<(), IoError>;
}
```

### Reactor 模式

```rust
// Reactor: 单线程事件循环
pub struct Reactor {
    event_loop: Box<dyn EventLoop>,
    handlers: HashMap<RawFd, Box<dyn EventHandler>>,
    tasks: Vec<Box<dyn Future>>,
}

impl Reactor {
    pub fn new() -> Result<Self, IoError> {
        let event_loop = platform_event_loop()?;
        Ok(Reactor {
            event_loop,
            handlers: HashMap::new(),
            tasks: vec![],
        })
    }

    /// 注册 IO 事件
    pub fn register(&mut self, fd: RawFd, handler: Box<dyn EventHandler>) -> Result<(), IoError> {
        self.event_loop.register(fd, Interest::Both)?;
        self.handlers.insert(fd, handler);
        Ok(())
    }

    /// 生成异步任务
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push(Box::pin(future));
    }

    /// 运行事件循环
    pub fn run(&mut self) -> Result<(), IoError> {
        loop {
            // 1. 处理 IO 事件
            self.event_loop.tick()?;

            // 2. 推进任务
            self.poll_tasks();

            // 3. 检查是否所有任务完成
            if self.tasks.is_empty() && self.handlers.is_empty() {
                break;
            }
        }
        Ok(())
    }

    fn poll_tasks(&mut self) {
        let mut i = 0;
        while i < self.tasks.len() {
            let mut task = self.tasks.swap_remove(i);

            // 尝试推进任务
            match Pin::new(&mut task).poll(&mut Context::none()) {
                Poll::Ready(()) => {
                    // 任务完成
                },
                Poll::Pending => {
                    // 任务未完成，放回列表
                    self.tasks.push(task);
                    i += 1;
                },
            }
        }
    }
}
```

---

## 平台特定实现

### Linux: io_uring

io_uring 是 Linux 5.1+ 引入的高性能异步 IO 接口。

```rust
// io_uring 事件循环
pub struct IoUringEventLoop {
    ring: io_uring,
    submissions: Vec<io_uring_sqe>,
    completions: Vec<io_uring_cqe>,
}

impl IoUringEventLoop {
    pub fn new(entries: u32) -> Result<Self, IoError> {
        let mut params = io_uring_params::default();
        let ring = io_uring_setup(entries, &params)?;

        Ok(IoUringEventLoop {
            ring,
            submissions: Vec::with_capacity(entries as usize),
            completions: Vec::with_capacity(entries as usize),
        })
    }
}

impl EventLoop for IoUringEventLoop {
    fn register(&mut self, fd: RawFd, interest: Interest) -> Result<(), IoError> {
        // io_uring 不需要显式注册，直接提交 SQE
        Ok(())
    }

    fn tick(&mut self) -> Result<bool, IoError> {
        // 1. 提交待处理的 SQE
        io_uring_submit(&mut self.ring)?;

        // 2. 等待完成
        let count = io_uring_wait_cqe(&mut self.ring, 1)?;

        // 3. 处理完成的 CQE
        for _ in 0..count {
            let cqe = io_uring_get_cqe(&mut self.ring)?;

            // 处理事件
            let user_data = cqe.user_data;
            let res = cqe.res;

            if let Some(handler) = self.handlers.get_mut(&(user_data as i32)) {
                handler.handle_event(Event::new(user_data, res))?;
            }

            io_uring_cqe_seen(&mut self.ring, cqe);
        }

        Ok(count > 0)
    }
}

// 异步文件读取使用 io_uring
pub struct IoUringFile {
    fd: RawFd,
    ring: Arc<Mutex<IoUringEventLoop>>,
}

impl AsyncRead for IoUringFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        let future = IoUringReadFuture {
            fd: self.fd,
            buf: buf.as_mut_ptr(),
            len: buf.len(),
            ring: self.ring.clone(),
        };

        future.await
    }
}

struct IoUringReadFuture {
    fd: RawFd,
    buf: *mut u8,
    len: usize,
    ring: Arc<Mutex<IoUringEventLoop>>,
}

impl Future for IoUringReadFuture {
    type Output = Result<usize, IoError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut ring = self.ring.lock().unwrap();

        // 准备 SQE
        let sqe = io_uring_get_sqe(&mut ring.ring)?;
        io_uring_prep_read(sqe, self.fd, self.buf, self.len, 0);
        sqe.user_data = self as *mut _ as u64;

        // 提交
        io_uring_submit(&mut ring.ring)?;

        // 等待完成
        let cqe = io_uring_wait_cqe(&mut ring.ring, 1)?;
        let res = cqe.res;
        io_uring_cqe_seen(&mut ring.ring, cqe);

        Poll::Ready(Ok(res as usize))
    }
}
```

**优势**:
- 零拷贝，减少系统调用
- 批量提交和完成
- 更高的吞吐量
- 更低的延迟

### Linux: epoll

epoll 是 Linux 传统的 IO 事件通知机制。

```rust
// epoll 事件循环
pub struct EpollEventLoop {
    epoll_fd: RawFd,
    events: Vec<epoll_event>,
    handlers: HashMap<RawFd, Box<dyn EventHandler>>,
}

impl EpollEventLoop {
    pub fn new() -> Result<Self, IoError> {
        let epoll_fd = epoll_create1(epoll_flags::EPOLL_CLOEXEC)?;

        Ok(EpollEventLoop {
            epoll_fd,
            events: vec![epoll_event::zero(); 1024],
            handlers: HashMap::new(),
        })
    }
}

impl EventLoop for EpollEventLoop {
    fn register(&mut self, fd: RawFd, interest: Interest) -> Result<(), IoError> {
        let mut event = epoll_event {
            events: match interest {
                Interest::Readable => EPOLLIN | EPOLLET,
                Interest::Writable => EPOLLOUT | EPOLLET,
                Interest::Both => EPOLLIN | EPOLLOUT | EPOLLET,
            },
            data: epoll_data { fd: fd as u64 },
            ..epoll_event::zero()
        };

        epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, fd, &mut event)?;
        Ok(())
    }

    fn tick(&mut self) -> Result<bool, IoError> {
        let nfds = epoll_wait(
            self.epoll_fd,
            self.events.as_mut_ptr(),
            self.events.len() as i32,
            0,  // 非阻塞
        )?;

        for i in 0..nfds {
            let event = &self.events[i as usize];
            let fd = event.data.fd as i32;

            if let Some(handler) = self.handlers.get_mut(&fd) {
                handler.handle_event(Event::from_epoll(event))?;
            }
        }

        Ok(nfds > 0)
    }

    fn run(&mut self) -> Result<(), IoError> {
        loop {
            let nfds = epoll_wait(
                self.epoll_fd,
                self.events.as_mut_ptr(),
                self.events.len() as i32,
                -1,  // 无限等待
            )?;

            for i in 0..nfds {
                let event = &self.events[i as usize];
                let fd = event.data.fd as i32;

                if let Some(handler) = self.handlers.get_mut(&fd) {
                    handler.handle_event(Event::from_epoll(event))?;
                }
            }
        }
    }
}
```

**特性**:
- 边缘触发 (EPOLLET) - 减少无效唤醒
- 支持 EPOLLONESHOT - 避免饥饿
- 支持 EPOLLPRI - 紧急数据

### Windows: IOCP

IOCP (I/O Completion Ports) 是 Windows 的高性能异步 IO 接口。

```rust
// IOCP 事件循环
pub struct IocpEventLoop {
    iocp_handle: HANDLE,
    handlers: HashMap<HANDLE, Box<dyn EventHandler>>,
    active_ios: HashSet<usize>,
}

impl IocpEventLoop {
    pub fn new() -> Result<Self, IoError> {
        let iocp_handle = CreateIoCompletionPort(
            INVALID_HANDLE_VALUE,
            null_mut(),
            0,
            0,  // 并发线程数（0 = CPU 核心数)
        )?;

        Ok(IocpEventLoop {
            iocp_handle,
            handlers: HashMap::new(),
            active_ios: HashSet::new(),
        })
    }

    pub fn associate_handle(&mut self, handle: HANDLE) -> Result<(), IoError> {
        CreateIoCompletionPort(handle, self.iocp_handle, 0, 0)?;
        Ok(())
    }
}

impl EventLoop for IocpEventLoop {
    fn tick(&mut self) -> Result<bool, IoError> {
        let mut bytes_transferred = 0;
        let mut completion_key = 0;
        let mut overlapped = null_mut();

        // 非阻塞等待
        let success = GetQueuedCompletionStatus(
            self.iocp_handle,
            &mut bytes_transferred,
            &mut completion_key,
            &mut overlapped,
            0,  // 非阻塞
        );

        if success == FALSE {
            let error = GetLastError();
            if error == WAIT_TIMEOUT {
                return Ok(false);
            }
            return Err(IoError::from_raw_os_error(error));
        }

        // 处理完成的事件
        let overlapped_ptr = overlapped as *const Overlapped;
        if let Some(handler) = self.handlers.get_mut(&completion_key) {
            handler.handle_event(Event::from_iocp(overlapped_ptr, bytes_transferred))?;
        }

        Ok(true)
    }

    fn run(&mut self) -> Result<(), IoError> {
        loop {
            let mut bytes_transferred = 0;
            let mut completion_key = 0;
            let mut overlapped = null_mut();

            GetQueuedCompletionStatus(
                self.iocp_handle,
                &mut bytes_transferred,
                &mut completion_key,
                &mut overlapped,
                INFINITE,
            );

            let overlapped_ptr = overlapped as *const Overlapped;
            if let Some(handler) = self.handlers.get_mut(&completion_key) {
                handler.handle_event(Event::from_iocp(overlapped_ptr, bytes_transferred))?;
            }
        }
    }
}

// 异步文件读取使用 IOCP
pub struct IocpFile {
    handle: HANDLE,
    event_loop: Arc<Mutex<IocpEventLoop>>,
}

impl AsyncRead for IocpFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        let future = IocpReadFuture {
            handle: self.handle,
            buf: buf.as_mut_ptr(),
            len: buf.len(),
            event_loop: self.event_loop.clone(),
        };

        future.await
    }
}

struct IocpReadFuture {
    handle: HANDLE,
    buf: *mut u8,
    len: usize,
    event_loop: Arc<Mutex<IocpEventLoop>>,
}

impl Future for IocpReadFuture {
    type Output = Result<usize, IoError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut overlapped = Overlapped::default();
        let mut bytes_read = 0;

        let success = ReadFile(
            self.handle,
            self.buf,
            self.len,
            &mut bytes_read,
            &mut overlapped,
        );

        if success == FALSE {
            let error = GetLastError();
            if error != ERROR_IO_PENDING {
                return Poll::Ready(Err(IoError::from_raw_os_error(error)));
            }
        }

        // 等待完成
        let mut loop = self.event_loop.lock().unwrap();
        loop {
            match loop.tick() {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }

        Poll::Ready(Ok(bytes_read as usize))
    }
}
```

### macOS/BSD: kqueue

kqueue 是 BSD 系统（包括 macOS）的事件通知机制。

```rust
// kqueue 事件循环
pub struct KqueueEventLoop {
    kq: RawFd,
    handlers: HashMap<RawFd, Box<dyn EventHandler>>,
    changes: Vec<kevent>,
}

impl KqueueEventLoop {
    pub fn new() -> Result<Self, IoError> {
        let kq = kqueue()?;

        Ok(KqueueEventLoop {
            kq,
            handlers: HashMap::new(),
            changes: vec![],
        })
    }
}

impl EventLoop for KqueueEventLoop {
    fn register(&mut self, fd: RawFd, interest: Interest) -> Result<(), IoError> {
        let mut filter = match interest {
            Interest::Readable => EVFILT_READ,
            Interest::Writable => EVFILT_WRITE,
            Interest::Both => EVFILT_READ,
        };

        let kevent = kevent {
            ident: fd as uintptr_t,
            filter: filter,
            flags: EV_ADD | EV_ENABLE | EV_ONESHOT,
            fflags: 0,
            data: 0,
            udata: null_mut(),
        };

        self.changes.push(kevent);
        kevent(self.kq, &self.changes, 1, null_mut(), 0, null_mut())?;
        self.changes.clear();

        Ok(())
    }

    fn tick(&mut self) -> Result<bool, IoError> {
        let mut events = vec![kevent::zero(); 1024];

        let nev = kevent(
            self.kq,
            null_mut(),
            0,
            events.as_mut_ptr(),
            events.len() as i32,
            null_mut(),  // 非阻塞
        )?;

        for i in 0..nev {
            let event = &events[i as usize];
            let fd = event.ident as RawFd;

            if let Some(handler) = self.handlers.get_mut(&fd) {
                handler.handle_event(Event::from_kevent(event))?;
            }
        }

        Ok(nev > 0)
    }

    fn run(&mut self) -> Result<(), IoError> {
        let mut events = vec![kevent::zero(); 1024];

        loop {
            let nev = kevent(
                self.kq,
                null_mut(),
                0,
                events.as_mut_ptr(),
                events.len() as i32,
                null_mut(),  // 无限等待
            )?;

            for i in 0..nev {
                let event = &events[i as usize];
                let fd = event.ident as RawFd;

                if let Some(handler) = self.handlers.get_mut(&fd) {
                    handler.handle_event(Event::from_kevent(event))?;
                }
            }
        }
    }
}
```

---

## Channel 选择机制

### Select 语句

ZULON 提供 `select` 语句用于等待多个 channel。

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

### 实现原理

```rust
// Select 运行时
pub fn select<T>(channels: &[&dyn Channel]) -> Option<(usize, T)> {
    // 1. 注册所有 channel 到事件循环
    for (i, channel) in channels.iter().enumerate() {
        channel.register_interest(i)?;
    }

    // 2. 等待任意 channel 就绪
    loop {
        for (i, channel) in channels.iter().enumerate() {
            if let Some(value) = channel.try_recv() {
                return Some((i, value));
            }
        }

        // 让出 CPU，等待事件
        yield_thread();
    }
}

// Channel trait
pub trait Channel {
    fn register_interest(&self, index: usize) -> Result<(), IoError>;
    fn try_recv(&self) -> Option<Value>;
}

// Receiver 实现
impl<T> Channel for Receiver<T> {
    fn register_interest(&self, index: usize) -> Result<(), IoError> {
        EVENT_LOOP.register(self.fd, Interest::Readable)?;
        Ok(())
    }

    fn try_recv(&self) -> Option<Value> {
        self.inner.try_recv().ok()
    }
}
```

---

## 性能优化

### 批量处理

```rust
// 批量提交 IO 请求
pub struct BatchedIoUring {
    ring: io_uring,
    pending: Vec<io_uring_sqe>,
    batch_size: usize,
}

impl BatchedIoUring {
    pub fn new(entries: u32, batch_size: usize) -> Result<Self, IoError> {
        let ring = io_uring_setup(entries, &io_uring_params::default())?;

        Ok(BatchedIoUring {
            ring,
            pending: Vec::with_capacity(batch_size),
            batch_size,
        })
    }

    pub fn submit_read(&mut self, fd: RawFd, buf: &mut [u8]) -> impl Future<Output = Result<usize, IoError>> {
        let sqe = io_uring_get_sqe(&mut self.ring)?;
        io_uring_prep_read(sqe, fd, buf.as_mut_ptr(), buf.len(), 0);

        self.pending.push(*sqe);

        if self.pending.len() >= self.batch_size {
            self.flush()?;
        }

        IoUringFuture { sqe }
    }

    fn flush(&mut self) -> Result<(), IoError> {
        io_uring_submit(&mut self.ring)?;
        self.pending.clear();
        Ok(())
    }
}
```

### 零拷贝

```rust
// 零拷贝文件传输
pub async fn sendfile(out: &mut TcpStream, file: &mut File, len: usize) -> Result<usize, IoError> {
    #[cfg(target_os = "linux")]
    {
        // 使用 sendfile 系统调用
        let mut offset = 0;
        let mut sent = 0;

        while sent < len {
            let n = sendfile(
                out.as_raw_fd(),
                file.as_raw_fd(),
                &mut offset,
                len - sent,
                null_mut(),
                0,
            )?;

            if n == 0 {
                break;
            }

            sent += n;
        }

        Ok(sent)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // 回退到普通读取
        let mut buf = vec![0u8; 8192];
        let mut total = 0;

        while total < len {
            let n = file.read(&mut buf).await?;
            if n == 0 {
                break;
            }

            out.write_all(&buf[0..n]).await?;
            total += n;
        }

        Ok(total)
    }
}
```

### SPSC (Single Producer Single Consumer) 无锁队列

```rust
// 无锁 SPSC 队列
pub struct SpscQueue<T> {
    buffer: Vec<Option<T>>,
    mask: usize,
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
}

impl<T> SpscQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());

        let buffer = (0..capacity)
            .map(|_| None)
            .collect();

        SpscQueue {
            buffer,
            mask: capacity - 1,
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if tail - head > self.mask {
            return Err(value);  // 队列满
        }

        let index = tail & self.mask;
        self.buffer[index] = Some(value);

        self.tail.store(tail + 1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if head == tail {
            return None;  // 队列空
        }

        let index = head & self.mask;
        let value = self.buffer[index].take()?;

        self.head.store(head + 1, Ordering::Release);
        Some(value)
    }
}
```

---

**文档版本**: 1.0
**最后更新**: 2026-01-07
**维护者**: ZULON Language Team
