use wirecrab::gpui::*;
use wirecrab::gpui_component::{ActiveTheme, StyledExt};
use wirecrab::flow::{Flow, Endpoint, IPAddress, Protocol, Packet, FlowKey};
use wirecrab::gui::components::{
    FlowTable, PacketTable, PacketBytesView, SearchBar, Toolbar, 
    histogram_from_flows, render_histogram, ProtocolCategory
};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Introduction,
    FlowTable,
    PacketTable,
    PacketBytes,
    SearchBar,
    Toolbar,
    Histogram,
}

impl Page {
    fn label(&self) -> &'static str {
        match self {
            Page::Introduction => "Introduction",
            Page::FlowTable => "Flow Table",
            Page::PacketTable => "Packet Table",
            Page::PacketBytes => "Packet Bytes",
            Page::SearchBar => "Search Bar",
            Page::Toolbar => "Toolbar",
            Page::Histogram => "Histogram",
        }
    }

    fn all() -> Vec<Page> {
        vec![
            Page::Introduction,
            Page::FlowTable,
            Page::PacketTable,
            Page::PacketBytes,
            Page::SearchBar,
            Page::Toolbar,
            Page::Histogram,
        ]
    }
}

pub struct StoryView {
    active_page: Page,
    search_bar: SearchBar,
    flow_table: FlowTable,
    packet_table: PacketTable,
    packet_bytes_list: ListState,
    packet_bytes_data: Vec<u8>,
    flows: HashMap<FlowKey, Flow>,
    histogram_collapsed: bool,
}

impl StoryView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_bar = SearchBar::create(window, cx);
        
        // Mock Data
        let flows = generate_mock_flows();
        let flow_vec: Vec<(FlowKey, Flow)> = flows.iter().map(|(k, v)| (*k, v.clone())).collect();
        
        let flow_table = FlowTable::create(window, cx, flow_vec.clone(), None, None);
        
        let mock_flow = flow_vec.first().map(|(_, f)| f.clone()).unwrap_or_default();
        let packet_table = PacketTable::create(window, cx, &mock_flow, None);

        let packet_bytes_data = (0..512).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        let packet_bytes_list = PacketBytesView::create_list_state(&packet_bytes_data);

        Self {
            active_page: Page::Introduction,
            search_bar,
            flow_table,
            packet_table,
            packet_bytes_list,
            packet_bytes_data,
            flows,
            histogram_collapsed: false,
        }
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(200.0))
            .h_full()
            .bg(cx.theme().colors.secondary)
            .text_color(cx.theme().colors.foreground)
            .border_r_1()
            .border_color(cx.theme().colors.border)
            .child(
                div()
                    .p_4()
                    .text_lg()
                    .font_bold()
                    .child("Wirecrab Story")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .children(Page::all().into_iter().map(|page| {
                        let is_active = self.active_page == page;
                        div()
                            .id(page.label())
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().colors.secondary_hover))
                            .bg(if is_active { cx.theme().colors.secondary_active } else { wirecrab::gpui::transparent_black() })
                            .child(page.label())
                            .on_click(cx.listener(move |this, _, _window, _cx| {
                                this.active_page = page;
                            }))
                    }))
            )
    }

    fn render_content(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.active_page {
            Page::Introduction => div().child("Welcome to the Wirecrab Component Storybook."),
            Page::FlowTable => div().size_full().child(self.flow_table.clone()),
            Page::PacketTable => div().size_full().child(self.packet_table.clone()),
            Page::PacketBytes => div().size_full().border_1().border_color(cx.theme().colors.border).child(
                PacketBytesView::new(
                    Some(self.packet_bytes_list.clone()),
                    Some(self.packet_bytes_data.clone())
                )
            ),
            Page::SearchBar => div().p_4().child(self.search_bar.clone()),
            Page::Toolbar => div().p_4().child(
                Toolbar::new()
                    .left(div().child("Left Item"))
                    .center(div().child("Center Item"))
                    .right(div().child("Right Item"))
            ),
            Page::Histogram => {
                let flows_vec: Vec<(FlowKey, Flow)> = self.flows.iter().map(|(k, v)| (*k, v.clone())).collect();
                let buckets = histogram_from_flows(&flows_vec, None);
                let collapsed = self.histogram_collapsed;
                let on_toggle = cx.listener(|this, _, _window, _cx| {
                    this.histogram_collapsed = !this.histogram_collapsed;
                });
                let on_legend = |_: ProtocolCategory, _: &mut Window, _: &mut App| {};
                
                div().child(render_histogram(buckets, collapsed, on_toggle, on_legend, cx))
            }
        };

        div()
            .flex_1()
            .bg(cx.theme().colors.background)
            .text_color(cx.theme().colors.foreground)
            .child(content)
    }
}

impl Render for StoryView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .child(self.render_sidebar(cx))
            .child(self.render_content(cx))
    }
}

#[unsafe(no_mangle)]
pub fn create_story_view(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| StoryView::new(window, cx)).into()
}

// --- Mock Data Helpers ---

fn generate_mock_flows() -> HashMap<FlowKey, Flow> {
    let mut flows = HashMap::new();
    
    let ep1 = Endpoint { ip: IPAddress::V4([192, 168, 1, 10]), port: 443 };
    let ep2 = Endpoint { ip: IPAddress::V4([10, 0, 0, 5]), port: 12345 };
    
    let flow1 = Flow {
        timestamp: 1678886400.0,
        protocol: Protocol::TCP,
        source: ep1,
        destination: ep2,
        packets: generate_mock_packets(50),
    };
    
    let key1 = FlowKey::from_endpoints(ep1, ep2, Protocol::TCP);
    flows.insert(key1, flow1);

    let ep3 = Endpoint { ip: IPAddress::V4([8, 8, 8, 8]), port: 53 };
    let ep4 = Endpoint { ip: IPAddress::V4([192, 168, 1, 10]), port: 54321 };
    
    let flow2 = Flow {
        timestamp: 1678886405.0,
        protocol: Protocol::UDP,
        source: ep3,
        destination: ep4,
        packets: generate_mock_packets(10),
    };
    
    let key2 = FlowKey::from_endpoints(ep3, ep4, Protocol::UDP);
    flows.insert(key2, flow2);

    flows
}

fn generate_mock_packets(count: usize) -> Vec<Packet> {
    (0..count).map(|i| {
        Packet {
            timestamp: 1678886400.0 + (i as f64 * 0.1),
            src_ip: IPAddress::V4([192, 168, 1, 10]),
            dst_ip: IPAddress::V4([10, 0, 0, 5]),
            src_port: Some(443),
            dst_port: Some(12345),
            length: 64 + (i % 1000) as u16,
            data: (0..64).map(|b| (b % 255) as u8).collect(),
        }
    }).collect()
}
