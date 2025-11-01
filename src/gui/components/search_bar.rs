use gpui::*;
use gpui_component::{
    Icon, IconName,
    input::{InputState, TextInput},
};

#[derive(IntoElement)]
pub struct SearchBar {
    input_state: Entity<InputState>,
}

impl SearchBar {
    pub fn new(input_state: &Entity<InputState>) -> Self {
        Self {
            input_state: input_state.clone(),
        }
    }
}

impl RenderOnce for SearchBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .p_2()
            .bg(rgb(0x252525))
            .border_b_1()
            .border_color(rgb(0x444444))
            .child(
                div().flex_1().child(
                    TextInput::new(&self.input_state)
                        .prefix(Icon::new(IconName::Search))
                        .cleanable(),
                ),
            )
    }
}
