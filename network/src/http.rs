extern crate alloc;

use crate::error::NetError;
use crate::stack::NetworkStack;
#[cfg(feature = "tls")]
use crate::tls::TlsConnection;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, Socket as TcpSocket, State as TcpState};
use smoltcp::wire::{IpAddress, IpEndpoint, Ipv4Address};

const DEFAULT_CONNECT_TIMEOUT_MS: i64 = 10_000;
const DEFAULT_READ_TIMEOUT_MS: i64 = 30_000;
const DEFAULT_MAX_HEADER_BYTES: usize = 32 * 1024;
const DEFAULT_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUrl<'a> {
    pub scheme: Scheme,
    pub host: &'a str,
    pub port: u16,
    pub path_and_query: &'a str,
}

#[derive(Debug)]
pub enum HttpError {
    InvalidUrl(String),

    UnsupportedScheme(String),

    InvalidResponse(String),

    HeaderTooLarge,

    BodyTooLarge,

    ReadTimeout,

    Net(NetError),
}

impl From<NetError> for HttpError {
    fn from(value: NetError) -> Self {
        Self::Net(value)
    }
}

impl core::fmt::Display for HttpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HttpError::InvalidUrl(s) => write!(f, "invalid url: {s}"),
            HttpError::UnsupportedScheme(s) => write!(f, "unsupported URL scheme: {s}"),
            HttpError::InvalidResponse(s) => write!(f, "invalid HTTP response: {s}"),
            HttpError::HeaderTooLarge => write!(f, "response header too large"),
            HttpError::BodyTooLarge => write!(f, "response body too large"),
            HttpError::ReadTimeout => write!(f, "HTTP read timeout"),
            HttpError::Net(e) => write!(f, "network error: {e}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

pub struct HttpClient {
    dns_server: Ipv4Address,
    connect_timeout_ms: i64,
    read_timeout_ms: i64,
    max_header_bytes: usize,
    max_body_bytes: usize,
}

impl HttpClient {
    pub fn new(dns_server: Ipv4Address) -> Self {
        Self {
            dns_server,
            connect_timeout_ms: DEFAULT_CONNECT_TIMEOUT_MS,
            read_timeout_ms: DEFAULT_READ_TIMEOUT_MS,
            max_header_bytes: DEFAULT_MAX_HEADER_BYTES,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        }
    }

    pub fn with_timeouts(mut self, connect_timeout_ms: i64, read_timeout_ms: i64) -> Self {
        self.connect_timeout_ms = connect_timeout_ms;
        self.read_timeout_ms = read_timeout_ms;
        self
    }

    pub fn with_limits(mut self, max_header_bytes: usize, max_body_bytes: usize) -> Self {
        self.max_header_bytes = max_header_bytes;
        self.max_body_bytes = max_body_bytes;
        self
    }

    pub fn post_json<F, S>(
        &self,
        stack: &mut NetworkStack,
        url: &str,
        body: &str,
        headers: &[(&str, &str)],
        mut get_time_ms: F,
        mut sleep_ms: Option<S>,
    ) -> Result<HttpResponse, HttpError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        let mut merged_headers: Vec<(&str, &str)> = Vec::with_capacity(headers.len() + 2);
        merged_headers.extend_from_slice(headers);
        if !headers_contain(headers, "Content-Type") {
            merged_headers.push(("Content-Type", "application/json"));
        }
        if !headers_contain(headers, "Accept") {
            merged_headers.push(("Accept", "application/json"));
        }

        self.request(
            stack,
            "POST",
            url,
            Some(body.as_bytes()),
            &merged_headers,
            &mut get_time_ms,
            sleep_ms.as_mut(),
        )
    }

    pub fn request<F, S>(
        &self,
        stack: &mut NetworkStack,
        method: &str,
        url: &str,
        body: Option<&[u8]>,
        headers: &[(&str, &str)],
        get_time_ms: &mut F,
        sleep_ms: Option<&mut S>,
    ) -> Result<HttpResponse, HttpError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        let parsed = parse_url(url)?;
        let ip = resolve_host_ipv4(
            stack,
            parsed.host,
            self.dns_server,
            self.connect_timeout_ms,
            get_time_ms,
            sleep_ms,
        )?;

        let request_bytes = build_request_bytes(&parsed, method, headers, body);

        match parsed.scheme {
            Scheme::Https => {
                #[cfg(feature = "tls")]
                {
                    let mut tls = TlsConnection::connect(
                        stack,
                        parsed.host,
                        ip,
                        parsed.port,
                        self.connect_timeout_ms,
                        &mut *get_time_ms,
                        sleep_ms,
                    )?;

                    tls.write(stack, &request_bytes, &mut *get_time_ms, sleep_ms)?;

                    let mut read_fn = |buf: &mut [u8]| -> Result<usize, HttpError> {
                        let n = tls.read(stack, buf, &mut *get_time_ms, sleep_ms)?;
                        Ok(n)
                    };

                    let response = read_http_response(
                        &mut read_fn,
                        self.read_timeout_ms,
                        self.max_header_bytes,
                        self.max_body_bytes,
                        &mut *get_time_ms,
                        sleep_ms,
                    )?;
                    tls.close(stack);
                    Ok(response)
                }

                #[cfg(not(feature = "tls"))]
                {
                    Err(HttpError::UnsupportedScheme(
                        "https requires the `network/tls` feature".into(),
                    ))
                }
            }
            Scheme::Http => {
                let mut tcp = TcpConnection::connect(
                    stack,
                    ip,
                    parsed.port,
                    self.connect_timeout_ms,
                    &mut *get_time_ms,
                    sleep_ms,
                )?;
                tcp.write_all(
                    stack,
                    &request_bytes,
                    self.read_timeout_ms,
                    &mut *get_time_ms,
                    sleep_ms,
                )?;

                let mut read_fn = |buf: &mut [u8]| -> Result<usize, HttpError> {
                    let n = tcp.read(
                        stack,
                        buf,
                        self.read_timeout_ms,
                        &mut *get_time_ms,
                        sleep_ms,
                    )?;
                    Ok(n)
                };

                let response = read_http_response(
                    &mut read_fn,
                    self.read_timeout_ms,
                    self.max_header_bytes,
                    self.max_body_bytes,
                    &mut *get_time_ms,
                    sleep_ms,
                )?;
                tcp.close(stack);
                Ok(response)
            }
        }
    }
}

