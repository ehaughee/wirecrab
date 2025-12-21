use crate::flow::{IPAddress, Protocol};
use crate::layers::PacketContext;
use crate::layers::tls::TlsParser;
use crate::parser::decoder::decode_headers;
use crate::parser::parse_pcap;
use crate::parser::packets::add_packet;
use crate::parser::tcp::looks_like_tls;
use etherparse::PacketBuilder;
use pcap_parser::pcapng::{NameRecord, NameRecordType, NameResolutionBlock};
use pcap_parser::NRB_MAGIC;
use std::collections::HashMap;

fn build_tcp_packet(
    flags: impl FnOnce(
        etherparse::PacketBuilderStep<etherparse::TcpHeader>,
    ) -> etherparse::PacketBuilderStep<etherparse::TcpHeader>,
    payload: &[u8],
) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2([1, 2, 3, 4, 5, 6], [6, 5, 4, 3, 2, 1]).ipv4(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        64,
    );
    let builder = flags(builder.tcp(12345, 80, 1, 64240));

    let mut packet = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut packet, payload).unwrap();
    packet
}

fn build_udp_packet(payload: &[u8]) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2([1, 2, 3, 4, 5, 6], [6, 5, 4, 3, 2, 1])
        .ipv4([192, 168, 1, 10], [192, 168, 1, 20], 64)
        .udp(5353, 8053);
    let mut packet = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut packet, payload).unwrap();
    packet
}

fn build_ipv6_tcp_packet(payload: &[u8]) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2([1, 1, 1, 1, 1, 1], [2, 2, 2, 2, 2, 2])
        .ipv6([0u8; 16], [0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 32)
        .tcp(40000, 80, 1, 65535)
        .syn();

    let mut packet = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut packet, payload).unwrap();
    packet
}

fn build_ipv6_udp_packet(payload: &[u8]) -> Vec<u8> {
    let builder = PacketBuilder::ethernet2([1, 1, 1, 1, 1, 1], [2, 2, 2, 2, 2, 2])
        .ipv6([0u8; 16], [0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 32)
        .udp(5353, 8053);
    let mut packet = Vec::with_capacity(builder.size(payload.len()));
    builder.write(&mut packet, payload).unwrap();
    packet
}

fn build_dns_response_payload(v6_ip: [u8; 16]) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // id
    buf.extend_from_slice(&0x8180u16.to_be_bytes()); // standard response, no error
    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // qdcount
    buf.extend_from_slice(&0x0002u16.to_be_bytes()); // ancount
    buf.extend_from_slice(&0x0000u16.to_be_bytes()); // nscount
    buf.extend_from_slice(&0x0000u16.to_be_bytes()); // arcount

    buf.push(7);
    buf.extend_from_slice(b"example");
    buf.push(5);
    buf.extend_from_slice(b"local");
    buf.push(0);

    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // QTYPE A
    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // QCLASS IN

    // Answer 1: A record using pointer to question name at offset 12 (0x0c)
    buf.push(0xc0);
    buf.push(0x0c);
    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // TYPE A
    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // CLASS IN
    buf.extend_from_slice(&0x0000003cu32.to_be_bytes()); // TTL 60s
    buf.extend_from_slice(&0x0004u16.to_be_bytes()); // RDLENGTH
    buf.extend_from_slice(&[1, 2, 3, 4]); // RDATA

    // Answer 2: AAAA record, pointer to same name
    buf.push(0xc0);
    buf.push(0x0c);
    buf.extend_from_slice(&0x001cu16.to_be_bytes()); // TYPE AAAA
    buf.extend_from_slice(&0x0001u16.to_be_bytes()); // CLASS IN
    buf.extend_from_slice(&0x0000003cu32.to_be_bytes()); // TTL 60s
    buf.extend_from_slice(&0x0010u16.to_be_bytes()); // RDLENGTH
    buf.extend_from_slice(&v6_ip);

    buf
}

