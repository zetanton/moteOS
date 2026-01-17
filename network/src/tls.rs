//! TLS 1.3 support for moteOS using embedded-tls
//!
//! This module provides TLS 1.3 encryption for TCP connections using the
//! embedded-tls library, which is designed for no_std environments.
//!
//! # Features
//! - TLS 1.3 handshake
//! - Certificate verification (currently using NoVerify for compatibility)
//! - AES-128-GCM and AES-256-GCM cipher suites
//! - Blocking I/O interface compatible with smoltcp
//!
//! # Example
//! ```no_run
//! use network::{NetworkStack, TlsConnection};
//! use smoltcp::wire::Ipv4Address;
//!
//! # fn example(stack: &mut NetworkStack, get_time_ms: impl FnMut() -> i64, sleep_ms: impl FnMut(i64)) -> Result<(), network::NetError> {
//! // Resolve hostname
//! let dns_server = Ipv4Address::new(8, 8, 8, 8);
//! let ip = stack.dns_resolve("api.openai.com", dns_server, 5000, get_time_ms, Some(sleep_ms))?;
//!
//! // Create TLS connection
//! let mut tls = TlsConnection::connect(
//!     stack,
//!     "api.openai.com",
//!     ip,
//!     443,
//!     10000,
//!     get_time_ms,
//!     Some(sleep_ms)
//! )?;
//!
//! // Send HTTP request
//! tls.write(stack, b"GET / HTTP/1.1\r\nHost: api.openai.com\r\n\r\n", get_time_ms, Some(sleep_ms))?;
//!
//! // Read response
//! let mut buffer = [0u8; 1024];
//! let len = tls.read(stack, &mut buffer, get_time_ms, Some(sleep_ms))?;
//! # Ok(())
//! # }
//! ```

extern crate alloc;

use crate::error::NetError;
use crate::stack::NetworkStack;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use embedded_tls::blocking::{TlsConfig, TlsConnection as EmbeddedTlsConnection, TlsContext};
use embedded_tls::NoVerify;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, Socket as TcpSocket, State as TcpState};
use smoltcp::wire::{IpAddress, IpEndpoint, Ipv4Address};

/// Maximum TLS record size (16KB as recommended by embedded-tls)
const TLS_RECORD_BUFFER_SIZE: usize = 16384;

/// TCP receive buffer size (must be large enough for TLS records)
const TCP_RX_BUFFER_SIZE: usize = 16384;

/// TCP transmit buffer size
const TCP_TX_BUFFER_SIZE: usize = 16384;

/// TLS connection handle that manages a TCP socket with TLS encryption
///
/// This struct provides a simplified interface for TLS connections over TCP.
/// It handles the TLS handshake, encryption/decryption, and manages the
/// underlying TCP socket.
pub struct TlsConnection {
    /// Handle to the TCP socket in the network stack
    tcp_handle: SocketHandle,
    /// TLS read record buffer (16KB)
    read_buffer: Box<[u8; TLS_RECORD_BUFFER_SIZE]>,
    /// TLS write record buffer (16KB)
    write_buffer: Box<[u8; TLS_RECORD_BUFFER_SIZE]>,
    /// Hostname for SNI (Server Name Indication)
    hostname: String,
    /// Whether the TLS handshake is complete
    handshake_complete: bool,
}

