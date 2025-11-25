use gpui::{Result, TextSystem};
use std::borrow::Cow;

pub const JETBRAINS_MONO_FAMILY: &str = "JetBrains Mono";
pub const JETBRAINS_MONO_ASSET_PATH: &str = "fonts/JetBrainsMono-Regular.ttf";
const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");

pub fn register_with(text_system: &TextSystem) -> Result<()> {
    text_system.add_fonts(vec![Cow::Borrowed(JETBRAINS_MONO_BYTES)])
}

pub fn bytes_for(path: &str) -> Option<&'static [u8]> {
    let normalized = path.trim_start_matches('/');
    match normalized {
        JETBRAINS_MONO_ASSET_PATH => Some(JETBRAINS_MONO_BYTES),
        _ => None,
    }
}
