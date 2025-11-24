use crate::flow::filter::FlowFilter;
use crate::flow::*;
use crate::gui::assets::Assets;
use crate::gui::components::{FlowTable, PacketBytesView, PacketTable, SearchBar, Toolbar};
use crate::gui::layout::{BottomSplit, Layout};
use crate::loader::{FlowLoadController, FlowLoadStatus};
use gpui::AsyncApp;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::InputEvent;
use gpui_component::progress::Progress;
use gpui_component::resizable::ResizableState;
use gpui_component::table::TableEvent;
use gpui_component::{ActiveTheme, Disableable, Icon, IconName, Root, StyledExt};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

struct FlowStore {
    flows: HashMap<FlowKey, Flow>,
    start_timestamp: Option<f64>,
    selected_flow: Option<FlowKey>,
}

impl FlowStore {
    fn new() -> Self {
        Self {
            flows: HashMap::new(),
            start_timestamp: None,
            selected_flow: None,
        }
    }

    fn ingest(&mut self, flows: HashMap<FlowKey, Flow>, start_timestamp: Option<f64>) {
        let min_ts = flows
            .values()
            .map(|flow| flow.timestamp)
            .fold(f64::INFINITY, |acc, ts| acc.min(ts));

        let effective_start =
            start_timestamp.or_else(|| (min_ts != f64::INFINITY).then_some(min_ts));

        if self.start_timestamp.is_none() {
            self.start_timestamp = effective_start;
        }

        self.flows = flows;
        info!(flow_count = self.flows.len(), "Flow store updated");
    }

    fn filtered_flows(&self, search_text: &str) -> Vec<(FlowKey, Flow)> {
        let filter = FlowFilter::new(search_text, self.start_timestamp);
        self.flows
            .iter()
            .filter(|(_, flow)| filter.matches_flow(flow))
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    fn select_flow(&mut self, flow_key: FlowKey) {
        self.selected_flow = Some(flow_key);
    }

    fn clear_selection(&mut self) {
        self.selected_flow = None;
    }

    fn selected_flow(&self) -> Option<FlowKey> {
        self.selected_flow
    }

    fn current_flow(&self) -> Option<Flow> {
        self.selected_flow
            .and_then(|key| self.flows.get(&key).cloned())
    }

    fn start_timestamp(&self) -> Option<f64> {
        self.start_timestamp
    }

    fn total_flows(&self) -> usize {
        self.flows.len()
    }
}

struct LoaderState {
    controller: FlowLoadController,
    progress: Option<f32>,
    error: Option<String>,
}

impl LoaderState {
    fn new(path: PathBuf) -> Self {
        Self {
            controller: FlowLoadController::new(path),
            progress: Some(0.0),
            error: None,
        }
    }

    fn poll(&mut self) -> FlowLoadStatus {
        let status = self.controller.poll();
        match &status {
            FlowLoadStatus::Loading { progress } => {
                self.progress = Some(*progress);
            }
            FlowLoadStatus::Ready { .. } | FlowLoadStatus::Idle => {
                self.progress = None;
            }
            FlowLoadStatus::Error(error) => {
                self.progress = None;
                self.error = Some(error.clone());
            }
        }
        status
    }

    fn progress(&self) -> Option<f32> {
        self.progress
    }

    fn error(&self) -> Option<&String> {
        self.error.as_ref()
    }
}

struct FlowView {
    search_bar: SearchBar,
    table: FlowTable,
}

impl FlowView {
    fn new(window: &mut Window, cx: &mut Context<WirecrabApp>) -> Self {
        let search_bar = SearchBar::create(window, cx);
        let table = FlowTable::create(window, cx, Vec::new(), None, None);

        cx.subscribe_in(
            search_bar.entity(),
            window,
            |_view, _state, event, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            table.entity(),
            window,
            |app, table_state, event, _window, cx| {
                if let TableEvent::SelectRow(row_ix) = event {
                    let state = table_state.read(cx);
                    if let Some((key, _)) = state.delegate().flows.get(*row_ix) {
                        debug!(row = *row_ix, flow = ?key, "Flow row selected");
                        app.on_flow_selected(*key);
                        cx.notify();
                    } else {
                        warn!(row = *row_ix, "Flow row selection out of bounds");
                    }
                }
            },
        )
        .detach();

        Self { search_bar, table }
    }

    fn query(&self, cx: &App) -> String {
        self.search_bar.entity().read(cx).value().to_string()
    }

    fn search_bar(&self) -> SearchBar {
        self.search_bar.clone()
    }

    fn table(&self) -> FlowTable {
        self.table.clone()
    }