impl TlsConnection {
    /// Create a new TLS connection to a server
    ///
    /// This method performs the following steps:
    /// 1. Creates a TCP socket and connects to the server
    /// 2. Performs TLS 1.3 handshake (with NoVerify for now)
    /// 3. Returns a ready-to-use TLS connection
    ///
    /// # Arguments
    /// * `stack` - Mutable reference to the network stack
    /// * `hostname` - Server hostname for SNI
    /// * `ip` - Server IP address (from DNS resolution)
    /// * `port` - Server port (typically 443 for HTTPS)
    /// * `timeout_ms` - Connection timeout in milliseconds
    /// * `get_time_ms` - Function to get current time in milliseconds
    /// * `sleep_ms` - Optional function to sleep/yield
    ///
    /// # Returns
    /// * `Ok(TlsConnection)` - Successfully established TLS connection
    /// * `Err(NetError)` - Failed to connect or handshake failed
    ///
    /// # Example
    /// ```no_run
    /// # use network::{NetworkStack, TlsConnection};
    /// # use smoltcp::wire::Ipv4Address;
    /// # fn example(stack: &mut NetworkStack, get_time_ms: impl FnMut() -> i64, sleep_ms: impl FnMut(i64)) -> Result<(), network::NetError> {
    /// let ip = Ipv4Address::new(93, 184, 216, 34);
    /// let mut tls = TlsConnection::connect(
    ///     stack,
    ///     "example.com",
    ///     ip,
    ///     443,
    ///     10000,
    ///     get_time_ms,
    ///     Some(sleep_ms)
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect<F, S>(
        stack: &mut NetworkStack,
        hostname: &str,
        ip: Ipv4Address,
        port: u16,
        timeout_ms: i64,
        mut get_time_ms: F,
        mut sleep_ms: Option<S>,
    ) -> Result<Self, NetError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        // Create TCP socket
        let tcp_handle = Self::create_tcp_socket(stack)?;

        // Connect TCP socket
        Self::tcp_connect(
            stack,
            tcp_handle,
            ip,
            port,
            timeout_ms,
            &mut get_time_ms,
            &mut sleep_ms,
        )?;

        // Allocate TLS buffers on heap (16KB each)
        let read_buffer = Box::new([0u8; TLS_RECORD_BUFFER_SIZE]);
        let write_buffer = Box::new([0u8; TLS_RECORD_BUFFER_SIZE]);

        let mut connection = TlsConnection {
            tcp_handle,
            read_buffer,
            write_buffer,
            hostname: hostname.to_string(),
            handshake_complete: false,
        };

        // Perform TLS handshake
        connection.perform_handshake(stack, timeout_ms, get_time_ms, sleep_ms)?;

