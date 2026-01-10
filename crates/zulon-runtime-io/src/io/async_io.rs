// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Async I/O primitives
//!
//! This module provides async/await-based wrappers around the poll-based I/O types,
//! enabling ergonomic async network programming.

use crate::error::{IoError, IoResult};
use crate::io::tcp::{TcpStream as PollTcpStream, TcpListener as PollTcpListener, AsyncPending};
use crate::event_loop::{EventLoop, EventSource, Interest, Token};
use zulon_async_futures::{Future, Poll, Context, Waker};
use std::net::SocketAddr;
use std::os::unix::io::RawFd;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Async TCP stream
///
/// This is an async/await wrapper around the poll-based TcpStream.
pub struct AsyncTcpStream {
    inner: PollTcpStream,
    state: Arc<Mutex<StreamState>>,
}

/// Internal state for async operations
struct StreamState {
    waker: Option<Waker>,
    readable: bool,
    writable: bool,
}

impl AsyncTcpStream {
    /// Create a new async TCP stream connected to the specified address
    ///
    /// This returns a Future that completes when the connection is established.
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address to connect to
    ///
    /// # Returns
    ///
    /// A Future that resolves to a connected AsyncTcpStream
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zulon_runtime_io::io::async_io::AsyncTcpStream;
    /// use zulon_async_futures::Executor;
    ///
    /// # async fn example() {
    /// let stream = AsyncTcpStream::connect("127.0.0.1:8080".parse().unwrap()).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # }
    /// ```
    pub fn connect(addr: SocketAddr) -> ConnectFuture {
        ConnectFuture {
            addr,
            state: ConnectState::Start,
        }
    }

    /// Read data from the stream asynchronously
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read into
    ///
    /// # Returns
    ///
    /// A Future that resolves to the number of bytes read
    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> ReadFuture<'a> {
        ReadFuture {
            stream: self,
            buf,
            state: ReadState::Start,
        }
    }

    /// Write data to the stream asynchronously
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to write
    ///
    /// # Returns
    ///
    /// A Future that resolves to the number of bytes written
    pub fn write<'a>(&'a self, buf: &'a [u8]) -> WriteFuture<'a> {
        WriteFuture {
            stream: self,
            buf,
            state: WriteState::Start,
        }
    }

    /// Get the raw file descriptor
    pub fn raw_fd(&self) -> RawFd {
        self.inner.raw_fd()
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> SocketAddr {
        self.inner.local_addr()
    }

    /// Get the peer socket address
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.inner.peer_addr()
    }

    /// Create from a poll-based TcpStream
    pub fn from_poll(stream: PollTcpStream) -> Self {
        Self {
            inner: stream,
            state: Arc::new(Mutex::new(StreamState {
                waker: None,
                readable: false,
                writable: false,
            })),
        }
    }

    /// Get the inner poll-based TcpStream
    pub fn into_inner(self) -> PollTcpStream {
        self.inner
    }

    /// Handle readable event from event loop
    pub(crate) fn handle_readable(&self) {
        let mut state = self.state.lock().unwrap();
        state.readable = true;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }

    /// Handle writable event from event loop
    pub(crate) fn handle_writable(&self) {
        let mut state = self.state.lock().unwrap();
        state.writable = true;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

/// Future for connecting to a remote address
pub struct ConnectFuture {
    addr: SocketAddr,
    state: ConnectState,
}

enum ConnectState {
    Start,
    Pending(Option<PollTcpStream>),
    Done,
}

impl Future for ConnectFuture {
    type Output = Result<AsyncTcpStream, IoError>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match &mut self.state {
            ConnectState::Start => {
                // Start connection
                match PollTcpStream::connect(self.addr) {
                    Ok(AsyncPending::Ready(stream)) => {
                        self.state = ConnectState::Done;
                        Poll::Ready(Ok(AsyncTcpStream::from_poll(stream)))
                    }
                    Ok(AsyncPending::Pending(stream)) => {
                        self.state = ConnectState::Pending(Some(stream));
                        Poll::Pending
                    }
                    Err(e) => Poll::Ready(Err(e)),
                }
            }
            ConnectState::Pending(stream_opt) => {
                // Check if connection completed
                let stream = stream_opt.as_ref().unwrap();
                match stream.try_complete_connect() {
                    Ok(true) => {
                        // Take the stream out
                        let stream = stream_opt.take().unwrap();
                        self.state = ConnectState::Done;
                        Poll::Ready(Ok(AsyncTcpStream::from_poll(stream)))
                    }
                    Ok(false) => Poll::Pending,
                    Err(e) => {
                        self.state = ConnectState::Done;
                        Poll::Ready(Err(e))
                    }
                }
            }
            ConnectState::Done => {
                panic!("ConnectFuture polled after completion");
            }
        }
    }
}

