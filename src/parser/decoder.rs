use crate::flow::{IPAddress, Protocol};
use crate::layers::PacketContext;
use crate::layers::tls::TlsParser;
use etherparse::{NetHeaders, PacketHeaders, TransportHeader};
use tracing::trace;

pub fn decode_headers(packet: &[u8], tls_parser: &TlsParser) -> Result<PacketContext, String> {
    let mut context = PacketContext::default();

    let headers = PacketHeaders::from_ethernet_slice(packet).map_err(|err| {
        trace!(error = ?err, "Failed to parse packet headers");
        format!("header parse error: {err:?}")
    })?;

    if let Some(net) = &headers.net {
        match net {
            NetHeaders::Ipv4(ip, _) => {
                context.src_ip = Some(IPAddress::V4(ip.source));
                context.dst_ip = Some(IPAddress::V4(ip.destination));
            }
            NetHeaders::Ipv6(ip, _) => {
                context.src_ip = Some(IPAddress::V6(ip.source));
                context.dst_ip = Some(IPAddress::V6(ip.destination));
            }
            _ => {}
        }
    }

    let payload = headers.payload.slice();

    if let Some(transport) = headers.transport {
        match transport {
            TransportHeader::Tcp(tcp) => {
                context.src_port = Some(tcp.source_port);
                context.dst_port = Some(tcp.destination_port);
                context.protocol = Some(Protocol::TCP);
                context.is_syn = tcp.syn;
                context.is_ack = tcp.ack;

                if tcp.syn {
                    if tcp.ack {
                        context.tags.push("SYN-ACK".to_string());
                    } else {
                        context.tags.push("SYN".to_string());
                    }
                } else if tcp.fin {
                    context.tags.push("FIN".to_string());
                } else if tcp.rst {
                    context.tags.push("RST".to_string());
                } else if tcp.ack && payload.is_empty() {
                    context.tags.push("ACK".to_string());
                }

                if looks_like_tls(payload) {
                    tls_parser.parse(payload, &mut context);
                }
            }
            TransportHeader::Udp(udp) => {
                context.src_port = Some(udp.source_port);
                context.dst_port = Some(udp.destination_port);
                context.protocol = Some(Protocol::UDP);
            }
            _ => {}
        }
    }

    Ok(context)
}

fn looks_like_tls(payload: &[u8]) -> bool {
    if payload.len() < 5 {
        return false;
    }
    let content_type = payload[0];
    let version_major = payload[1];
    (20..=23).contains(&content_type) && version_major == 3
}
