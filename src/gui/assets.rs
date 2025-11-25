use crate::gui::fonts::{self, JETBRAINS_MONO_ASSET_PATH};
use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default)]
pub struct Assets;

impl Assets {
    fn normalized(path: &str) -> &str {
        path.trim_start_matches('/')
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = fonts::bytes_for(Self::normalized(path)) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }

        AssetSource::load(&gpui_component_assets::Assets, path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut entries = AssetSource::list(&gpui_component_assets::Assets, path)?;

        if Self::normalized(path) == "fonts"
            && !entries
                .iter()
                .any(|entry| entry.as_str() == "JetBrainsMono-Regular.ttf")
        {
            entries.push(SharedString::from(
                JETBRAINS_MONO_ASSET_PATH
                    .rsplit('/')
                    .next()
                    .unwrap_or("JetBrainsMono-Regular.ttf"),
            ));
        }

        Ok(entries)
    }
}
