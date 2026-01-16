#![no_std]

//! DNS resolver implementation for moteOS
//!
//! This module provides DNS (Domain Name System) resolution functionality
//! for resolving hostnames to IPv4 addresses.
//!
//! The DNS resolver:
//! - Creates UDP sockets for DNS queries
//! - Sends DNS queries for A records (IPv4 addresses)
//! - Parses DNS responses
//! - Returns the resolved IP address
//!
//! See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.8 for details.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;

/// DNS query types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum QueryType {
    /// IPv4 address (A record)
    A = 1,
    /// IPv6 address (AAAA record)
    AAAA = 28,
}

/// DNS query class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum QueryClass {
    /// Internet class
    IN = 1,
}

/// DNS response codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResponseCode {
    /// No error
    NoError = 0,
    /// Format error
    FormatError = 1,
    /// Server failure
    ServerFailure = 2,
    /// Name error (domain doesn't exist)
    NameError = 3,
    /// Not implemented
    NotImplemented = 4,
    /// Refused
    Refused = 5,
}

impl ResponseCode {
    /// Convert from u8 to ResponseCode
    pub fn from_u8(code: u8) -> Option<Self> {
        match code {
            0 => Some(ResponseCode::NoError),
            1 => Some(ResponseCode::FormatError),
            2 => Some(ResponseCode::ServerFailure),
            3 => Some(ResponseCode::NameError),
            4 => Some(ResponseCode::NotImplemented),
            5 => Some(ResponseCode::Refused),
            _ => None,
        }
    }
}

/// DNS header structure (12 bytes)
///
/// Format (RFC 1035):
/// ```text
///                                 1  1  1  1  1  1
///   0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                      ID                       |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    QDCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    ANCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    NSCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    ARCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DnsHeader {
    /// Transaction ID
    pub id: u16,
    /// Flags (QR, Opcode, AA, TC, RD, RA, Z, RCODE)
    pub flags: u16,
    /// Number of questions
    pub qdcount: u16,
    /// Number of answers
    pub ancount: u16,
    /// Number of authority records
    pub nscount: u16,
    /// Number of additional records
    pub arcount: u16,
}

impl DnsHeader {
    /// Create a new DNS header for a query
    pub fn new_query(id: u16) -> Self {
        Self {
            id,
            flags: 0x0100, // Standard query with recursion desired (RD=1)
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    /// Serialize the header to bytes (big-endian)
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..2].copy_from_slice(&self.id.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.flags.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.arcount.to_be_bytes());
        bytes
    }

    /// Parse header from bytes (big-endian)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 12 {
            return Err("DNS header too short");
        }

        Ok(Self {
            id: u16::from_be_bytes([bytes[0], bytes[1]]),
            flags: u16::from_be_bytes([bytes[2], bytes[3]]),
            qdcount: u16::from_be_bytes([bytes[4], bytes[5]]),
            ancount: u16::from_be_bytes([bytes[6], bytes[7]]),
            nscount: u16::from_be_bytes([bytes[8], bytes[9]]),
            arcount: u16::from_be_bytes([bytes[10], bytes[11]]),
        })
    }

    /// Check if this is a response (QR bit set)
    pub fn is_response(&self) -> bool {
        (self.flags & 0x8000) != 0
    }

    /// Get the response code (RCODE)
    pub fn rcode(&self) -> u8 {
        (self.flags & 0x000F) as u8
    }

    /// Check if recursion is available (RA bit)
    pub fn recursion_available(&self) -> bool {
        (self.flags & 0x0080) != 0
    }
}

/// Encode a domain name in DNS format
///
/// Example: "example.com" becomes [7]example[3]com[0]
pub fn encode_domain_name(domain: &str) -> Vec<u8> {
    let mut encoded = Vec::new();

    for label in domain.split('.') {
        if label.is_empty() {
            continue;
        }

        // Length prefix (max 63 bytes per label)
        let len = label.len().min(63) as u8;
        encoded.push(len);

        // Label bytes
        encoded.extend_from_slice(&label.as_bytes()[..len as usize]);
    }

    // Null terminator
    encoded.push(0);

    encoded
}

