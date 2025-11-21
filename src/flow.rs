use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IPAddress {
    V4([u8; 4]),
    V6([u8; 16]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    TCP,
    UDP,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub ip: IPAddress,
    pub port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowEndpoints {
    pub first: Endpoint,
    pub second: Endpoint,
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub timestamp: f64,
    pub protocol: Protocol,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub packets: Vec<Packet>,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub timestamp: f64,
    pub src_ip: IPAddress,
    pub dst_ip: IPAddress,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub length: u32,
    pub data: Vec<u8>,
}

impl Default for Flow {
    fn default() -> Self {
        Flow {
            timestamp: 0.0,
            protocol: Protocol::Other(0),
            source: Endpoint {
                ip: IPAddress::V4([0, 0, 0, 0]),
                port: 0,
            },
            destination: Endpoint {
                ip: IPAddress::V4([0, 0, 0, 0]),
                port: 0,
            },
            packets: Vec::new(),
        }
    }
}

impl fmt::Display for Flow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ↔ {} ({:?})",
            self.source, self.destination, self.protocol
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowKey {
    pub endpoints: FlowEndpoints,
    pub protocol: Protocol,
}

impl FlowKey {}

impl fmt::Display for IPAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IPAddress::V4(bytes) => {
                write!(f, "{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
            }
            IPAddress::V6(bytes) => {
                let segments: Vec<String> = bytes
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                    .map(|segment| format!("{:x}", segment))
                    .collect();
                write!(f, "{}", segments.join(":"))
            }
        }
    }
}

impl IPAddress {
    fn cmp_bytes(&self, other: &Self) -> Ordering {
        match (self, other) {
            (IPAddress::V4(a), IPAddress::V4(b)) => a.cmp(b),
            (IPAddress::V4(_), IPAddress::V6(_)) => Ordering::Less,
            (IPAddress::V6(_), IPAddress::V4(_)) => Ordering::Greater,
            (IPAddress::V6(a), IPAddress::V6(b)) => a.cmp(b),
        }
    }
}

impl Ord for IPAddress {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_bytes(other)
    }
}

impl PartialOrd for IPAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Endpoint {
    pub fn new(ip: IPAddress, port: u16) -> Self {
        Self { ip, port }
    }
}

impl Ord for Endpoint {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.ip.cmp(&other.ip) {
            Ordering::Equal => self.port.cmp(&other.port),
            ord => ord,
        }
    }
}

impl PartialOrd for Endpoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FlowEndpoints {
    pub fn new(a: Endpoint, b: Endpoint) -> Self {
        FlowEndpoints {
            first: std::cmp::min(a, b),
            second: std::cmp::max(a, b),
        }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

impl fmt::Display for FlowEndpoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ↔ {}", self.first, self.second)
    }
}
