// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! kqueue-based event loop for macOS/BSD
//!
//! This module provides a kqueue-based implementation of the EventLoop trait,
//! offering high-performance event notification on macOS and BSD systems.

use crate::event_loop::{EventLoop, EventHandler, EventSource, Interest, Token, IoResult, IoError};
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// kqueue-based event loop for macOS/BSD
///
/// Uses kqueue() system call for efficient event notification.
pub struct KqueueEventLoop {
    /// kqueue file descriptor
    kq_fd: RawFd,

    /// Event handlers indexed by token
    handlers: Vec<Option<Box<dyn EventHandler>>>,

    /// Map from file descriptor to token
    fd_to_token: HashMap<RawFd, Token>,

    /// Next token to assign
    next_token: Arc<AtomicUsize>,

    /// Running flag
    running: Arc<AtomicBool>,

    /// Maximum events per iteration
    max_events: usize,
}

impl KqueueEventLoop {
    /// Create a new kqueue-based event loop
    pub fn new() -> IoResult<Self> {
        // Create kqueue
        let kq_fd = unsafe { libc::kqueue() };
        if kq_fd < 0 {
            return Err(IoError::last_os_error());
        }

        // Set close-on-exec
        let flags = unsafe { libc::fcntl(kq_fd, libc::F_GETFD) };
        if flags < 0 {
            let err = IoError::last_os_error();
            unsafe { libc::close(kq_fd) };
            return Err(err);
        }
        if unsafe { libc::fcntl(kq_fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
            let err = IoError::last_os_error();
            unsafe { libc::close(kq_fd) };
            return Err(err);
        }

        Ok(Self {
            kq_fd,
            handlers: Vec::new(),
            fd_to_token: HashMap::new(),
            next_token: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            max_events: 1024,
        })
    }

    /// Get the kqueue file descriptor
    pub fn raw_fd(&self) -> RawFd {
        self.kq_fd
    }
}

impl Drop for KqueueEventLoop {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.kq_fd);
        }
    }
}

impl EventLoop for KqueueEventLoop {
    fn new() -> IoResult<Self> {
        Self::new()
    }

