use crate::flow::{Flow, Packet};
use gpui::*;
use gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};
use std::ops::Range;

pub struct PacketTableDelegate {
    pub packets: Vec<Packet>,
    pub columns: Vec<Column>,
    pub active_sort: Option<(usize, ColumnSort)>,
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
            active_sort: None,
        }
    }

    pub fn set_flow(&mut self, flow: Option<&Flow>) {
        self.packets = flow.map_or_else(Vec::new, |f| f.packets.clone());
        if let Some((col_ix, sort)) = self.active_sort {
            self.sort_data(col_ix, sort);
        }
    }

    fn sort_data(&mut self, col_ix: usize, sort: ColumnSort) {
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
            "src_ip" => match sort {
                ColumnSort::Ascending => self.packets.sort_by(|a, b| a.src_ip.cmp(&b.src_ip)),
                ColumnSort::Descending => self.packets.sort_by(|a, b| b.src_ip.cmp(&a.src_ip)),
                ColumnSort::Default => {}
            },
            "dst_ip" => match sort {
                ColumnSort::Ascending => self.packets.sort_by(|a, b| a.dst_ip.cmp(&b.dst_ip)),
                ColumnSort::Descending => self.packets.sort_by(|a, b| b.dst_ip.cmp(&a.dst_ip)),
                ColumnSort::Default => {}
            },
            "src_port" => match sort {
                ColumnSort::Ascending => self.packets.sort_by(|a, b| a.src_port.cmp(&b.src_port)),
                ColumnSort::Descending => self.packets.sort_by(|a, b| b.src_port.cmp(&a.src_port)),
                ColumnSort::Default => {}
            },
            "dst_port" => match sort {
                ColumnSort::Ascending => self.packets.sort_by(|a, b| a.dst_port.cmp(&b.dst_port)),
                ColumnSort::Descending => self.packets.sort_by(|a, b| b.dst_port.cmp(&a.dst_port)),
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
        self.active_sort = Some((col_ix, sort));
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
