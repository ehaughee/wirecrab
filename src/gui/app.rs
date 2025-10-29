use crate::flow::*;
use crate::gui::widgets::{flow_table, helpers, packet_table, search_bar};
use iced::{
    Element, Length, Task, Theme,
    widget::{column, container, pane_grid, text},
};
use std::collections::HashMap;

// Content for pane grid
#[derive(Debug, Clone)]
pub enum PaneContent {
    FlowsTable,
    PacketDetails,
}

// Application state
#[derive(Debug)]
pub struct WirecrabApp {
    flows: HashMap<FlowKey, Flow>,
    search: String,
    filtered_flows: Vec<(FlowKey, Flow)>,
    selected_flow: Option<FlowKey>,
    panes: pane_grid::State<PaneContent>,
}

// Messages that the application can handle
#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    FlowSelected(FlowKey),
    PaneResized(pane_grid::ResizeEvent),
    ClosePacketView,
}

impl WirecrabApp {
    fn with_flows(flows: HashMap<FlowKey, Flow>) -> (Self, Task<Message>) {
        let (panes, _) = pane_grid::State::new(PaneContent::FlowsTable);

        let mut app = WirecrabApp {
            flows,
            search: String::new(),
            filtered_flows: Vec::new(),
            selected_flow: None,
            panes,
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
            Message::FlowSelected(flow_key) => {
                self.selected_flow = Some(flow_key);
                // Create horizontal split when a flow is selected
                if self.panes.panes.len() == 1 {
                    let first_pane = self.panes.panes.iter().next().map(|(id, _)| *id).unwrap();
                    let _ = self.panes.split(
                        pane_grid::Axis::Horizontal,
                        first_pane,
                        PaneContent::PacketDetails,
                    );
                }
            }
            Message::ClosePacketView => {
                // Remove the packet details pane and clear selection
                self.selected_flow = None;
                // Reset panes to just the flows table
                self.panes = pane_grid::State::new(PaneContent::FlowsTable).0;
            }
            Message::PaneResized(resize_event) => {
                self.panes.resize(resize_event.split, resize_event.ratio);
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
            // Show search bar and pane grid with flows and packet details
            let title = text(format!(
                "Wirecrab: {} flows loaded ({} shown)",
                flow_count, filtered_count
            ))
            .size(20);

            let search = search_bar::search_bar(
                "Search flows (IP, port, protocol)...",
                &self.search,
                Message::SearchChanged,
            );

            let pane_grid = pane_grid::PaneGrid::new(&self.panes, |_id, pane, _is_maximized| {
                let content = match pane {
                    PaneContent::FlowsTable => flow_table::flow_table(
                        &self.filtered_flows,
                        self.selected_flow,
                        Message::FlowSelected,
                    ),
                    PaneContent::PacketDetails => {
                        packet_table::packet_table(self.selected_flow.as_ref(), &self.flows, || {
                            Message::ClosePacketView
                        })
                    }
                };
                pane_grid::Content::new(content)
            })
            .on_resize(10, Message::PaneResized);

            column![title, search, pane_grid]
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
                    let src_ip = helpers::format_ip_address(&flow.src_ip).to_lowercase();
                    let dst_ip = helpers::format_ip_address(&flow.dst_ip).to_lowercase();
                    let protocol = helpers::format_protocol(&flow.protocol).to_lowercase();

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
}

pub fn run_ui(initial_flows: HashMap<FlowKey, Flow>) -> Result<(), Box<dyn std::error::Error>> {
    // Create an initialization function that captures the flows
    let init_fn = move || WirecrabApp::with_flows(initial_flows);

    iced::application("Wirecrab", WirecrabApp::update, WirecrabApp::view)
        .theme(|_state| Theme::Dark)
        .run_with(init_fn)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
