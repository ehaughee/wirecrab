#[cfg(feature = "ui")]
use gpui::{App, SharedString};
#[cfg(feature = "ui")]
use gpui_component::{Theme, ThemeRegistry};
#[cfg(feature = "ui")]
use std::path::PathBuf;

#[cfg(feature = "ui")]
pub fn init(cx: &mut App) {
    let theme_name = SharedString::from("Flexoki Dark");
    // Load and watch themes from ./themes directory
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        eprintln!("Failed to watch themes directory: {}", err);
    }
}
