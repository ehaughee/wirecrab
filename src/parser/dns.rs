use crate::flow::{IPAddress, Protocol};
use crate::layers::PacketContext;
use pcap_parser::pcapng::{NameRecordType, NameResolutionBlock};
use std::collections::HashMap;
use tracing::debug;

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

pub fn handle_dns_response(
    context: &PacketContext,
    name_resolutions: &mut HashMap<IPAddress, Vec<String>>,
) {
    let payload = match (&context.protocol, &context.udp_payload) {
        (Some(Protocol::UDP), Some(data)) => data.as_slice(),
        _ => return,
    };

    let from_dns_port = matches!(context.src_port, Some(53)) || matches!(context.dst_port, Some(53));
    if !from_dns_port {
        return;
    }

    for (ip, name) in parse_dns_answers(payload) {
        add_name_resolution(ip, name, name_resolutions);
    }
}

fn parse_dns_answers(payload: &[u8]) -> Vec<(IPAddress, String)> {
    if payload.len() < 12 {
        return Vec::new();
    }

    let flags = u16::from_be_bytes([payload[2], payload[3]]);
    let is_response = flags & 0x8000 != 0;
    if !is_response {
        return Vec::new();
    }

    let qdcount = u16::from_be_bytes([payload[4], payload[5]]) as usize;
    let ancount = u16::from_be_bytes([payload[6], payload[7]]) as usize;

    let mut offset = 12usize;

    for _ in 0..qdcount {
        if let Some((_, next)) = read_dns_name(payload, offset) {
            offset = next;
        } else {
            return Vec::new();
        }

        if offset + 4 > payload.len() {
            return Vec::new();
        }
        offset += 4; // type + class
    }

    let mut results = Vec::new();
    for _ in 0..ancount {
        let (name, next) = match read_dns_name(payload, offset) {
            Some(n) => n,
            None => break,
        };
        offset = next;

        if offset + 10 > payload.len() {
            break;
        }

        let rtype = u16::from_be_bytes([payload[offset], payload[offset + 1]]);
        let class = u16::from_be_bytes([payload[offset + 2], payload[offset + 3]]);
        let rdlength = u16::from_be_bytes([payload[offset + 8], payload[offset + 9]]) as usize;
        offset += 10;

        if offset + rdlength > payload.len() {
            break;
        }

        let rdata = &payload[offset..offset + rdlength];
        if class == 1 {
            match (rtype, rdlength) {
                (1, 4) => results.push((IPAddress::V4(rdata.try_into().unwrap()), name.clone())),
                (28, 16) => results.push((IPAddress::V6(rdata.try_into().unwrap()), name.clone())),
                _ => {}
            }
        }

        offset += rdlength;
    }

    results
}

fn read_dns_name(packet: &[u8], start: usize) -> Option<(String, usize)> {
    let mut labels = Vec::new();
    let mut offset = start;
    let mut jumped = false;
    let mut next_offset = start;
    let mut depth = 0;

    loop {
        if offset >= packet.len() {
            return None;
        }

        let len = packet[offset];
        if len & 0xC0 == 0xC0 {
            if offset + 1 >= packet.len() {
                return None;
            }
            let pointer = (((len & 0x3F) as usize) << 8) | packet[offset + 1] as usize;
            if depth > 20 {
                return None;
            }
            depth += 1;
            if !jumped {
                next_offset = offset + 2;
                jumped = true;
            }
            offset = pointer;
            continue;
        }

        if len == 0 {
            offset += 1;
            if !jumped {
                next_offset = offset;
            }
            break;
        }

        offset += 1;
        if offset + len as usize > packet.len() {
            return None;
        }

        let label = &packet[offset..offset + len as usize];
        labels.push(String::from_utf8_lossy(label).into_owned());
        offset += len as usize;
        if !jumped {
            next_offset = offset;
        }
    }

    Some((labels.join("."), next_offset))
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
    let inserted = if !entry.iter().any(|existing| existing == &name) {
        entry.push(name);
        true
    } else {
        false
    };

    let names = entry.join(", ");
    debug!(ip = %ip, names = %names, inserted, "Name resolution updated");
}