    fn update_table(
        &self,
        flows: Vec<(FlowKey, Flow)>,
        selected: Option<FlowKey>,
        start_timestamp: Option<f64>,
        cx: &mut App,
    ) {
        self.table.update(cx, move |table, cx| {
            let delegate = table.delegate_mut();
            delegate.set_start_timestamp(start_timestamp);
            delegate.set_flows(flows);
            delegate.selected_flow = selected;
            table.refresh(cx);
            cx.notify();
        });
    }
}

struct DetailPane {
    packet_table: Option<PacketTable>,
    split_state: Entity<ResizableState>,
    selected_packet: Option<Packet>,
}

impl DetailPane {
    fn new(cx: &mut Context<WirecrabApp>) -> Self {
        Self {
            packet_table: None,
            split_state: cx.new(|_| ResizableState::default()),
            selected_packet: None,
        }
    }

    fn ensure_table(
        &mut self,
        window: &mut Window,
        cx: &mut Context<WirecrabApp>,
        flow: &Flow,
        start_timestamp: Option<f64>,
    ) {
        if let Some(table) = &mut self.packet_table {
            table.update(flow, start_timestamp, cx);
        } else {
            let packet_table = PacketTable::create(window, cx, flow, start_timestamp);
            Self::subscribe_to_selection(&packet_table, window, cx);
            self.packet_table = Some(packet_table);
            self.split_state = cx.new(|_| ResizableState::default());
            self.selected_packet = None;
        }
    }

    fn subscribe_to_selection(
        packet_table: &PacketTable,
        window: &mut Window,
        cx: &mut Context<WirecrabApp>,
    ) {
        cx.subscribe_in(
            packet_table.entity(),
            window,
            |app, table, event, _window, cx| {
                if let TableEvent::SelectRow(row_ix) = event {
                    let state = table.read(cx);
                    let packet = state.delegate().packets.get(*row_ix).cloned();
                    if packet.is_some() {
                        debug!(row = *row_ix, "Packet row selected");
                    } else {
                        warn!(row = *row_ix, "Packet row selection out of bounds");
                    }
                    app.on_packet_selected(packet);
                    cx.notify();
                }
            },
        )
        .detach();
    }

    fn packet_table(&self) -> Option<PacketTable> {
        self.packet_table.clone()
    }

    fn split_state(&self) -> Entity<ResizableState> {
        self.split_state.clone()
    }

    fn selected_packet_bytes(&self) -> Option<&[u8]> {
        self.selected_packet
            .as_ref()
            .map(|packet| packet.data.as_slice())
    }

    fn set_selected_packet(&mut self, packet: Option<Packet>) {
        self.selected_packet = packet;
    }

    fn has_content(&self) -> bool {
        self.packet_table.is_some()
    }

    fn close(&mut self, cx: &mut Context<WirecrabApp>) {
        self.packet_table = None;
        self.selected_packet = None;
        self.split_state = cx.new(|_| ResizableState::default());
        debug!("Detail pane closed");
    }
}

pub struct WirecrabApp {
    path: String,
    loader: LoaderState,
    flows: FlowStore,
    flow_view: FlowView,
    detail_pane: DetailPane,
    main_split_state: Entity<ResizableState>,
}

impl WirecrabApp {
    fn new(path: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let loader = LoaderState::new(path.clone());
        let flow_view = FlowView::new(window, cx);
        let detail_pane = DetailPane::new(cx);
        let main_split_state = cx.new(|_| ResizableState::default());

        cx.spawn(|view: gpui::WeakEntity<WirecrabApp>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(30))
                        .await;
                    let result = view.update(
                        &mut cx,
                        |app: &mut WirecrabApp, cx: &mut Context<WirecrabApp>| app.check_loader(cx),
                    );

                    match result {
                        Ok(true) => continue,
                        _ => break,
                    }
                }
                Ok::<(), anyhow::Error>(())
            }
        })
        .detach();

        Self {
            path: path.to_string_lossy().to_string(),
            loader,
            flows: FlowStore::new(),
            flow_view,
            detail_pane,
            main_split_state,
        }
    }

    fn check_loader(&mut self, cx: &mut Context<Self>) -> bool {
        match self.loader.poll() {
            FlowLoadStatus::Loading { .. } => {
                cx.notify();
                true
            }
            FlowLoadStatus::Ready {
                flows,
                start_timestamp,
            } => {
                info!(flow_count = flows.len(), "Loader ready with parsed flows");
                self.flows.ingest(flows, start_timestamp);
                cx.notify();
                false
            }
            FlowLoadStatus::Error(_) => {
                warn!("Loader encountered an error");
                cx.notify();
                false
            }
            FlowLoadStatus::Idle => false,
        }
    }

    fn on_flow_selected(&mut self, flow_key: FlowKey) {
        debug!(flow = ?flow_key, "Flow selected");
        self.flows.select_flow(flow_key);
        self.detail_pane.set_selected_packet(None);
    }

    fn on_packet_selected(&mut self, packet: Option<Packet>) {
        if let Some(packet) = &packet {
            debug!(
                timestamp = packet.timestamp,
                length = packet.length,
                "Packet selected"
            );
        } else {
            debug!("Packet selection cleared");
        }
        self.detail_pane.set_selected_packet(packet);
    }

    fn close_details(&mut self, cx: &mut Context<Self>) {
        debug!("Clearing flow selection and closing details");
        self.flows.clear_selection();
        self.detail_pane.close(cx);
    }

    fn render_loader_status_bar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if let Some(progress) = self.loader.progress() {
            let progress_percent = (progress * 100.0).clamp(0.0, 100.0);
            let headline = format!("Loading {}", self.path);

            let status = div()
                .id("loader_status_progress")
                .bg(cx.theme().colors.secondary)
                .border_t_1()
                .border_color(cx.theme().colors.border)
                .px_3()
                .py_2()
                .flex()
                .items_center()
                .gap_4()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().font_bold().child(headline))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().colors.muted_foreground)
                                .child("Parsing flows. Large captures can take a minute."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .w(px(220.0))
                        .child(div().h_3().child(Progress::new().value(progress_percent)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().colors.muted_foreground)
                                .flex()
                                .justify_end()
                                .child(format!("{progress_percent:.0}%")),
                        ),
                );

            return Some(status.into_any_element());
        }

        if let Some(error) = self.loader.error() {
            let message = format!("Wirecrab could not open {}.", self.path);

            let status = div()
                .id("loader_status_error")
                .bg(cx.theme().colors.secondary)
                .border_t_1()
                .border_color(cx.theme().colors.border)
                .px_3()
                .py_2()
                .flex()
                .items_center()
                .gap_3()
                .child(Icon::new(IconName::TriangleAlert))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().font_bold().child(message))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().colors.muted_foreground)
                                .child(error.clone()),
                        ),
                );

            return Some(status.into_any_element());
        }

        None
    }
}

