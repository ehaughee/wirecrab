#[cfg(feature = "tui")]
pub mod widgets;

#[cfg(feature = "tui")]
pub mod theme;

#[cfg(feature = "tui")]
mod app;

#[cfg(feature = "tui")]
pub use app::run_tui;

#[cfg(feature = "tui")]
pub fn to_color(hex: u32) -> ratatui::style::Color {
    let r = ((hex >> 16) & 0xFF) as u8;
    let g = ((hex >> 8) & 0xFF) as u8;
    let b = (hex & 0xFF) as u8;
    ratatui::style::Color::Rgb(r, g, b)
}

#[cfg(not(feature = "tui"))]
pub fn run_tui(_path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("TUI feature is disabled. Rebuild with --features tui to enable the Ratatui TUI.");
    Ok(())
}
