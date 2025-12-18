use crate::flow::{Flow, FlowKey};
use crate::flow::IPAddress;
use std::collections::HashMap;

#[derive(Default)]
pub struct ParseState {
    pub flows: HashMap<FlowKey, Flow>,
    pub first_packet_ts: Option<f64>,
    pub packet_count: usize,
    pub name_resolutions: HashMap<IPAddress, Vec<String>>,
}

pub fn update_first_timestamp(first_packet_ts: &mut Option<f64>, timestamp: f64) {
    match first_packet_ts {
        None => *first_packet_ts = Some(timestamp),
        Some(current) if timestamp < *current => *first_packet_ts = Some(timestamp),
        _ => {}
    }
}