/// Decode a domain name from DNS format
///
/// Supports compression pointers (RFC 1035 section 4.1.4)
pub fn decode_domain_name(data: &[u8], offset: usize) -> Result<(String, usize), &'static str> {
    let mut name = String::new();
    let mut pos = offset;
    let mut jumped = false;
    let mut jumps = 0;
    let max_jumps = 5; // Prevent infinite loops
    let original_pos = offset;

    loop {
        if pos >= data.len() {
            return Err("Unexpected end of data while decoding domain name");
        }

        let len = data[pos];

        // Check for compression pointer (top 2 bits set)
        if (len & 0xC0) == 0xC0 {
            if pos + 1 >= data.len() {
                return Err("Incomplete compression pointer");
            }

            // Calculate pointer offset
            let pointer = ((u16::from(len & 0x3F) << 8) | u16::from(data[pos + 1])) as usize;

            if pointer >= data.len() {
                return Err("Invalid compression pointer");
            }

            // Jump to pointer location
            pos = pointer;
            jumped = true;
            jumps += 1;

            if jumps > max_jumps {
                return Err("Too many compression jumps");
            }

            continue;
        }

        // Null terminator - end of domain name
        if len == 0 {
            pos += 1;
            break;
        }

        // Regular label
        if len > 63 {
            return Err("Label length too long");
        }

        pos += 1;

        if pos + len as usize > data.len() {
            return Err("Label extends beyond data");
        }

        // Add dot separator if not first label
        if !name.is_empty() {
            name.push('.');
        }

        // Add label to name
        for i in 0..len as usize {
            name.push(data[pos + i] as char);
        }

        pos += len as usize;
    }

    // Return the decoded name and the position after the name
    // If we jumped, return position after the pointer; otherwise after the null terminator
    let final_pos = if jumped {
        original_pos + 2 // Pointer is 2 bytes
    } else {
        pos
    };

    Ok((name, final_pos))
}

/// DNS query structure
pub struct DnsQuery {
    /// Domain name to query
    pub name: String,
    /// Query type (A, AAAA, etc.)
    pub qtype: QueryType,
    /// Query class (IN for Internet)
    pub qclass: QueryClass,
}

impl DnsQuery {
    /// Create a new DNS query for an A record
    pub fn new_a(domain: &str) -> Self {
        Self {
            name: domain.into(),
            qtype: QueryType::A,
            qclass: QueryClass::IN,
        }
    }

    /// Serialize the query to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Encode domain name
        bytes.extend_from_slice(&encode_domain_name(&self.name));

        // Query type (2 bytes)
        bytes.extend_from_slice(&(self.qtype as u16).to_be_bytes());

        // Query class (2 bytes)
        bytes.extend_from_slice(&(self.qclass as u16).to_be_bytes());

        bytes
    }
}

/// DNS answer/resource record
#[derive(Debug, Clone)]
pub struct DnsAnswer {
    /// Domain name
    pub name: String,
    /// Record type
    pub rtype: u16,
    /// Record class
    pub rclass: u16,
    /// Time to live (seconds)
    pub ttl: u32,
    /// Resource data
    pub rdata: Vec<u8>,
}

impl DnsAnswer {
    /// Parse an answer from bytes
    pub fn from_bytes(data: &[u8], offset: usize) -> Result<(Self, usize), &'static str> {
        // Decode domain name
        let (name, mut pos) = decode_domain_name(data, offset)?;

        // Check we have enough bytes for the rest of the record
        if pos + 10 > data.len() {
            return Err("Incomplete DNS answer");
        }

        // Parse type (2 bytes)
        let rtype = u16::from_be_bytes([data[pos], data[pos + 1]]);
        pos += 2;

        // Parse class (2 bytes)
        let rclass = u16::from_be_bytes([data[pos], data[pos + 1]]);
        pos += 2;

        // Parse TTL (4 bytes)
        let ttl = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;

        // Parse data length (2 bytes)
        let rdlength = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // Check we have enough bytes for rdata
        if pos + rdlength > data.len() {
            return Err("Incomplete DNS answer data");
        }

