use crate::flow::*;
use gpui::*;
use gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};
use std::ops::Range;

pub struct FlowTableDelegate {
    pub flows: Vec<(FlowKey, Flow)>,
    pub selected_flow: Option<FlowKey>,
    pub columns: Vec<Column>,
    pub active_sort: Option<(usize, ColumnSort)>,
    pub start_timestamp: Option<f64>,
}

impl FlowTableDelegate {
    pub fn new(
        flows: Vec<(FlowKey, Flow)>,
        selected_flow: Option<FlowKey>,
        start_timestamp: Option<f64>,
    ) -> Self {
        Self {
            flows,
            selected_flow,
            columns: vec![
                Column::new("timestamp", "Timestamp").width(100.).sortable(),
                Column::new("protocol", "Protocol").width(100.).sortable(),
                Column::new("source", "Source").width(170.).sortable(),
                Column::new("source_port", "Src Port")
                    .width(100.)
                    .sortable(),
                Column::new("destination", "Destination")
                    .width(170.)
                    .sortable(),
                Column::new("destination_port", "Dst Port")
                    .width(100.)
                    .sortable(),
                Column::new("packets", "Packets").width(100.).sortable(),
                Column::new("bytes", "Bytes").width(120.).sortable(),
            ],
            active_sort: Some((0, ColumnSort::Ascending)),
            start_timestamp,
        }
    }

    pub fn set_flows(&mut self, flows: Vec<(FlowKey, Flow)>) {
        self.flows = flows;
        if let Some((col_ix, sort)) = self.active_sort {
            self.sort_data(col_ix, sort);
        }
    }

    pub fn set_start_timestamp(&mut self, timestamp: Option<f64>) {
        self.start_timestamp = timestamp;
    }

    fn sort_data(&mut self, col_ix: usize, sort: ColumnSort) {
        let col = &self.columns[col_ix];

        match col.key.as_ref() {
            "timestamp" => match sort {
                ColumnSort::Ascending | ColumnSort::Default => self
                    .flows
                    .sort_by(|a, b| a.1.timestamp.partial_cmp(&b.1.timestamp).unwrap()),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.timestamp.partial_cmp(&a.1.timestamp).unwrap()),
            },
            "protocol" => match sort {
                ColumnSort::Ascending => self.flows.sort_by(|a, b| {
                    format!("{:?}", a.1.protocol).cmp(&format!("{:?}", b.1.protocol))
                }),
                ColumnSort::Descending => self.flows.sort_by(|a, b| {
                    format!("{:?}", b.1.protocol).cmp(&format!("{:?}", a.1.protocol))
                }),
                ColumnSort::Default => {}
            },
            "source" => match sort {
                ColumnSort::Ascending => {
                    self.flows.sort_by(|a, b| a.1.initiator.cmp(&b.1.initiator))
                }
                ColumnSort::Descending => {
                    self.flows.sort_by(|a, b| b.1.initiator.cmp(&a.1.initiator))
                }
                ColumnSort::Default => {}
            },
            "source_port" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.initiator.port.cmp(&b.1.initiator.port)),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.initiator.port.cmp(&a.1.initiator.port)),
                ColumnSort::Default => {}
            },
            "destination" => match sort {
                ColumnSort::Ascending => self.flows.sort_by(|a, b| {
                    let a_dst = if a.1.endpoints.first == a.1.initiator {
                        a.1.endpoints.second
                    } else {
                        a.1.endpoints.first
                    };
                    let b_dst = if b.1.endpoints.first == b.1.initiator {
                        b.1.endpoints.second
                    } else {
                        b.1.endpoints.first
                    };
                    a_dst.cmp(&b_dst)
                }),
                ColumnSort::Descending => self.flows.sort_by(|a, b| {
                    let a_dst = if a.1.endpoints.first == a.1.initiator {
                        a.1.endpoints.second
                    } else {
                        a.1.endpoints.first
                    };
                    let b_dst = if b.1.endpoints.first == b.1.initiator {
                        b.1.endpoints.second
                    } else {
                        b.1.endpoints.first
                    };
                    b_dst.cmp(&a_dst)
                }),
                ColumnSort::Default => {}
            },
            "packets" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.packets.len().cmp(&b.1.packets.len())),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.packets.len().cmp(&a.1.packets.len())),
                ColumnSort::Default => {}
            },
            "bytes" => match sort {
                ColumnSort::Ascending => self.flows.sort_by(|a, b| {
                    let a_bytes: u64 = a.1.packets.iter().map(|p| p.length as u64).sum();
                    let b_bytes: u64 = b.1.packets.iter().map(|p| p.length as u64).sum();
                    a_bytes.cmp(&b_bytes)
                }),
                ColumnSort::Descending => self.flows.sort_by(|a, b| {
                    let a_bytes: u64 = a.1.packets.iter().map(|p| p.length as u64).sum();
                    let b_bytes: u64 = b.1.packets.iter().map(|p| p.length as u64).sum();
                    b_bytes.cmp(&a_bytes)
                }),
                ColumnSort::Default => {}
            },
            _ => {}
        }
    }

    pub fn create_entity<Owner>(
        window: &mut Window,
        cx: &mut Context<Owner>,
        flows: Vec<(FlowKey, Flow)>,
        selected_flow: Option<FlowKey>,
        start_timestamp: Option<f64>,
    ) -> Entity<TableState<Self>> {
        cx.new(move |cx| {
            TableState::new(
                FlowTableDelegate::new(flows, selected_flow, start_timestamp),
                window,
                cx,
            )
        })
    }
}

impl TableDelegate for FlowTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.flows.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> &Column {
        &self.columns[col_ix]
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> impl IntoElement {
        let (_key, flow) = &self.flows[row_ix];
        let col = &self.columns[col_ix];

        let content = match col.key.as_ref() {
            "timestamp" => {
                if let Some(start) = self.start_timestamp {
                    format!("{:.6}", flow.timestamp - start)
                } else {
                    format!("{:.6}", flow.timestamp)
                }
            }
            "protocol" => format!("{:?}", flow.protocol),
            "source" => flow.initiator.to_string(),
            "source_port" => flow.initiator.port.to_string(),
            "destination" => {
                if flow.endpoints.first == flow.initiator {
                    flow.endpoints.second.to_string()
                } else {
                    flow.endpoints.first.to_string()
                }
            }
            "destination_port" => {
                let dst_endpoint = if flow.endpoints.first == flow.initiator {
                    flow.endpoints.second
                } else {
                    flow.endpoints.first
                };
                dst_endpoint.port.to_string()
            }
            "packets" => flow.packets.len().to_string(),
            "bytes" => {
                let total: u64 = flow.packets.iter().map(|p| p.length as u64).sum();
                total.to_string()
            }
            _ => String::new(),
        };

        div().child(content)
    }

    fn render_tr(&self, row_ix: usize, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        div().id(row_ix)
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        self.active_sort = Some((col_ix, sort));
        for (i, col) in self.columns.iter_mut().enumerate() {
            if i == col_ix {
                col.sort = Some(sort);
            } else {
                col.sort = Some(ColumnSort::Default);
            }
        }
        self.sort_data(col_ix, sort);
    }

    fn visible_rows_changed(
        &mut self,
        _visible_range: Range<usize>,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        // Optional: can be used for lazy loading or other optimizations
    }
}
