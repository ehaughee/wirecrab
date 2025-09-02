use super::widgets::table::string_table;
use crate::flow::*;
use std::collections::HashMap;
use xilem::view::{Axis, flex, label, portal, textbox};
use xilem::{EventLoop, WidgetView, Xilem};

// Simple app state that holds a list of strings to render.
#[derive(Default)]
pub struct AppState {
    pub flows: HashMap<FlowKey, Flow>,
    pub search: String,
}

// Build the UI view from the state
fn app_logic(data: &mut AppState) -> impl WidgetView<AppState> + use<> {
    // Header label
    let title_lbl = label("Wirecrab Flows");

    // Filter bar (horizontal): label + textbox
    let filter_bar = flex((
        label("Filter:"),
        textbox(data.search.clone(), |state: &mut AppState, new_value| {
            state.search = new_value;
        }),
    ))
    .direction(Axis::Horizontal)
    .must_fill_major_axis(true);

    // Collect, format, filter, and sort keys for stable ordering
    let mut items: Vec<(FlowKey, String)> = data
        .flows
        .keys()
        .map(|k| (*k, k.to_display()))
        .filter(|(_, s)| {
            if data.search.is_empty() {
                true
            } else {
                s.to_lowercase().contains(&data.search.to_lowercase())
            }
        })
        .collect();

    // Sort by timestamp (oldest first)
    items.sort_unstable_by(|a, b| {
        let flow_a = data.flows.get(&a.0);
        let flow_b = data.flows.get(&b.0);
        match (flow_a, flow_b) {
            (Some(fa), Some(fb)) => fa.timestamp.total_cmp(&fb.timestamp),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    // Build a simple table using string_table: header + rows (Flow, Packets, Bytes)
    let table_data: Vec<Vec<String>> = items
        .into_iter()
        .filter_map(|(key, text)| {
            data.flows.get(&key).map(|flow| {
                let pkt_count = flow.packets.len();
                let byte_count: usize = flow.packets.iter().map(|p| p.len()).sum();
                vec![text, format!("{}", pkt_count), format!("{}", byte_count)]
            })
        })
        .collect();

    let table = string_table::<AppState>(vec!["Flow", "Packets", "Bytes"], table_data);
    let scroll = portal(table);

    // Compose vertically
    flex((title_lbl, filter_bar, scroll)).direction(Axis::Vertical)
}

pub fn run_ui(initial_items: HashMap<FlowKey, Flow>) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        flows: initial_items,
        search: String::new(),
    };
    let app = Xilem::new(state, app_logic);
    app.run_windowed(EventLoop::with_user_event(), "Wirecrab".into())?;
    Ok(())
}
