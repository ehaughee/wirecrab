#[cfg(feature = "ui")]
pub mod app;
#[cfg(feature = "ui")]
pub mod widgets;

#[cfg(feature = "ui")]
pub use app::run_ui;

// Fallback stub when `ui` feature is disabled
#[cfg(not(feature = "ui"))]
pub fn run_ui(
    _items: std::collections::HashMap<crate::flow::FlowKey, crate::flow::Flow>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("UI feature is disabled. Rebuild with --features ui to enable the GUI.");
    Ok(())
}
