use crate::flow::{Flow, Packet};
use gpui::*;
use gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};
use std::ops::Range;

pub struct PacketTableDelegate {
    pub packets: Vec<Packet>,
    pub columns: Vec<Column>,
}

impl PacketTableDelegate {
    pub fn new(flow: Option<&Flow>) -> Self {
        Self {
            packets: flow.map_or(vec![], |f| f.packets.clone()),
            columns: vec![
                make_packet_col("timestamp", "Timestamp", 120.),
                make_packet_col("src_ip", "Source IP", 150.),
                make_packet_col("dst_ip", "Dest IP", 150.),
                make_packet_col("src_port", "Src Port", 100.),
                make_packet_col("dst_port", "Dst Port", 100.),
                make_packet_col("length", "Length", 100.),
            ],
        }
    }

    pub fn set_flow(&mut self, flow: Option<&Flow>) {
        self.packets = flow.map_or_else(Vec::new, |f| f.packets.clone());
    }

    pub fn create_entity<Owner>(
        window: &mut Window,
        cx: &mut Context<Owner>,
        flow: Option<Flow>,
    ) -> Entity<TableState<Self>> {
        cx.new(move |cx| TableState::new(PacketTableDelegate::new(flow.as_ref()), window, cx))
    }
}

impl TableDelegate for PacketTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.packets.len()
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
        let packet = &self.packets[row_ix];
        let col = &self.columns[col_ix];

        let content = match col.key.as_ref() {
            "timestamp" => format!("{:.6}", packet.timestamp),
            "src_ip" => packet.src_ip.to_string(),
            "dst_ip" => packet.dst_ip.to_string(),
            "src_port" => packet
                .src_port
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
            "dst_port" => packet
                .dst_port
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
            "length" => packet.length.to_string(),
            _ => String::new(),
        };

        div().child(content)
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
            "timestamp" => match sort {
                ColumnSort::Ascending => self
                    .packets
                    .sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap()),
                ColumnSort::Descending => self
                    .packets
                    .sort_by(|a, b| b.timestamp.partial_cmp(&a.timestamp).unwrap()),
                ColumnSort::Default => {}
            },
            "length" => match sort {
                ColumnSort::Ascending => self.packets.sort_by(|a, b| a.length.cmp(&b.length)),
                ColumnSort::Descending => self.packets.sort_by(|a, b| b.length.cmp(&a.length)),
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

fn make_packet_col(
    key: impl Into<SharedString>,
    name: impl Into<SharedString>,
    width: impl Into<Pixels>,
) -> Column {
    Column::new(key, name)
        .width(width)
        .sortable()
        .movable(false)
        .resizable(true)
}
