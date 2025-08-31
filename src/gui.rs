#[cfg(feature = "ui")]
mod ui_impl {
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
        // Header
        let header = label("Wirecrab Flows");

        // Filter bar (horizontal): label + textbox
        let filter_bar = flex((
            label("Filter:"),
            textbox(data.search.clone(), |state: &mut AppState, new_value| {
                state.search = new_value;
            }),
        ))
        .direction(Axis::Horizontal);

        // Collect, format, filter, and sort keys for stable ordering
        let mut items: Vec<String> = data
            .flows
            .keys()
            .map(|k| k.to_display())
            .filter(|s| {
                if data.search.is_empty() {
                    true
                } else {
                    s.to_lowercase().contains(&data.search.to_lowercase())
                }
            })
            .collect();
        items.sort_unstable();

        // Build list of labels and wrap in a scrollable portal
        let list = flex(items.into_iter().map(label).collect::<Vec<_>>()).direction(Axis::Vertical);
        let scroll = portal(list);

        // Compose vertically
        flex((header, filter_bar, scroll)).direction(Axis::Vertical)
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
}

#[cfg(feature = "ui")]
pub use ui_impl::run_ui;

// Fallback stub when `ui` feature is disabled
#[cfg(not(feature = "ui"))]
pub fn run_ui(
    _items: std::collections::HashMap<crate::flow::FlowKey, crate::flow::Flow>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("UI feature is disabled. Rebuild with --features ui to enable the GUI.");
    Ok(())
}