impl Render for WirecrabApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let loader_status = self.render_loader_status_bar(cx);

        let query = self.flow_view.query(cx);
        let flows_vec = self.flows.filtered_flows(&query);
        let selected_flow = self.flows.selected_flow();
        let start_timestamp = self.flows.start_timestamp();

        self.flow_view
            .update_table(flows_vec, selected_flow, start_timestamp, cx);

        let current_flow = self.flows.current_flow();

        if let Some(ref flow) = current_flow {
            self.detail_pane
                .ensure_table(window, cx, flow, start_timestamp);
        } else if self.detail_pane.has_content() {
            self.detail_pane.close(cx);
        }

        let toolbar = {
            let flow_count = self.flows.total_flows();
            let file_info = div()
                .flex()
                .items_center()
                .gap_2()
                .child(Icon::new(IconName::FolderOpen))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0()
                        .child(div().text_sm().child(self.path.clone()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().colors.muted_foreground)
                                .child(format!("{flow_count} flows")),
                        ),
                );

            let clear_selection =
                cx.listener(|app: &mut WirecrabApp, &_event: &(), _window, cx| {
                    app.close_details(cx);
                    cx.notify();
                });

            let clear_button = Button::new("clear_selection_button")
                .icon(Icon::new(IconName::CircleX))
                .label("Clear selection")
                .disabled(selected_flow.is_none())
                .on_click(move |_event, window, cx| {
                    clear_selection(&(), window, cx);
                });

            Toolbar::new()
                .left(file_info)
                .center(self.flow_view.search_bar())
                .right(clear_button)
        };

        let mut layout = Layout::new(self.main_split_state.clone())
            .header(toolbar)
            .main(self.flow_view.table());

        if let (Some(flow), Some(packet_table)) =
            (current_flow.as_ref(), self.detail_pane.packet_table())
        {
            let header_content = PacketTable::pane_header(flow, cx);
            let close_handler = cx.listener(|app: &mut WirecrabApp, &_event: &(), _window, cx| {
                app.close_details(cx);
                cx.notify();
            });

            let bytes_view = PacketBytesView::new(self.detail_pane.selected_packet_bytes());

            let split = BottomSplit::new(
                "packet_detail_split",
                self.detail_pane.split_state(),
                packet_table,
                bytes_view,
            )
            .left_size(px(420.0))
            .left_range(px(280.0)..Pixels::MAX)
            .right_range(px(240.0)..Pixels::MAX);

            layout = layout.bottom_closable_split(header_content, split, close_handler);
        }

        if let Some(status) = loader_status {
            layout = layout.status_bar(status);
        }

        div().size_full().child(layout)
    }
}

pub fn run_ui(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new().with_assets(Assets);
    info!("Launching GPUI application");

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);
        crate::gui::theme::init(cx);
        let win_opts = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(String::from("Wirecrab").into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        cx.open_window(win_opts, move |window, cx| {
            let app = cx.new(|cx| WirecrabApp::new(path.clone(), window, cx));

            cx.new(move |cx| Root::new(app, window, cx))
        })
        .ok();
    });
    info!("GPUI application exited");
    Ok(())
}
