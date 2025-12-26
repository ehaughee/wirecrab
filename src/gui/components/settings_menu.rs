use crate::gui::theme::ThemeMode;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_component::{Icon, IconName};
use std::rc::Rc;

type ToggleHandler = Rc<dyn Fn(&(), &mut Window, &mut App)>;
type ThemeHandler = Rc<dyn Fn(ThemeMode, &mut Window, &mut App)>;

/// Compact settings dropdown inspired by gpui-component story examples.
#[derive(IntoElement, Clone)]
pub struct SettingsMenu {
    prefer_names: bool,
    on_toggle_names: ToggleHandler,
    theme_mode: ThemeMode,
    on_theme_change: ThemeHandler,
}

impl SettingsMenu {
    pub fn new(
        prefer_names: bool,
        on_toggle_names: impl Fn(&(), &mut Window, &mut App) + 'static,
        theme_mode: ThemeMode,
        on_theme_change: impl Fn(ThemeMode, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            prefer_names,
            on_toggle_names: Rc::new(on_toggle_names),
            theme_mode,
            on_theme_change: Rc::new(on_theme_change),
        }
    }
}

impl RenderOnce for SettingsMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let prefer_names = self.prefer_names;
        let on_toggle_names = self.on_toggle_names;
        let theme_mode = self.theme_mode;
        let on_theme_change = self.on_theme_change;

        Button::new("settings_menu_button")
            .icon(Icon::new(IconName::Settings))
            .ghost()
            .compact()
            .dropdown_menu_with_anchor(Corner::TopRight, move |menu: PopupMenu, _window, _cx| {
                menu.label("Display")
                    .item(
                        PopupMenuItem::new("Dark Mode")
                            .checked(matches!(theme_mode, ThemeMode::Dark))
                            .on_click({
                                let handler = on_theme_change.clone();
                                move |_event, window, cx| {
                                    let next = if matches!(theme_mode, ThemeMode::Dark) {
                                        ThemeMode::Light
                                    } else {
                                        ThemeMode::Dark
                                    };
                                    handler(next, window, cx);
                                }
                            }),
                    )
                    .separator()
                    .label("Preferences")
                    .item(
                        PopupMenuItem::new("Resolve Names")
                            .checked(prefer_names)
                            .on_click({
                                let handler = on_toggle_names.clone();
                                move |_event, window, cx| {
                                    handler(&(), window, cx);
                                }
                            }),
                    )
            })
            .into_any_element()
    }
}