        // Extract rdata
        let rdata = data[pos..pos + rdlength].to_vec();
        pos += rdlength;

        Ok((
            Self {
                name,
                rtype,
                rclass,
                ttl,
                rdata,
            },
            pos,
        ))
    }

    /// Extract IPv4 address from A record data
    pub fn as_ipv4(&self) -> Option<[u8; 4]> {
        if self.rtype == QueryType::A as u16 && self.rdata.len() == 4 {
            Some(self.rdata.as_slice().try_into().ok()?)
        } else {
            None
        }
    }
}

/// DNS response structure
#[derive(Debug)]
pub struct DnsResponse {
    /// Response header
    pub header: DnsHeader,
    /// Answer records
    pub answers: Vec<DnsAnswer>,
}

impl DnsResponse {
    /// Parse a DNS response from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 12 {
            return Err("DNS response too short");
        }

        // Parse header
        let header = DnsHeader::from_bytes(data)?;

        // Check if this is a response
        if !header.is_response() {
            return Err("Not a DNS response");
        }

        // Start parsing questions (skip them - we know what we asked)
        let mut pos = 12;
        for _ in 0..header.qdcount {
            // Skip question name
            let (_, new_pos) = decode_domain_name(data, pos)?;
            pos = new_pos;

            // Skip qtype and qclass (4 bytes)
            if pos + 4 > data.len() {
                return Err("Incomplete question section");
            }
            pos += 4;
        }

        // Parse answer records
        let mut answers = Vec::new();
        for _ in 0..header.ancount {
            let (answer, new_pos) = DnsAnswer::from_bytes(data, pos)?;
            answers.push(answer);
            pos = new_pos;
        }

        Ok(Self { header, answers })
    }

    /// Get the first IPv4 address from the response
    pub fn first_ipv4(&self) -> Option<[u8; 4]> {
        for answer in &self.answers {
            if let Some(ip) = answer.as_ipv4() {
                return Some(ip);
            }
        }
        None
    }
}

/// Build a complete DNS query packet
pub fn build_query(hostname: &str, transaction_id: u16) -> Vec<u8> {
    let mut packet = Vec::new();

    // Header
    let header = DnsHeader::new_query(transaction_id);
    packet.extend_from_slice(&header.to_bytes());

    // Question
    let query = DnsQuery::new_a(hostname);
    packet.extend_from_slice(&query.to_bytes());

    packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_domain_name() {
        let encoded = encode_domain_name("example.com");
        assert_eq!(encoded, vec![7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm', 0]);
    }

    #[test]
    fn test_encode_domain_name_empty() {
        let encoded = encode_domain_name("");
        assert_eq!(encoded, vec![0]);
    }

    #[test]
    fn test_dns_header_query() {
        let header = DnsHeader::new_query(0x1234);
        assert_eq!(header.id, 0x1234);
        assert_eq!(header.flags, 0x0100); // Recursion desired
        assert_eq!(header.qdcount, 1);
        assert_eq!(header.ancount, 0);
        assert!(!header.is_response());
    }

    #[test]
    fn test_dns_header_serialization() {
        let header = DnsHeader::new_query(0x1234);
        let bytes = header.to_bytes();
        let parsed = DnsHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.id, header.id);
        assert_eq!(parsed.flags, header.flags);
        assert_eq!(parsed.qdcount, header.qdcount);
    }

    #[test]
    fn test_build_query() {
        let packet = build_query("example.com", 0x1234);

        // Should have header (12 bytes) + question
        assert!(packet.len() > 12);

        // Check header
        let header = DnsHeader::from_bytes(&packet[..12]).unwrap();
        assert_eq!(header.id, 0x1234);
        assert_eq!(header.qdcount, 1);
    }

    #[test]
    fn test_response_code_conversion() {
        assert_eq!(ResponseCode::from_u8(0), Some(ResponseCode::NoError));
        assert_eq!(ResponseCode::from_u8(2), Some(ResponseCode::ServerFailure));
        assert_eq!(ResponseCode::from_u8(3), Some(ResponseCode::NameError));
        assert_eq!(ResponseCode::from_u8(99), None);
    }
}
