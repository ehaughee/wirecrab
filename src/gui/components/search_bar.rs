use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName,
    input::{Input, InputState},
};

#[derive(IntoElement)]
pub struct SearchBar {
    input_state: Entity<InputState>,
}

impl SearchBar {
    const PLACEHOLDER: &'static str = "Search by IP or protocol...";
    pub fn create_state<Owner>(window: &mut Window, cx: &mut Context<Owner>) -> Entity<InputState> {
        let placeholder = SharedString::from(Self::PLACEHOLDER);
        cx.new(move |cx| InputState::new(window, cx).placeholder(placeholder.clone()))
    }

    pub fn new(input_state: &Entity<InputState>) -> Self {
        Self {
            input_state: input_state.clone(),
        }
    }
}

impl RenderOnce for SearchBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .p_1()
            .bg(cx.theme().colors.popover)
            .border_b_1()
            .border_color(cx.theme().colors.border)
            .child(
                div().flex_1().child(
                    Input::new(&self.input_state)
                        .prefix(Icon::new(IconName::Search))
                        .cleanable(true),
                ),
            )
    }
}