pub fn parse_url(url: &str) -> Result<ParsedUrl<'_>, HttpError> {
    let (scheme, rest) = if let Some(r) = url.strip_prefix("https://") {
        (Scheme::Https, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (Scheme::Http, r)
    } else {
        return Err(HttpError::UnsupportedScheme(url.to_string()));
    };

    if rest.is_empty() {
        return Err(HttpError::InvalidUrl("missing host".into()));
    }

    let (authority, path_and_query) = split_authority_path(rest);
    let (host, port) = split_host_port(authority, scheme)?;

    let path_and_query = if path_and_query.is_empty() {
        "/"
    } else {
        path_and_query
    };

    Ok(ParsedUrl {
        scheme,
        host,
        port,
        path_and_query,
    })
}

fn split_authority_path(rest: &str) -> (&str, &str) {
    let mut authority_end = rest.len();
    for (i, b) in rest.bytes().enumerate() {
        if b == b'/' || b == b'?' || b == b'#' {
            authority_end = i;
            break;
        }
    }

    let authority = &rest[..authority_end];
    let mut path = &rest[authority_end..];

    if let Some(hash) = path.find('#') {
        path = &path[..hash];
    }

    (authority, path)
}

fn split_host_port<'a>(authority: &'a str, scheme: Scheme) -> Result<(&'a str, u16), HttpError> {
    if authority.is_empty() {
        return Err(HttpError::InvalidUrl("missing host".into()));
    }

    let default_port = match scheme {
        Scheme::Http => 80,
        Scheme::Https => 443,
    };

    if let Some(host) = authority.strip_prefix('[') {
        // IPv6 literal: [::1]:443
        let Some(end) = host.find(']') else {
            return Err(HttpError::InvalidUrl("unterminated IPv6 literal".into()));
        };
        let host_part = &authority[..(end + 2)];
        let tail = &host[(end + 1)..];
        let host_inner = &host[..end];
        if tail.is_empty() {
            return Ok((host_inner, default_port));
        }
        let Some(port_str) = tail.strip_prefix(':') else {
            return Err(HttpError::InvalidUrl(
                "invalid IPv6 literal authority".into(),
            ));
        };
        let port = parse_port(port_str)?;
        let _ = host_part; // keep logic clear; host_inner is returned
        return Ok((host_inner, port));
    }

    let Some((host, port_str)) = authority.rsplit_once(':') else {
        return Ok((authority, default_port));
    };

    if host.is_empty() {
        return Err(HttpError::InvalidUrl("missing host".into()));
    }

    let port = parse_port(port_str)?;
    Ok((host, port))
}

