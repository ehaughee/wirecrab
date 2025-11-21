use gpui::*;
use gpui_component::resizable::{ResizableState, resizable_panel, v_resizable};

const MIN_BOTTOM_PANE_HEIGHT: f32 = 160.0;
const DEFAULT_BOTTOM_PANE_HEIGHT: f32 = 320.0;

#[derive(IntoElement)]
pub struct Layout {
    header: AnyElement,
    main: AnyElement,
    bottom: Option<AnyElement>,
    resizable_state: Entity<ResizableState>,
}

impl Layout {
    pub fn new(resizable_state: Entity<ResizableState>) -> Self {
        Self {
            header: div().into_any_element(),
            main: div().into_any_element(),
            bottom: None,
            resizable_state,
        }
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = header.into_any_element();
        self
    }

    pub fn main(mut self, main: impl IntoElement) -> Self {
        self.main = main.into_any_element();
        self
    }

    pub fn bottom(mut self, bottom: impl IntoElement) -> Self {
        self.bottom = Some(bottom.into_any_element());
        self
    }
}

impl RenderOnce for Layout {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let top_panel_content = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .child(self.header)
            .child(div().flex_1().overflow_hidden().child(self.main));

        let mut container = v_resizable("main_split")
            .with_state(&self.resizable_state)
            .child(
                resizable_panel()
                    .size(px(DEFAULT_BOTTOM_PANE_HEIGHT))
                    .child(top_panel_content),
            );

        if let Some(bottom) = self.bottom {
            container = container.child(
                resizable_panel()
                    .size(px(DEFAULT_BOTTOM_PANE_HEIGHT))
                    .size_range(px(MIN_BOTTOM_PANE_HEIGHT)..px(f32::MAX))
                    .child(bottom),
            );
        }

        div().size_full().child(container)
    }
}
