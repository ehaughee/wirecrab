use crate::flow::filter::FlowFilter;
use crate::flow::*;
use crate::gui::assets::Assets;
use crate::gui::components::{FlowTable, PacketTable, SearchBar};

use crate::gui::layout::Layout;
use crate::loader::{LoadStatus, Loader};
use gpui::AsyncApp;
use gpui::*;
use gpui_component::input::InputEvent;
use gpui_component::progress::Progress;
use gpui_component::resizable::ResizableState;
use gpui_component::table::TableEvent;
use gpui_component::{ActiveTheme, Root};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct WirecrabApp {
    path: String,
    flows: HashMap<FlowKey, Flow>,
    loader: Option<Loader>,
    loading_progress: Option<f32>,
    error_message: Option<String>,
    selected_flow: Option<FlowKey>,
    search_bar: SearchBar,
    flow_table: FlowTable,
    packet_table: Option<PacketTable>,
    resizable_state: Entity<ResizableState>,
    start_timestamp: Option<f64>,
}

impl WirecrabApp {
    fn new(path: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_bar = SearchBar::create(window, cx);
        let loader = Loader::new(path.clone());
        let resizable_state = cx.new(|_| ResizableState::default());

        // Start with empty flows
        let flows = HashMap::new();
        let initial_view: Vec<(FlowKey, Flow)> = Vec::new();

        let flow_table = FlowTable::create(window, cx, initial_view, None, None);

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
            flow_table.entity(),
            window,
            |view, table, event, _window, cx| {
                if let TableEvent::SelectRow(row_ix) = event {
                    let table_state = table.read(cx);
                    let delegate = table_state.delegate();
                    if let Some((key, _flow)) = delegate.flows.get(*row_ix) {
                        view.select_flow(*key);
                        cx.notify();
                    }
                }
            },
        )
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
            search_bar,
            flow_table,
            packet_table: None,
            resizable_state,
            start_timestamp: None,
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
                    LoadStatus::Loaded(flows, start_ts) => {
                        loaded_flows = Some(flows);
                        if let Some(ts) = start_ts {
                            self.start_timestamp = Some(ts);
                        }
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
            // If start_timestamp was not set by the loader (or is None), calculate it from flows
            if self.start_timestamp.is_none() {
                let min_ts = flows
                    .values()
                    .map(|f| f.timestamp)
                    .fold(f64::INFINITY, |a, b| a.min(b));

                if min_ts != f64::INFINITY {
                    self.start_timestamp = Some(min_ts);
                }
            }

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
        let start_timestamp = self.start_timestamp;
        self.flow_table.update(cx, move |table, cx| {
            let delegate = table.delegate_mut();
            delegate.set_start_timestamp(start_timestamp);
            delegate.set_flows(flows_vec);
            delegate.selected_flow = selected_flow;
            table.refresh(cx);
            cx.notify();
        });
    }

    fn select_flow(&mut self, flow_key: FlowKey) {
        self.selected_flow = Some(flow_key);
    }

    fn filtered_flows(&self, cx: &App) -> Vec<(FlowKey, Flow)> {
        let search_text = self
            .search_bar
            .entity()
            .read(cx)
            .value()
            .to_string();
        let filter = FlowFilter::new(&search_text, self.start_timestamp);

        self.flows
            .iter()
            .filter(|(_, flow)| filter.matches_flow(flow))
            .map(|(k, v)| (*k, v.clone()))
            .collect()
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
                .bg(cx.theme().colors.background)
                .text_color(cx.theme().colors.foreground)
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
                        .text_color(cx.theme().colors.muted_foreground)
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
                .bg(cx.theme().colors.background)
                .text_color(cx.theme().colors.foreground)
                .child(div().text_xl().mb_4().child("Error loading file"))
                .child(div().text_sm().child(error.clone()));
        }

        let flows_vec = self.filtered_flows(cx);
        let selected_flow = self.selected_flow;

        self.flow_table.update(cx, move |table, _cx| {
            let delegate = table.delegate_mut();
            delegate.set_flows(flows_vec);
            delegate.selected_flow = selected_flow;
        });

        let current_flow = self
            .selected_flow
            .and_then(|key| self.flows.get(&key))
            .cloned();

        match current_flow.as_ref() {
            Some(flow) => {
                if let Some(packet_table) = &mut self.packet_table {
                    packet_table.update(flow, self.start_timestamp, cx);
                } else {
                    self.packet_table =
                        Some(PacketTable::create(window, cx, flow, self.start_timestamp));
                    self.resizable_state = cx.new(|_| ResizableState::default());
                }
            }
            None => {
                self.packet_table = None;
            }
        }

        let header = self.search_bar.clone();
        let main = self.flow_table.clone();

        let mut layout = Layout::new(self.resizable_state.clone())
            .header(header)
            .main(main);

        if let (Some(packet_table), Some(flow)) =
            (self.packet_table.as_ref(), current_flow.as_ref())
        {
            let header_content = PacketTable::pane_header(flow, cx);
            let close_handler = cx.listener(|app: &mut WirecrabApp, &_event: &(), _window, cx| {
                app.selected_flow = None;
                app.packet_table = None;
                cx.notify();
            });

            layout =
                layout.bottom_closable(header_content, packet_table.clone(), move |window, cx| {
                    close_handler(&(), window, cx);
                });
        }

        div().size_full().child(layout)
    }
}

pub fn run_ui(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::new().with_assets(Assets);

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
    Ok(())
}