fn parse_port(port: &str) -> Result<u16, HttpError> {
    if port.is_empty() || !port.bytes().all(|b| b.is_ascii_digit()) {
        return Err(HttpError::InvalidUrl(format!("invalid port: {port}")));
    }
    port.parse::<u16>()
        .map_err(|_| HttpError::InvalidUrl(format!("invalid port: {port}")))
}

fn headers_contain(headers: &[(&str, &str)], name: &str) -> bool {
    headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(name))
}

fn build_request_bytes(
    url: &ParsedUrl<'_>,
    method: &str,
    headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> Vec<u8> {
    let body_len = body.map(|b| b.len()).unwrap_or(0);

    let mut out = Vec::with_capacity(256 + body_len);
    out.extend_from_slice(method.as_bytes());
    out.extend_from_slice(b" ");
    out.extend_from_slice(url.path_and_query.as_bytes());
    out.extend_from_slice(b" HTTP/1.1\r\n");

    if !headers_contain(headers, "Host") {
        out.extend_from_slice(b"Host: ");
        out.extend_from_slice(url.host.as_bytes());
        if (url.scheme == Scheme::Http && url.port != 80)
            || (url.scheme == Scheme::Https && url.port != 443)
        {
            out.extend_from_slice(b":");
            out.extend_from_slice(url.port.to_string().as_bytes());
        }
        out.extend_from_slice(b"\r\n");
    }

    if !headers_contain(headers, "User-Agent") {
        out.extend_from_slice(b"User-Agent: moteOS/1.0\r\n");
    }

    if !headers_contain(headers, "Connection") {
        out.extend_from_slice(b"Connection: close\r\n");
    }

    if body.is_some() && !headers_contain(headers, "Content-Length") {
        out.extend_from_slice(b"Content-Length: ");
        out.extend_from_slice(body_len.to_string().as_bytes());
        out.extend_from_slice(b"\r\n");
    }

    for (k, v) in headers {
        out.extend_from_slice(k.as_bytes());
        out.extend_from_slice(b": ");
        out.extend_from_slice(v.as_bytes());
        out.extend_from_slice(b"\r\n");
    }

    out.extend_from_slice(b"\r\n");
    if let Some(body) = body {
        out.extend_from_slice(body);
    }
    out
}

fn resolve_host_ipv4<F, S>(
    stack: &mut NetworkStack,
    host: &str,
    dns_server: Ipv4Address,
    timeout_ms: i64,
    get_time_ms: &mut F,
    sleep_ms: Option<&mut S>,
) -> Result<Ipv4Address, HttpError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    if let Some(ip) = parse_ipv4_literal(host) {
        return Ok(ip);
    }
    let ip = stack.dns_resolve(host, dns_server, timeout_ms, &mut *get_time_ms, sleep_ms)?;
    Ok(ip)
}