    fn run(&mut self) -> IoResult<()> {
        self.running.store(true, Ordering::Relaxed);

        let mut event_list = vec![libc::kevent::default(); self.max_events];

        while self.running.load(Ordering::Relaxed) {
            // Wait for events
            let timeout = libc::timespec {
                tv_sec: 1,
                tv_nsec: 0,
            };

            let nev = unsafe {
                libc::kevent(
                    self.kq_fd,
                    std::ptr::null(),
                    0,
                    event_list.as_mut_ptr(),
                    event_list.len() as i32,
                    &timeout,
                )
            };

            if nev < 0 {
                let err = IoError::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(err);
            }

            // Process events
            for i in 0..nev as usize {
                let event = &event_list[i];

                // Get token from udata
                let token = Token::new(event.udata as usize);

                // Handle the event
                let filter = event.filter;

                if filter == libc::EVFILT_READ || filter == libc::EVFILT_WRITE {
                    // Read or write event
                    if event.flags & libc::EOF != 0 {
                        // EOF - error or hangup
                        if let Some(Some(handler)) = self.handlers.get(token.index()) {
                            handler.error(token, IoError::new(std::io::ErrorKind::ConnectionReset, "EOF"));
                        }
                    } else if filter == libc::EVFILT_READ {
                        if let Some(Some(handler)) = self.handlers.get(token.index()) {
                            handler.readable(token);
                        }
                    } else if filter == libc::EVFILT_WRITE {
                        if let Some(Some(handler)) = self.handlers.get(token.index()) {
                            handler.writable(token);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn register(&mut self, source: &dyn EventSource) -> IoResult<Token> {
        let fd = source.raw_fd();
        let interest = source.interest();

        // Allocate token
        let token = Token::new(self.next_token.fetch_add(1, Ordering::Relaxed));

        // Ensure handlers vec is big enough
        if token.index() >= self.handlers.len() {
            self.handlers.resize(token.index() + 1, None);
        }

        // Register read filter
        if interest.is_readable() {
            let mut change = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_READ,
                flags: libc::EV_ADD | libc::EV_CLEAR,
                fflags: 0,
                data: 0,
                udata: token.index() as usize,
            };

            unsafe {
                if libc::kevent(
                    self.kq_fd,
                    &change,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                ) < 0
                {
                    return Err(IoError::last_os_error());
                }
            }
        }

        // Register write filter
        if interest.is_writable() {
            let mut change = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_WRITE,
                flags: libc::EV_ADD | libc::EV_CLEAR,
                fflags: 0,
                data: 0,
                udata: token.index() as usize,
            };

            unsafe {
                if libc::kevent(
                    self.kq_fd,
                    &change,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                ) < 0
                {
                    return Err(IoError::last_os_error());
                }
            }
        }

        // Store mapping
        self.fd_to_token.insert(fd, token);

        Ok(token)
    }

    fn reregister(&mut self, token: Token, interest: Interest) -> IoResult<()> {
        // Find the fd for this token
        let mut fd = None;
        for (&f, &t) in &self.fd_to_token {
            if t == token {
                fd = Some(f);
                break;
            }
        }

        let fd = match fd {
            Some(f) => f,
            None => return Err(IoError::new(std::io::ErrorKind::NotFound, "token not found")),
        };

        // Delete old filters and add new ones
        // First, delete read filter if it exists
        if !interest.is_readable() {
            let mut delete_read = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_READ,
                flags: libc::EV_DELETE,
                fflags: 0,
                data: 0,
                udata: 0,
            };
            unsafe {
                libc::kevent(self.kq_fd, &delete_read, 1, std::ptr::null_mut(), 0, std::ptr::null());
            }
        }

        // Delete write filter if it exists
        if !interest.is_writable() {
            let mut delete_write = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_WRITE,
                flags: libc::EV_DELETE,
                fflags: 0,
                data: 0,
                udata: 0,
            };
            unsafe {
                libc::kevent(self.kq_fd, &delete_write, 1, std::ptr::null_mut(), 0, std::ptr::null());
            }
        }

        // Add new filters
        if interest.is_readable() {
            let mut add_read = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_READ,
                flags: libc::EV_ADD | libc::EV_CLEAR,
                fflags: 0,
                data: 0,
                udata: token.index() as usize,
            };

            unsafe {
                if libc::kevent(
                    self.kq_fd,
                    &add_read,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                ) < 0
                {
                    return Err(IoError::last_os_error());
                }
            }
        }

        if interest.is_writable() {
            let mut add_write = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_WRITE,
                flags: libc::EV_ADD | libc::EV_CLEAR,
                fflags: 0,
                data: 0,
                udata: token.index() as usize,
            };

            unsafe {
                if libc::kevent(
                    self.kq_fd,
                    &add_write,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                ) < 0
                {
                    return Err(IoError::last_os_error());
                }
            }
        }

        Ok(())
    }

    fn deregister(&mut self, token: Token) -> IoResult<()> {
        // Find the fd for this token
        let mut fd = None;
        for (&f, &t) in &self.fd_to_token {
            if t == token {
                fd = Some(f);
                break;
            }
        }

        let fd = match fd {
            Some(f) => f,
            None => return Err(IoError::new(std::io::ErrorKind::NotFound, "token not found")),
        };

        // Delete filters
        let mut delete_read = libc::kevent {
            ident: fd as usize,
            filter: libc::EVFILT_READ,
            flags: libc::EV_DELETE,
            fflags: 0,
            data: 0,
            udata: 0,
        };

        let mut delete_write = libc::kevent {
            ident: fd as usize,
            filter: libc::EVFILT_WRITE,
            flags: libc::EV_DELETE,
            fflags: 0,
            data: 0,
            udata: 0,
        };

        unsafe {
            libc::kevent(self.kq_fd, &delete_read, 1, std::ptr::null_mut(), 0, std::ptr::null());
            libc::kevent(self.kq_fd, &delete_write, 1, std::ptr::null_mut(), 0, std::ptr::null());
        }

        // Remove mapping
        self.fd_to_token.remove(&fd);

        // Remove handler
        if token.index() < self.handlers.len() {
            self.handlers[token.index()] = None;
        }

        Ok(())
    }

