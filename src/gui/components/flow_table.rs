use crate::flow::*;
use gpui::*;
use gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};
use std::ops::Range;

pub struct FlowTableDelegate {
    pub flows: Vec<(FlowKey, Flow)>,
    pub selected_flow: Option<FlowKey>,
    pub columns: Vec<Column>,
}

impl FlowTableDelegate {
    pub fn new(flows: Vec<(FlowKey, Flow)>, selected_flow: Option<FlowKey>) -> Self {
        Self {
            flows,
            selected_flow,
            columns: vec![
                Column::new("protocol", "Protocol").width(100.).sortable(),
                Column::new("src_ip", "Source IP").width(150.).sortable(),
                Column::new("dst_ip", "Dest IP").width(150.).sortable(),
                Column::new("packets", "Packets").width(100.).sortable(),
                Column::new("bytes", "Bytes").width(120.).sortable(),
            ],
        }
    }

    pub fn create_entity<Owner>(
        window: &mut Window,
        cx: &mut Context<Owner>,
        flows: Vec<(FlowKey, Flow)>,
        selected_flow: Option<FlowKey>,
    ) -> Entity<TableState<Self>> {
        cx.new(move |cx| TableState::new(FlowTableDelegate::new(flows, selected_flow), window, cx))
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
            "protocol" => format!("{:?}", flow.protocol),
            "src_ip" => flow.src_ip.to_string(),
            "dst_ip" => flow.dst_ip.to_string(),
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
        let col = &self.columns[col_ix];

        match col.key.as_ref() {
            "protocol" => match sort {
                ColumnSort::Ascending => self.flows.sort_by(|a, b| {
                    format!("{:?}", a.1.protocol).cmp(&format!("{:?}", b.1.protocol))
                }),
                ColumnSort::Descending => self.flows.sort_by(|a, b| {
                    format!("{:?}", b.1.protocol).cmp(&format!("{:?}", a.1.protocol))
                }),
                ColumnSort::Default => {}
            },
            "src_ip" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.src_ip.to_string().cmp(&b.1.src_ip.to_string())),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.src_ip.to_string().cmp(&a.1.src_ip.to_string())),
                ColumnSort::Default => {}
            },
            "dst_ip" => match sort {
                ColumnSort::Ascending => self
                    .flows
                    .sort_by(|a, b| a.1.dst_ip.to_string().cmp(&b.1.dst_ip.to_string())),
                ColumnSort::Descending => self
                    .flows
                    .sort_by(|a, b| b.1.dst_ip.to_string().cmp(&a.1.dst_ip.to_string())),
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

    fn visible_rows_changed(
        &mut self,
        _visible_range: Range<usize>,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        // Optional: can be used for lazy loading or other optimizations
    }
}