/// Future for reading from a stream
pub struct ReadFuture<'a> {
    stream: &'a AsyncTcpStream,
    buf: &'a mut [u8],
    state: ReadState,
}

enum ReadState {
    Start,
    Pending,
    Done,
}

impl<'a> Future for ReadFuture<'a> {
    type Output = IoResult<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.state {
            ReadState::Start => {
                // Try to read immediately
                match self.stream.inner.read(self.buf) {
                    Ok(n) => {
                        self.state = ReadState::Done;
                        Poll::Ready(Ok(n))
                    }
                    Err(e) if e.kind() == crate::error::IoErrorKind::WouldBlock => {
                        // Need to wait for readability
                        let mut state = self.stream.state.lock().unwrap();
                        state.waker = Some(cx.waker().clone());
                        state.readable = false;
                        self.state = ReadState::Pending;
                        Poll::Pending
                    }
                    Err(e) => {
                        self.state = ReadState::Done;
                        Poll::Ready(Err(e))
                    }
                }
            }
            ReadState::Pending => {
                // Check if readable
                let state = self.stream.state.lock().unwrap();
                if state.readable {
                    drop(state);
                    // Try reading again
                    self.state = ReadState::Start;
                    Pin::new(&mut *self).poll(cx)
                } else {
                    Poll::Pending
                }
            }
            ReadState::Done => {
                panic!("ReadFuture polled after completion");
            }
        }
    }
}

/// Future for writing to a stream
pub struct WriteFuture<'a> {
    stream: &'a AsyncTcpStream,
    buf: &'a [u8],
    state: WriteState,
}

enum WriteState {
    Start,
    Pending,
    Done,
}

impl<'a> Future for WriteFuture<'a> {
    type Output = IoResult<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.state {
            WriteState::Start => {
                // Try to write immediately
                match self.stream.inner.write(self.buf) {
                    Ok(n) => {
                        self.state = WriteState::Done;
                        Poll::Ready(Ok(n))
                    }
                    Err(e) if e.kind() == crate::error::IoErrorKind::WouldBlock => {
                        // Need to wait for writability
                        let mut state = self.stream.state.lock().unwrap();
                        state.waker = Some(cx.waker().clone());
                        state.writable = false;
                        self.state = WriteState::Pending;
                        Poll::Pending
                    }
                    Err(e) => {
                        self.state = WriteState::Done;
                        Poll::Ready(Err(e))
                    }
                }
            }
            WriteState::Pending => {
                // Check if writable
                let state = self.stream.state.lock().unwrap();
                if state.writable {
                    drop(state);
                    // Try writing again
                    self.state = WriteState::Start;
                    Pin::new(&mut *self).poll(cx)
                } else {
                    Poll::Pending
                }
            }
            WriteState::Done => {
                panic!("WriteFuture polled after completion");
            }
        }
    }
}

/// Async TCP listener
///
/// This is an async/await wrapper around the poll-based TcpListener.
pub struct AsyncTcpListener {
    inner: PollTcpListener,
    state: Arc<Mutex<ListenerState>>,
}

