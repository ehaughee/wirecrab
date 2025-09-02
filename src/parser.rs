use crate::flow::*;
use pcap_parser::traits::{PcapNGPacketBlock, PcapReaderIterator};
use pcap_parser::*;
use std::collections::HashMap;
use std::fs::File;

struct InterfaceDescription {
    linktype: Linktype,
    ts_resolution: u8,
    ts_offset: i64,
}

pub fn parse(file_path: &str) -> HashMap<FlowKey, Flow> {
    // TODO: Return a result from parse and handle errors in main instead of panicking
    let file = File::open(file_path).expect("Failed to open file");
    let mut num_blocks = 0;
    let mut reader = PcapNGReader::new(65536, file).expect("Failed to create reader");
    let mut flows: HashMap<FlowKey, Flow> = HashMap::new();
    let mut interfaces: Vec<InterfaceDescription> = Vec::new();

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                num_blocks += 1;
                match block {
                    PcapBlockOwned::NG(Block::SectionHeader(ref _shb)) => {
                        // New section: reset interface tracking
                        interfaces.clear();
                    }
                    PcapBlockOwned::NG(Block::InterfaceDescription(idb)) => {
                        // Remember linktype and timestamp configuration for this interface (epb.if_id maps to index here)
                        interfaces.push(InterfaceDescription {
                            linktype: idb.linktype,
                            ts_resolution: idb.if_tsresol,
                            ts_offset: idb.if_tsoffset,
                        });
                        println!(
                            "Interface #{}: linktype={:?}, name={:?}",
                            interfaces.len() - 1,
                            idb.linktype,
                            idb.if_name()
                        );
                    }
                    PcapBlockOwned::NG(Block::EnhancedPacket(ref epb)) => {
                        // Validate interface id and link type
                        let if_id = epb.if_id as usize;
                        if if_id >= interfaces.len() {
                            println!(
                                "Warning: EPB references unknown interface id {}, skipping",
                                if_id
                            );
                        } else {
                            let interface = &interfaces[if_id];
                            if interface.linktype == pcap_parser::Linktype::ETHERNET {
                                let epb_packet_data = epb.packet_data();
                                // Try parsing even if truncated; parser will error if too short
                                match etherparse::PacketHeaders::from_ethernet_slice(
                                    epb_packet_data,
                                ) {
                                    Ok(headers) => {
                                        // Build a flow only when IP + TCP/UDP present
                                        let mut flow = Flow::default();

                                        // Parse the timestamp from the packet using the interface data
                                        flow.timestamp = epb.decode_ts_f64(
                                            interface.ts_offset as u64,
                                            interface.ts_resolution as u64,
                                        );

                                        // Extract source and destination IPs
                                        match headers.net {
                                            Some(etherparse::NetHeaders::Ipv4(ipv4, _)) => {
                                                flow.src_ip = IPAddress::V4(ipv4.source);
                                                flow.dst_ip = IPAddress::V4(ipv4.destination);
                                            }
                                            Some(etherparse::NetHeaders::Ipv6(ipv6, _)) => {
                                                flow.src_ip = IPAddress::V6(ipv6.source);
                                                flow.dst_ip = IPAddress::V6(ipv6.destination);
                                            }
                                            _ => {
                                                // Non-IP
                                            }
                                        }

                                        // Extract source and destination ports
                                        match headers.transport {
                                            Some(etherparse::TransportHeader::Tcp(tcp)) => {
                                                flow.src_port = Some(tcp.source_port);
                                                flow.dst_port = Some(tcp.destination_port);
                                                flow.protocol = Protocol::TCP;
                                            }
                                            Some(etherparse::TransportHeader::Udp(udp)) => {
                                                flow.src_port = Some(udp.source_port);
                                                flow.dst_port = Some(udp.destination_port);
                                                flow.protocol = Protocol::UDP;
                                            }
                                            _ => {
                                                // Not TCP/UDP
                                            }
                                        }

                                        // Create a flow key from the flow data
                                        if let Some(key) = FlowKey::try_from_flow(&flow) {
                                            // Collect packet data per flow, creating a new flow if one does not exist for this 5-tuple
                                            flows
                                                .entry(key)
                                                .or_insert(flow)
                                                .packets
                                                .push(epb_packet_data.to_vec());
                                        }
                                    }
                                    Err(e) => {
                                        println!("Failed to parse packet: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    PcapBlockOwned::NG(Block::SimplePacket(_)) => {
                        println!("unsupported block type: 'SimplePacket'")
                    }
                    PcapBlockOwned::NG(Block::NameResolution(_)) => {
                        println!("unsupported block type: 'NameResolution'")
                    }
                    PcapBlockOwned::NG(Block::InterfaceStatistics(_)) => {
                        println!("unsupported block type: 'InterfaceStatistics'")
                    }
                    PcapBlockOwned::NG(Block::DecryptionSecrets(_)) => {
                        println!("unsupported block type: 'DecryptionSecrets'")
                    }
                    PcapBlockOwned::NG(Block::Custom(_)) => {
                        println!("unsupported block type: 'Custom'")
                    }
                    PcapBlockOwned::NG(Block::Unknown(_)) => {
                        println!("unsupported block type: 'Unknown'")
                    }
                    PcapBlockOwned::NG(Block::SystemdJournalExport(_)) => {
                        println!("unsupported block type: 'SystemdJournalExport'")
                    }
                    PcapBlockOwned::NG(Block::ProcessInformation(_)) => {
                        println!("unsupported block type: 'ProcessInformation'")
                    }
                    PcapBlockOwned::Legacy(_legacy_pcap_block) => {
                        println!("unsupported block type: 'Legacy'")
                    }
                    PcapBlockOwned::LegacyHeader(_pcap_header) => {
                        println!("unsupported block type: 'LegacyHeader'")
                    }
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader.refill().expect("Failed to refill reader");
            }
            Err(e) => eprintln!("Error while reading: {:?}", e),
        }
    }
    println!("Total blocks: {}", num_blocks);
    // Avoid dumping all packet bytes; print a concise summary instead.
    println!("Unique flows: {}", flows.len());
    // for (key, flow) in flows.iter().take(20) {
    //     let bytes: usize = flow.packets.iter().map(|p| p.len()).sum();
    //     println!(
    //         "{:?} -> packets={}, bytes={}",
    //         key,
    //         flow.packets.len(),
    //         bytes
    //     );
    // }
    // if flows.len() > 20 {
    //     println!("... {} more flows not shown", flows.len() - 20);
    // }

    return flows;
}
