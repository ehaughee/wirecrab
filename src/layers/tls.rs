use crate::layers::{LayerParser, PacketContext, ParseResult};

pub struct TlsParser;

impl LayerParser for TlsParser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        if data.len() < 5 {
            return ParseResult::Final;
        }

        let content_type = data[0];
        let version_major = data[1];
        let version_minor = data[2];
        let length = u16::from_be_bytes([data[3], data[4]]) as usize;

        if data.len() < 5 + length {
            return ParseResult::Final; // Incomplete record
        }

        let version_str = match (version_major, version_minor) {
            (3, 0) => "SSL 3.0",
            (3, 1) => "TLS 1.0",
            (3, 2) => "TLS 1.1",
            (3, 3) => "TLS 1.2",
            (3, 4) => "TLS 1.3",
            _ => "TLS Unknown",
        };

        match content_type {
            20 => context.tags.push(format!("ChangeCipherSpec ({})", version_str)),
            21 => context.tags.push(format!("Alert ({})", version_str)),
            22 => {
                // Handshake
                if length > 0 {
                    let handshake_type = data[5];
                    let handshake_str = match handshake_type {
                        1 => "Client Hello",
                        2 => "Server Hello",
                        11 => "Certificate",
                        12 => "Server Key Exchange",
                        13 => "Certificate Request",
                        14 => "Server Hello Done",
                        15 => "Certificate Verify",
                        16 => "Client Key Exchange",
                        20 => "Finished",
                        _ => "Handshake",
                    };
                    context.tags.push(format!("{} ({})", handshake_str, version_str));
                } else {
                    context.tags.push(format!("Handshake ({})", version_str));
                }
            }
            23 => context.tags.push(format!("Application Data ({})", version_str)),
            _ => return ParseResult::Final, // Not TLS or unknown content type
        }

        // For now, we don't parse inner TLS layers or multiple records in one packet deeply
        // We just mark it as TLS and stop.
        // In a real implementation, we might want to iterate over records.
        
        ParseResult::Final
    }
}
