use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Cell, Row},
};
use std::collections::{HashMap, HashSet};

use crate::flow::{Flow, FlowKey, IPAddress, Protocol};

pub struct PacketTableState {
    pub expanded_flows: HashSet<FlowKey>,
    flow_order: Vec<FlowKey>,
    flows: HashMap<FlowKey, Flow>,
    row_to_flow_map: Vec<Option<FlowKey>>, // Maps table row index to flow key
}

impl PacketTableState {
    pub fn new(flows: HashMap<FlowKey, Flow>) -> Self {
        let mut flow_order: Vec<FlowKey> = flows.keys().copied().collect();

        // Sort by timestamp (oldest first)
        flow_order.sort_unstable_by(|a, b| {
            let flow_a = flows.get(a);
            let flow_b = flows.get(b);
            match (flow_a, flow_b) {
                (Some(fa), Some(fb)) => fa.timestamp.total_cmp(&fb.timestamp),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        Self {
            expanded_flows: HashSet::new(),
            flow_order,
            flows,
            row_to_flow_map: Vec::new(),
        }
    }

    pub fn get_selected_flow_key(
        &self,
        table_state: &ratatui::widgets::TableState,
    ) -> Option<FlowKey> {
        table_state
            .selected()
            .and_then(|i| self.row_to_flow_map.get(i))
            .and_then(|flow_key_opt| *flow_key_opt)
    }

    pub fn toggle_selected_flow(&mut self, table_state: &ratatui::widgets::TableState) {
        if let Some(flow_key) = self.get_selected_flow_key(table_state) {
            if self.expanded_flows.contains(&flow_key) {
                self.expanded_flows.remove(&flow_key);
            } else {
                self.expanded_flows.insert(flow_key);
            }
        }
    }

    pub fn next_flow(&mut self, table_state: &mut ratatui::widgets::TableState) {
        let i = match table_state.selected() {
            Some(i) => {
                if i >= self.flow_order.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        table_state.select(Some(i));
    }

    pub fn previous_flow(&mut self, table_state: &mut ratatui::widgets::TableState) {
        let i = match table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.flow_order.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        table_state.select(Some(i));
    }

    pub fn get_filtered_table_data(&mut self, filter: &str) -> (Vec<Row>, Vec<Constraint>) {
        // Create mapping and rows separately to avoid borrowing conflicts
        let mut rows = Vec::new();
        let mut row_to_flow_map = Vec::new();
        let filter_lower = filter.to_lowercase();

        for flow_key in &self.flow_order.clone() {
            if let Some(flow) = self.flows.get(flow_key) {
                // Determine source (initiator) and destination
                let (src_endpoint, dst_endpoint) = if flow.endpoints.first == flow.initiator {
                    (flow.endpoints.first, flow.endpoints.second)
                } else {
                    (flow.endpoints.second, flow.endpoints.first)
                };

                // Check if this flow matches the filter
                let timestamp_str = format_timestamp(flow.timestamp);
                let endpoint_a_ip = format_ip_address(&src_endpoint.ip);
                let endpoint_b_ip = format_ip_address(&dst_endpoint.ip);
                let endpoint_a_port = src_endpoint.port.to_string();
                let endpoint_b_port = dst_endpoint.port.to_string();
                let protocol_str = format_protocol(&flow.protocol);

                // If filter is empty or any field contains the filter text, include this flow
                let matches_filter = filter.is_empty()
                    || timestamp_str.to_lowercase().contains(&filter_lower)
                    || endpoint_a_ip.to_lowercase().contains(&filter_lower)
                    || endpoint_b_ip.to_lowercase().contains(&filter_lower)
                    || endpoint_a_port.to_lowercase().contains(&filter_lower)
                    || endpoint_b_port.to_lowercase().contains(&filter_lower)
                    || protocol_str.to_lowercase().contains(&filter_lower);

                if matches_filter {
                    let total_bytes: u64 = flow.packets.iter().map(|p| p.length as u64).sum();

                    // Main flow row
                    let main_row = Row::new(vec![
                        Cell::from(timestamp_str),
                        Cell::from(endpoint_a_ip),
                        Cell::from(endpoint_a_port),
                        Cell::from(endpoint_b_ip),
                        Cell::from(endpoint_b_port),
                        Cell::from(protocol_str),
                        Cell::from(flow.packets.len().to_string()),
                        Cell::from(total_bytes.to_string()),
                    ]);

                    rows.push(main_row);
                    row_to_flow_map.push(Some(*flow_key)); // Main flow row maps to the flow

                    // If expanded, add packet detail rows
                    if self.expanded_flows.contains(flow_key) {
                        for (_i, packet) in flow.packets.iter().enumerate() {
                            let packet_row = Row::new(vec![
                                Cell::from(format!("  {}", format_timestamp(packet.timestamp))),
                                Cell::from(format_ip_address(&packet.src_ip)),
                                Cell::from(
                                    packet.src_port.map(|p| p.to_string()).unwrap_or_default(),
                                ),
                                Cell::from(format_ip_address(&packet.dst_ip)),
                                Cell::from(
                                    packet.dst_port.map(|p| p.to_string()).unwrap_or_default(),
                                ),
                                Cell::from(""),
                                Cell::from(""),
                                Cell::from(format!("{}", packet.length)),
                            ])
                            .style(Style::default().fg(Color::Gray));
                            rows.push(packet_row);
                            row_to_flow_map.push(Some(*flow_key)); // Packet detail rows also map to their parent flow
                        }
                    }
                }
            }
        }

        self.row_to_flow_map = row_to_flow_map;
        let widths = vec![
            Constraint::Length(20), // Timestamp
            Constraint::Length(15), // Endpoint A IP
            Constraint::Length(8),  // Endpoint A Port
            Constraint::Length(15), // Endpoint B IP
            Constraint::Length(8),  // Endpoint B Port
            Constraint::Length(8),  // Protocol
            Constraint::Length(8),  // Packets
            Constraint::Length(10), // Bytes
        ];
        (rows, widths)
    }
}

fn format_timestamp(timestamp: f64) -> String {
    // Convert to a readable format (you might want to use chrono for better formatting)
    format!("{:.6}", timestamp)
}

fn format_ip_address(ip: &IPAddress) -> String {
    match ip {
        IPAddress::V4(addr) => format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]),
        IPAddress::V6(addr) => {
            format!(
                "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                addr[0],
                addr[1],
                addr[2],
                addr[3],
                addr[4],
                addr[5],
                addr[6],
                addr[7],
                addr[8],
                addr[9],
                addr[10],
                addr[11],
                addr[12],
                addr[13],
                addr[14],
                addr[15]
            )
        }
    }
}

fn format_protocol(protocol: &Protocol) -> String {
    match protocol {
        Protocol::TCP => "TCP".to_string(),
        Protocol::UDP => "UDP".to_string(),
        Protocol::Other(n) => format!("Proto-{}", n),
    }
}
