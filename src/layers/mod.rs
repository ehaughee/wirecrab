use crate::flow::{IPAddress, Protocol};

pub mod tls;

#[derive(Default, Debug, Clone)]
pub struct PacketContext {
    pub src_ip: Option<IPAddress>,
    pub dst_ip: Option<IPAddress>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: Option<Protocol>,
    pub is_syn: bool,
    pub is_ack: bool,
    pub tags: Vec<String>,
}

// Context populated while decoding packets; shared by decoders.
