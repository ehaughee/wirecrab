use ratatui::{
    layout::Constraint,
    style::Style,
    widgets::{Cell, Row},
};
use std::collections::{HashMap, HashSet};

use crate::flow::filter::{FlowFilter, FlowFormatter};
use crate::flow::{Flow, FlowKey};
use crate::tui::theme::flexoki;
use crate::tui::to_color;

pub struct PacketTableState {
    pub expanded_flows: HashSet<FlowKey>,
    flow_order: Vec<FlowKey>,
    flows: HashMap<FlowKey, Flow>,
    row_to_flow_map: Vec<Option<FlowKey>>, // Maps table row index to flow key
    start_timestamp: Option<f64>,
}

impl PacketTableState {
    pub fn new(flows: HashMap<FlowKey, Flow>, start_timestamp: Option<f64>) -> Self {
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
            start_timestamp,
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

    pub fn get_filtered_table_data(&'_ mut self, filter: &str) -> (Vec<Row<'_>>, Vec<Constraint>) {
        let mut rows = Vec::new();
        let mut row_to_flow_map = Vec::new();
        let flow_filter = FlowFilter::new(filter, self.start_timestamp, false, None);
        let timestamp_origin = flow_filter.timestamp_origin();

        for flow_key in self.flow_order.clone() {
            if let Some(flow) = self.flows.get(&flow_key) {
                if !flow_filter.matches_flow(flow) {
                    continue;
                }

                let timestamp_str = FlowFormatter::timestamp(flow.timestamp, timestamp_origin);
                let endpoint_a_ip = FlowFormatter::ip_address(&flow.source.ip, false, None);
                let endpoint_b_ip = FlowFormatter::ip_address(&flow.destination.ip, false, None);
                let endpoint_a_port = FlowFormatter::port(flow.source.port);
                let endpoint_b_port = FlowFormatter::port(flow.destination.port);
                let protocol_str = FlowFormatter::protocol(&flow.protocol);
                let total_bytes = flow.total_bytes();

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
                row_to_flow_map.push(Some(flow_key));

                if self.expanded_flows.contains(&flow_key) {
                    for packet in &flow.packets {
                        let packet_row = Row::new(vec![
                            Cell::from(format!(
                                "  {}",
                                FlowFormatter::timestamp(packet.timestamp, timestamp_origin)
                            )),
                            Cell::from(FlowFormatter::ip_address(&packet.src_ip, false, None)),
                            Cell::from(
                                packet.src_port.map(FlowFormatter::port).unwrap_or_default(),
                            ),
                            Cell::from(FlowFormatter::ip_address(&packet.dst_ip, false, None)),
                            Cell::from(
                                packet.dst_port.map(FlowFormatter::port).unwrap_or_default(),
                            ),
                            Cell::from(""),
                            Cell::from(""),
                            Cell::from(packet.length.to_string()),
                        ])
                        .style(Style::default().fg(to_color(flexoki::BASE_500)));
                        rows.push(packet_row);
                        row_to_flow_map.push(Some(flow_key));
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