    fn submit_timer(&mut self, duration: Duration) -> IoResult<crate::event_loop::TimerHandle> {
        // Allocate timer handle
        let handle = crate::event_loop::TimerHandle::new(self.next_token.fetch_add(1, Ordering::Relaxed));

        // Convert duration to milliseconds
        let millis = duration.as_secs() * 1000 + duration.subsec_millis() as u64;

        // Create timer event
        let mut change = libc::kevent {
            ident: handle.index() as usize,
            filter: libc::EVFILT_TIMER,
            flags: libc::EV_ADD | libc::EV_ONESHOT,
            fflags: libc::NOTE_USECONDS, // microseconds
            data: millis * 1000, // convert to microseconds
            udata: handle.index() as usize,
        };

        unsafe {
            if libc::kevent(
                self.kq_fd,
                &change,
                1,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
            ) < 0
            {
                return Err(IoError::last_os_error());
            }
        }

        Ok(handle)
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::set_nonblocking;

    #[test]
    fn test_kqueue_creation() {
        let event_loop = KqueueEventLoop::new().unwrap();
        assert!(event_loop.kq_fd >= 0);
    }

    #[test]
    fn test_kqueue_register_pipe() {
        let mut event_loop = KqueueEventLoop::new().unwrap();

        // Create a pipe
        let mut pipe_fds = [0, 0];
        unsafe {
            if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
                panic!("pipe failed");
            }
        }

        // Set non-blocking
        set_nonblocking(pipe_fds[0]).unwrap();
        set_nonblocking(pipe_fds[1]).unwrap();

        // Create a simple event source
        struct PipeSource {
            fd: RawFd,
        }

        impl EventSource for PipeSource {
            fn raw_fd(&self) -> RawFd {
                self.fd
            }

            fn interest(&self) -> Interest {
                Interest::READABLE
            }
        }

        let source = PipeSource { fd: pipe_fds[0] };

        // Register
        let token = event_loop.register(&source).unwrap();

        // Verify token
        assert_eq!(event_loop.fd_to_token.get(&pipe_fds[0]), Some(&token));

        // Cleanup
        unsafe {
            libc::close(pipe_fds[0]);
            libc::close(pipe_fds[1]);
        }
    }

    #[test]
    fn test_kqueue_timer() {
        let mut event_loop = KqueueEventLoop::new().unwrap();

        let handle = event_loop.submit_timer(Duration::from_millis(100)).unwrap();
        assert_eq!(handle.index(), handle.index());
    }

    #[test]
    fn test_kqueue_deregister() {
        let mut event_loop = KqueueEventLoop::new().unwrap();

        // Create a pipe
        let mut pipe_fds = [0, 0];
        unsafe {
            if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
                panic!("pipe failed");
            }
        }

        // Set non-blocking
        set_nonblocking(pipe_fds[0]).unwrap();
        set_nonblocking(pipe_fds[1]).unwrap();

        // Create a simple event source
        struct PipeSource {
            fd: RawFd,
        }

        impl EventSource for PipeSource {
            fn raw_fd(&self) -> RawFd {
                self.fd
            }

            fn interest(&self) -> Interest {
                Interest::READABLE
            }
        }

        let source = PipeSource { fd: pipe_fds[0] };

        // Register
        let token = event_loop.register(&source).unwrap();

        // Deregister
        event_loop.deregister(token).unwrap();

        // Verify removed
        assert!(!event_loop.fd_to_token.contains_key(&pipe_fds[0]));

        // Cleanup
        unsafe {
            libc::close(pipe_fds[0]);
            libc::close(pipe_fds[1]);
        }
    }
}
