// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! TCP networking types for non-blocking I/O
//!
//! This module provides TCP stream and listener types that integrate
//! with the event loop for asynchronous I/O operations.

use crate::error::{IoError, IoResult};
use crate::io::poll::set_nonblocking;
use crate::event_loop::{EventSource, Interest};

use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};
use std::os::unix::io::RawFd;

/// TCP stream for non-blocking network communication
///
/// A TCP stream represents a network connection to a remote endpoint.
/// It can be used with the event loop for asynchronous read/write operations.
pub struct TcpStream {
    fd: RawFd,
    local_addr: SocketAddr,
    peer_addr: Option<SocketAddr>,
}

impl TcpStream {
    /// Create a new TCP stream connected to the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address to connect to
    ///
    /// # Returns
    ///
    /// Returns `Ok(AsyncPending::Ready(stream))` if immediately connected,
    /// or `Ok(AsyncPending::Pending(stream))` if connection is in progress
    /// and should be registered with the event loop for writability.
    ///
    /// # Errors
    ///
    /// Returns an error if socket creation or connection fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zulon_runtime_io::io::tcp::TcpStream;
    /// use std::net::SocketAddr;
    ///
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// match TcpStream::connect(addr) {
    ///     Ok(AsyncPending::Ready(stream)) => {
    ///         // Immediately connected
    ///     }
    ///     Ok(AsyncPending::Pending(stream)) => {
    ///         // Need to wait for writability
    ///         // Register with event loop...
    ///     }
    ///     Err(e) => {
    ///         // Connection failed
    ///     }
    /// }
    /// ```
    pub fn connect(addr: SocketAddr) -> Result<AsyncPending<TcpStream>, IoError> {
        // Create socket
        let fd = match addr {
            SocketAddr::V4(_) => unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) },
            SocketAddr::V6(_) => unsafe { libc::socket(libc::AF_INET6, libc::SOCK_STREAM, 0) },
        };

        if fd < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Set non-blocking
        if let Err(e) = set_nonblocking(fd) {
            unsafe { libc::close(fd); }
            return Err(e);
        }

        // Initiate connection
        let (sockaddr, sockaddr_len) = addr_to_libc(&addr);
        let result = unsafe {
            libc::connect(fd, sockaddr, sockaddr_len)
        };

        if result < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINPROGRESS) {
                // Connection in progress
                return Ok(AsyncPending::Pending(TcpStream {
                    fd,
                    local_addr: SocketAddr::V4(std::net::SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
                    peer_addr: Some(addr),
                }));
            }
            unsafe { libc::close(fd); }
            return Err(IoError::from(err));
        }

        // Immediately connected
        Ok(AsyncPending::Ready(TcpStream {
            fd,
            local_addr: SocketAddr::V4(std::net::SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
            peer_addr: Some(addr),
        }))
    }

    /// Try to complete a pending connection
    ///
    /// Should be called after the event loop signals writability.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if connection completed successfully
    /// - `Ok(false)` if still in progress
    /// - `Err(e)` if connection failed
    pub fn try_complete_connect(&self) -> Result<bool, IoError> {
        let mut error: i32 = 0;
        let mut error_len = std::mem::size_of::<i32>() as libc::socklen_t;

        let result = unsafe {
            libc::getsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                &mut error as *mut i32 as *mut libc::c_void,
                &mut error_len,
            )
        };

        if result < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        if error == 0 {
            Ok(true) // Connected
        } else {
            Err(IoError::new(crate::error::IoErrorKind::Other))
        }
    }

    /// Read data from the stream
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read into
    ///
    /// # Returns
    ///
    /// Returns the number of bytes read. Returns 0 if the connection
    /// was closed by the remote end.
    ///
    /// # Errors
    ///
    /// Returns `IoErrorKind::WouldBlock` if no data is available
    /// and the stream is in non-blocking mode.
    pub fn read(&self, buf: &mut [u8]) -> IoResult<usize> {
        let result = unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };

        if result < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EAGAIN) ||
               err.raw_os_error() == Some(libc::EWOULDBLOCK) {
                return Err(IoError::new(crate::error::IoErrorKind::WouldBlock));
            }
            return Err(IoError::from(err));
        }

        Ok(result as usize)
    }

    /// Write data to the stream
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to write
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written. May write less than
    /// the full buffer if the operating system's send buffer is full.
    ///
    /// # Errors
    ///
    /// Returns `IoErrorKind::WouldBlock` if the send buffer is full
    /// and the stream is in non-blocking mode.
    pub fn write(&self, buf: &[u8]) -> IoResult<usize> {
        let result = unsafe {
            libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len())
        };

        if result < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EAGAIN) ||
               err.raw_os_error() == Some(libc::EWOULDBLOCK) {
                return Err(IoError::new(crate::error::IoErrorKind::WouldBlock));
            }
            return Err(IoError::from(err));
        }

        Ok(result as usize)
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Get the peer socket address
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr
    }

    /// Shutdown the stream
    ///
    /// # Arguments
    ///
    /// * `how` - Whether to shut down reads, writes, or both
    pub fn shutdown(&self, how: Shutdown) -> IoResult<()> {
        let how_raw = match how {
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
            Shutdown::Both => libc::SHUT_RDWR,
        };

        let result = unsafe { libc::shutdown(self.fd, how_raw) };

        if result < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        Ok(())
    }
}

