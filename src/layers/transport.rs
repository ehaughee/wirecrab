use super::{LayerParser, LayerType, PacketContext, ParseResult};
use crate::flow::Protocol;
use etherparse::{TcpHeader, UdpHeader};

pub struct TcpParser;

impl LayerParser for TcpParser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        match TcpHeader::from_slice(data) {
            Ok((header, rest)) => {
                context.src_port = Some(header.source_port);
                context.dst_port = Some(header.destination_port);
                context.protocol = Some(Protocol::TCP);
                context.is_syn = header.syn;
                context.is_ack = header.ack;

                if header.syn {
                    if header.ack {
                        context.tags.push("SYN-ACK".to_string());
                    } else {
                        context.tags.push("SYN".to_string());
                    }
                } else if header.fin {
                    context.tags.push("FIN".to_string());
                } else if header.rst {
                    context.tags.push("RST".to_string());
                } else if header.ack && rest.is_empty() {
                    context.tags.push("ACK".to_string());
                }

                // TCP payload might be empty or contain L7 data
                if rest.is_empty() {
                    ParseResult::Final
                } else {
                    // Check for TLS
                    if rest.len() >= 5 {
                        let content_type = rest[0];
                        let version_major = rest[1];
                        
                        if (20..=23).contains(&content_type) && version_major == 3 {
                             return ParseResult::NextLayer {
                                 next_layer: LayerType::TLS,
                                 payload: rest,
                             };
                        }
                    }
                    ParseResult::Final
                }
            }
            Err(e) => ParseResult::Error(format!("TCP parse error: {}", e)),
        }
    }
}

pub struct UdpParser;

impl LayerParser for UdpParser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        match UdpHeader::from_slice(data) {
            Ok((header, _)) => {
                context.src_port = Some(header.source_port);
                context.dst_port = Some(header.destination_port);
                context.protocol = Some(Protocol::UDP);

                ParseResult::Final
            }
            Err(e) => ParseResult::Error(format!("UDP parse error: {}", e)),
        }
    }
}
