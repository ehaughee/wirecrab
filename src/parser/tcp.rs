use crate::flow::Protocol;
use crate::layers::PacketContext;
use etherparse::TcpHeader;

pub fn tag_tcp(header: &TcpHeader, payload_len: usize, context: &mut PacketContext) {
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
    } else if header.ack && payload_len == 0 {
        context.tags.push("ACK".to_string());
    }
}

pub fn looks_like_tls(payload: &[u8]) -> bool {
    if payload.len() < 5 {
        return false;
    }
    let content_type = payload[0];
    let version_major = payload[1];
    (20..=23).contains(&content_type) && version_major == 3
}