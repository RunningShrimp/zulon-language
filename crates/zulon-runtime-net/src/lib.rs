// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # zulon-runtime-net
//!
//! Network stack for ZULON language with TCP/UDP sockets and async I/O.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use thiserror::Error;

/// Network error
#[derive(Debug, Error)]
pub enum NetworkError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection error
    #[error("connection error: {0}")]
    Connection(String),

    /// Address error
    #[error("address error: {0}")]
    Address(String),

    /// Timeout error
    #[error("operation timed out")]
    Timeout,

    /// Not supported
    #[error("operation not supported")]
    NotSupported,
}

/// TCP socket wrapper
pub struct TcpSocket {
    stream: TcpStream,
}

impl TcpSocket {
    /// Connect to an address
    pub fn connect(addr: SocketAddr) -> Result<Self, NetworkError> {
        let stream = TcpStream::connect(addr).map_err(NetworkError::Io)?;

        Ok(TcpSocket { stream })
    }

    /// Read data from socket
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetworkError> {
        self.stream.read(buf).map_err(NetworkError::Io)
    }

    /// Write data to socket
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, NetworkError> {
        self.stream.write(buf).map_err(NetworkError::Io)
    }

    /// Close socket
    pub fn close(self) -> Result<(), NetworkError> {
        self.stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(NetworkError::Io)
    }
}

/// TCP listener wrapper
pub struct TcpListenerWrapper {
    listener: TcpListener,
}

impl TcpListenerWrapper {
    /// Bind to an address
    pub fn bind(addr: SocketAddr) -> Result<Self, NetworkError> {
        let listener = TcpListener::bind(addr).map_err(NetworkError::Io)?;

        Ok(TcpListenerWrapper { listener })
    }

    /// Accept a connection
    pub fn accept(&self) -> Result<TcpSocket, NetworkError> {
        let (stream, _addr) = self.listener.accept().map_err(NetworkError::Io)?;
        Ok(TcpSocket { stream })
    }

    /// Close listener
    pub fn close(self) -> Result<(), NetworkError> {
        drop(self.listener);
        Ok(())
    }
}

/// UDP socket wrapper
pub struct UdpSocketWrapper {
    socket: UdpSocket,
}

impl UdpSocketWrapper {
    /// Bind to an address
    pub fn bind(addr: SocketAddr) -> Result<Self, NetworkError> {
        let socket = UdpSocket::bind(addr).map_err(NetworkError::Io)?;

        Ok(UdpSocketWrapper { socket })
    }

    /// Send data
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, NetworkError> {
        self.socket.send_to(buf, addr).map_err(NetworkError::Io)
    }

    /// Receive data
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, NetworkError> {
        self.socket.recv(buf).map_err(NetworkError::Io)
    }

    /// Close socket
    pub fn close(self) -> Result<(), NetworkError> {
        drop(self.socket);
        Ok(())
    }
}

/// Dns resolution
pub struct DnsResolver;

impl DnsResolver {
    pub fn new() -> Self {
        DnsResolver
    }

    /// Resolve a hostname to IP address
    pub fn resolve(&self, hostname: &str) -> Result<Ipv4Addr, NetworkError> {
        use std::net::ToSocketAddrs;

        let addrs: Vec<SocketAddr> = (hostname, 0)
            .to_socket_addrs()
            .map_err(NetworkError::Io)?
            .collect();

        for addr in addrs {
            if let SocketAddr::V4(sockaddr) = addr {
                return Ok(*sockaddr.ip());
            }
        }

        Err(NetworkError::Connection(format!(
            "could not resolve: {}",
            hostname
        )))
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_socket_creation() {
        let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        let result = TcpSocket::connect(addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dns_resolution() {
        let resolver = DnsResolver::new();
        let result = resolver.resolve("localhost");
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert_eq!(addr, Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn test_udp_socket() {
        let addr = "127.0.0.1:0".parse::<SocketAddr>().unwrap();
        let result = UdpSocketWrapper::bind(addr);
        assert!(result.is_ok());
    }
}