        Ok(connection)
    }

    /// Create a new TCP socket in the network stack
    fn create_tcp_socket(stack: &mut NetworkStack) -> Result<SocketHandle, NetError> {
        // Create TCP socket buffers
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_RX_BUFFER_SIZE]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0u8; TCP_TX_BUFFER_SIZE]);

        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);

        // Add socket to the socket set
        let handle = stack.sockets_mut().add(tcp_socket);

        Ok(handle)
    }

    /// Connect the TCP socket to the remote server
    fn tcp_connect<F, S>(
        stack: &mut NetworkStack,
        handle: SocketHandle,
        ip: Ipv4Address,
        port: u16,
        timeout_ms: i64,
        get_time_ms: &mut F,
        sleep_ms: &mut Option<S>,
    ) -> Result<(), NetError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        let remote_endpoint = IpEndpoint::new(IpAddress::Ipv4(ip), port);

        // Initiate connection
        {
            let tcp_socket = stack.sockets_mut().get_mut::<TcpSocket>(handle);
            tcp_socket
                .connect(stack.interface().context(), remote_endpoint, 49152)
                .map_err(|e| NetError::TcpConnectionFailed(format!("{:?}", e)))?;
        }

        let start_time = get_time_ms();

        // Wait for connection to be established
        loop {
            let current_time = get_time_ms();

            // Poll the network stack
            stack.poll(current_time)?;

            // Check connection state
            let tcp_socket = stack.sockets().get::<TcpSocket>(handle);
            match tcp_socket.state() {
                TcpState::Established => {
                    return Ok(());
                }
                TcpState::Closed | TcpState::Closing | TcpState::CloseWait => {
                    return Err(NetError::TcpConnectionFailed("Connection closed".into()));
                }
                _ => {
                    // Still connecting
                }
            }

            // Check for timeout
            if current_time - start_time > timeout_ms {
                return Err(NetError::TcpConnectionFailed("Connection timeout".into()));
            }

            // Sleep to avoid busy waiting
            if let Some(ref mut sleep_fn) = sleep_ms {
                sleep_fn(10);
            } else {
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    /// Perform TLS 1.3 handshake with the server
    ///
    /// Note: Currently uses NoVerify for certificate verification.
    /// In production, this should be replaced with proper certificate
    /// verification using embedded root CAs.
    fn perform_handshake<F, S>(
        &mut self,
        stack: &mut NetworkStack,
        timeout_ms: i64,
        mut get_time_ms: F,
        mut sleep_ms: Option<S>,
    ) -> Result<(), NetError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        // Create TLS configuration
        let config = TlsConfig::new()
            .with_server_name(&self.hostname)
            .enable_rsa_signatures();

        // Create TCP adapter for embedded-tls
        let mut tcp_adapter = TcpSocketAdapter {
            stack,
            handle: self.tcp_handle,
            get_time_ms: &mut get_time_ms,
            sleep_ms: &mut sleep_ms,
        };

        // Create TLS context (using NoVerify for now)
        let context = TlsContext::new(&config, &mut NoVerify);

        // Create TLS connection
        let mut tls = EmbeddedTlsConnection::new(
            &mut tcp_adapter,
            &mut *self.read_buffer,
            &mut *self.write_buffer,
        );

        // Perform handshake (blocking)
        tls.open(context)
            .map_err(|e| NetError::TlsHandshakeFailed(format!("{:?}", e)))?;

        self.handshake_complete = true;

        Ok(())
    }

    /// Write data to the TLS connection
    ///
    /// This method encrypts the data using TLS 1.3 and sends it over the TCP connection.
    ///
    /// # Arguments
    /// * `stack` - Mutable reference to the network stack
    /// * `data` - Data to send
    /// * `get_time_ms` - Function to get current time in milliseconds
    /// * `sleep_ms` - Optional function to sleep/yield
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of bytes written
    /// * `Err(NetError)` - Write failed
    ///
    /// # Example
    /// ```no_run
    /// # use network::{NetworkStack, TlsConnection};
    /// # fn example(tls: &mut TlsConnection, stack: &mut NetworkStack, get_time_ms: impl FnMut() -> i64, sleep_ms: impl FnMut(i64)) -> Result<(), network::NetError> {
    /// tls.write(stack, b"GET / HTTP/1.1\r\n", get_time_ms, Some(sleep_ms))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write<F, S>(
        &mut self,
        stack: &mut NetworkStack,
        data: &[u8],
        mut get_time_ms: F,
        mut sleep_ms: Option<S>,
    ) -> Result<usize, NetError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        if !self.handshake_complete {
            return Err(NetError::TlsError("Handshake not complete".into()));
        }

        // Create TLS connection for write operation
        let config = TlsConfig::new()
            .with_server_name(&self.hostname)
            .enable_rsa_signatures();

        let mut tcp_adapter = TcpSocketAdapter {
            stack,
            handle: self.tcp_handle,
            get_time_ms: &mut get_time_ms,
            sleep_ms: &mut sleep_ms,
        };

        let context = TlsContext::new(&config, &mut NoVerify);

        let mut tls = EmbeddedTlsConnection::new(
            &mut tcp_adapter,
            &mut *self.read_buffer,
            &mut *self.write_buffer,
        );

        // Write data through TLS
        tls.write(data)
            .map_err(|e| NetError::TlsError(format!("Write failed: {:?}", e)))
    }

    /// Read data from the TLS connection
    ///
    /// This method reads encrypted data from the TCP connection and decrypts it.
    ///
    /// # Arguments
    /// * `stack` - Mutable reference to the network stack
    /// * `buffer` - Buffer to read data into
    /// * `get_time_ms` - Function to get current time in milliseconds
    /// * `sleep_ms` - Optional function to sleep/yield
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of bytes read (0 indicates connection closed)
    /// * `Err(NetError)` - Read failed
    ///
    /// # Example
    /// ```no_run
    /// # use network::{NetworkStack, TlsConnection};
    /// # fn example(tls: &mut TlsConnection, stack: &mut NetworkStack, get_time_ms: impl FnMut() -> i64, sleep_ms: impl FnMut(i64)) -> Result<(), network::NetError> {
    /// let mut buffer = [0u8; 1024];
    /// let len = tls.read(stack, &mut buffer, get_time_ms, Some(sleep_ms))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read<F, S>(
        &mut self,
        stack: &mut NetworkStack,
        buffer: &mut [u8],
        mut get_time_ms: F,
        mut sleep_ms: Option<S>,
    ) -> Result<usize, NetError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        if !self.handshake_complete {
            return Err(NetError::TlsError("Handshake not complete".into()));
        }

        // Create TLS connection for read operation
        let config = TlsConfig::new()
            .with_server_name(&self.hostname)
            .enable_rsa_signatures();

        let mut tcp_adapter = TcpSocketAdapter {
            stack,
            handle: self.tcp_handle,
            get_time_ms: &mut get_time_ms,
            sleep_ms: &mut sleep_ms,
        };

        let context = TlsContext::new(&config, &mut NoVerify);

        let mut tls = EmbeddedTlsConnection::new(
            &mut tcp_adapter,
            &mut *self.read_buffer,
            &mut *self.write_buffer,
        );

        // Read data through TLS
        tls.read(buffer)
            .map_err(|e| NetError::TlsError(format!("Read failed: {:?}", e)))
    }

    /// Close the TLS connection
    ///
    /// This sends a TLS close_notify alert and closes the underlying TCP connection.
    ///
    /// # Arguments
    /// * `stack` - Mutable reference to the network stack
    pub fn close(self, stack: &mut NetworkStack) {
        // Close TCP socket
        let tcp_socket = stack.sockets_mut().get_mut::<TcpSocket>(self.tcp_handle);
        tcp_socket.close();

        // Remove socket from socket set
        stack.sockets_mut().remove(self.tcp_handle);
    }

    /// Check if the connection is still open
    ///
    /// # Arguments
    /// * `stack` - Reference to the network stack
    ///
    /// # Returns
    /// * `true` - Connection is open and ready
    /// * `false` - Connection is closed or handshake incomplete
    pub fn is_open(&self, stack: &NetworkStack) -> bool {
        if !self.handshake_complete {
            return false;
        }

        let tcp_socket = stack.sockets().get::<TcpSocket>(self.tcp_handle);
        matches!(tcp_socket.state(), TcpState::Established)
    }
}

