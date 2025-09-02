#[cfg(feature = "tui")]
pub mod widgets;

#[cfg(feature = "tui")]
mod app;

#[cfg(feature = "tui")]
pub use app::run_tui;

#[cfg(not(feature = "tui"))]
pub fn run_tui(
    _flows: std::collections::HashMap<crate::flow::FlowKey, crate::flow::Flow>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("TUI feature is disabled. Rebuild with --features tui to enable the Ratatui TUI.");
    Ok(())
}