fn parse_ipv4_literal(host: &str) -> Option<Ipv4Address> {
    let mut parts = [0u8; 4];
    let mut part_idx = 0usize;
    let mut current: u16 = 0;
    let mut saw_digit = false;

    for b in host.bytes() {
        match b {
            b'0'..=b'9' => {
                saw_digit = true;
                current = current * 10 + (b - b'0') as u16;
                if current > 255 {
                    return None;
                }
            }
            b'.' => {
                if !saw_digit || part_idx >= 4 {
                    return None;
                }
                parts[part_idx] = current as u8;
                part_idx += 1;
                current = 0;
                saw_digit = false;
            }
            _ => return None,
        }
    }

    if !saw_digit || part_idx != 3 {
        return None;
    }
    parts[3] = current as u8;
    Some(Ipv4Address::from_bytes(&parts))
}

fn read_http_response<F, S>(
    read: &mut impl FnMut(&mut [u8]) -> Result<usize, HttpError>,
    read_timeout_ms: i64,
    max_header_bytes: usize,
    max_body_bytes: usize,
    get_time_ms: &mut F,
    mut sleep_ms: Option<&mut S>,
) -> Result<HttpResponse, HttpError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    let start_time = get_time_ms();
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 1024];

    let header_end = loop {
        if let Some(idx) = find_subslice(&buf, b"\r\n\r\n") {
            break idx + 4;
        }

        if buf.len() >= max_header_bytes {
            return Err(HttpError::HeaderTooLarge);
        }

        if get_time_ms() - start_time > read_timeout_ms {
            return Err(HttpError::ReadTimeout);
        }

        let n = read(&mut tmp)?;
        if n == 0 {
            return Err(HttpError::InvalidResponse(
                "connection closed before headers".into(),
            ));
        }
        buf.extend_from_slice(&tmp[..n]);

        if let Some(ref mut sleep_fn) = sleep_ms {
            sleep_fn(1);
        }
    };

    let (status, headers) = parse_response_head(&buf[..header_end])?;
    let mut remainder = buf[header_end..].to_vec();

    let transfer_encoding =
        header_value(&headers, "Transfer-Encoding").map(|v| v.to_ascii_lowercase());
    let content_length =
        header_value(&headers, "Content-Length").and_then(|v| v.trim().parse::<usize>().ok());

    let body = if transfer_encoding
        .as_deref()
        .is_some_and(|v| v.contains("chunked"))
    {
        decode_chunked_body(
            &mut remainder,
            read,
            read_timeout_ms,
            max_body_bytes,
            get_time_ms,
            sleep_ms,
        )?
    } else if let Some(len) = content_length {
        read_fixed_body(
            &mut remainder,
            read,
            len,
            read_timeout_ms,
            max_body_bytes,
            get_time_ms,
            sleep_ms,
        )?
    } else {
        read_until_eof(
            &mut remainder,
            read,
            read_timeout_ms,
            max_body_bytes,
            get_time_ms,
            sleep_ms,
        )?
    };

    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