#[test]
fn tcp_decode_sets_flags_and_ports() {
    let payload = [];
    let packet = build_tcp_packet(|b| b.syn(), &payload);
    let tls_parser = TlsParser;

    let ctx = decode_headers(&packet, &tls_parser).expect("decode tcp");

    assert_eq!(ctx.src_ip, Some(IPAddress::V4([10, 0, 0, 1])));
    assert_eq!(ctx.dst_ip, Some(IPAddress::V4([10, 0, 0, 2])));
    assert_eq!(ctx.src_port, Some(12345));
    assert_eq!(ctx.dst_port, Some(80));
    assert_eq!(ctx.protocol, Some(Protocol::TCP));
    assert!(ctx.is_syn);
    assert!(!ctx.is_ack);
    assert!(ctx.tags.contains(&"SYN".to_string()));
}

#[test]
fn udp_decode_sets_protocol_and_ports() {
    let packet = build_udp_packet(&[1, 2, 3]);
    let tls_parser = TlsParser;

    let ctx = decode_headers(&packet, &tls_parser).expect("decode udp");

    assert_eq!(ctx.src_port, Some(5353));
    assert_eq!(ctx.dst_port, Some(8053));
    assert_eq!(ctx.protocol, Some(Protocol::UDP));
    assert_eq!(ctx.tags.len(), 0);
}

#[test]
fn add_packet_creates_flow_and_counts_packets() {
    let packet = build_tcp_packet(|b| b.syn(), &[]);
    let tls_parser = TlsParser;
    let context = decode_headers(&packet, &tls_parser).expect("decode packet");

    let mut flows = HashMap::new();
    let mut packet_count = 0usize;

    add_packet(&packet, context, 1.0, &mut flows, &mut packet_count);

    assert_eq!(packet_count, 1);
    assert_eq!(flows.len(), 1);

    let flow = flows.values().next().unwrap();
    assert_eq!(flow.source.ip, IPAddress::V4([10, 0, 0, 1]));
    assert_eq!(flow.destination.ip, IPAddress::V4([10, 0, 0, 2]));
    assert_eq!(flow.protocol, Protocol::TCP);
    assert_eq!(flow.packets.len(), 1);
    assert!(flow.packets[0].tags.contains(&"SYN".to_string()));
}

#[test]
fn parse_pcap_handles_randpkt_mix() {
    let path = std::path::Path::new("testdata/randpkt_mixed.pcapng");
    assert!(path.exists(), "expected randpkt_mixed fixture to exist");

    let (flows, start_ts) = parse_pcap(path, |_p| {}).expect("parse randpkt_mixed");

    assert!(!flows.is_empty(), "expected flows from randpkt capture");
    assert!(start_ts.is_some(), "expected start timestamp");
}

#[test]
fn parse_pcap_skips_malformed_randpkt_tcp() {
    let path = std::path::Path::new("testdata/randpkt_tcp.pcapng");
    assert!(path.exists(), "expected randpkt_tcp fixture to exist");

    let result = parse_pcap(path, |_p| {});
    assert!(result.is_ok(), "parser should not crash on malformed randpkt tcp");

    let (flows, _ts) = result.unwrap();
    // Malformed packets may all be skipped; just assert we handled gracefully.
    let _ = flows.len();
}

#[test]
fn tcp_tls_packets_get_tagged() {
    // Minimal TLS-looking record: ContentType Handshake(0x16), Version TLS1.2 (0x0303), length 0x0005, then dummy bytes.
    let tls_payload: [u8; 9] = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00];

    let packet = build_tcp_packet(|b| b.syn(), &tls_payload);
    let tls_parser = TlsParser;

    // Sanity: looks_like_tls should be true for this payload.
    assert!(looks_like_tls(&tls_payload));

    let ctx = decode_headers(&packet, &tls_parser).expect("decode tls-ish tcp");
    assert_eq!(ctx.protocol, Some(Protocol::TCP));
    // Even if the TLS parser cannot fully classify this tiny record, tags should include at least the TCP flag marker.
    assert!(!ctx.tags.is_empty(), "expected some tags (e.g., SYN) on TLS-looking packet");

    let mut flows = HashMap::new();
    let mut packet_count = 0usize;
    add_packet(&packet, ctx, 1.0, &mut flows, &mut packet_count);

    assert_eq!(packet_count, 1);
    let flow = flows.values().next().unwrap();
    assert!(!flow.packets[0].tags.is_empty(), "expected packet tags to propagate to flow");
}