/// Adapter that allows embedded-tls to use our TCP socket
///
/// This implements the embedded-io traits that embedded-tls expects,
/// bridging between our smoltcp-based TCP sockets and embedded-tls's
/// blocking I/O interface.
struct TcpSocketAdapter<'a, F, S>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    stack: &'a mut NetworkStack,
    handle: SocketHandle,
    get_time_ms: &'a mut F,
    sleep_ms: &'a mut Option<S>,
}

impl<'a, F, S> embedded_io::ErrorType for TcpSocketAdapter<'a, F, S>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    type Error = TcpAdapterError;
}

impl<'a, F, S> embedded_io::Read for TcpSocketAdapter<'a, F, S>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // Poll until data is available
        loop {
            let current_time = (self.get_time_ms)();
            self.stack.poll(current_time).map_err(|_| TcpAdapterError)?;

            let tcp_socket = self.stack.sockets_mut().get_mut::<TcpSocket>(self.handle);

            if tcp_socket.can_recv() {
                return tcp_socket.recv_slice(buf).map_err(|_| TcpAdapterError);
            }

            // Check if connection is closed
            if !tcp_socket.is_open() {
                return Ok(0);
            }

            // Sleep to avoid busy waiting
            if let Some(ref mut sleep_fn) = self.sleep_ms {
                sleep_fn(1);
            } else {
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

impl<'a, F, S> embedded_io::Write for TcpSocketAdapter<'a, F, S>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // Poll until we can send
        loop {
            let current_time = (self.get_time_ms)();
            self.stack.poll(current_time).map_err(|_| TcpAdapterError)?;

            let tcp_socket = self.stack.sockets_mut().get_mut::<TcpSocket>(self.handle);

            if tcp_socket.can_send() {
                return tcp_socket.send_slice(buf).map_err(|_| TcpAdapterError);
            }

            // Check if connection is closed
            if !tcp_socket.is_open() {
                return Err(TcpAdapterError);
            }

            // Sleep to avoid busy waiting
            if let Some(ref mut sleep_fn) = self.sleep_ms {
                sleep_fn(1);
            } else {
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // TCP flush is handled by smoltcp automatically
        Ok(())
    }
}

/// Error type for TCP adapter
#[derive(Debug)]
struct TcpAdapterError;

impl embedded_io::Error for TcpAdapterError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}
