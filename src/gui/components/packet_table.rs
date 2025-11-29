use crate::flow::{Flow, FlowKey, Packet};
use gpui::*;
use gpui_component::table::{Column, ColumnSort, Table, TableDelegate, TableState};
use gpui_component::tag::Tag;
use gpui_component::{ActiveTheme, ColorName, Sizable, StyledExt, h_flex};
use std::ops::Range;

#[derive(IntoElement, Clone)]
pub struct PacketTable {
    state: Entity<TableState<PacketTableDelegate>>,
    flow_key: Option<FlowKey>,
    packet_count: usize,
    last_start_timestamp: Option<f64>,
}

impl PacketTable {
    pub fn create<Owner>(
        window: &mut Window,
        cx: &mut Context<Owner>,
        flow: &Flow,
        start_timestamp: Option<f64>,
    ) -> Self {
        let state =
            PacketTableDelegate::create_entity(window, cx, Some(flow.clone()), start_timestamp);
        let flow_key = FlowKey::from_endpoints(flow.source, flow.destination, flow.protocol);
        Self {
            state,
            flow_key: Some(flow_key),
            packet_count: flow.packets.len(),
            last_start_timestamp: start_timestamp,
        }
    }

    pub fn update(&mut self, flow: &Flow, start_timestamp: Option<f64>, cx: &mut App) {
        let flow_key = FlowKey::from_endpoints(flow.source, flow.destination, flow.protocol);
        let packet_count = flow.packets.len();
        let needs_refresh = self.flow_key != Some(flow_key)
            || self.packet_count != packet_count
            || self.last_start_timestamp != start_timestamp;

        if !needs_refresh {
            return;
        }

        self.state.update(cx, move |table, cx| {
            let delegate = table.delegate_mut();
            delegate.set_flow(Some(&flow));
            delegate.set_start_timestamp(start_timestamp);
            table.refresh(cx);
        });

        self.flow_key = Some(flow_key);
        self.packet_count = packet_count;
        self.last_start_timestamp = start_timestamp;
    }

    pub fn entity(&self) -> &Entity<TableState<PacketTableDelegate>> {
        &self.state
    }

    pub fn pane_header(flow: &Flow, cx: &App) -> AnyElement {
        let flow_summary = flow.to_string();
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .text_color(cx.theme().colors.foreground)
                    .child("Flow Packets"),
            )
            .child(
                div()
                    .flex_grow()
                    .text_sm()
                    .text_color(cx.theme().colors.muted_foreground)
                    .child(flow_summary),
            )
            .into_any_element()
    }
}

impl RenderOnce for PacketTable {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let table = Table::new(&self.state).bordered(false).xsmall();

        div()
            .flex()
            .flex_col()
            .bg(cx.theme().colors.background)
            .border_t_1()
            .border_color(cx.theme().colors.border)
            .size_full()
            .child(
                div()
                    .size_full()
                    .overflow_hidden()
                    .rounded_none()
                    .border_1()
                    .border_color(cx.theme().colors.border)
                    .child(div().size_full().child(table)),
            )
    }
}

pub struct PacketTableDelegate {
    pub packets: Vec<Packet>,
    pub columns: Vec<Column>,
    pub active_sort: Option<(usize, ColumnSort)>,
    pub start_timestamp: Option<f64>,
}

impl PacketTableDelegate {
    pub fn new(flow: Option<&Flow>, start_timestamp: Option<f64>) -> Self {
        Self {
            packets: flow.map_or(vec![], |f| f.packets.clone()),
            columns: vec![
                make_packet_col("timestamp", "Timestamp", 110.),
                make_packet_col("src_ip", "Source IP", 150.),
                make_packet_col("src_port", "Src Port", 100.),
                make_packet_col("dst_ip", "Dest IP", 150.),
                make_packet_col("dst_port", "Dst Port", 100.),
                make_packet_col("size", "Size", 100.),
                make_packet_col("details", "Details", 300.),
            ],
            active_sort: Some((0, ColumnSort::Ascending)),
            start_timestamp,
        }
    }

    pub fn set_flow(&mut self, flow: Option<&Flow>) {
        self.packets = flow.map_or_else(Vec::new, |f| f.packets.clone());
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
            "size" => match sort {
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
        start_timestamp: Option<f64>,
    ) -> Entity<TableState<Self>> {
        cx.new(move |cx| {
            TableState::new(
                PacketTableDelegate::new(flow.as_ref(), start_timestamp),
                window,
                cx,
            )
        })
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
        cx: &mut App,
    ) -> impl IntoElement {
        let packet = &self.packets[row_ix];
        let col = &self.columns[col_ix];

        if col.key == "details" {
            return h_flex()
                .gap_1()
                .children(packet.tags.iter().map(|tag| {
                    Tag::color(get_tag_color(tag))
                        .with_size(px(12.0))
                        .px(px(1.0))
                        .py(px(1.0))
                        .text_size(px(12.0))
                        .text_color(cx.theme().colors.foreground)
                        .child(tag.clone())
                }))
                .into_any_element();
        }

        let content = match col.key.as_ref() {
            "timestamp" => {
                if let Some(start) = self.start_timestamp {
                    format!("{:.6}", packet.timestamp - start)
                } else {
                    format!("{:.6}", packet.timestamp)
                }
            }
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
            "size" => packet.length.to_string(),
            _ => String::new(),
        };

        div().child(content).into_any_element()
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

fn get_tag_color(tag: &str) -> ColorName {
    match tag {
        "ACK" => ColorName::Green,
        "SYN" => ColorName::Orange,
        "SYN-ACK" => ColorName::Blue,
        "FIN" => ColorName::Red,
        "RST" => ColorName::Pink,
        _ if tag.contains("TLS") => ColorName::Cyan,
        _ if tag.contains("Hello") => ColorName::Purple,
        _ => ColorName::Gray,
    }
}
