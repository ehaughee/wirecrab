use crate::layers::{LayerParser, PacketContext, ParseResult};
use std::convert::TryFrom;
use tracing::warn;

pub struct TlsParser;

impl LayerParser for TlsParser {
    fn parse<'a>(&self, data: &'a [u8], context: &mut PacketContext) -> ParseResult<'a> {
        let mut cursor = 0;

        while data.len().saturating_sub(cursor) >= 5 {
            let header = &data[cursor..];
            let content_type = header[0];
            let version_major = header[1];
            let version_minor = header[2];
            let length = u16::from_be_bytes([header[3], header[4]]) as usize;

            let total_len = 5 + length;
            if data.len() < cursor + total_len {
                break; // Incomplete record at end of packet
            }

            let record = &data[cursor..cursor + total_len];
            let version = tls_version(version_major, version_minor);

            match content_type {
                20 => context.tags.push(format!("ChangeCipherSpec ({})", version)),
                21 => {
                    let record_payload = &record[5..];
                    if let Ok(alert) = Alert::try_from(record_payload) {
                        warn!(
                            version,
                            level = alert_level(alert.level),
                            description = alert.description_name(),
                            code = alert.description,
                            "TLS alert"
                        );
                    } else {
                        warn!(version, len = record_payload.len(), "TLS alert record truncated");
                    }
                    context.tags.push(format!("Alert ({})", version));
                }
                22 => {
                    if length > 0 {
                        let handshake_type = record[5];
                        let handshake = handshake_type_name(handshake_type);
                        context.tags.push(format!("{} ({})", handshake, version));
                    } else {
                        context.tags.push(format!("Handshake ({})", version));
                    }
                }
                23 => context.tags.push(format!("Application Data ({})", version)),
                _ => break, // Not TLS or unknown content type
            }

            cursor += total_len;
        }

        ParseResult::Final
    }
}

#[derive(Debug, Clone, Copy)]
struct Alert {
    level: u8,
    description: u8,
}

impl TryFrom<&[u8]> for Alert {
    type Error = &'static str;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() < 2 {
            return Err("TLS alert truncated");
        }

        Ok(Alert {
            level: payload[0],
            description: payload[1],
        })
    }
}

impl Alert {
    fn description_name(&self) -> String {
        match self.description {
            0 => "close_notify".to_owned(),
            10 => "unexpected_message".to_owned(),
            20 => "bad_record_mac".to_owned(),
            21 => "decryption_failed".to_owned(),
            22 => "record_overflow".to_owned(),
            40 => "handshake_failure".to_owned(),
            41 => "no_certificate".to_owned(),
            42 => "bad_certificate".to_owned(),
            43 => "unsupported_certificate".to_owned(),
            44 => "certificate_revoked".to_owned(),
            45 => "certificate_expired".to_owned(),
            46 => "certificate_unknown".to_owned(),
            47 => "illegal_parameter".to_owned(),
            48 => "unknown_ca".to_owned(),
            49 => "access_denied".to_owned(),
            50 => "decode_error".to_owned(),
            51 => "decrypt_error".to_owned(),
            70 => "protocol_version".to_owned(),
            71 => "insufficient_security".to_owned(),
            80 => "internal_error".to_owned(),
            86 => "inappropriate_fallback".to_owned(),
            90 => "user_canceled".to_owned(),
            100 => "no_renegotiation".to_owned(),
            110 => "unsupported_extension".to_owned(),
            112 => "unrecognized_name".to_owned(),
            113 => "bad_certificate_status_response".to_owned(),
            115 => "unknown_psk_identity".to_owned(),
            116 => "certificate_required".to_owned(),
            120 => "no_application_protocol".to_owned(),
            _ => format!("unknown({})", self.description),
        }
    }
}

fn tls_version(major: u8, minor: u8) -> String {
    match (major, minor) {
        (3, 0) => "SSL 3.0".to_owned(),
        (3, 1) => "TLS 1.0".to_owned(),
        (3, 2) => "TLS 1.1".to_owned(),
        (3, 3) => "TLS 1.2".to_owned(),
        (3, 4) => "TLS 1.3".to_owned(),
        _ => format!("TLS Unknown ({}.{})", major, minor),
    }
}

fn handshake_type_name(handshake_type: u8) -> String {
    match handshake_type {
        1 => "Client Hello".to_owned(),
        2 => "Server Hello".to_owned(),
        11 => "Certificate".to_owned(),
        12 => "Server Key Exchange".to_owned(),
        13 => "Certificate Request".to_owned(),
        14 => "Server Hello Done".to_owned(),
        15 => "Certificate Verify".to_owned(),
        16 => "Client Key Exchange".to_owned(),
        20 => "Finished".to_owned(),
        _ => format!("Handshake Unknown ({})", handshake_type),
    }
}

fn alert_level(level: u8) -> &'static str {
    match level {
        1 => "warning",
        2 => "fatal",
        _ => "unknown",
    }
}