fn parse_response_head(head: &[u8]) -> Result<(u16, Vec<(String, String)>), HttpError> {
    let head_str = str::from_utf8(head)
        .map_err(|_| HttpError::InvalidResponse("headers not valid UTF-8".into()))?;
    let Some((lines_str, _)) = head_str.split_once("\r\n\r\n") else {
        return Err(HttpError::InvalidResponse(
            "missing header terminator".into(),
        ));
    };

    let mut lines = lines_str.split("\r\n");
    let Some(status_line) = lines.next() else {
        return Err(HttpError::InvalidResponse("missing status line".into()));
    };

    let mut parts = status_line.splitn(3, ' ');
    let version = parts.next().unwrap_or("");
    let status_str = parts.next().unwrap_or("");
    if !version.starts_with("HTTP/") {
        return Err(HttpError::InvalidResponse("invalid status line".into()));
    }
    let status: u16 = status_str
        .parse()
        .map_err(|_| HttpError::InvalidResponse("invalid status code".into()))?;

    let mut headers: Vec<(String, String)> = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(HttpError::InvalidResponse("malformed header line".into()));
        };
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }

    Ok((status, headers))
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn read_fixed_body<F, S>(
    remainder: &mut Vec<u8>,
    read: &mut impl FnMut(&mut [u8]) -> Result<usize, HttpError>,
    expected_len: usize,
    read_timeout_ms: i64,
    max_body_bytes: usize,
    get_time_ms: &mut F,
    mut sleep_ms: Option<&mut S>,
) -> Result<Vec<u8>, HttpError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    if expected_len > max_body_bytes {
        return Err(HttpError::BodyTooLarge);
    }

    let start_time = get_time_ms();
    let mut tmp = [0u8; 1024];
    while remainder.len() < expected_len {
        if get_time_ms() - start_time > read_timeout_ms {
            return Err(HttpError::ReadTimeout);
        }
        let n = read(&mut tmp)?;
        if n == 0 {
            return Err(HttpError::InvalidResponse(
                "connection closed mid-body".into(),
            ));
        }
        remainder.extend_from_slice(&tmp[..n]);
        if let Some(ref mut sleep_fn) = sleep_ms {
            sleep_fn(1);
        }
    }
    remainder.truncate(expected_len);
    Ok(core::mem::take(remainder))
}

fn read_until_eof<F, S>(
    remainder: &mut Vec<u8>,
    read: &mut impl FnMut(&mut [u8]) -> Result<usize, HttpError>,
    read_timeout_ms: i64,
    max_body_bytes: usize,
    get_time_ms: &mut F,
    mut sleep_ms: Option<&mut S>,
) -> Result<Vec<u8>, HttpError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    let start_time = get_time_ms();
    let mut tmp = [0u8; 1024];
    loop {
        if remainder.len() > max_body_bytes {
            return Err(HttpError::BodyTooLarge);
        }
        if get_time_ms() - start_time > read_timeout_ms {
            return Err(HttpError::ReadTimeout);
        }

        let n = read(&mut tmp)?;
        if n == 0 {
            break;
        }
        remainder.extend_from_slice(&tmp[..n]);
        if let Some(ref mut sleep_fn) = sleep_ms {
            sleep_fn(1);
        }
    }
    Ok(core::mem::take(remainder))
}

fn decode_chunked_body<F, S>(
    remainder: &mut Vec<u8>,
    read: &mut impl FnMut(&mut [u8]) -> Result<usize, HttpError>,
    read_timeout_ms: i64,
    max_body_bytes: usize,
    get_time_ms: &mut F,
    mut sleep_ms: Option<&mut S>,
) -> Result<Vec<u8>, HttpError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    let start_time = get_time_ms();
    let mut tmp = [0u8; 1024];
    let mut out: Vec<u8> = Vec::new();

    loop {
        let line = read_line_crlf(
            remainder,
            read,
            &mut tmp,
            read_timeout_ms,
            start_time,
            get_time_ms,
            sleep_ms.as_deref_mut(),
        )?;
        let size = parse_chunk_size(&line)?;
        if size == 0 {
            // Consume trailer headers (if any) until empty line.
            loop {
                let trailer_line = read_line_crlf(
                    remainder,
                    read,
                    &mut tmp,
                    read_timeout_ms,
                    start_time,
                    get_time_ms,
                    sleep_ms.as_deref_mut(),
                )?;
                if trailer_line.is_empty() {
                    break;
                }
            }
            break;
        }

        while remainder.len() < size + 2 {
            if get_time_ms() - start_time > read_timeout_ms {
                return Err(HttpError::ReadTimeout);
            }
            let n = read(&mut tmp)?;
            if n == 0 {
                return Err(HttpError::InvalidResponse(
                    "connection closed mid-chunk".into(),
                ));
            }
            remainder.extend_from_slice(&tmp[..n]);
            if let Some(ref mut sleep_fn) = sleep_ms {
                sleep_fn(1);
            }
        }

        if out.len().saturating_add(size) > max_body_bytes {
            return Err(HttpError::BodyTooLarge);
        }
        out.extend_from_slice(&remainder[..size]);

        // Consume the chunk and trailing CRLF.
        remainder.drain(..(size + 2));
    }

    Ok(out)
}

