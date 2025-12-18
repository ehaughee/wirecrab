use crate::layers::{LayerParser, PacketContext, ParseResult};
use tls_parser::{TlsMessage, TlsMessageHandshake, TlsRecordType, TlsVersion, parse_tls_plaintext};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentType {
    ChangeCipherSpec,
    Alert,
    Handshake,
    ApplicationData,
    Unknown(u8),
}

pub struct TlsParser;

impl LayerParser for TlsParser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        let mut input = data;

        while !input.is_empty() {
            match parse_tls_plaintext(input) {
                Ok((remaining, record)) => {
                    let content_type = map_record_type(record.hdr.record_type);
                    let version = tls_version_from_parser(record.hdr.version);

                    for msg in &record.msg {
                        handle_message(content_type, &version, msg, context);
                    }

                    if matches!(content_type, ContentType::Unknown(_)) {
                        break;
                    }

                    input = remaining;
                }
                Err(_) => break, // incomplete or invalid; stop at current packet boundary
            }
        }

        ParseResult::Final
    }
}

fn handle_message(
    content_type: ContentType,
    version: &str,
    msg: &TlsMessage,
    context: &mut PacketContext,
) {
    match content_type {
        ContentType::ChangeCipherSpec => {
            if matches!(msg, TlsMessage::ChangeCipherSpec) {
                context.tags.push(format!("ChangeCipherSpec ({})", version));
            }
        }
        ContentType::Alert => {
            if let TlsMessage::Alert(alert) = msg {
                warn!(
                    version,
                    level = ?alert.severity,
                    description = ?alert.code,
                    "TLS alert"
                );
            } else {
                warn!(version, "TLS alert record truncated or unexpected payload");
            }
            context.tags.push(format!("Alert ({})", version));
        }
        ContentType::Handshake => {
            if let TlsMessage::Handshake(hs) = msg {
                let handshake = handshake_label(hs);
                context.tags.push(format!("{} ({})", handshake, version));
            } else {
                context.tags.push(format!("Handshake ({})", version));
            }
        }
        ContentType::ApplicationData => {
            if matches!(msg, TlsMessage::ApplicationData(_)) {
                context.tags.push(format!("Application Data ({})", version));
            }
        }
        ContentType::Unknown(_) => {
            // stop parsing when we encounter non-TLS content types
        }
    }
}

fn tls_version_from_parser(version: TlsVersion) -> String {
    format!("{:?}", version)
}

fn handshake_label(msg: &TlsMessageHandshake) -> String {
    match msg {
        TlsMessageHandshake::ClientHello(_) => "Client Hello".to_owned(),
        TlsMessageHandshake::ServerHello(_) => "Server Hello".to_owned(),
        TlsMessageHandshake::Certificate(_) => "Certificate".to_owned(),
        TlsMessageHandshake::ServerKeyExchange(_) => "Server Key Exchange".to_owned(),
        TlsMessageHandshake::CertificateRequest(_) => "Certificate Request".to_owned(),
        TlsMessageHandshake::CertificateVerify(_) => "Certificate Verify".to_owned(),
        TlsMessageHandshake::ClientKeyExchange(_) => "Client Key Exchange".to_owned(),
        TlsMessageHandshake::Finished(_) => "Finished".to_owned(),
        TlsMessageHandshake::NewSessionTicket(_) => "New Session Ticket".to_owned(),
        TlsMessageHandshake::EndOfEarlyData => "End Of Early Data".to_owned(),
        TlsMessageHandshake::KeyUpdate(_) => "Key Update".to_owned(),
        TlsMessageHandshake::HelloRequest => "Hello Request".to_owned(),
        other => format!("Handshake {:?}", other),
    }
}

fn map_record_type(record_type: TlsRecordType) -> ContentType {
    match record_type {
        TlsRecordType::ChangeCipherSpec => ContentType::ChangeCipherSpec,
        TlsRecordType::Alert => ContentType::Alert,
        TlsRecordType::Handshake => ContentType::Handshake,
        TlsRecordType::ApplicationData => ContentType::ApplicationData,
        other => ContentType::Unknown(u8::from(other)),
    }
}