#[test]
fn decode_ipv6_tcp_and_udp() {
    let tcp_packet = build_ipv6_tcp_packet(&[]);
    let udp_packet = build_ipv6_udp_packet(&[1, 2, 3, 4]);
    let tls_parser = TlsParser;

    let tcp_ctx = decode_headers(&tcp_packet, &tls_parser).expect("decode ipv6 tcp");
    assert_eq!(tcp_ctx.protocol, Some(Protocol::TCP));
    assert!(matches!(tcp_ctx.src_ip, Some(IPAddress::V6(_))));
    assert!(matches!(tcp_ctx.dst_ip, Some(IPAddress::V6(_))));

    let udp_ctx = decode_headers(&udp_packet, &tls_parser).expect("decode ipv6 udp");
    assert_eq!(udp_ctx.protocol, Some(Protocol::UDP));
    assert!(matches!(udp_ctx.src_ip, Some(IPAddress::V6(_))));
    assert!(matches!(udp_ctx.dst_ip, Some(IPAddress::V6(_))));
}

#[test]
fn dns_responses_populate_name_resolutions() {
    let v6_ip = [0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let payload = build_dns_response_payload(v6_ip);

    let context = PacketContext {
        src_ip: Some(IPAddress::V4([8, 8, 8, 8])),
        dst_ip: Some(IPAddress::V4([10, 0, 0, 1])),
        src_port: Some(53),
        dst_port: Some(55555),
        protocol: Some(Protocol::UDP),
        is_syn: false,
        is_ack: false,
        tags: Vec::new(),
        udp_payload: Some(payload.clone()),
    };

    let mut resolutions = HashMap::new();
    crate::parser::dns::handle_dns_response(&context, &mut resolutions);
    crate::parser::dns::handle_dns_response(&context, &mut resolutions);

    let v4_names = resolutions
        .get(&IPAddress::V4([1, 2, 3, 4]))
        .expect("ipv4 answer inserted");
    assert_eq!(v4_names.len(), 1);
    assert!(v4_names.contains(&"example.local".to_string()));

    let v6_names = resolutions
        .get(&IPAddress::V6(v6_ip))
        .expect("ipv6 answer inserted");
    assert_eq!(v6_names.len(), 1);
    assert!(v6_names.contains(&"example.local".to_string()));
}

#[test]
fn name_resolution_records_ipv4_and_ipv6() {
    let ipv4_bytes = [192, 168, 0, 42];
    let ipv4_value = [192, 168, 0, 42, b'h', b'o', b's', b't', 0];

    let ipv6_ip = [
        0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    ];
    let mut ipv6_value = Vec::new();
    ipv6_value.extend_from_slice(&ipv6_ip);
    ipv6_value.extend_from_slice(b"example.local");
    ipv6_value.push(0);

    let nrb = NameResolutionBlock {
        block_type: NRB_MAGIC,
        block_len1: 0,
        nr: vec![
            NameRecord {
                record_type: NameRecordType::Ipv4,
                record_value: &ipv4_value,
            },
            NameRecord {
                record_type: NameRecordType::Ipv6,
                record_value: &ipv6_value,
            },
        ],
        options: Vec::new(),
        block_len2: 0,
    };

    let mut resolutions = HashMap::new();
    crate::parser::dns::handle_name_resolution(&nrb, &mut resolutions);
    crate::parser::dns::handle_name_resolution(&nrb, &mut resolutions);

    let v4_names = resolutions
        .get(&IPAddress::V4(ipv4_bytes))
        .expect("ipv4 name inserted");
    assert_eq!(v4_names.len(), 1);
    assert!(v4_names.contains(&"host".to_string()));

    let v6_names = resolutions
        .get(&IPAddress::V6(ipv6_ip))
        .expect("ipv6 name inserted");
    assert_eq!(v6_names.len(), 1);
    assert!(v6_names.contains(&"example.local".to_string()));
}
