use crate::flow::*;
use crate::layers::{LayerType, PacketContext, ParseResult, ParserRegistry};
use anyhow::{Context, Result};
use pcap_parser::traits::{PcapNGPacketBlock, PcapReaderIterator};
use pcap_parser::*;
use std::collections::HashMap;
use std::fs::File;

struct InterfaceDescription {
    linktype: Linktype,
    ts_resolution: u8,
    ts_offset: i64,
}

pub fn parse_pcap<F>(
    file_path: &std::path::Path,
    on_progress: F,
) -> Result<(HashMap<FlowKey, Flow>, Option<f64>)>
where
    F: Fn(f32),
{
    let file = File::open(file_path).context("Failed to open file")?;
    let file_size = file.metadata()?.len();
    let mut reader = PcapNGReader::new(65536, file)
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to create reader")?;
    let mut flows: HashMap<FlowKey, Flow> = HashMap::new();
    let mut interfaces: Vec<InterfaceDescription> = Vec::new();
    let registry = ParserRegistry::new();
    let mut bytes_read = 0;
    let mut last_progress_update = 0;
    let mut first_packet_ts: Option<f64> = None;

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                bytes_read += offset;
                // Report progress every ~1KB
                if bytes_read - last_progress_update > 1_000 {
                    on_progress(bytes_read as f32 / file_size as f32);
                    last_progress_update = bytes_read;
                }
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
                                let timestamp = parse_timestamp(epb, interface);

                                if first_packet_ts.is_none() {
                                    first_packet_ts = Some(timestamp);
                                } else if let Some(ts) = first_packet_ts {
                                    if timestamp < ts {
                                        first_packet_ts = Some(timestamp);
                                    }
                                }

                                let mut context = PacketContext::default();
                                let mut current_layer = LayerType::Ethernet;
                                let mut current_data = epb_packet_data;

                                loop {
                                    if let Some(parser) = registry.get(current_layer) {
                                        match parser.parse(&current_data, &mut context) {
                                            ParseResult::NextLayer {
                                                next_layer,
                                                payload,
                                            } => {
                                                current_layer = next_layer;
                                                current_data = payload;
                                            }
                                            ParseResult::Final => break,
                                            ParseResult::Error(_) => break,
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                if let (
                                    Some(src_ip),
                                    Some(dst_ip),
                                    Some(src_port),
                                    Some(dst_port),
                                    Some(protocol),
                                ) = (
                                    context.src_ip,
                                    context.dst_ip,
                                    context.src_port,
                                    context.dst_port,
                                    context.protocol,
                                ) {
                                    let src_ep = Endpoint::new(src_ip, src_port);
                                    let dst_ep = Endpoint::new(dst_ip, dst_port);
                                    let key = FlowKey::from_endpoints(src_ep, dst_ep, protocol);

                                    // Collect packet data per flow, creating a new flow if one does not exist for this 5-tuple
                                    let packet_length =
                                        u16::try_from(epb_packet_data.len()).unwrap_or(u16::MAX);
                                    let packet = Packet {
                                        timestamp,
                                        src_ip,
                                        dst_ip,
                                        src_port: Some(src_port),
                                        dst_port: Some(dst_port),
                                        length: packet_length,
                                        data: epb_packet_data.to_vec(),
                                    };

                                    let flow = flows.entry(key).or_insert_with(|| Flow {
                                        timestamp,
                                        protocol,
                                        source: src_ep,
                                        destination: dst_ep,
                                        packets: Vec::new(),
                                    });

                                    if protocol == Protocol::TCP
                                        && context.is_syn
                                        && !context.is_ack
                                    {
                                        flow.source = src_ep;
                                        flow.destination = dst_ep;
                                    }

                                    flow.packets.push(packet);
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
    return Ok((flows, first_packet_ts));
}

fn calculate_ts_unit(resolution: u8) -> u64 {
    if resolution & 0x80 != 0 {
        // Base 2 (High bit set)
        2u64.pow((resolution & 0x7F) as u32)
    } else {
        // Base 10
        10u64.pow(resolution as u32)
    }
}

fn parse_timestamp(
    epb: &pcap_parser::pcapng::EnhancedPacketBlock,
    interface: &InterfaceDescription,
) -> f64 {
    let unit = calculate_ts_unit(interface.ts_resolution);
    epb.decode_ts_f64(interface.ts_offset as u64, unit)
}
