use crate::flow::*;
use iced::{
    Element, Length, Task, Theme,
    widget::{column, container, row, scrollable, text, text_input},
};
use std::collections::HashMap;

// Application state
#[derive(Debug, Default)]
pub struct WirecrabApp {
    flows: HashMap<FlowKey, Flow>,
    search: String,
    filtered_flows: Vec<(FlowKey, Flow)>,
}

// Messages that the application can handle
#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
}

impl WirecrabApp {
    fn new() -> (Self, Task<Message>) {
        let mut app = WirecrabApp {
            flows: HashMap::new(),
            search: String::new(),
            filtered_flows: Vec::new(),
        };
        app.update_filtered_flows();
        (app, Task::none())
    }

    fn with_flows(flows: HashMap<FlowKey, Flow>) -> (Self, Task<Message>) {
        let mut app = WirecrabApp {
            flows,
            search: String::new(),
            filtered_flows: Vec::new(),
        };
        app.update_filtered_flows();
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(value) => {
                self.search = value;
                self.update_filtered_flows();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let flow_count = self.flows.len();
        let filtered_count = self.filtered_flows.len();

        if flow_count == 0 {
            // Show empty state
            let content = text("No flows loaded").size(20);

            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            // Show search bar and flows table
            let title = text(format!(
                "Wirecrab: {} flows loaded ({} shown)",
                flow_count, filtered_count
            ))
            .size(20);

            let search_bar = text_input("Search flows (IP, port, protocol)...", &self.search)
                .on_input(Message::SearchChanged)
                .padding(10)
                .size(16);

            let flows_table = self.create_flows_table();

            column![title, search_bar, flows_table]
                .spacing(10)
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}

impl WirecrabApp {
    fn update_filtered_flows(&mut self) {
        self.filtered_flows = if self.search.is_empty() {
            self.flows.iter().map(|(k, v)| (*k, v.clone())).collect()
        } else {
            self.flows
                .iter()
                .filter(|(key, flow)| {
                    let search_lower = self.search.to_lowercase();
                    let flow_display = key.to_display().to_lowercase();
                    let src_ip = format_ip_address(&flow.src_ip).to_lowercase();
                    let dst_ip = format_ip_address(&flow.dst_ip).to_lowercase();
                    let protocol = format_protocol(&flow.protocol).to_lowercase();

                    flow_display.contains(&search_lower)
                        || src_ip.contains(&search_lower)
                        || dst_ip.contains(&search_lower)
                        || protocol.contains(&search_lower)
                })
                .map(|(k, v)| (*k, v.clone()))
                .collect()
        };

        // Sort by timestamp (oldest first)
        self.filtered_flows.sort_by(|a, b| {
            a.1.timestamp
                .partial_cmp(&b.1.timestamp)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn create_flows_table(&self) -> Element<Message> {
        // Create table header
        let header = row![
            text("Timestamp").width(Length::FillPortion(2)),
            text("Source IP").width(Length::FillPortion(2)),
            text("Src Port").width(Length::FillPortion(1)),
            text("Destination IP").width(Length::FillPortion(2)),
            text("Dst Port").width(Length::FillPortion(1)),
            text("Protocol").width(Length::FillPortion(1)),
            text("Packets").width(Length::FillPortion(1)),
            text("Bytes").width(Length::FillPortion(1)),
        ]
        .spacing(10)
        .padding(10);

        // Create table rows
        let mut rows = column![].spacing(2);

        // Add header with styling
        let styled_header = container(header).padding(5);

        rows = rows.push(styled_header);

        // Add data rows
        for (_flow_key, flow) in &self.filtered_flows {
            let timestamp_str = format!("{:.6}", flow.timestamp);
            let src_ip_str = format_ip_address(&flow.src_ip);
            let dst_ip_str = format_ip_address(&flow.dst_ip);
            let src_port_str = flow.src_port.map_or("N/A".to_string(), |p| p.to_string());
            let dst_port_str = flow.dst_port.map_or("N/A".to_string(), |p| p.to_string());
            let protocol_str = format_protocol(&flow.protocol);
            let packet_count = flow.packets.len();
            let byte_count: usize = flow.packets.iter().map(|p| p.len()).sum();

            let data_row = row![
                text(timestamp_str).width(Length::FillPortion(2)),
                text(src_ip_str).width(Length::FillPortion(2)),
                text(src_port_str).width(Length::FillPortion(1)),
                text(dst_ip_str).width(Length::FillPortion(2)),
                text(dst_port_str).width(Length::FillPortion(1)),
                text(protocol_str).width(Length::FillPortion(1)),
                text(packet_count.to_string()).width(Length::FillPortion(1)),
                text(byte_count.to_string()).width(Length::FillPortion(1)),
            ]
            .spacing(10)
            .padding(5);

            let styled_row = container(data_row).padding(2);

            rows = rows.push(styled_row);
        }

        // Wrap in scrollable container
        scrollable(rows)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub fn run_ui(initial_flows: HashMap<FlowKey, Flow>) -> Result<(), Box<dyn std::error::Error>> {
    // Create an initialization function that captures the flows
    let init_fn = move || WirecrabApp::with_flows(initial_flows);

    iced::application("Wirecrab", WirecrabApp::update, WirecrabApp::view)
        .theme(|_state| Theme::Dark)
        .run_with(init_fn)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
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
