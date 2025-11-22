use super::{LayerParser, LayerType, PacketContext, ParseResult};
use crate::flow::IPAddress;
use etherparse::{Ipv4Header, Ipv6Header, IpNumber};

pub struct IPv4Parser;

impl LayerParser for IPv4Parser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        match Ipv4Header::from_slice(data) {
            Ok((header, rest)) => {
                context.src_ip = Some(IPAddress::V4(header.source));
                context.dst_ip = Some(IPAddress::V4(header.destination));
                
                let next_layer = match header.protocol {
                    IpNumber::TCP => LayerType::TCP,
                    IpNumber::UDP => LayerType::UDP,
                    val => LayerType::Unknown(val.0 as u32),
                };

                ParseResult::NextLayer {
                    next_layer,
                    payload: rest,
                }
            }
            Err(e) => ParseResult::Error(format!("IPv4 parse error: {}", e)),
        }
    }
}

pub struct IPv6Parser;

impl LayerParser for IPv6Parser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        match Ipv6Header::from_slice(data) {
            Ok((header, rest)) => {
                context.src_ip = Some(IPAddress::V6(header.source));
                context.dst_ip = Some(IPAddress::V6(header.destination));

                let next_layer = match header.next_header {
                    IpNumber::TCP => LayerType::TCP,
                    IpNumber::UDP => LayerType::UDP,
                    val => LayerType::Unknown(val.0 as u32),
                };

                ParseResult::NextLayer {
                    next_layer,
                    payload: rest,
                }
            }
            Err(e) => ParseResult::Error(format!("IPv6 parse error: {}", e)),
        }
    }
}