impl EventSource for TcpStream {
    fn raw_fd(&self) -> RawFd {
        self.fd
    }

    fn interest(&self) -> Interest {
        // Initially interested in both read and write
        Interest::READABLE | Interest::WRITABLE
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

/// TCP listener for accepting incoming connections
///
/// A TCP listener listens for incoming connections on a specific address.
pub struct TcpListener {
    fd: RawFd,
    addr: SocketAddr,
}

impl TcpListener {
    /// Create a new TCP listener bound to the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - Local address to bind to
    ///
    /// # Errors
    ///
    /// Returns an error if socket creation, binding, or listening fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use zulon_runtime_io::io::tcp::TcpListener;
    /// use std::net::SocketAddr;
    ///
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// let listener = TcpListener::bind(addr).unwrap();
    /// // Register with event loop for readability...
    /// ```
    pub fn bind(addr: SocketAddr) -> IoResult<Self> {
        // Create socket
        let fd = match addr {
            SocketAddr::V4(_) => unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) },
            SocketAddr::V6(_) => unsafe { libc::socket(libc::AF_INET6, libc::SOCK_STREAM, 0) },
        };

        if fd < 0 {
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Set reuse address
        let reuse: i32 = 1;
        let result = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &reuse as *const i32 as *const libc::c_void,
                std::mem::size_of::<i32>() as libc::socklen_t,
            )
        };

        if result < 0 {
            unsafe { libc::close(fd); }
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Bind
        let (sockaddr, sockaddr_len) = addr_to_libc(&addr);
        let result = unsafe {
            libc::bind(fd, sockaddr, sockaddr_len)
        };

        if result < 0 {
            unsafe { libc::close(fd); }
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Listen
        let result = unsafe { libc::listen(fd, 128) };

        if result < 0 {
            unsafe { libc::close(fd); }
            return Err(IoError::from(std::io::Error::last_os_error()));
        }

        // Set non-blocking
        set_nonblocking(fd)?;

        Ok(TcpListener { fd, addr })
    }

    /// Accept an incoming connection
    ///
    /// # Returns
    ///
    /// - `Ok(Some(stream))` - A new connection was accepted
    /// - `Ok(None)` - No pending connections (WouldBlock)
    /// - `Err(e)` - An error occurred
    pub fn accept(&self) -> IoResult<Option<TcpStream>> {
        let mut sockaddr_storage: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
        let mut sockaddr_len = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;

        let client_fd = unsafe {
            libc::accept(
                self.fd,
                &mut sockaddr_storage as *mut libc::sockaddr_storage as *mut libc::sockaddr,
                &mut sockaddr_len,
            )
        };

        if client_fd < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EAGAIN) ||
               err.raw_os_error() == Some(libc::EWOULDBLOCK) {
                return Ok(None); // No pending connections
            }
            return Err(IoError::from(err));
        }

        // Set non-blocking
        set_nonblocking(client_fd)?;

