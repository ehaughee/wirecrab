use super::decoder::decode_headers;
use super::{dns, packets, state};
use crate::flow::{Flow, FlowKey};
use crate::layers::tls::TlsParser;
use anyhow::{Context, Result};
use pcap_parser::pcapng::EnhancedPacketBlock;
use pcap_parser::traits::{PcapNGPacketBlock, PcapReaderIterator};
use pcap_parser::*;
use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;
use tracing::{debug, error, info, warn};

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
    info!(path = ?file_path, size_bytes = file_size, "Starting PCAP parse");
    let mut reader = PcapNGReader::new(65536, file)
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to create reader")?;
    let mut state = state::ParseState::default();
    let mut interfaces: Vec<InterfaceDescription> = Vec::new();
    let mut bytes_read = 0;
    let mut last_progress_update = 0;
    let start_time = Instant::now();
    let tls_parser = TlsParser;

    loop {
        match reader.next() {
            Ok((offset, block)) => {
                bytes_read += offset;
                if bytes_read - last_progress_update > 1_000 {
                    on_progress(bytes_read as f32 / file_size as f32);
                    last_progress_update = bytes_read;
                }
                match block {
                    PcapBlockOwned::NG(Block::SectionHeader(_)) => {
                        debug!("Encountered SectionHeader; clearing interface descriptions");
                        interfaces.clear();
                    }
                    PcapBlockOwned::NG(Block::InterfaceDescription(idb)) => {
                        interfaces.push(InterfaceDescription {
                            linktype: idb.linktype,
                            ts_resolution: idb.if_tsresol,
                            ts_offset: idb.if_tsoffset,
                        });
                        debug!(
                            if_id = interfaces.len() - 1,
                            "Registered interface description"
                        );
                    }
                    PcapBlockOwned::NG(Block::EnhancedPacket(ref epb)) => {
                        let if_id = epb.if_id as usize;
                        if if_id >= interfaces.len() {
                            warn!(
                                if_id = if_id,
                                "EPB references unknown interface; skipping packet"
                            );
                        } else {
                            let interface = &interfaces[if_id];
                            if interface.linktype == pcap_parser::Linktype::ETHERNET {
                                let epb_packet_data = epb.packet_data();
                                handle_enhanced_packet(
                                    epb,
                                    interface,
                                    &tls_parser,
                                    epb_packet_data,
                                    &mut state,
                                );
                            }
                        }
                    }
                    PcapBlockOwned::NG(Block::SimplePacket(_)) => {
                        debug!("Unsupported block type: SimplePacket")
                    }
                    PcapBlockOwned::NG(Block::NameResolution(nrb)) => {
                        dns::handle_name_resolution(&nrb, &mut state.name_resolutions);
                    }
                    PcapBlockOwned::NG(Block::InterfaceStatistics(_)) => {
                        debug!("Unsupported block type: InterfaceStatistics")
                    }
                    PcapBlockOwned::NG(Block::DecryptionSecrets(_)) => {
                        debug!("Unsupported block type: DecryptionSecrets")
                    }
                    PcapBlockOwned::NG(Block::Custom(_)) => {
                        debug!("Unsupported block type: Custom")
                    }
                    PcapBlockOwned::NG(Block::Unknown(_)) => {
                        debug!("Unsupported block type: Unknown")
                    }
                    PcapBlockOwned::NG(Block::SystemdJournalExport(_)) => {
                        debug!("Unsupported block type: SystemdJournalExport")
                    }
                    PcapBlockOwned::NG(Block::ProcessInformation(_)) => {
                        debug!("Unsupported block type: ProcessInformation")
                    }
                    PcapBlockOwned::Legacy(_legacy_pcap_block) => {
                        debug!("Unsupported block type: Legacy")
                    }
                    PcapBlockOwned::LegacyHeader(_pcap_header) => {
                        debug!("Unsupported block type: LegacyHeader")
                    }
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader.refill().expect("Failed to refill reader");
            }
            Err(e) => error!(error = ?e, "Error while reading packet data"),
        }
    }
    let elapsed = start_time.elapsed();
    info!(
        path = ?file_path,
        packets = state.packet_count,
        flows = state.flows.len(),
        elapsed_ms = elapsed.as_millis(),
        "Completed PCAP parse"
    );
    Ok((state.flows, state.first_packet_ts))
}

fn calculate_ts_unit(resolution: u8) -> u64 {
    if resolution & 0x80 != 0 {
        2u64.pow((resolution & 0x7F) as u32)
    } else {
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

fn handle_enhanced_packet(
    epb: &EnhancedPacketBlock,
    interface: &InterfaceDescription,
    tls_parser: &TlsParser,
    epb_packet_data: &[u8],
    state: &mut state::ParseState,
) {
    let timestamp = parse_timestamp(epb, interface);
    state::update_first_timestamp(&mut state.first_packet_ts, timestamp);

    if let Ok(context) = decode_headers(epb_packet_data, tls_parser) {
        dns::handle_dns_response(&context, &mut state.name_resolutions);
        packets::add_packet(
            epb_packet_data,
            context,
            timestamp,
            &mut state.flows,
            &mut state.packet_count,
        );
    }
}
