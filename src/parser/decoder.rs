use crate::flow::{IPAddress, Protocol};
use crate::layers::PacketContext;
use crate::layers::tls::TlsParser;
use crate::parser::tcp::{looks_like_tls, tag_tcp};
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
                tag_tcp(&tcp, payload.len(), &mut context);
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
