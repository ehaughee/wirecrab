use crate::flow::{IPAddress, Protocol};
use std::collections::HashMap;

pub mod ethernet;
pub mod ip;
pub mod tls;
pub mod transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    Ethernet,
    IPv4,
    IPv6,
    TCP,
    UDP,
    TLS,
    Unknown(u32),
}

#[derive(Debug)]
pub enum ParseResult<'a> {
    NextLayer {
        next_layer: LayerType,
        payload: &'a [u8],
    },
    Final,
    Error(String),
}

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

pub trait LayerParser: Send + Sync {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a>;
}

pub struct ParserRegistry {
    parsers: HashMap<LayerType, Box<dyn LayerParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: HashMap::new(),
        };

        registry.register(LayerType::Ethernet, Box::new(ethernet::EthernetParser));
        registry.register(LayerType::IPv4, Box::new(ip::IPv4Parser));
        registry.register(LayerType::IPv6, Box::new(ip::IPv6Parser));
        registry.register(LayerType::TCP, Box::new(transport::TcpParser));
        registry.register(LayerType::UDP, Box::new(transport::UdpParser));
        registry.register(LayerType::TLS, Box::new(tls::TlsParser));

        registry
    }

    pub fn register(&mut self, layer: LayerType, parser: Box<dyn LayerParser>) {
        self.parsers.insert(layer, parser);
    }

    pub fn get(&self, layer: LayerType) -> Option<&dyn LayerParser> {
        self.parsers.get(&layer).map(|p| p.as_ref())
    }
}
