use super::filter::FlowFilter;
use super::*;

fn sample_flow() -> Flow {
    Flow {
        timestamp: 5.0,
        protocol: Protocol::TCP,
        source: Endpoint::new(IPAddress::V4([10, 0, 0, 1]), 12345),
        destination: Endpoint::new(IPAddress::V4([10, 0, 0, 2]), 80),
        packets: vec![],
    }
}

#[test]
fn total_bytes_sums_packet_lengths() {
    let packets = vec![
        Packet {
            timestamp: 0.0,
            src_ip: IPAddress::V4([10, 0, 0, 1]),
            dst_ip: IPAddress::V4([10, 0, 0, 2]),
            src_port: Some(10),
            dst_port: Some(20),
            length: 64,
            data: vec![],
            tags: vec![],
        },
        Packet {
            timestamp: 0.1,
            src_ip: IPAddress::V4([10, 0, 0, 1]),
            dst_ip: IPAddress::V4([10, 0, 0, 2]),
            src_port: Some(10),
            dst_port: Some(20),
            length: 128,
            data: vec![],
            tags: vec![],
        },
    ];

    let flow = Flow {
        timestamp: 0.0,
        protocol: Protocol::TCP,
        source: Endpoint::new(IPAddress::V4([10, 0, 0, 1]), 10),
        destination: Endpoint::new(IPAddress::V4([10, 0, 0, 2]), 20),
        packets,
    };

    assert_eq!(flow.total_bytes(), 64 + 128);
}

#[test]
fn match_all_accepts_everything() {
    let filter = FlowFilter::new("   ", None);
    assert!(filter.matches_flow(&sample_flow()));
}

#[test]
fn matches_ip_port_and_protocol() {
    let flow = sample_flow();

    assert!(FlowFilter::new("10.0.0.1", None).matches_flow(&flow));
    assert!(FlowFilter::new("80", None).matches_flow(&flow));
    assert!(FlowFilter::new("tcp", None).matches_flow(&flow));
}

#[test]
fn matches_relative_timestamp() {
    let flow = sample_flow();
    let filter = FlowFilter::new("3.000000", Some(2.0));
    assert!(filter.matches_flow(&flow));
}

#[test]
fn matches_ipv6_and_other_protocol() {
    let flow = Flow {
        timestamp: 1.0,
        protocol: Protocol::Other(99),
        source: Endpoint::new(IPAddress::V6([0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]), 443),
        destination: Endpoint::new(IPAddress::V6([0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]), 8443),
        packets: vec![],
    };

    assert!(FlowFilter::new("fe80:0:0:0:0:0:0:1", None).matches_flow(&flow));
    assert!(FlowFilter::new("proto-99", None).matches_flow(&flow));
}
