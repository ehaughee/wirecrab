use crate::flow::{Endpoint, Flow, FlowKey, IPAddress, Protocol};
use crate::layers::PacketContext;
use std::collections::HashMap;

pub fn add_packet(
    epb_packet_data: &[u8],
    context: PacketContext,
    timestamp: f64,
    flows: &mut HashMap<FlowKey, Flow>,
    packet_count: &mut usize,
) {
    if let Some((src_ip, dst_ip, src_port, dst_port, protocol)) = unpack_context(&context) {
        let src_ep = Endpoint::new(src_ip, src_port);
        let dst_ep = Endpoint::new(dst_ip, dst_port);
        let key = FlowKey::from_endpoints(src_ep, dst_ep, protocol);
        let packet_length = u16::try_from(epb_packet_data.len()).unwrap_or(u16::MAX);

        let packet = crate::flow::Packet {
            timestamp,
            src_ip,
            dst_ip,
            src_port: Some(src_port),
            dst_port: Some(dst_port),
            length: packet_length,
            data: epb_packet_data.to_vec(),
            tags: context.tags,
        };

        let flow = flows.entry(key).or_insert_with(|| Flow {
            timestamp,
            protocol,
            source: src_ep,
            destination: dst_ep,
            packets: Vec::new(),
        });

        if protocol == Protocol::TCP && context.is_syn && !context.is_ack {
            flow.source = src_ep;
            flow.destination = dst_ep;
        }

        flow.packets.push(packet);
        *packet_count += 1;
    }
}

fn unpack_context(context: &PacketContext) -> Option<(IPAddress, IPAddress, u16, u16, Protocol)> {
    match (
        context.src_ip,
        context.dst_ip,
        context.src_port,
        context.dst_port,
        context.protocol,
    ) {
        (Some(src_ip), Some(dst_ip), Some(src_port), Some(dst_port), Some(protocol)) => {
            Some((src_ip, dst_ip, src_port, dst_port, protocol))
        }
        _ => None,
    }
}