fn read_line_crlf<F, S>(
    buffer: &mut Vec<u8>,
    read: &mut impl FnMut(&mut [u8]) -> Result<usize, HttpError>,
    tmp: &mut [u8; 1024],
    read_timeout_ms: i64,
    start_time: i64,
    get_time_ms: &mut F,
    mut sleep_ms: Option<&mut S>,
) -> Result<String, HttpError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    loop {
        if let Some(idx) = find_subslice(buffer, b"\r\n") {
            let line_bytes: Vec<u8> = buffer.drain(..idx).collect();
            buffer.drain(..2); // CRLF
            let line = str::from_utf8(&line_bytes)
                .map_err(|_| HttpError::InvalidResponse("chunk line not valid UTF-8".into()))?;
            return Ok(line.to_string());
        }

        if get_time_ms() - start_time > read_timeout_ms {
            return Err(HttpError::ReadTimeout);
        }

        let n = read(tmp)?;
        if n == 0 {
            return Err(HttpError::InvalidResponse(
                "connection closed while reading line".into(),
            ));
        }
        buffer.extend_from_slice(&tmp[..n]);
        if let Some(ref mut sleep_fn) = sleep_ms {
            sleep_fn(1);
        }
    }
}

fn parse_chunk_size(line: &str) -> Result<usize, HttpError> {
    let size_str = line.split(';').next().unwrap_or("").trim();
    if size_str.is_empty() {
        return Err(HttpError::InvalidResponse("empty chunk size".into()));
    }
    usize::from_str_radix(size_str, 16)
        .map_err(|_| HttpError::InvalidResponse("invalid chunk size".into()))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

struct TcpConnection {
    handle: SocketHandle,
}

impl TcpConnection {
    fn connect<F, S>(
        stack: &mut NetworkStack,
        ip: Ipv4Address,
        port: u16,
        timeout_ms: i64,
        get_time_ms: &mut F,
        mut sleep_ms: Option<&mut S>,
    ) -> Result<Self, HttpError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        let rx = tcp::SocketBuffer::new(vec![0u8; 8192]);
        let tx = tcp::SocketBuffer::new(vec![0u8; 8192]);
        let socket = TcpSocket::new(rx, tx);
        let handle = stack.sockets_mut().add(socket);

        let remote = IpEndpoint::new(IpAddress::Ipv4(ip), port);
        {
            // smoltcp requires `&mut Context` for connect; `NetworkStack` doesn't expose a safe
            // way to borrow the interface context and socket set simultaneously.
            let ctx_ptr = stack.interface_mut().context() as *mut _;
            let sock = stack.sockets_mut().get_mut::<TcpSocket>(handle);
            // SAFETY: `iface` and `sockets` are disjoint fields of `NetworkStack`.
            unsafe { sock.connect(&mut *ctx_ptr, remote, 49152) }
                .map_err(|e| NetError::TcpConnectionFailed(format!("{:?}", e)))?;
        }

        let start = get_time_ms();
        loop {
            let now = get_time_ms();
            stack.poll(now)?;

            let sock = stack.sockets().get::<TcpSocket>(handle);
            match sock.state() {
                TcpState::Established => break,
                TcpState::Closed | TcpState::Closing | TcpState::CloseWait => {
                    return Err(NetError::TcpConnectionFailed("Connection closed".into()).into());
                }
                _ => {}
            }

            if now - start > timeout_ms {
                return Err(NetError::TcpConnectionFailed("Connection timeout".into()).into());
            }

            if let Some(ref mut sleep_fn) = sleep_ms {
                sleep_fn(10);
            } else {
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }

        Ok(Self { handle })
    }

    fn write_all<F, S>(
        &mut self,
        stack: &mut NetworkStack,
        mut data: &[u8],
        timeout_ms: i64,
        get_time_ms: &mut F,
        mut sleep_ms: Option<&mut S>,
    ) -> Result<(), HttpError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        let start = get_time_ms();
        while !data.is_empty() {
            let now = get_time_ms();
            stack.poll(now)?;
            if now - start > timeout_ms {
                return Err(HttpError::ReadTimeout);
            }

            let can_send = stack.sockets().get::<TcpSocket>(self.handle).can_send();
            if !can_send {
                if let Some(ref mut sleep_fn) = sleep_ms {
                    sleep_fn(1);
                }
                continue;
            }

            let sent = stack
                .sockets_mut()
                .get_mut::<TcpSocket>(self.handle)
                .send_slice(data)
                .map_err(|_| NetError::TcpSendBufferFull)?;
            data = &data[sent..];
        }
        Ok(())
    }

    fn read<F, S>(
        &mut self,
        stack: &mut NetworkStack,
        buf: &mut [u8],
        timeout_ms: i64,
        get_time_ms: &mut F,
        mut sleep_ms: Option<&mut S>,
    ) -> Result<usize, HttpError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        let start = get_time_ms();
        loop {
            let now = get_time_ms();
            stack.poll(now)?;
            if now - start > timeout_ms {
                return Err(HttpError::ReadTimeout);
            }

            let can_recv = stack.sockets().get::<TcpSocket>(self.handle).can_recv();
            if can_recv {
                let n = stack
                    .sockets_mut()
                    .get_mut::<TcpSocket>(self.handle)
                    .recv_slice(buf)
                    .map_err(|_| NetError::TcpReceiveError)?;
                return Ok(n);
            }

            let sock = stack.sockets().get::<TcpSocket>(self.handle);
            if sock.state() == TcpState::CloseWait || sock.state() == TcpState::Closed {
                return Ok(0);
            }

            if let Some(ref mut sleep_fn) = sleep_ms {
                sleep_fn(1);
            } else {
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    fn close(self, stack: &mut NetworkStack) {
        let sock = stack.sockets_mut().get_mut::<TcpSocket>(self.handle);
        sock.close();
        stack.sockets_mut().remove(self.handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_parse_https_default_port_and_path() {
        let u = parse_url("https://example.com").unwrap();
        assert_eq!(u.scheme, Scheme::Https);
        assert_eq!(u.host, "example.com");
        assert_eq!(u.port, 443);
        assert_eq!(u.path_and_query, "/");
    }

    #[test]
    fn url_parse_http_custom_port_and_query() {
        let u = parse_url("http://example.com:8080/path?q=1").unwrap();
        assert_eq!(u.scheme, Scheme::Http);
        assert_eq!(u.host, "example.com");
        assert_eq!(u.port, 8080);
        assert_eq!(u.path_and_query, "/path?q=1");
    }

    #[test]
    fn parse_response_content_length() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nX-Test: a\r\n\r\nhello";
        let (status, headers) = parse_response_head(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(header_value(&headers, "content-length"), Some("5"));
        assert_eq!(header_value(&headers, "x-test"), Some("a"));
    }

    #[test]
    fn decode_chunked_basic() {
        // "Wikipedia" chunked example: 4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n
        let mut buf = b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n".to_vec();
        let mut read = |_out: &mut [u8]| -> Result<usize, HttpError> { Ok(0) };
        let mut time = || 0i64;
        let body = decode_chunked_body(
            &mut buf,
            &mut read,
            1000,
            1024,
            &mut time,
            None::<&mut fn(i64)>,
        )
        .unwrap();
        assert_eq!(body, b"Wikipedia");
    }
}
