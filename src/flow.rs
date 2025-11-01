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

#[derive(Debug, Clone)]
pub struct Flow {
    pub timestamp: f64,
    pub src_ip: IPAddress,
    pub dst_ip: IPAddress,
    // TODO: Make ports required instead of optional?
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: Protocol,
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
            src_ip: IPAddress::V4([0, 0, 0, 0]),
            dst_ip: IPAddress::V4([0, 0, 0, 0]),
            src_port: None,
            dst_port: None,
            protocol: Protocol::Other(0),
            packets: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowKey {
    pub src_ip: IPAddress,
    pub dst_ip: IPAddress,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: Protocol,
}

impl FlowKey {
    pub fn try_from_flow(flow: &Flow) -> Option<Self> {
        let (Some(src_port), Some(dst_port)) = (flow.src_port, flow.dst_port) else {
            return None;
        };
        Some(FlowKey {
            src_ip: flow.src_ip,
            dst_ip: flow.dst_ip,
            src_port,
            dst_port,
            protocol: flow.protocol,
        })
    }

    // pub fn to_display(&self) -> String {
    //     fn fmt_ip(ip: &IPAddress) -> String {
    //         match ip {
    //             IPAddress::V4(b) => format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]),
    //             IPAddress::V6(b) => {
    //                 // Represent as 8 hextets (no zero-compression)
    //                 let parts: Vec<String> = b
    //                     .chunks(2)
    //                     .map(|c| format!("{:x}", u16::from_be_bytes([c[0], c[1]])))
    //                     .collect();
    //                 parts.join(":")
    //             }
    //         }
    //     }
    //     let proto = match self.protocol {
    //         Protocol::TCP => "TCP",
    //         Protocol::UDP => "UDP",
    //         Protocol::Other(p) => return format!("OTHER({}) {:?}", p, self),
    //     };
    //     format!(
    //         "{} {}:{} -> {}:{}",
    //         proto,
    //         fmt_ip(&self.src_ip),
    //         self.src_port,
    //         fmt_ip(&self.dst_ip),
    //         self.dst_port
    //     )
    // }
}

use std::fmt;

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
