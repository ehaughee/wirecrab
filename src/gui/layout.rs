use gpui::*;
use gpui_component::button::Button;
use gpui_component::resizable::{resizable_panel, v_resizable, ResizableState};
use gpui_component::{ActiveTheme, IconName};
use std::rc::Rc;

const MIN_BOTTOM_PANE_HEIGHT: f32 = 160.0;
const DEFAULT_BOTTOM_PANE_HEIGHT: f32 = 320.0;

#[derive(IntoElement)]
pub struct Layout {
    header: AnyElement,
    main: AnyElement,
    bottom: Option<BottomPane>,
    resizable_state: Entity<ResizableState>,
}

enum BottomPane {
    Plain(AnyElement),
    Closable {
        header: AnyElement,
        content: AnyElement,
        on_close: Rc<dyn Fn(&mut Window, &mut App)>,
    },
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
        self.bottom = Some(BottomPane::Plain(bottom.into_any_element()));
        self
    }

    pub fn bottom_closable(
        mut self,
        header: impl IntoElement,
        content: impl IntoElement,
        on_close: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.bottom = Some(BottomPane::Closable {
            header: header.into_any_element(),
            content: content.into_any_element(),
            on_close: Rc::new(on_close),
        });
        self
    }
}

impl RenderOnce for Layout {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let header_content = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().colors.background)
            .text_color(cx.theme().colors.foreground)
            .child(self.header)
            .child(div().flex_1().overflow_hidden().child(self.main));

        let content = if let Some(bottom) = self.bottom {
            let bottom_content = match bottom {
                BottomPane::Plain(content) => content,
                BottomPane::Closable {
                    header,
                    content,
                    on_close,
                } => {
                    let close_cb = on_close.clone();
                    div()
                        .flex()
                        .flex_col()
                        .bg(cx.theme().colors.background)
                        .border_t_1()
                        .border_color(cx.theme().colors.border)
                        .size_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .py_1()
                                .px_2()
                                .bg(cx.theme().colors.secondary)
                                .border_b_1()
                                .border_color(cx.theme().colors.border)
                                .child(div().flex_grow().child(header))
                                .child(
                                    Button::new("bottom_pane_close_button")
                                        .icon(IconName::WindowClose)
                                        .on_click(move |_event, window, cx| {
                                            close_cb(window, cx);
                                        }),
                                ),
                        )
                        .child(content)
                        .into_any_element()
                }
            };

            v_resizable("main_split")
                .with_state(&self.resizable_state)
                .child(resizable_panel().child(header_content))
                .child(
                    resizable_panel()
                        .size(px(DEFAULT_BOTTOM_PANE_HEIGHT))
                        .size_range(px(MIN_BOTTOM_PANE_HEIGHT)..px(f32::MAX))
                        .child(bottom_content),
                )
                .into_any_element()
        } else {
            header_content.into_any_element()
        };

        div().size_full().child(content)
    }
}
