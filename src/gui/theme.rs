#[cfg(feature = "ui")]
use gpui::{App, SharedString};
#[cfg(feature = "ui")]
use gpui_component::{Theme, ThemeRegistry};
#[cfg(feature = "ui")]
use std::path::PathBuf;
#[cfg(feature = "ui")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "ui")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[cfg(feature = "ui")]
const FLEXOKI_LIGHT: &str = "Flexoki Light";
#[cfg(feature = "ui")]
const FLEXOKI_DARK: &str = "Flexoki Dark";

#[cfg(feature = "ui")]
static CURRENT_THEME: OnceLock<Mutex<SharedString>> = OnceLock::new();

#[cfg(feature = "ui")]
fn current_theme() -> SharedString {
    CURRENT_THEME
        .get_or_init(|| Mutex::new(SharedString::from(FLEXOKI_DARK)))
        .lock()
        .expect("current theme lock poisoned")
        .clone()
}

#[cfg(feature = "ui")]
fn set_current_theme(name: SharedString) {
    if let Some(lock) = CURRENT_THEME.get()
        && let Ok(mut guard) = lock.lock()
    {
        *guard = name;
    }
}

#[cfg(feature = "ui")]
fn theme_name(mode: ThemeMode) -> SharedString {
    match mode {
        ThemeMode::Light => SharedString::from(FLEXOKI_LIGHT),
        ThemeMode::Dark => SharedString::from(FLEXOKI_DARK),
    }
}

#[cfg(feature = "ui")]
pub fn apply_theme(mode: ThemeMode, cx: &mut App) {
    let name = theme_name(mode);
    set_current_theme(name.clone());
    if let Some(theme) = ThemeRegistry::global(cx).themes().get(&name).cloned() {
        Theme::global_mut(cx).apply_config(&theme);
    }
}

#[cfg(feature = "ui")]
pub fn init(cx: &mut App) {
    // Load and watch themes from ./themes directory
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        let name = current_theme();
        if let Some(theme) = ThemeRegistry::global(cx).themes().get(&name).cloned() {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        eprintln!("Failed to watch themes directory: {}", err);
    }

    apply_theme(ThemeMode::Dark, cx);
}
