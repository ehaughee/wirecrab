use crate::flow::*;
use crate::gui::assets::Assets;
use crate::gui::components::{FlowTableDelegate, PacketTableDelegate, SearchBar};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{InputEvent, InputState};
use gpui_component::table::{Table, TableEvent, TableState};
use gpui_component::{IconName, Root};
use std::collections::HashMap;

const MIN_PACKET_PANE_HEIGHT: f32 = 160.0;
const DEFAULT_PACKET_PANE_HEIGHT: f32 = 320.0;
const MIN_FLOW_REGION_HEIGHT: f32 = 200.0;

struct ResizeDragState {
    start_height: f32,
    start_mouse_y: f32,
}

struct WirecrabApp {
    flows: HashMap<FlowKey, Flow>,
    selected_flow: Option<FlowKey>,
    search_input: Entity<InputState>,
    flow_table: Entity<TableState<FlowTableDelegate>>,
    packet_table: Option<Entity<TableState<PacketTableDelegate>>>,
    packet_pane_height: f32,
    resize_state: Option<ResizeDragState>,
}

impl WirecrabApp {
    fn new(flows: HashMap<FlowKey, Flow>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = SearchBar::create_state(window, cx);

        let initial_view: Vec<(FlowKey, Flow)> =
            flows.iter().map(|(k, v)| (*k, v.clone())).collect();

        let flow_table = FlowTableDelegate::create_entity(window, cx, initial_view.clone(), None);

        cx.subscribe_in(
            &search_input,
            window,
            |_view, _state, event, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(&flow_table, window, |view, table, event, _window, cx| {
            if let TableEvent::SelectRow(row_ix) = event {
                let table_state = table.read(cx);
                let delegate = table_state.delegate();
                if let Some((key, _flow)) = delegate.flows.get(*row_ix) {
                    view.select_flow(*key);
                    cx.notify();
                }
            }
        })
        .detach();

        Self {
            flows,
            selected_flow: None,
            search_input,
            flow_table,
            packet_table: None,
            packet_pane_height: DEFAULT_PACKET_PANE_HEIGHT,
            resize_state: None,
        }
    }

    fn select_flow(&mut self, flow_key: FlowKey) {
        self.selected_flow = Some(flow_key);
    }

    fn current_flow(&self) -> Option<&Flow> {
        self.selected_flow.and_then(|key| self.flows.get(&key))
    }

    fn filtered_flows(&self, cx: &App) -> Vec<(FlowKey, Flow)> {
        let search_text = self.search_input.read(cx).value().to_lowercase();

        if search_text.is_empty() {
            self.flows.iter().map(|(k, v)| (*k, v.clone())).collect()
        } else {
            self.flows
                .iter()
                .filter(|(_, flow)| {
                    let endpoints = [flow.endpoints.first, flow.endpoints.second];
                    endpoints
                        .iter()
                        .any(|endpoint| Self::endpoint_matches(*endpoint, &search_text))
                        || format!("{:?}", flow.protocol)
                            .to_lowercase()
                            .contains(&search_text)
                })
                .map(|(k, v)| (*k, v.clone()))
                .collect()
        }
    }

    fn endpoint_matches(endpoint: Endpoint, needle: &str) -> bool {
        endpoint.to_string().to_lowercase().contains(needle)
            || endpoint.port.to_string().contains(needle)
    }

    fn begin_resize(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }

        let mouse_y: f32 = event.position.y.into();
        self.resize_state = Some(ResizeDragState {
            start_height: self.packet_pane_height,
            start_mouse_y: mouse_y,
        });

        window.prevent_default();
        cx.notify();
    }

    fn update_resize(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(state) = &self.resize_state {
            if !event.dragging() {
                self.resize_state = None;
                return;
            }

            let mouse_y: f32 = event.position.y.into();
            let delta = state.start_mouse_y - mouse_y;
            let viewport_height: f32 = window.viewport_size().height.into();
            let max_height = (viewport_height - MIN_FLOW_REGION_HEIGHT).max(MIN_PACKET_PANE_HEIGHT);
            let new_height = (state.start_height + delta).clamp(MIN_PACKET_PANE_HEIGHT, max_height);

            if (new_height - self.packet_pane_height).abs() > 0.5 {
                self.packet_pane_height = new_height;
                cx.notify();
            }
        }
    }

