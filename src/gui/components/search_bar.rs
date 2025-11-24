use gpui::*;
use gpui_component::{
    Icon, IconName,
    input::{Input, InputState},
};

#[derive(IntoElement, Clone)]
pub struct SearchBar {
    input_state: Entity<InputState>,
}

impl SearchBar {
    const PLACEHOLDER: &'static str = "Search by IP or protocol...";

    pub fn create<Owner>(window: &mut Window, cx: &mut Context<Owner>) -> Self {
        let placeholder = SharedString::from(Self::PLACEHOLDER);
        let input_state =
            cx.new(move |cx| InputState::new(window, cx).placeholder(placeholder.clone()));
        Self { input_state }
    }

    pub fn entity(&self) -> &Entity<InputState> {
        &self.input_state
    }
}

impl RenderOnce for SearchBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().flex().flex_1().items_center().child(
            Input::new(&self.input_state)
                .prefix(Icon::new(IconName::Search))
                .cleanable(true),
        )
    }
}
