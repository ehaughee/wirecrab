use crate::flow::*;
use gpui::*;
use gpui_component::table::{Column, ColumnSort, Table, TableDelegate, TableState};
use gpui_component::{ActiveTheme, Sizable, table};
use std::ops::Range;

#[derive(IntoElement, Clone)]
pub struct FlowTable {
    state: Entity<TableState<FlowTableDelegate>>,
}

impl FlowTable {
    pub fn create<Owner>(
        window: &mut Window,
        cx: &mut Context<Owner>,
        flows: Vec<(FlowKey, Flow)>,
        selected_flow: Option<FlowKey>,
        start_timestamp: Option<f64>,
    ) -> Self {
        let state =
            FlowTableDelegate::create_entity(window, cx, flows, selected_flow, start_timestamp);
        Self { state }
    }

    pub fn update<F, R>(&self, cx: &mut App, f: F) -> R
    where
        F: FnOnce(
            &mut TableState<FlowTableDelegate>,
            &mut Context<TableState<FlowTableDelegate>>,
        ) -> R,
    {
        self.state.update(cx, f)
    }

    pub fn entity(&self) -> &Entity<TableState<FlowTableDelegate>> {
        &self.state
    }
}

impl RenderOnce for FlowTable {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let table = Table::new(&self.state).bordered(false).xsmall();

        div()
            .size_full()
            .overflow_hidden()
            .rounded_none()
            .border_1()
            .border_color(cx.theme().colors.border)
            .child(table)
    }
}

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
                Column::new("timestamp", "Timestamp").width(110.).sortable(),
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
                ColumnSort::Ascending => self.flows.sort_by(|a, b| a.1.source.cmp(&b.1.source)),
                ColumnSort::Descending => self.flows.sort_by(|a, b| b.1.source.cmp(&a.1.source)),
                ColumnSort::Default => {}
            },
            "source_port" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.source.port.cmp(&b.1.source.port)),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.source.port.cmp(&a.1.source.port)),
                ColumnSort::Default => {}
            },
            "destination" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.destination.cmp(&b.1.destination)),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.destination.cmp(&a.1.destination)),
                ColumnSort::Default => {}
            },
            "destination_port" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.destination.port.cmp(&b.1.destination.port)),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.destination.port.cmp(&a.1.destination.port)),
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
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.total_bytes().cmp(&b.1.total_bytes())),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.total_bytes().cmp(&a.1.total_bytes())),
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
            "source" => flow.source.to_string(),
            "source_port" => flow.source.port.to_string(),
            "destination" => flow.destination.to_string(),
            "destination_port" => flow.destination.port.to_string(),
            "packets" => flow.packets.len().to_string(),
            "bytes" => flow.total_bytes().to_string(),
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