/// Internal state for listener
struct ListenerState {
    waker: Option<Waker>,
    readable: bool,
}

impl AsyncTcpListener {
    /// Create a new async TCP listener bound to the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - Local address to bind to
    ///
    /// # Returns
    ///
    /// A Future that resolves to a bound AsyncTcpListener
    pub fn bind(addr: SocketAddr) -> BindFuture {
        BindFuture {
            addr,
            state: BindState::Start,
        }
    }

    /// Accept an incoming connection asynchronously
    ///
    /// # Returns
    ///
    /// A Future that resolves to an accepted connection
    pub fn accept<'a>(&'a self) -> AcceptFuture<'a> {
        AcceptFuture {
            listener: self,
            state: AcceptState::Start,
        }
    }

    /// Get the raw file descriptor
    pub fn raw_fd(&self) -> RawFd {
        self.inner.raw_fd()
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> SocketAddr {
        self.inner.local_addr()
    }

    /// Create from a poll-based TcpListener
    pub fn from_poll(listener: PollTcpListener) -> Self {
        Self {
            inner: listener,
            state: Arc::new(Mutex::new(ListenerState {
                waker: None,
                readable: false,
            })),
        }
    }

    /// Handle readable event from event loop
    pub(crate) fn handle_readable(&self) {
        let mut state = self.state.lock().unwrap();
        state.readable = true;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

/// Future for binding a listener
pub struct BindFuture {
    addr: SocketAddr,
    state: BindState,
}

enum BindState {
    Start,
    Done,
}

impl Future for BindFuture {
    type Output = IoResult<AsyncTcpListener>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match self.state {
            BindState::Start => {
                match PollTcpListener::bind(self.addr) {
                    Ok(listener) => {
                        self.state = BindState::Done;
                        Poll::Ready(Ok(AsyncTcpListener::from_poll(listener)))
                    }
                    Err(e) => {
                        self.state = BindState::Done;
                        Poll::Ready(Err(e))
                    }
                }
            }
            BindState::Done => {
                panic!("BindFuture polled after completion");
            }
        }
    }
}

/// Future for accepting a connection
pub struct AcceptFuture<'a> {
    listener: &'a AsyncTcpListener,
    state: AcceptState,
}

enum AcceptState {
    Start,
    Pending,
    Done,
}

impl<'a> Future for AcceptFuture<'a> {
    type Output = IoResult<(AsyncTcpStream, SocketAddr)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.state {
            AcceptState::Start => {
                // Try to accept immediately
                match self.listener.inner.accept() {
                    Ok(Some((stream, addr))) => {
                        self.state = AcceptState::Done;
                        Poll::Ready(Ok((AsyncTcpStream::from_poll(stream), addr)))
                    }
                    Ok(None) => {
                        // WouldBlock - need to wait for readability
                        let mut state = self.listener.state.lock().unwrap();
                        state.waker = Some(cx.waker().clone());
                        state.readable = false;
                        self.state = AcceptState::Pending;
                        Poll::Pending
                    }
                    Err(e) => {
                        self.state = AcceptState::Done;
                        Poll::Ready(Err(e))
                    }
                }
            }
            AcceptState::Pending => {
                // Check if readable
                let state = self.listener.state.lock().unwrap();
                if state.readable {
                    drop(state);
                    // Try accepting again
                    self.state = AcceptState::Start;
                    Pin::new(&mut *self).poll(cx)
                } else {
                    Poll::Pending
                }
            }
            AcceptState::Done => {
                panic!("AcceptFuture polled after completion");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_tcp_stream_creation() {
        // This is a compile-time test to ensure the types work
        // Actual async tests would require an executor
        let _addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        // AsyncTcpStream::connect(_addr); // Would need executor to run
    }

    #[test]
    fn test_async_tcp_listener_creation() {
        let _addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        // AsyncTcpListener::bind(_addr); // Would need executor to run
    }
}
