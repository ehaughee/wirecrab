use gpui::*;
use gpui_component::ActiveTheme;

/// Flexible toolbar layout with left/center/right slots.
#[derive(IntoElement)]
pub struct Toolbar {
    left: AnyElement,
    center: AnyElement,
    right: AnyElement,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            left: div().into_any_element(),
            center: div().into_any_element(),
            right: div().into_any_element(),
        }
    }

    pub fn left(mut self, left: impl IntoElement) -> Self {
        self.left = left.into_any_element();
        self
    }

    pub fn center(mut self, center: impl IntoElement) -> Self {
        self.center = center.into_any_element();
        self
    }

    pub fn right(mut self, right: impl IntoElement) -> Self {
        self.right = right.into_any_element();
        self
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let left = div().flex().items_center().gap_2().child(self.left);

        let center = div()
            .flex()
            .items_center()
            .flex_1()
            .gap_2()
            .child(self.center);

        let right = div().flex().items_center().gap_2().child(self.right);

        div()
            .flex()
            .items_center()
            .gap_4()
            .px_4()
            .py_2()
            .bg(cx.theme().colors.secondary)
            .border_b_1()
            .border_color(cx.theme().colors.border)
            .child(left)
            .child(center)
            .child(right)
    }
}
