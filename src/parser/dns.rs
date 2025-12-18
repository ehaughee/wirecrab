use crate::flow::IPAddress;
use pcap_parser::pcapng::{NameRecordType, NameResolutionBlock};
use std::collections::HashMap;

pub fn handle_name_resolution(
    nrb: &NameResolutionBlock,
    name_resolutions: &mut HashMap<IPAddress, Vec<String>>,
) {
    for record in &nrb.nr {
        match record.record_type {
            NameRecordType::Ipv4 => {
                if let Some((ip, name)) = parse_name_record_value(record.record_value, 4) {
                    add_name_resolution(ip, name, name_resolutions);
                }
            }
            NameRecordType::Ipv6 => {
                if let Some((ip, name)) = parse_name_record_value(record.record_value, 16) {
                    add_name_resolution(ip, name, name_resolutions);
                }
            }
            _ => {}
        }
    }
}

fn parse_name_record_value(record_value: &[u8], ip_len: usize) -> Option<(IPAddress, String)> {
    if record_value.len() < ip_len + 1 {
        return None;
    }

    let (ip_bytes, rest) = record_value.split_at(ip_len);
    let name_bytes = rest.split(|b| *b == 0).next().unwrap_or(&[]);
    if name_bytes.is_empty() {
        return None;
    }

    let name = String::from_utf8_lossy(name_bytes).trim().to_string();
    if name.is_empty() {
        return None;
    }

    let ip = match ip_len {
        4 => IPAddress::V4(ip_bytes.try_into().ok()?),
        16 => IPAddress::V6(ip_bytes.try_into().ok()?),
        _ => return None,
    };

    Some((ip, name))
}

fn add_name_resolution(
    ip: IPAddress,
    name: String,
    name_resolutions: &mut HashMap<IPAddress, Vec<String>>,
) {
    let entry = name_resolutions.entry(ip).or_default();
    if !entry.iter().any(|existing| existing == &name) {
        entry.push(name);
    }
}
