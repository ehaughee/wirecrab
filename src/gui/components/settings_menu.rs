use gpui::*;
use gpui_component::button::Button;
use gpui_component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_component::{Icon, IconName};
use std::rc::Rc;

type ToggleHandler = Rc<dyn Fn(&(), &mut Window, &mut App)>;

/// Small settings dropdown that can broadcast toggles to the app.
#[derive(IntoElement, Clone)]
pub struct SettingsMenu {
    prefer_names: bool,
    on_toggle_names: ToggleHandler,
}

impl SettingsMenu {
    pub fn new(
        prefer_names: bool,
        on_toggle_names: impl Fn(&(), &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            prefer_names,
            on_toggle_names: Rc::new(on_toggle_names),
        }
    }
}

impl RenderOnce for SettingsMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let prefer_names = self.prefer_names;
        let on_toggle_names = self.on_toggle_names;

        Button::new("settings_menu_button")
            .icon(Icon::new(IconName::Settings))
            .label("Settings")
            .dropdown_caret(true)
            .dropdown_menu_with_anchor(Corner::BottomLeft, move |menu: PopupMenu, _window, _cx| {
                menu.item(
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
