use crate::flow::*;
use crate::gui::assets::Assets;
use crate::gui::components::{FlowTableDelegate, PacketPane, PacketTableDelegate, SearchBar};
use crate::gui::layout::Layout;
use crate::loader::{LoadStatus, Loader};
use gpui::AsyncApp;
use gpui::*;
use gpui_component::Root;
use gpui_component::input::{InputEvent, InputState};
use gpui_component::progress::Progress;
use gpui_component::resizable::ResizableState;
use gpui_component::table::{Table, TableEvent, TableState};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct WirecrabApp {
    path: String,
    flows: HashMap<FlowKey, Flow>,
    loader: Option<Loader>,
    loading_progress: Option<f32>,
    error_message: Option<String>,
    selected_flow: Option<FlowKey>,
    search_input: Entity<InputState>,
    flow_table: Entity<TableState<FlowTableDelegate>>,
    packet_table: Option<Entity<TableState<PacketTableDelegate>>>,
    resizable_state: Entity<ResizableState>,
}

impl WirecrabApp {
    fn new(path: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = SearchBar::create_state(window, cx);
        let loader = Loader::new(path.clone());
        let resizable_state = cx.new(|_| ResizableState::default());

        // Start with empty flows
        let flows = HashMap::new();
        let initial_view: Vec<(FlowKey, Flow)> = Vec::new();

        let flow_table = FlowTableDelegate::create_entity(window, cx, initial_view, None);

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

        // Schedule the loader check loop
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
            path: path.clone().to_string_lossy().to_string(),
            flows,
            loader: Some(loader),
            loading_progress: Some(0.0),
            error_message: None,
            selected_flow: None,
            search_input,
            flow_table,
            packet_table: None,
            resizable_state,
        }
    }

    fn check_loader(&mut self, cx: &mut Context<Self>) -> bool {
        let mut loaded_flows = None;
        let mut error = None;
        let mut progress = None;

        if let Some(loader) = &self.loader {
            while let Some(status) = loader.try_recv() {
                match status {
                    LoadStatus::Progress(p) => progress = Some(p),
                    LoadStatus::Loaded(flows) => {
                        loaded_flows = Some(flows);
                        break;
                    }
                    LoadStatus::Error(e) => {
                        error = Some(e);
                        break;
                    }
                }
            }
        } else {
            return false;
        }

        let mut needs_notify = false;
        let mut finished = false;

        if let Some(p) = progress {
            self.loading_progress = Some(p);
            needs_notify = true;
        }

        if let Some(flows) = loaded_flows {
            self.flows = flows;
            self.loading_progress = None;
            self.loader = None;
            self.update_flow_table(cx);
            needs_notify = true;
            finished = true;
        } else if let Some(e) = error {
            self.error_message = Some(e);
            self.loading_progress = None;
            self.loader = None;
            needs_notify = true;
            finished = true;
        }

        if needs_notify {
            cx.notify();
        }

        !finished
    }

    fn update_flow_table(&mut self, cx: &mut Context<Self>) {
        let flows_vec = self.filtered_flows(cx);
        let selected_flow = self.selected_flow;
        self.flow_table.update(cx, move |table, _cx| {
            let delegate = table.delegate_mut();
            delegate.set_flows(flows_vec);
            delegate.selected_flow = selected_flow;
        });
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
                    self.resizable_state = cx.new(|_| ResizableState::default());
                }
            }
            None => {
                self.packet_table = None;
            }
        }
    }
}

impl Render for WirecrabApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(progress) = self.loading_progress {
            let progress_percent = progress * 100.0;
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .bg(rgb(0x1e1e1e))
                .text_color(rgb(0xffffff))
                .child(
                    div()
                        .text_xl()
                        .mb_4()
                        .child(format!("Loading {}...", self.path)),
                )
                .child(
                    div()
                        .w(window.bounds().size.width * 0.8)
                        .h_4()
                        .child(Progress::new().value(progress_percent).size_full()),
                )
                .child(
                    div()
                        .mt_2()
                        .text_sm()
                        .text_color(rgb(0xaaaaaa))
                        .child(format!("{:.0}%", progress_percent)),
                );
        }

        if let Some(error) = &self.error_message {
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .bg(rgb(0x1e1e1e))
                .text_color(rgb(0xff5555))
                .child(div().text_xl().mb_4().child("Error loading file"))
                .child(div().text_sm().child(error.clone()));
        }

        let flows_vec = self.filtered_flows(cx);
        // let filtered_count = flows_vec.len();
        let selected_flow = self.selected_flow;

        self.flow_table.update(cx, move |table, _cx| {
            let delegate = table.delegate_mut();
            delegate.set_flows(flows_vec);
            delegate.selected_flow = selected_flow;
        });

        // let total_flows = self.flows.len();
        let current_flow = self.current_flow().cloned();
        self.sync_packet_table(current_flow.as_ref(), window, cx);

        let header = div()
            // .text_xl()
            // .py_1()
            // .px_2()
            // .bg(rgb(0x252525))
            // .border_b_1()
            // .border_color(rgb(0x444444))
            // .child(format!(
            //     "Wirecrab: {} flows ({} shown)",
            //     total_flows, filtered_count
            // ))
            .child(SearchBar::new(&self.search_input));

        let main = Table::new(&self.flow_table);

        let mut layout = Layout::new(self.resizable_state.clone())
            .header(header)
            .main(main);

        match self.current_flow() {
            Some(flow) => {
                layout = layout.bottom(PacketPane::new(
                    self.packet_table.as_ref().unwrap().clone(),
                    flow.clone(),
                    cx.listener(|view, _event, _window, cx| {
                        view.selected_flow = None;
                        view.packet_table = None;
                        cx.notify();
                    }),
                ));
            }
            None => {}
        }

        div().size_full().child(layout)
    }
}

pub fn run_ui(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
            let app = cx.new(|cx| WirecrabApp::new(path.clone(), window, cx));
            cx.new(move |cx| Root::new(app, window, cx))
        })
        .ok();
    });
    Ok(())
}