        // Convert sockaddr to SocketAddr
        let peer_addr = sockaddr_to_addr(&sockaddr_storage, sockaddr_len)?;

        Ok(Some(TcpStream {
            fd: client_fd,
            local_addr: self.addr,
            peer_addr: Some(peer_addr),
        }))
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl EventSource for TcpListener {
    fn raw_fd(&self) -> RawFd {
        self.fd
    }

    fn interest(&self) -> Interest {
        Interest::READABLE // Only interested in incoming connections
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

/// Whether to shut down reads, writes, or both
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shutdown {
    /// Shut down the reading side
    Read,
    /// Shut down the writing side
    Write,
    /// Shut down both reading and writing
    Both,
}

/// Result of an async operation
///
/// Indicates whether an operation completed immediately
/// or is pending and needs to be registered with the event loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsyncPending<T> {
    /// Operation completed immediately
    Ready(T),
    /// Operation is pending
    Pending(T),
}

// Helper functions

fn addr_to_libc(addr: &SocketAddr) -> (*const libc::sockaddr, libc::socklen_t) {
    match addr {
        SocketAddr::V4(addr) => {
            let mut sockaddr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
            sockaddr.sin_family = libc::AF_INET as libc::sa_family_t;
            sockaddr.sin_port = addr.port().to_be();
            sockaddr.sin_addr.s_addr = u32::from_ne_bytes(addr.ip().octets());
            let ptr = &sockaddr as *const libc::sockaddr_in as *const libc::sockaddr;
            (ptr, std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t)
        }
        SocketAddr::V6(addr) => {
            let mut sockaddr: libc::sockaddr_in6 = unsafe { std::mem::zeroed() };
            sockaddr.sin6_family = libc::AF_INET6 as libc::sa_family_t;
            sockaddr.sin6_port = addr.port().to_be();
            sockaddr.sin6_addr.s6_addr = addr.ip().octets();
            let ptr = &sockaddr as *const libc::sockaddr_in6 as *const libc::sockaddr;
            (ptr, std::mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t)
        }
    }
}

fn sockaddr_to_addr(storage: &libc::sockaddr_storage, _len: libc::socklen_t) -> IoResult<SocketAddr> {
    match storage.ss_family as i32 {
        libc::AF_INET => {
            let addr = unsafe {
                *(storage as *const libc::sockaddr_storage as *const libc::sockaddr_in)
            };
            let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));
            let port = u16::from_be(addr.sin_port);
            Ok(SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)))
        }
        libc::AF_INET6 => {
            let addr = unsafe {
                *(storage as *const libc::sockaddr_storage as *const libc::sockaddr_in6)
            };
            let ip = Ipv6Addr::from(addr.sin6_addr.s6_addr);
            let port = u16::from_be(addr.sin6_port);
            Ok(SocketAddr::V6(std::net::SocketAddrV6::new(ip, port, addr.sin6_flowinfo, addr.sin6_scope_id)))
        }
        _ => Err(IoError::new(crate::error::IoErrorKind::Other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_listener_bind() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr);
        assert!(listener.is_ok());
    }

    #[test]
    fn test_tcp_listener_local_addr() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).unwrap();
        let local_addr = listener.local_addr();
        // Port 0 means OS assigns a port, but we haven't implemented getsockname yet
        // So we just check the IP for now
        assert_eq!(local_addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn test_tcp_listener_event_source() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).unwrap();
        assert!(listener.raw_fd() >= 0);
        assert_eq!(listener.interest(), Interest::READABLE);
    }

    #[test]
    fn test_async_pending() {
        let ready = AsyncPending::Ready(42);
        let pending = AsyncPending::Pending(42);

        match ready {
            AsyncPending::Ready(v) => assert_eq!(v, 42),
            _ => panic!("Expected Ready"),
        }

        match pending {
            AsyncPending::Pending(v) => assert_eq!(v, 42),
            _ => panic!("Expected Pending"),
        }
    }

    #[test]
    fn test_shutdown_enum() {
        assert_eq!(Shutdown::Read, Shutdown::Read);
        assert_eq!(Shutdown::Write, Shutdown::Write);
        assert_eq!(Shutdown::Both, Shutdown::Both);
    }
}
