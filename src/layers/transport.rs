use super::{LayerParser, PacketContext, ParseResult};
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

                // TCP payload might be empty or contain L7 data
                if rest.is_empty() {
                    ParseResult::Final
                } else {
                    // For now, we don't have L7 parsers, so we stop here or return Unknown
                    // But to support future L7, we could return NextLayer with Unknown/L7 type
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
            Ok((header, rest)) => {
                context.src_port = Some(header.source_port);
                context.dst_port = Some(header.destination_port);
                context.protocol = Some(Protocol::UDP);

                if rest.is_empty() {
                    ParseResult::Final
                } else {
                    ParseResult::Final
                }
            }
            Err(e) => ParseResult::Error(format!("UDP parse error: {}", e)),
        }
    }
}
