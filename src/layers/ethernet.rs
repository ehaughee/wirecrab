use super::{LayerParser, LayerType, PacketContext, ParseResult};
use etherparse::{Ethernet2Header, EtherType};

pub struct EthernetParser;

impl LayerParser for EthernetParser {
    fn parse(&self, data: &[u8], _context: &mut PacketContext) -> ParseResult {
        match Ethernet2Header::from_slice(data) {
            Ok((header, rest)) => {
                let next_layer = match header.ether_type {
                    EtherType::IPV4 => LayerType::IPv4,
                    EtherType::IPV6 => LayerType::IPv6,
                    val => LayerType::Unknown(val.0 as u32),
                };
                ParseResult::NextLayer {
                    next_layer,
                    payload: rest.to_vec(),
                }
            }
            Err(e) => ParseResult::Error(format!("Ethernet parse error: {}", e)),
        }
    }
}