    fn end_resize(&mut self, event: &MouseUpEvent) {
        if event.button == MouseButton::Left {
            self.resize_state = None;
        }
    }

    fn sync_packet_table(
        &mut self,
        selected_flow: Option<&Flow>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match selected_flow {
            Some(flow) => {
                if let Some(packet_table) = &self.packet_table {
                    packet_table.update(cx, |table, table_cx| {
                        let delegate = table.delegate_mut();
                        delegate.set_flow(Some(flow));
                        table.refresh(table_cx);
                        table_cx.notify();
                    });
                } else {
                    self.packet_table = Some(PacketTableDelegate::create_entity(
                        window,
                        cx,
                        Some(flow.clone()),
                    ));
                }
            }
            None => {
                self.packet_table = None;
                self.resize_state = None;
            }
        }
    }

    fn render_packet_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<Div> {
        let packet_table = self.packet_table.clone()?;
        let viewport_height: f32 = window.viewport_size().height.into();
        let max_height = (viewport_height - MIN_FLOW_REGION_HEIGHT).max(MIN_PACKET_PANE_HEIGHT);
        self.packet_pane_height = self
            .packet_pane_height
            .clamp(MIN_PACKET_PANE_HEIGHT, max_height);

        let selected_flow = self.current_flow()?;
        let flow_summary = format!("{} ({:?})", selected_flow.endpoints, selected_flow.protocol);

        Some(
            div()
                .flex()
                .flex_col()
                .bg(rgb(0x202020))
                .border_t_1()
                .border_color(rgb(0x444444))
                .h(px(self.packet_pane_height))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .p_2()
                        .bg(rgb(0x252525))
                        .border_b_1()
                        .border_color(rgb(0x444444))
                        .cursor(CursorStyle::ResizeRow)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|view, event, window, cx| {
                                view.begin_resize(event, window, cx);
                            }),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xffffff))
                                .child("Flow Packets"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xaaaaaa))
                                .child(flow_summary),
                        )
                        .child(
                            Button::new("flow_close_button")
                                .icon(IconName::WindowClose)
                                .on_click(cx.listener(|view, _event, _window, cx| {
                                    view.selected_flow = None;
                                    view.packet_table = None;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .child(Table::new(&packet_table)),
                ),
        )
    }
}

impl Render for WirecrabApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let flows_vec = self.filtered_flows(cx);
        let filtered_count = flows_vec.len();
        let selected_flow = self.selected_flow;

        self.flow_table.update(cx, move |table, _cx| {
            let delegate = table.delegate_mut();
            delegate.set_flows(flows_vec);
            delegate.selected_flow = selected_flow;
        });

        let total_flows = self.flows.len();
        let current_flow = self.current_flow().cloned();
        self.sync_packet_table(current_flow.as_ref(), window, cx);

        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .on_mouse_move(cx.listener(|view, event, window, cx| {
                view.update_resize(event, window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|view, event, _window, _cx| {
                    view.end_resize(event);
                }),
            )
            .child(
                div()
                    .text_xl()
                    .p_2()
                    .bg(rgb(0x252525))
                    .border_b_1()
                    .border_color(rgb(0x444444))
                    .child(format!(
                        "Wirecrab: {} flows loaded ({} shown)",
                        total_flows, filtered_count
                    )),
            )
            .child(SearchBar::new(&self.search_input))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(Table::new(&self.flow_table)),
            );

        if let Some(packet_pane) = self.render_packet_pane(window, cx) {
            container = container.child(packet_pane);
        }

        container
    }
}

pub fn run_ui(initial_flows: HashMap<FlowKey, Flow>) -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);
        let win_opts = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(String::from("Wirecrab").into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        cx.open_window(win_opts, move |window, cx| {
            let app = cx.new(|cx| WirecrabApp::new(initial_flows.clone(), window, cx));
            cx.new(move |cx| Root::new(app, window, cx))
        })
        .ok();
    });
    Ok(())
}
