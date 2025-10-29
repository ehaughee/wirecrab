use crate::flow::{IPAddress, Protocol};

pub fn format_ip_address(ip: &IPAddress) -> String {
    match ip {
        IPAddress::V4(addr) => format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]),
        IPAddress::V6(addr) => {
            format!(
                "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                addr[0],
                addr[1],
                addr[2],
                addr[3],
                addr[4],
                addr[5],
                addr[6],
                addr[7],
                addr[8],
                addr[9],
                addr[10],
                addr[11],
                addr[12],
                addr[13],
                addr[14],
                addr[15]
            )
        }
    }
}

pub fn format_protocol(protocol: &Protocol) -> String {
    match protocol {
        Protocol::TCP => "TCP".to_string(),
        Protocol::UDP => "UDP".to_string(),
        Protocol::Other(n) => format!("Proto-{}", n),
    }
}