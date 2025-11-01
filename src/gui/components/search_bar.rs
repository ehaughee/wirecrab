use gpui::*;
use gpui_component::input::{InputState, TextInput};

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
        let input = self.input_state.clone();

        div()
            .flex()
            .items_center()
            .gap_2()
            .p_2()
            .bg(rgb(0x252525))
            .border_b_1()
            .border_color(rgb(0x444444))
            .child(div().text_sm().text_color(rgb(0xcccccc)).child("Search:"))
            .child(
                div()
                    .flex_1()
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        // Focus the input when the container is clicked
                        input.update(cx, |state, cx| {
                            state.focus(window, cx);
                        });
                    })
                    .child(TextInput::new(&self.input_state).cleanable()),
            )
    }
}
