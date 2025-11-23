use gpui::*;
use gpui_component::button::Button;
use gpui_component::resizable::{ResizableState, h_resizable, resizable_panel, v_resizable};
use gpui_component::{ActiveTheme, IconName};
use std::ops::Range;

const MIN_BOTTOM_PANE_HEIGHT: f32 = 160.0;
const DEFAULT_BOTTOM_PANE_HEIGHT: f32 = 320.0;

type CloseHandler = Box<
    dyn for<'event, 'window, 'app> Fn(&'event (), &'window mut Window, &'app mut App) + 'static,
>;

#[derive(IntoElement)]
pub struct Layout {
    header: AnyElement,
    main: AnyElement,
    bottom: Option<ClosableBottomPane>,
    resizable_state: Entity<ResizableState>,
}

struct ClosableBottomPane {
    header: AnyElement,
    content: BottomSplit,
    on_close: CloseHandler,
}

pub struct BottomSplit {
    id: SharedString,
    state: Entity<ResizableState>,
    left: AnyElement,
    right: AnyElement,
    left_size: Option<Pixels>,
    right_size: Option<Pixels>,
    left_range: Option<Range<Pixels>>,
    right_range: Option<Range<Pixels>>,
}

impl BottomSplit {
    pub fn new(
        id: impl Into<SharedString>,
        state: Entity<ResizableState>,
        left: impl IntoElement,
        right: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            state,
            left: left.into_any_element(),
            right: right.into_any_element(),
            left_size: None,
            right_size: None,
            left_range: None,
            right_range: None,
        }
    }

    pub fn left_size(mut self, size: impl Into<Pixels>) -> Self {
        self.left_size = Some(size.into());
        self
    }

    pub fn left_range(mut self, range: Range<Pixels>) -> Self {
        self.left_range = Some(range);
        self
    }

    pub fn right_range(mut self, range: Range<Pixels>) -> Self {
        self.right_range = Some(range);
        self
    }
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

    pub fn bottom_closable_split(
        mut self,
        header: impl IntoElement,
        split: BottomSplit,
        on_close: impl for<'event, 'window, 'app> Fn(&'event (), &'window mut Window, &'app mut App)
        + 'static,
    ) -> Self {
        self.bottom = Some(ClosableBottomPane {
            header: header.into_any_element(),
            content: split,
            on_close: Box::new(on_close),
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
            let ClosableBottomPane {
                header,
                content,
                on_close,
            } = bottom;

            let pane_body = render_split(content);
            let bottom_content = div()
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
                                    on_close(&(), window, cx);
                                }),
                        ),
                )
                .child(pane_body)
                .into_any_element();

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

fn render_split(split: BottomSplit) -> AnyElement {
    let BottomSplit {
        id,
        state,
        left,
        right,
        left_size,
        right_size,
        left_range,
        right_range,
    } = split;

    let mut left_panel = resizable_panel().child(left);
    if let Some(size) = left_size {
        left_panel = left_panel.size(size);
    }
    if let Some(range) = left_range {
        left_panel = left_panel.size_range(range);
    }

    let mut right_panel = resizable_panel().child(right);
    if let Some(size) = right_size {
        right_panel = right_panel.size(size);
    }
    if let Some(range) = right_range {
        right_panel = right_panel.size_range(range);
    }

    h_resizable(id)
        .with_state(&state)
        .child(left_panel)
        .child(right_panel)
        .into_any_element()
}
